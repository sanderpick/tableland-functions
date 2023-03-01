use serde_json::{from_slice, Value};

use crate::deps::OwnedDeps;
use crate::errors::StdResult;
use crate::traits::Api;
use crate::types::{BlockInfo, Env};

const RESPONSE: &[u8] = include_bytes!("../../testdata/response.json");

/// Creates all external requirements that can be injected for unit tests.
pub fn mock_dependencies() -> OwnedDeps<MockApi> {
    OwnedDeps {
        api: MockApi::default(),
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

/// Returns a default enviroment with height, time, chain_id, and contract address
/// You can submit as is to most contracts, or modify height/time if you want to
/// test for expiration.
///
/// This is intended for use in test code only.
pub fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            chain_id: "yoyo".to_string(),
        },
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
