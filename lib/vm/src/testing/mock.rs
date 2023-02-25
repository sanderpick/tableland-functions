use tableland_std::{BlockInfo, Env};

use crate::{Backend, BackendApi, BackendResult, GasInfo};

const GAS_COST_HELLO: u64 = 55;

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
    fn hello(&self, input: &str) -> BackendResult<Vec<u8>> {
        let gas_info = GasInfo::with_cost(GAS_COST_HELLO);
        let out = Vec::from(input.to_string().as_bytes());
        (Ok(out), gas_info)
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
    use crate::BackendError;

    #[test]
    fn hello_works() {
        let api = MockApi::default();

        api.hello("foobar123").0.unwrap();
    }

    #[test]
    fn hello_max_input_length() {
        let api = MockApi::default();
        let human = "longer-than-the-address-length-supported-by-this-api-longer-than-54";
        match api.hello(human).0.unwrap_err() {
            BackendError::UserErr { msg } => assert!(msg.contains("too long")),
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
