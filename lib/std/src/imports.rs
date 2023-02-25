use std::vec::Vec;

use crate::errors::{StdError, StdResult};
use crate::memory::{alloc, build_region, consume_region, Region};
use crate::traits::Api;

const QUERY_RESULT_BUFFER_LENGTH: usize = 100_000;

// This interface will compile into required Wasm imports.
// A complete documentation those functions is available in the VM that provides them:
// https://github.com/CosmWasm/cosmwasm/blob/v1.0.0-beta/packages/vm/src/instance.rs#L89-L206
extern "C" {
    #[cfg(feature = "abort")]
    fn abort(source_ptr: u32);

    fn hello(source_ptr: u32, destination_ptr: u32) -> u32;

    /// Writes a debug message (UFT-8 encoded) to the host for debugging purposes.
    /// The host is free to log or process this in any way it considers appropriate.
    /// In production environments it is expected that those messages are discarded.
    fn debug(source_ptr: u32);
}

/// A stateless convenience wrapper around imports provided by the VM
#[derive(Copy, Clone)]
pub struct ExternalApi {}

impl ExternalApi {
    pub fn new() -> ExternalApi {
        ExternalApi {}
    }
}

impl Api for ExternalApi {
    fn hello(&self, input: &str) -> StdResult<Vec<u8>> {
        let send = build_region(input.as_bytes());
        let send_ptr = &*send as *const Region as u32;
        let out = alloc(QUERY_RESULT_BUFFER_LENGTH);

        let result = unsafe { hello(send_ptr, out as u32) };
        if result != 0 {
            let error = unsafe { consume_string_region_written_by_vm(result as *mut Region) };
            return Err(StdError::generic_err(format!("hello errored: {}", error)));
        }

        let data = unsafe { consume_region(out) };
        Ok(data)
    }

    fn debug(&self, message: &str) {
        // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
        let region = build_region(message.as_bytes());
        let region_ptr = region.as_ref() as *const Region as u32;
        unsafe { debug(region_ptr) };
    }
}

/// Takes a pointer to a Region and reads the data into a String.
/// This is for trusted string sources only.
unsafe fn consume_string_region_written_by_vm(from: *mut Region) -> String {
    let data = consume_region(from);
    // We trust the VM/chain to return correct UTF-8, so let's save some gas
    String::from_utf8_unchecked(data)
}

#[cfg(feature = "abort")]
pub fn handle_panic(message: &str) {
    // keep the boxes in scope, so we free it at the end (don't cast to pointers same line as build_region)
    let region = build_region(message.as_bytes());
    let region_ptr = region.as_ref() as *const Region as u32;
    unsafe { abort(region_ptr) };
}
