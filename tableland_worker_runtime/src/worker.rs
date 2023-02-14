#[cfg(not(feature = "wasi"))]
use crate::spec::bindings::Runtime;
#[cfg(feature = "wasi")]
use crate::wasi_spec::bindings::Runtime;
use bytes::Bytes;
use fp_bindgen_support::host::errors::{InvocationError, RuntimeError};
use reqwest::Client;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use stretto::AsyncCache;

const IPFS_GATEWATE: &str = "http://localhost:8081/ipfs";

#[derive(Clone)]
pub struct Worker {
    http_client: Client,
    runtime_cache: AsyncCache<String, Runtime>,
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            http_client: Client::builder()
                .timeout(Duration::new(5, 0))
                .build()
                .unwrap(),
            runtime_cache: AsyncCache::new(12960, 1e6 as i64, tokio::spawn).unwrap(),
        }
    }

    pub async fn add_runtime(&self, cid: String) -> Result<Runtime, WorkerError> {
        let module = self
            .http_client
            .get(format!("{}/{}", IPFS_GATEWATE, cid))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await
            .map_err(|e| WorkerError::Ipfs(e.to_string()))?
            .to_vec();

        let file_name = format!("./tableland_worker_runtime/workers/{}.wasm", cid);
        tokio::fs::write(&file_name, &module).await?;

        self.new_runtime(cid, module).await
    }

    pub async fn get_runtime(&self, cid: String) -> Result<Runtime, WorkerError> {
        match self.runtime_cache.get(cid.as_str()) {
            Some(v) => Ok(v.value().clone()),
            None => self.load_runtime(cid).await,
        }
    }

    async fn load_runtime(&self, cid: String) -> Result<Runtime, WorkerError> {
        let file_name = format!("./tableland_worker_runtime/workers/{}.wasm", cid);
        let module = tokio::fs::read(&file_name)
            .await
            .map_err(|e| WorkerError::Ipfs(e.to_string()))?;

        self.new_runtime(cid, module).await
    }

    async fn new_runtime(&self, cid: String, module: Vec<u8>) -> Result<Runtime, WorkerError> {
        let rt = Runtime::new(module)?;
        rt.init()?;

        match self.runtime_cache.insert(cid, rt.clone(), 1).await {
            true => Ok(rt),
            false => Err(WorkerError::Cache("failed to cache runtime".to_string())),
        }
    }
}

#[derive(Debug)]
pub enum WorkerError {
    Runtime(RuntimeError),
    Invocation(InvocationError),
    Ipfs(String),
    Cache(String),
}

impl From<RuntimeError> for WorkerError {
    fn from(value: RuntimeError) -> Self {
        WorkerError::Runtime(value)
    }
}

impl From<InvocationError> for WorkerError {
    fn from(value: InvocationError) -> Self {
        WorkerError::Invocation(value)
    }
}

impl From<std::io::Error> for WorkerError {
    fn from(value: std::io::Error) -> Self {
        WorkerError::Ipfs(value.to_string())
    }
}

impl From<reqwest::Error> for WorkerError {
    fn from(value: reqwest::Error) -> Self {
        WorkerError::Ipfs(value.to_string())
    }
}

impl Display for WorkerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match &self {
            WorkerError::Runtime(e) => write!(f, "{}", e.to_string()),
            WorkerError::Invocation(e) => write!(f, "{}", e.to_string()),
            WorkerError::Ipfs(s) => write!(f, "{}", s),
            WorkerError::Cache(s) => write!(f, "{}", s),
        }
    }
}
