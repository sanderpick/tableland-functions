use serde_json::{from_slice, Value};
use tableland_std::Request;

use crate::{Backend, BackendApi, BackendError, BackendResult, GasInfo};

const RESPONSE: &[u8] = include_bytes!("../../testdata/response.json");

const GAS_COST_QUERY_FLAT: u64 = 100_000;
/// Gas per request byte
const GAS_COST_QUERY_REQUEST_MULTIPLIER: u64 = 0;
/// Gas per reponse byte
const GAS_COST_QUERY_RESPONSE_MULTIPLIER: u64 = 100;

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_backend() -> Backend<MockApi> {
    Backend {
        api: MockApi::default(),
    }
}

/// Zero-pads all human addresses to make them fit the canonical_length and
/// trims off zeros for the reverse operation.
/// This is not really smart, but allows us to see a difference (and consistent length for canonical adddresses).
#[derive(Copy, Clone)]
pub struct MockApi {
    /// When set, all calls to the API fail with BackendError::Unknown containing this message
    #[allow(dead_code)]
    backend_error: Option<&'static str>,
}

impl MockApi {
    pub fn new_failing(backend_error: &'static str) -> Self {
        MockApi {
            backend_error: Some(backend_error),
            ..MockApi::default()
        }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            backend_error: None,
        }
    }
}

impl BackendApi for MockApi {
    fn read(&self, statement: &str, gas_limit: u64) -> BackendResult<Value> {
        let mut gas_info = GasInfo::with_externally_used(
            GAS_COST_QUERY_FLAT + (GAS_COST_QUERY_REQUEST_MULTIPLIER * (statement.len() as u64)),
        );
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        let response = match from_slice(RESPONSE) {
            Ok(b) => b,
            Err(e) => return (Err(BackendError::UserErr { msg: e.to_string() }), gas_info),
        };

        gas_info.externally_used += GAS_COST_QUERY_RESPONSE_MULTIPLIER * (RESPONSE.len() as u64);
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        (Ok(response), gas_info)
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
    use crate::BackendError;

    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;

    #[test]
    fn read_works() {
        let api = MockApi::default();

        api.read("select * from my_table", DEFAULT_QUERY_GAS_LIMIT)
            .0
            .unwrap();
    }
}
