use serde_json::{from_slice, Value};

use crate::ctx::OwnedCtx;
use crate::errors::StdResult;
use crate::traits::Api;
use crate::types::Request;

const RESPONSE: &[u8] = include_bytes!("../../testdata/response.json");

/// Creates all external requirements that can be injected for unit tests.
pub fn mock_dependencies() -> OwnedCtx<MockApi> {
    OwnedCtx {
        tableland: MockApi::default(),
    }
}

// MockPrecompiles zero pads all human addresses to make them fit the canonical_length
// it trims off zeros for the reverse operation.
// not really smart, but allows us to see a difference (and consistent length for canonical adddresses)
#[derive(Copy, Clone)]
pub struct MockApi {}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {}
    }
}

impl Api for MockApi {
    fn read(&self, _statement: &str) -> StdResult<Value> {
        Ok(from_slice(RESPONSE).unwrap())
    }

    fn debug(&self, message: &str) {
        println!("{}", message);
    }
}

pub fn mock_request() -> Request {
    Request {
        path: "/".to_string(),
        method: "GET".to_string(),
    }
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
