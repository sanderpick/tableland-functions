#![cfg(not(target_arch = "wasm32"))]

// Exposed for testing only
// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.

mod assertions;
mod mock;
mod shuffle;

pub use assertions::assert_approx_eq_impl;
pub use mock::{mock_dependencies, mock_env, MockApi};
pub use shuffle::riffle_shuffle;
