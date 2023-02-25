#![feature(error_generic_member_access)]
#![feature(provide_any)]
#![cfg_attr(feature = "backtraces", feature(backtrace))]

// Exposed on all platforms

mod assertions;
mod binary;
mod conversion;
mod deps;
mod errors;
mod hex_binary;
mod import_helpers;
mod math;
mod never;
mod panic;
mod results;
mod sections;
mod serde;
mod traits;
mod types;

pub use crate::binary::Binary;
pub use crate::deps::{Deps, DepsMut, OwnedDeps};
pub use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyRatioError, ConversionOverflowError, DivideByZeroError,
    OverflowError, OverflowOperation, StdError, StdResult, SystemError,
};
pub use crate::hex_binary::HexBinary;
pub use crate::math::{
    Decimal, Decimal256, Decimal256RangeExceeded, DecimalRangeExceeded, Fraction, Isqrt, Uint128,
    Uint256, Uint512, Uint64,
};
pub use crate::never::Never;
pub use crate::results::{ContractResult, Empty, Response, SystemResult};
pub use crate::serde::{from_binary, from_slice, to_binary, to_vec};
pub use crate::traits::Api;
pub use crate::types::{BlockInfo, Env};

// Exposed in wasm build only

#[cfg(target_arch = "wasm32")]
mod exports;
#[cfg(target_arch = "wasm32")]
mod imports;
#[cfg(target_arch = "wasm32")]
mod memory; // Used by exports and imports only. This assumes pointers are 32 bit long, which makes it untestable on dev machines.

#[cfg(target_arch = "wasm32")]
pub use crate::exports::do_fetch;
#[cfg(target_arch = "wasm32")]
pub use crate::imports::ExternalApi;

// Exposed for testing only
// Both unit tests and integration tests are compiled to native code, so everything in here does not need to compile to Wasm.
#[cfg(not(target_arch = "wasm32"))]
pub mod testing;

// Re-exports

pub use tableland_derive::entry_point;
