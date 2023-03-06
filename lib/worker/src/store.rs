use reqwest::Client;
use tableland_client::{Tableland, TablelandClient};
use tableland_std::{FuncResult, Request, Response};
use tableland_vm::{call_fetch, GasReport, Instance, VmResult};

use crate::backend::Api;
use crate::config::Config;
use crate::errors::StoreError;
use crate::instance::{instance_with_options, ApiInstanceOptions};

#[derive(Clone)]
pub struct Store {
    config: Config,
    http_client: Client,
    fn_cache: stretto::AsyncCache<String, Instance<Api<TablelandClient>>>,
}

impl Store {
    pub fn new(config: Config) -> Self {
        Store {
            config,
            http_client: Client::builder()
                .timeout(std::time::Duration::new(5, 0))
                .build()
                .unwrap(),
            fn_cache: stretto::AsyncCache::new(12960, 1e6 as i64, tokio::spawn).unwrap(),
        }
    }

    pub async fn add(&self, cid: String) -> Result<bool, StoreError> {
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

        self.save(cid.clone(), module).await
    }

    pub async fn run(
        &self,
        cid: String,
        req: Request,
    ) -> (Result<Response, StoreError>, GasReport) {
        let value = match self.fn_cache.get_mut(cid.as_str()) {
            Some(v) => v,
            None => {
                if let Err(e) = self.load(cid.clone()).await {
                    return (Err(e), GasReport::default());
                };
                match self.fn_cache.get_mut(cid.as_str()) {
                    Some(v) => v,
                    None => {
                        return (
                            Err(StoreError::cache_err("failed to get runtime")),
                            GasReport::default(),
                        );
                    }
                }
            }
        };
        let mut instance = value.clone_inner();

        let vmr = match tokio::task::spawn_blocking(
            move || -> (VmResult<FuncResult<Response>>, GasReport) {
                let res = call_fetch(&mut instance, &req);
                let report = instance.create_gas_report();
                (res, report)
            },
        )
        .await
        {
            Ok(v) => v,
            Err(e) => return (Err(StoreError::from(e)), GasReport::default()),
        };
        match vmr.0 {
            Ok(r) => match r {
                FuncResult::Ok(r) => (Ok(r), vmr.1),
                FuncResult::Err(s) => (Err(StoreError::func_err(s)), vmr.1),
            },
            Err(e) => (Err(StoreError::from(e)), vmr.1),
        }
    }

    async fn load(&self, cid: String) -> Result<bool, StoreError> {
        let file_name = format!("{}/{}.wasm", self.config.cache.directory, cid);
        let module = tokio::fs::read(&file_name).await?;

        self.save(cid, module).await
    }

    async fn save(&self, cid: String, module: Vec<u8>) -> Result<bool, StoreError> {
        let chain_id = self.config.clone().chain.id;
        let instance = tokio::task::spawn_blocking(move || -> Instance<Api<TablelandClient>> {
            instance_with_options(
                module.as_slice(),
                ApiInstanceOptions::default(),
                TablelandClient::new(chain_id),
            )
        })
        .await?;

        if self.fn_cache.insert(cid, instance, 1).await {
            self.fn_cache.wait().await.unwrap();
            Ok(true)
        } else {
            Err(StoreError::cache_err("failed to cache runtime"))
        }
    }
}
