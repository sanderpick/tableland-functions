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
}
