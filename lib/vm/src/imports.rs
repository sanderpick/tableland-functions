//! Import implementations
use crate::backend::{BackendApi, BackendError};
use crate::conversion::{ref_to_u32, to_u32};
use crate::environment::{process_gas_info, Environment};
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::memory::{read_region, write_region};
#[allow(unused_imports)]
use crate::sections::encode_sections;

/// A kibi (kilo binary)
const KI: usize = 1024;
/// A mibi (mega binary)
const MI: usize = 1024 * 1024;

const MAX_LENGTH_QUERY_REQUEST: usize = 64 * KI;

/// Max length for a debug message
const MAX_LENGTH_DEBUG: usize = 2 * MI;

/// Max length for an abort message
const MAX_LENGTH_ABORT: usize = 2 * MI;

// Import implementations
//
// This block of do_* prefixed functions is tailored for Wasmer's
// Function::new_native_with_env interface. Those require an env in the first
// argument and cannot capture other variables. Thus everything is accessed
// through the env.

pub fn do_hello<A: BackendApi>(
    env: &Environment<A>,
    source_ptr: u32,
    destination_ptr: u32,
) -> VmResult<u32> {
    let source_data = read_region(&env.memory(), source_ptr, MAX_LENGTH_QUERY_REQUEST)?;
    if source_data.is_empty() {
        return write_to_contract::<A>(env, b"Input is empty");
    }

    let source_string = match String::from_utf8(source_data) {
        Ok(s) => s,
        Err(_) => return write_to_contract::<A>(env, b"Input is not valid UTF-8"),
    };

    let (result, gas_info) = env.api.hello(&source_string);
    process_gas_info::<A>(env, gas_info)?;
    match result {
        Ok(data) => {
            write_region(&env.memory(), destination_ptr, data.as_slice())?;
            Ok(0)
        }
        Err(BackendError::UserErr { msg, .. }) => Ok(write_to_contract::<A>(env, msg.as_bytes())?),
        Err(err) => Err(VmError::from(err)),
    }
}

/// Prints a debug message to console.
/// This does not charge gas, so debug printing should be disabled when used in a blockchain module.
pub fn do_debug<A: BackendApi>(env: &Environment<A>, message_ptr: u32) -> VmResult<()> {
    if env.print_debug {
        let message_data = read_region(&env.memory(), message_ptr, MAX_LENGTH_DEBUG)?;
        let msg = String::from_utf8_lossy(&message_data);
        println!("{}", msg);
    }
    Ok(())
}

/// Aborts the contract and shows the given error message
pub fn do_abort<A: BackendApi>(env: &Environment<A>, message_ptr: u32) -> VmResult<()> {
    let message_data = read_region(&env.memory(), message_ptr, MAX_LENGTH_ABORT)?;
    let msg = String::from_utf8_lossy(&message_data);
    Err(VmError::aborted(msg))
}

/// Creates a Region in the contract, writes the given data to it and returns the memory location
fn write_to_contract<A: BackendApi>(env: &Environment<A>, input: &[u8]) -> VmResult<u32> {
    let out_size = to_u32(input.len())?;
    let result = env.call_function1("allocate", &[out_size.into()])?;
    let target_ptr = ref_to_u32(&result)?;
    if target_ptr == 0 {
        return Err(CommunicationError::zero_address().into());
    }
    write_region(&env.memory(), target_ptr, input)?;
    Ok(target_ptr)
}
