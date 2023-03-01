use tableland_std::{FuncResult, Env, Response};
use wasmer::Value;

use crate::backend::BackendApi;
use crate::conversion::ref_to_u32;
use crate::errors::VmResult;
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

/// The limits in here protect the host from allocating an unreasonable amount of memory
/// and copying an unreasonable amount of data.
///
/// A JSON deserializer would want to set the limit to a much smaller value because
/// deserializing JSON is more expensive. As a consequence, any sane contract should hit
/// the deserializer limit before the read limit.
mod read_limits {
    /// A mibi (mega binary)
    const MI: usize = 1024 * 1024;
    pub const RESULT_QUERY: usize = 64 * MI;
}

/// The limits for the JSON deserialization.
///
/// Those limits are not used when the Rust JSON deserializer is bypassed by using the
/// public `call_*_raw` functions directly.
mod deserialization_limits {
    /// A kibi (kilo binary)
    const KI: usize = 1024;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 256 * KI;
}

pub fn call_fetch<A>(instance: &mut Instance<A>, env: &Env) -> VmResult<FuncResult<Response>>
where
    A: BackendApi + 'static,
{
    let env = to_vec(env)?;
    let data = call_fetch_raw(instance, &env)?;
    from_slice::<FuncResult<Response>>(&data, deserialization_limits::RESULT_QUERY)
}

pub fn call_fetch_raw<A>(instance: &mut Instance<A>, env: &[u8]) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
{
    call_raw(instance, "fetch", &[env], read_limits::RESULT_QUERY)
}

/// Calls a function with the given arguments.
/// The exported function must return exactly one result (an offset to the result Region).
pub(crate) fn call_raw<A>(
    instance: &mut Instance<A>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
{
    let mut arg_region_ptrs = Vec::<Value>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr.into());
    }
    let result = instance.call_function1(name, &arg_region_ptrs)?;
    let res_region_ptr = ref_to_u32(&result)?;
    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}
