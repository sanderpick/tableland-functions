// The external interface is `use tableland_vm::testing::X` for all integration testing symbols, no matter where they live internally.

mod calls;
mod instance;
mod mock;

pub use calls::fetch;
pub use instance::{
    mock_instance, mock_instance_options, mock_instance_with_failing_api,
    mock_instance_with_gas_limit, mock_instance_with_options, test_io, MockInstanceOptions,
};
pub use mock::{mock_backend, mock_get_request, MockApi};
