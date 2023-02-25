pub mod chains;
use reqwest::{Client, Error};
use serde_json::Value as JsonValue;
use std::time::Duration;
use tableland_client_types::*;

const QUERY_PATH: &str = "/api/v1/query";

pub struct TablelandClient {
    http_client: Client,
    chain: chains::Chain,
}

impl TablelandClient {
    pub fn new(chain_id: chains::ChainID) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::new(30, 0))
            .build()
            .unwrap();
        let chain = chains::get_chain(chain_id);
        Self { http_client, chain }
    }

    pub async fn read(&self, query: &str, opts: ReadOptions) -> Result<JsonValue, Error> {
        let format = format!("{}", opts.format);
        let mut params = vec![("statement", query), ("format", format.as_str())];
        if opts.extract {
            params.push(("extract", "true"));
        }
        if opts.unwrap {
            params.push(("unwrap", "true"));
        }
        self.http_client
            .get(format!("{}{}", self.chain.endpoint.to_string(), QUERY_PATH))
            .query(&params)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_client_works() {
        let client = TablelandClient::new(chains::ChainID::Ethereum);
        assert_eq!(client.chain.id, 1);
    }

    #[tokio::test]
    async fn read_works() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client
            .read("select * from politicians_31337_7;", ReadOptions::default())
            .await;
        assert_eq!(result.is_ok(), true);
    }

    #[tokio::test]
    async fn read_fails_on_server_error() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client.read("bad query;", ReadOptions::default()).await;
        assert_eq!(result.is_err(), true);
    }
}
