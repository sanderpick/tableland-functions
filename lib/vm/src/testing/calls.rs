//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
// use schemars::JsonSchema;
// use serde::{de::DeserializeOwned, Serialize};

use tableland_std::{ContractResult, Env, Response};

use crate::calls::call_fetch;
use crate::instance::Instance;
// use crate::serde::to_vec;
use crate::BackendApi;

/// Mimicks the call signature of the smart contracts.
/// Thus it moves env and msg rather than take them as reference.
/// This is inefficient here, but only used in test code.
pub fn fetch<A>(
    instance: &mut Instance<A>,
    env: Env,
    // info: MessageInfo,
    // msg: M,
) -> ContractResult<Response>
where
    A: BackendApi + 'static,
    // S: Storage + 'static,
    // Q: Querier + 'static,
    // M: Serialize + JsonSchema,
    // U: DeserializeOwned + CustomMsg,
{
    // let serialized_msg = to_vec(&msg).expect("Testing error: Could not seralize request message");
    call_fetch(instance, &env).expect("VM error")
}
