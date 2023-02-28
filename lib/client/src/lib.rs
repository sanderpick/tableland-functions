pub mod chains;
#[cfg(feature = "blocking")]
use reqwest::blocking::Client;
#[cfg(not(feature = "blocking"))]
use reqwest::Client;
use reqwest::Error;
use serde_json::Value as JsonValue;
use std::time::Duration;
use tableland_client_types::*;

const QUERY_PATH: &str = "/api/v1/query";

#[derive(Clone)]
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

    #[cfg(not(feature = "blocking"))]
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

    #[cfg(feature = "blocking")]
    pub fn read(&self, query: &str, opts: ReadOptions) -> Result<JsonValue, Error> {
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
            .send()?
            .error_for_status()?
            .json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "blocking"))]
    fn new_client_works() {
        let client = TablelandClient::new(chains::ChainID::Ethereum);
        assert_eq!(client.chain.id, 1);
    }

    #[tokio::test]
    #[cfg(not(feature = "blocking"))]
    async fn read_works() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client
            .read("select * from politicians_31337_6;", ReadOptions::default())
            .await;
        assert_eq!(result.is_ok(), true);
    }

    #[tokio::test]
    #[cfg(not(feature = "blocking"))]
    async fn read_fails_on_server_error() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client.read("bad query;", ReadOptions::default()).await;
        assert_eq!(result.is_err(), true);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn new_client_works() {
        let client = TablelandClient::new(chains::ChainID::Ethereum);
        assert_eq!(client.chain.id, 1);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn read_works() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client.read("select * from politicians_31337_6;", ReadOptions::default());
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn read_fails_on_server_error() {
        let client = TablelandClient::new(chains::ChainID::Local);
        let result = client.read("bad query;", ReadOptions::default());
        assert_eq!(result.is_err(), true);
    }
}
