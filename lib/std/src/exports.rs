//! exports exposes the public wasm API
//!
//! interface_version_8, allocate and deallocate turn into Wasm exports
//! as soon as tableland_std is `use`d in the function, even privately.
//!
//! `do_fetch` should be wrapped with a extern "C" entry point including
//! the contract-specific function pointer. This is done via the `#[entry_point]`
//! macro attribute from tableland-derive.
use serde_json::{from_slice, to_vec};
use std::vec::Vec;

use crate::ctx::OwnedCtx;
use crate::http::{Request, Response};
use crate::imports::ExternalApi;
use crate::memory::{alloc, consume_region, release_buffer, Region};
#[cfg(feature = "abort")]
use crate::panic::install_panic_handler;
use crate::results::FuncResult;
use crate::CtxMut;

/// interface_version_* exports mark which Wasm VM interface level this contract is compiled for.
/// They can be checked by tableland_vm.
/// Update this whenever the Wasm VM interface breaks.
#[no_mangle]
extern "C" fn interface_version_8() -> () {}

/// allocate reserves the given number of bytes in wasm memory and returns a pointer
/// to a Region defining this data. This space is managed by the calling process
/// and should be accompanied by a corresponding deallocate
#[no_mangle]
extern "C" fn allocate(size: usize) -> u32 {
    alloc(size) as u32
}

/// deallocate expects a pointer to a Region created with allocate.
/// It will free both the Region and the memory referenced by the Region.
#[no_mangle]
extern "C" fn deallocate(pointer: u32) {
    // auto-drop Region on function end
    let _ = unsafe { consume_region(pointer as *mut Region) };
}

// TODO: replace with https://doc.rust-lang.org/std/ops/trait.Try.html once stabilized
macro_rules! r#try_into_func_result {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                return FuncResult::Err(err.to_string());
            }
        }
    };
    ($expr:expr,) => {
        $crate::try_into_func_result!($expr)
    };
}

/// This should be wrapped in an external "C" export, containing a contract-specific function as an argument.
///
/// - `E`: error type for responses
pub fn do_fetch<E>(fetch_fn: &dyn Fn(Request, CtxMut) -> Result<Response, E>, req_ptr: u32) -> u32
where
    E: ToString,
{
    #[cfg(feature = "abort")]
    install_panic_handler();
    let res = _do_fetch::<E>(fetch_fn, req_ptr as *mut Region);
    let v = to_vec(&res).unwrap();
    release_buffer(v) as u32
}

fn _do_fetch<E>(
    fetch_fn: &dyn Fn(Request, CtxMut) -> Result<Response, E>,
    req_ptr: *mut Region,
) -> FuncResult<Response>
where
    E: ToString,
{
    let req: Vec<u8> = unsafe { consume_region(req_ptr) };
    let req: Request = try_into_func_result!(from_slice(&req));

    let mut ctx = make_ctx();
    fetch_fn(req, ctx.as_mut()).into()
}

/// Makes all bridges to external dependencies (i.e. Wasm imports) that are injected by the VM
pub(crate) fn make_ctx() -> OwnedCtx<ExternalApi> {
    OwnedCtx {
        tableland: ExternalApi::new(),
    }
}
