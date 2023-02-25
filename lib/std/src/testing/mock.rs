use crate::deps::OwnedDeps;
use crate::errors::StdResult;
use crate::traits::Api;
use crate::types::{BlockInfo, Env};

/// Creates all external requirements that can be injected for unit tests.
///
/// See also [`mock_dependencies_with_balance`] and [`mock_dependencies_with_balances`]
/// if you want to start with some initial balances.
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
    fn hello(&self, input: &str) -> StdResult<Vec<u8>> {
        Ok(Vec::from(input.to_string().as_bytes()))
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

pub fn digit_sum(input: &[u8]) -> usize {
    input.iter().fold(0, |sum, val| sum + (*val as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_works() {
        let api = MockApi::default();
        api.hello("foobar123").unwrap();
    }

    #[test]
    fn addr_canonicalize_max_input_length() {
        let api = MockApi::default();
        let human =
            String::from("some-extremely-long-address-not-supported-by-this-api-longer-than-supported------------------------");
        let err = api.hello(&human).unwrap_err();
        assert!(err
            .to_string()
            .contains("human address too long for this mock implementation (must be <= 90)"));
    }

    #[test]
    fn digit_sum_works() {
        assert_eq!(digit_sum(&[]), 0);
        assert_eq!(digit_sum(&[0]), 0);
        assert_eq!(digit_sum(&[0, 0]), 0);
        assert_eq!(digit_sum(&[0, 0, 0]), 0);

        assert_eq!(digit_sum(&[1, 0, 0]), 1);
        assert_eq!(digit_sum(&[0, 1, 0]), 1);
        assert_eq!(digit_sum(&[0, 0, 1]), 1);

        assert_eq!(digit_sum(&[1, 2, 3]), 6);

        assert_eq!(digit_sum(&[255, 1]), 256);
    }
}
