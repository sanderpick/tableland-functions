use serde_json::{from_slice, Value};
use tableland_client_types::ReadOptions;

use crate::ctx::OwnedCtx;
use crate::http::{Request, Result};
use crate::traits::Api;

const RESPONSE: &[u8] = include_bytes!("../../testdata/response.json");

/// Creates all external requirements that can be injected for unit tests.
pub fn mock_dependencies() -> OwnedCtx<MockApi> {
    OwnedCtx {
        tableland: MockApi::default(),
    }
}

#[derive(Copy, Clone)]
pub struct MockApi {}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {}
    }
}

impl Api for MockApi {
    fn read(&self, _statement: &str, _options: ReadOptions) -> Result<Value> {
        Ok(from_slice(RESPONSE).unwrap())
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
        let api = MockApi::default();
        let json = api.read("select * from my_table;").unwrap();
        println!("{}", serde_json::to_string(&json).unwrap());
    }
}
