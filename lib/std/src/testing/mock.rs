use serde_json::{from_slice, Value};
use tableland_client_types::ReadOptions;

use crate::ctx::OwnedCtx;
use crate::http::{Request, Result};
use crate::traits::Api;

/// Creates all external requirements that can be injected for unit tests.
pub fn mock_dependencies(data: Vec<u8>) -> OwnedCtx<MockApi> {
    OwnedCtx {
        tableland: MockApi::new(data),
    }
}

#[derive(Clone)]
pub struct MockApi {
    data: Vec<u8>,
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi { data: Vec::new() }
    }
}

impl MockApi {
    fn new(data: Vec<u8>) -> Self {
        MockApi { data }
    }
}

impl Api for MockApi {
    fn read(&self, _statement: &str, _options: ReadOptions) -> Result<Value> {
        Ok(from_slice(self.data.as_slice()).unwrap())
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
    }
}

pub fn mock_get_request(path: &'static str) -> Request {
    Request::new(
        http::uri::Uri::from_static(path),
        http::method::Method::GET,
        http::header::HeaderMap::default(),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_works() {
        let api = MockApi::new(b"[]".to_vec());
        let json = api
            .read("select * from my_table;", ReadOptions::default())
            .unwrap();
        println!("{}", serde_json::to_string(&json).unwrap());
    }
}
