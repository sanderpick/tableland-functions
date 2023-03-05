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
            .get(format!("{}{}", self.chain.endpoint.to_string(), QUERY_PATH))
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
            .get(format!("{}{}", self.chain.endpoint.to_string(), QUERY_PATH))
            .query(&params)
            .send()?
            .error_for_status()?;
        let len = res.content_length().ok_or(ClientError::NoContentLength)?;
        Ok((res.json()?, len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "blocking"))]
    fn new_client_works() {
        let client = TablelandClient::new(ChainID::Ethereum);
        assert_eq!(client.chain.id, 1);
    }

    #[tokio::test]
    #[cfg(not(feature = "blocking"))]
    async fn read_works() {
        let client = TablelandClient::new(ChainID::Local);
        let result = client
            .read("select * from politicians_31337_6;", ReadOptions::default())
            .await;
        assert_eq!(result.is_ok(), true);
    }

    #[tokio::test]
    #[cfg(not(feature = "blocking"))]
    async fn read_fails_on_server_error() {
        let client = TablelandClient::new(ChainID::Local);
        let result = client.read("bad query;", ReadOptions::default()).await;
        assert_eq!(result.is_err(), true);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn new_client_works() {
        let client = TablelandClient::new(ChainID::Ethereum);
        assert_eq!(client.chain.id, 1);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn read_works() {
        let client = TablelandClient::new(ChainID::Local);
        let result = client.read("select * from politicians_31337_6;", ReadOptions::default());
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn read_fails_on_server_error() {
        let client = TablelandClient::new(ChainID::Local);
        let result = client.read("bad query;", ReadOptions::default());
        assert_eq!(result.is_err(), true);
    }
}
