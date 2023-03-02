#![feature(error_generic_member_access)]
#![feature(provide_any)]
#![cfg_attr(feature = "backtraces", feature(backtrace))]

// Exposed on all platforms

mod ctx;
mod http;
mod panic;
mod results;
mod traits;

pub use crate::ctx::{Ctx, CtxMut, OwnedCtx};
pub use crate::http::{Error, Request, Response, Result, Router};
pub use crate::results::FuncResult;
pub use crate::traits::Api;

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
