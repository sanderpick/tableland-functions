use serde_json::{from_slice, Value};
use std::any::type_name;

use crate::errors::{StdError, StdResult};
use crate::memory::{build_region, consume_region, Region};
use crate::traits::Api;

// This interface will compile into required Wasm imports.
extern "C" {
    /// Performs a Tableland read query.
    fn read(source_ptr: u32) -> u32;

    /// Writes a debug message (UFT-8 encoded) to the host for debugging purposes.
    /// The host is free to log or process this in any way it considers appropriate.
    /// In production environments it is expected that those messages are discarded.
    fn debug(source_ptr: u32);

    #[cfg(feature = "abort")]
    fn abort(source_ptr: u32);
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
    fn read(&self, statement: &str) -> StdResult<Value> {
        let req = build_region(statement.as_bytes());
        let request_ptr = &*req as *const Region as u32;

        let response_ptr = unsafe { read(request_ptr) };
        let response = unsafe { consume_region(response_ptr as *mut Region) };

        let data = from_slice(response.as_slice())
            .map_err(|e| StdError::parse_err(type_name::<Value>(), e))?;
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
#[allow(dead_code)]
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
