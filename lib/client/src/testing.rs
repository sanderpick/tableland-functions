// The external interface is `use tableland_client::testing::X` for all integration testing symbols, no matter where they live internally.

mod mock;

pub use mock::MockClient;
