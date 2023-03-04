#![cfg(not(target_arch = "wasm32"))]

// Exposed for testing only
// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.

mod mock;

pub use mock::{mock_dependencies, mock_get_request, MockApi};
