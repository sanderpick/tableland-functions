//! This module contains the messages that are sent from the contract to the VM as an execution result

mod func_result;
mod response;

pub use func_result::FuncResult;
pub use response::Response;
