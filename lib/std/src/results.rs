//! This module contains the messages that are sent from the contract to the VM as an execution result

mod contract_result;
mod empty;
mod response;
mod system_result;

pub use contract_result::ContractResult;
pub use empty::Empty;
pub use response::Response;
pub use system_result::SystemResult;
