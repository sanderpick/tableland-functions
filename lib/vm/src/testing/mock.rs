use serde_json::Value;
use tableland_client_types::ReadOptions;
use tableland_std::Request;

use crate::serde::from_slice;
use crate::{Backend, BackendApi, BackendError, BackendResult, GasInfo};

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
#[derive(Clone, Default)]
pub struct MockApi {
    /// Data for mock query response.
    data: Vec<u8>,
}

impl MockApi {
    pub fn new(data: Vec<u8>) -> Self {
        MockApi { data }
    }
}

impl BackendApi for MockApi {
    fn read(&self, statement: &str, _options: ReadOptions, gas_limit: u64) -> BackendResult<Value> {
        let mut gas_info = GasInfo::with_externally_used(
            GAS_COST_QUERY_FLAT + (GAS_COST_QUERY_REQUEST_MULTIPLIER * (statement.len() as u64)),
        );
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        let response = match from_slice(self.data.as_slice(), 1024) {
            Ok(b) => b,
            Err(e) => return (Err(BackendError::UserErr { msg: e.to_string() }), gas_info),
        };

        gas_info.externally_used += GAS_COST_QUERY_RESPONSE_MULTIPLIER * (self.data.len() as u64);
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        (Ok(response), gas_info)
    }
}

pub fn mock_get_request(path: &'static str) -> Request {
    Request::new(
        "mock_id".to_string(),
        http::uri::Uri::from_static(path),
        http::method::Method::GET,
        http::header::HeaderMap::default(),
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;

    #[test]
    fn read_works() {
        let api = MockApi::new(b"[]".to_vec());

        api.read(
            "select * from my_table;",
            ReadOptions::default(),
            DEFAULT_QUERY_GAS_LIMIT,
        )
        .0
        .unwrap();
    }
}
