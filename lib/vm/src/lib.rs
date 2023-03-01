#![feature(error_generic_member_access)]
#![feature(provide_any)]
#![cfg_attr(feature = "backtraces", feature(backtrace))]

mod backend;
mod calls;
mod capabilities;
mod checksum;
mod compatibility;
mod conversion;
mod environment;
mod errors;
mod imports;
mod instance;
mod limited;
mod memory;
mod modules;
mod serde;
mod size;
mod static_analysis;
pub mod testing;
mod wasm_backend;

pub use crate::backend::{Backend, BackendApi, BackendError, BackendResult, GasInfo};
pub use crate::calls::{call_fetch, call_fetch_raw};
pub use crate::capabilities::capabilities_from_csv;
pub use crate::checksum::Checksum;
pub use crate::compatibility::check_wasm;
pub use crate::errors::{
    CommunicationError, CommunicationResult, RegionValidationError, RegionValidationResult,
    VmError, VmResult,
};
pub use crate::instance::{GasReport, Instance, InstanceOptions};
pub use crate::serde::{from_slice, to_vec};
pub use crate::size::Size;

#[doc(hidden)]
pub mod internals {
    //! We use the internals module for exporting types that are only
    //! intended to be used in internal crates / utils.
    //! Please don't use any of these types directly, as
    //! they might change frequently or be removed in the future.

    pub use crate::compatibility::check_wasm;
    pub use crate::instance::instance_from_module;
    pub use crate::wasm_backend::{compile, make_runtime_store};
}
