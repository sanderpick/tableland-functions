//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use cosmwasm_vm::testing::X
use tableland_std::{ContractResult, Env, Response};

use crate::calls::call_fetch;
use crate::instance::Instance;
use crate::BackendApi;

/// Mimicks the call signature of the smart contracts.
/// Thus it moves env and msg rather than take them as reference.
/// This is inefficient here, but only used in test code.
pub fn fetch<A>(instance: &mut Instance<A>, env: Env) -> ContractResult<Response>
where
    A: BackendApi + 'static,
{
    call_fetch(instance, &env).expect("VM error")
}
