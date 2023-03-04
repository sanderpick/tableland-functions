use serde_json::{from_slice, to_vec, Value};
use tableland_client_types::ReadOptions;

use crate::http::Result;
use crate::memory::{build_region, consume_region, Region};
use crate::tableland::ReadRequest;
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
    fn read(&self, statement: &str, options: ReadOptions) -> Result<Value> {
        let req = ReadRequest {
            stm: statement.to_string(),
            opts: options,
        };
        let data = to_vec(&req).unwrap();
        let src = build_region(&data);
        let src_ptr = &*src as *const Region as u32;

        let res_ptr = unsafe { read(src_ptr) };
        let res = unsafe { consume_region(res_ptr as *mut Region) };

        let data = from_slice(&res)?;
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
