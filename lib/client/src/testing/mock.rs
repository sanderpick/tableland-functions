use async_trait::async_trait;
use serde_json::Value;
use tableland_client_types::ReadOptions;

use crate::chains::{get_chain, Chain, ChainID};
use crate::errors::ClientError;
use crate::Tableland;

#[derive(Clone)]
pub struct MockClient {
    #[allow(dead_code)]
    chain: Chain,
    data: Vec<u8>,
}

impl MockClient {
    pub fn respond_with(&mut self, data: Vec<u8>) {
        self.data = data;
    }
}

#[async_trait]
impl Tableland for MockClient {
    fn new(chain_id: ChainID) -> Self {
        let chain = get_chain(chain_id);
        Self {
            chain,
            data: Vec::new(),
        }
    }

    #[cfg(not(feature = "blocking"))]
    async fn read(
        &self,
        _statement: &str,
        _options: ReadOptions,
    ) -> Result<(Value, u64), ClientError> {
        if self.data.is_empty() {
            panic!("Set data with 'MockClient::respond_with'")
        }

        let res = serde_json::from_slice(self.data.as_slice()).unwrap();
        Ok((res, self.data.len() as u64))
    }

    #[cfg(feature = "blocking")]
    fn read(&self, _statement: &str, _options: ReadOptions) -> Result<(Value, u64), ClientError> {
        if self.data.is_empty() {
            panic!("Set data with 'MockClient::respond_with'")
        }

        let res = serde_json::from_slice(self.data.as_slice()).unwrap();
        Ok((res, self.data.len() as u64))
    }

    fn chain(&self) -> Chain {
        self.chain.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "blocking"))]
    fn new_client_works() {
        let client = MockClient::new(ChainID::Ethereum);
        assert_eq!(client.chain().id, 1);
    }

    #[tokio::test]
    #[cfg(not(feature = "blocking"))]
    async fn read_works() {
        let mut client = MockClient::new(ChainID::Local);
        client.respond_with(b"[{}]".to_vec());
        let result = client
            .read("select * from my_table;", ReadOptions::default())
            .await;
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn new_client_works() {
        let client = MockClient::new(ChainID::Ethereum);
        assert_eq!(client.chain().id, 1);
    }

    #[test]
    #[cfg(feature = "blocking")]
    fn read_works() {
        let mut client = MockClient::new(ChainID::Local);
        client.respond_with(b"[{}]".to_vec());
        let result = client.read("select * from my_table;", ReadOptions::default());
        assert_eq!(result.is_ok(), true);
    }
}
