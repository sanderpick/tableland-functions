use reqwest::Client;
use tableland_std::{FuncResult, Request, Response};
use tableland_vm::{call_fetch, Instance, VmError, VmResult};
use thiserror::Error;

use crate::backend::Api;
use crate::config::Config;
use crate::instance::{instance_with_options, ApiInstanceOptions};

#[derive(Clone)]
pub struct Worker {
    config: Config,
    http_client: Client,
    runtime_cache: stretto::AsyncCache<String, Instance<Api>>,
}

impl Worker {
    pub fn new(config: Config) -> Self {
        Worker {
            config,
            http_client: Client::builder()
                .timeout(std::time::Duration::new(5, 0))
                .build()
                .unwrap(),
            runtime_cache: stretto::AsyncCache::new(12960, 1e6 as i64, tokio::spawn).unwrap(),
        }
    }

    pub async fn add_runtime(&self, cid: String) -> Result<bool, WorkerError> {
        let module = self
            .http_client
            .get(format!("{}/{}", self.config.ipfs.gateway, cid))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec();

        let file_name = format!("{}/{}.wasm", self.config.cache.directory, cid);
        tokio::fs::write(&file_name, &module).await?;

        self.new_runtime(cid.clone(), module).await
    }

    pub async fn run_runtime(&self, cid: String, req: Request) -> Result<Response, WorkerError> {
        let value = match self.runtime_cache.get_mut(cid.as_str()) {
            Some(v) => v,
            None => {
                self.load_runtime(cid.clone()).await?;
                self.runtime_cache
                    .get_mut(cid.as_str())
                    .ok_or(WorkerError::cache_err("failed to get runtime"))?
            }
        };
        let mut instance = value.clone_inner();

        match tokio::task::spawn_blocking(move || -> VmResult<FuncResult<Response>> {
            call_fetch(&mut instance, &req)
        })
        .await??
        {
            FuncResult::Ok(r) => Ok(r),
            FuncResult::Err(s) => Err(WorkerError::func_err(s)),
        }
    }

    async fn load_runtime(&self, cid: String) -> Result<bool, WorkerError> {
        let file_name = format!("{}/{}.wasm", self.config.cache.directory, cid);
        let module = tokio::fs::read(&file_name).await?;

        self.new_runtime(cid, module).await
    }

    async fn new_runtime(&self, cid: String, module: Vec<u8>) -> Result<bool, WorkerError> {
        let instance = tokio::task::spawn_blocking(move || -> Instance<Api> {
            instance_with_options(module.as_slice(), ApiInstanceOptions::default())
        })
        .await?;

        match self.runtime_cache.insert(cid, instance, 1).await {
            true => Ok(true),
            false => Err(WorkerError::cache_err("failed to cache runtime")),
        }
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("VM error: {0}")]
    Vm(String),
    #[error("Function error: {0}")]
    Func(String),
    #[error("IPFS error: {0}")]
    Ipfs(String),
    #[error("WASM cache error: {0}")]
    Cache(String),
    #[error("Tokie task join error: {0}")]
    TaskJoin(String),
}

impl WorkerError {
    pub fn vm_err(msg: impl Into<String>) -> Self {
        WorkerError::Vm { 0: msg.into() }
    }

    pub fn func_err(msg: impl Into<String>) -> Self {
        WorkerError::Func { 0: msg.into() }
    }

    pub fn ipfs_err(msg: impl Into<String>) -> Self {
        WorkerError::Ipfs { 0: msg.into() }
    }

    pub fn cache_err(msg: impl Into<String>) -> Self {
        WorkerError::Cache { 0: msg.into() }
    }

    pub fn task_join_err(msg: impl Into<String>) -> Self {
        WorkerError::TaskJoin { 0: msg.into() }
    }
}

impl From<std::io::Error> for WorkerError {
    fn from(e: std::io::Error) -> Self {
        WorkerError::cache_err(e.to_string())
    }
}

impl From<reqwest::Error> for WorkerError {
    fn from(e: reqwest::Error) -> Self {
        WorkerError::ipfs_err(e.to_string())
    }
}

impl From<VmError> for WorkerError {
    fn from(e: VmError) -> Self {
        WorkerError::vm_err(e.to_string())
    }
}

impl From<tokio::task::JoinError> for WorkerError {
    fn from(e: tokio::task::JoinError) -> Self {
        WorkerError::task_join_err(e.to_string())
    }
}
