use async_trait::async_trait;
#[cfg(feature = "blocking")]
use reqwest::blocking::Client;
#[cfg(not(feature = "blocking"))]
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tableland_client_types::ReadOptions;

use crate::chains::{get_chain, Chain, ChainID};
use crate::errors::ClientError;
use crate::Tableland;

const QUERY_PATH: &str = "/api/v1/query";

#[derive(Clone)]
pub struct TablelandClient {
    http_client: Client,
    chain: Chain,
}

#[async_trait]
impl Tableland for TablelandClient {
    fn new(chain_id: ChainID) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::new(5, 0))
            .build()
            .unwrap();
        let chain = get_chain(chain_id);
        Self { http_client, chain }
    }

    #[cfg(not(feature = "blocking"))]
    async fn read(
        &self,
        statement: &str,
        options: ReadOptions,
    ) -> Result<(Value, u64), ClientError> {
        let format = format!("{}", options.format);
        let mut params = vec![("statement", statement), ("format", format.as_str())];
        if options.extract {
            params.push(("extract", "true"));
        }
        if options.unwrap {
            params.push(("unwrap", "true"));
        }
        let res = self
            .http_client
            .get(format!("{}{}", self.chain.endpoint, QUERY_PATH))
            .query(&params)
            .send()
            .await?
            .error_for_status()?;
        let len = res.content_length().ok_or(ClientError::NoContentLength)?;
        Ok((res.json().await?, len))
    }

    #[cfg(feature = "blocking")]
    fn read(&self, statement: &str, options: ReadOptions) -> Result<(Value, u64), ClientError> {
        let format = format!("{}", options.format);
        let mut params = vec![("statement", statement), ("format", format.as_str())];
        if options.extract {
            params.push(("extract", "true"));
        }
        if options.unwrap {
            params.push(("unwrap", "true"));
        }

        let res = self
            .http_client
            .get(format!("{}{}", self.chain.endpoint, QUERY_PATH))
            .query(&params)
            .send()?
            .error_for_status()?;
        let len = res.content_length().ok_or(ClientError::NoContentLength)?;
        Ok((res.json()?, len))
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }
}
