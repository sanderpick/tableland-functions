//! Internal details to be used by instance.rs only
use std::borrow::{Borrow, BorrowMut};
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use wasmer::{HostEnvInitError, Instance as WasmerInstance, Memory, Val, WasmerEnv};
use wasmer_middlewares::metering::{get_remaining_points, set_remaining_points, MeteringPoints};

use crate::backend::{BackendApi, GasInfo};
use crate::errors::{VmError, VmResult};

/// Never can never be instantiated.
/// Replace this with the [never primitive type](https://doc.rust-lang.org/std/primitive.never.html) when stable.
#[derive(Debug)]
pub enum Never {}

/** gas config data */

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GasConfig {
    /// Gas costs of VM (not Backend) provided functionality
    /// secp256k1 signature verification cost
    pub secp256k1_verify_cost: u64,
    /// secp256k1 public key recovery cost
    pub secp256k1_recover_pubkey_cost: u64,
    /// ed25519 signature verification cost
    pub ed25519_verify_cost: u64,
    /// ed25519 batch signature verification cost
    pub ed25519_batch_verify_cost: u64,
    /// ed25519 batch signature verification cost (single public key)
    pub ed25519_batch_verify_one_pubkey_cost: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        // Target is 10^12 per millisecond (see GAS.md), i.e. 10^9 gas per µ second.
        const GAS_PER_US: u64 = 1_000_000_000;
        Self {
            // ~154 us in crypto benchmarks
            secp256k1_verify_cost: 154 * GAS_PER_US,
            // ~162 us in crypto benchmarks
            secp256k1_recover_pubkey_cost: 162 * GAS_PER_US,
            // ~63 us in crypto benchmarks
            ed25519_verify_cost: 63 * GAS_PER_US,
            // Gas cost factors, relative to ed25519_verify cost
            // From https://docs.rs/ed25519-zebra/2.2.0/ed25519_zebra/batch/index.html
            ed25519_batch_verify_cost: 63 * GAS_PER_US / 2,
            ed25519_batch_verify_one_pubkey_cost: 63 * GAS_PER_US / 4,
        }
    }
}

/** context data **/

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation, including internally and externally used gas.
    /// This is set when the Environment is created and never mutated.
    ///
    /// Measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    /// Tracking the gas used in the Cosmos SDK, in CosmWasm gas units.
    pub externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }
}

/// A environment that provides access to the ContextData.
/// The environment is clonable but clones access the same underlying data.
pub struct Environment<A: BackendApi> {
    pub api: A,
    pub print_debug: bool,
    pub gas_config: GasConfig,
    data: Arc<RwLock<ContextData>>,
}

unsafe impl<A: BackendApi> Send for Environment<A> {}

unsafe impl<A: BackendApi> Sync for Environment<A> {}

impl<A: BackendApi> Clone for Environment<A> {
    fn clone(&self) -> Self {
        Environment {
            api: self.api.clone(),
            print_debug: self.print_debug,
            gas_config: self.gas_config.clone(),
            data: self.data.clone(),
        }
    }
}

impl<A: BackendApi> WasmerEnv for Environment<A> {
    fn init_with_instance(&mut self, _instance: &WasmerInstance) -> Result<(), HostEnvInitError> {
        Ok(())
    }
}

impl<A: BackendApi> Environment<A> {
    pub fn new(api: A, gas_limit: u64, print_debug: bool) -> Self {
        Environment {
            api,
            print_debug,
            gas_config: GasConfig::default(),
            data: Arc::new(RwLock::new(ContextData::new(gas_limit))),
        }
    }

    fn with_context_data_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut ContextData) -> R,
    {
        let mut guard = self.data.as_ref().write().unwrap();
        let context_data = guard.borrow_mut();
        callback(context_data)
    }

    fn with_context_data<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&ContextData) -> R,
    {
        let guard = self.data.as_ref().read().unwrap();
        let context_data = guard.borrow();
        callback(context_data)
    }

    pub fn with_gas_state<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&GasState) -> R,
    {
        self.with_context_data(|context_data| callback(&context_data.gas_state))
    }

    pub fn with_gas_state_mut<C, R>(&self, callback: C) -> R
    where
        C: FnOnce(&mut GasState) -> R,
    {
        self.with_context_data_mut(|context_data| callback(&mut context_data.gas_state))
    }

    pub fn with_wasmer_instance<C, R>(&self, callback: C) -> VmResult<R>
    where
        C: FnOnce(&WasmerInstance) -> VmResult<R>,
    {
        self.with_context_data(|context_data| match context_data.wasmer_instance {
            Some(instance_ptr) => {
                let instance_ref = unsafe { instance_ptr.as_ref() };
                callback(instance_ref)
            }
            None => Err(VmError::uninitialized_context_data("wasmer_instance")),
        })
    }

    /// Calls a function with the given name and arguments.
    /// The number of return values is variable and controlled by the guest.
    /// Usually we expect 0 or 1 return values. Use [`Self::call_function0`]
    /// or [`Self::call_function1`] to ensure the number of return values is checked.
    fn call_function(&self, name: &str, args: &[Val]) -> VmResult<Box<[Val]>> {
        // Clone function before calling it to avoid dead locks
        let func = self.with_wasmer_instance(|instance| {
            let func = instance.exports.get_function(name)?;
            Ok(func.clone())
        })?;
        func.call(args).map_err(|runtime_err| -> VmError {
            self.with_wasmer_instance::<_, Never>(|instance| {
                let err: VmError = match get_remaining_points(instance) {
                    MeteringPoints::Remaining(_) => VmError::from(runtime_err),
                    MeteringPoints::Exhausted => VmError::gas_depletion(),
                };
                Err(err)
            })
            .unwrap_err() // with_wasmer_instance can only succeed if the callback succeeds
        })
    }

    pub fn call_function0(&self, name: &str, args: &[Val]) -> VmResult<()> {
        let result = self.call_function(name, args)?;
        let expected = 0;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(())
    }

    pub fn call_function1(&self, name: &str, args: &[Val]) -> VmResult<Val> {
        let result = self.call_function(name, args)?;
        let expected = 1;
        let actual = result.len();
        if actual != expected {
            return Err(VmError::result_mismatch(name, expected, actual));
        }
        Ok(result[0].clone())
    }

    /// Creates a back reference from a contact to its partent instance
    pub fn set_wasmer_instance(&self, wasmer_instance: Option<NonNull<WasmerInstance>>) {
        self.with_context_data_mut(|context_data| {
            context_data.wasmer_instance = wasmer_instance;
        });
    }

    pub fn get_gas_left(&self) -> u64 {
        self.with_wasmer_instance(|instance| {
            Ok(match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            })
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    pub fn set_gas_left(&self, new_value: u64) {
        self.with_wasmer_instance(|instance| {
            set_remaining_points(instance, new_value);
            Ok(())
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }

    /// Decreases gas left by the given amount.
    /// If the amount exceeds the available gas, the remaining gas is set to 0 and
    /// an VmError::GasDepletion error is returned.
    #[allow(unused)] // used in tests
    pub fn decrease_gas_left(&self, amount: u64) -> VmResult<()> {
        self.with_wasmer_instance(|instance| {
            let remaining = match get_remaining_points(instance) {
                MeteringPoints::Remaining(count) => count,
                MeteringPoints::Exhausted => 0,
            };
            if amount > remaining {
                set_remaining_points(instance, 0);
                Err(VmError::gas_depletion())
            } else {
                set_remaining_points(instance, remaining - amount);
                Ok(())
            }
        })
    }

    pub fn memory(&self) -> Memory {
        self.with_wasmer_instance(|instance| {
            let first: Option<Memory> = instance
                .exports
                .iter()
                .memories()
                .next()
                .map(|pair| pair.1.clone());
            // Every contract in CosmWasm must have exactly one exported memory.
            // This is ensured by `check_wasm`/`check_wasm_memories`, which is called for every
            // contract added to the Cache as well as in integration tests.
            // It is possible to bypass this check when using `Instance::from_code` but then you
            // learn the hard way when this panics, or when trying to upload the contract to chain.
            let memory = first.expect("A contract must have exactly one exported memory.");
            Ok(memory)
        })
        .expect("Wasmer instance is not set. This is a bug in the lifecycle.")
    }
}

pub struct ContextData {
    gas_state: GasState,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
}

impl ContextData {
    pub fn new(gas_limit: u64) -> Self {
        ContextData {
            gas_state: GasState::with_limit(gas_limit),
            wasmer_instance: None,
        }
    }
}

pub fn process_gas_info<A: BackendApi>(env: &Environment<A>, info: GasInfo) -> VmResult<()> {
    let gas_left = env.get_gas_left();

    let new_limit = env.with_gas_state_mut(|gas_state| {
        gas_state.externally_used_gas += info.externally_used;
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        gas_left
            .saturating_sub(info.externally_used)
            .saturating_sub(info.cost)
    });

    // This tells wasmer how much more gas it can consume from this point in time.
    env.set_gas_left(new_limit);

    if info.externally_used + info.cost > gas_left {
        Err(VmError::gas_depletion())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversion::ref_to_u32;
    use crate::errors::VmError;
    use crate::size::Size;
    use crate::testing::MockApi;
    use crate::wasm_backend::compile;
    use wasmer::{imports, Function, Instance as WasmerInstance};

    static CONTRACT: &[u8] = include_bytes!("../testdata/json.wasm");

    const TESTING_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));

    fn make_instance(gas_limit: u64) -> (Environment<MockApi>, Box<WasmerInstance>) {
        let env = Environment::new(MockApi::default(), gas_limit, false);

        let module = compile(CONTRACT, TESTING_MEMORY_LIMIT, &[]).unwrap();
        let store = module.store();
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "read" => Function::new_native(store, |_a: u32| -> u32 { 0 }),
                "debug" => Function::new_native(store, |_a: u32| {}),
                "abort" => Function::new_native(store, |_a: u32| {}),
            },
        };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(gas_limit);

        (env, instance)
    }

    #[test]
    fn process_gas_info_works_for_cost() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, GasInfo::with_cost(70)).unwrap();
        assert_eq!(env.get_gas_left(), 30);
        process_gas_info(&env, GasInfo::with_cost(4)).unwrap();
        assert_eq!(env.get_gas_left(), 26);
        process_gas_info(&env, GasInfo::with_cost(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_cost(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_cost(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn process_gas_info_works_for_externally_used() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Consume all the Gas that we allocated
        process_gas_info(&env, GasInfo::with_externally_used(70)).unwrap();
        assert_eq!(env.get_gas_left(), 30);
        process_gas_info(&env, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(), 26);
        process_gas_info(&env, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn process_gas_info_works_for_cost_and_externally_used() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 0);

        process_gas_info(&env, GasInfo::new(17, 4)).unwrap();
        assert_eq!(env.get_gas_left(), 79);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, GasInfo::new(9, 0)).unwrap();
        assert_eq!(env.get_gas_left(), 70);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 4);

        process_gas_info(&env, GasInfo::new(0, 70)).unwrap();
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More cost fail but do not change stats
        match process_gas_info(&env, GasInfo::new(1, 0)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 74);

        // More externally used fails and changes stats
        match process_gas_info(&env, GasInfo::new(0, 1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
        assert_eq!(env.get_gas_left(), 0);
        let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
        assert_eq!(gas_state.gas_limit, 100);
        assert_eq!(gas_state.externally_used_gas, 75);
    }

    #[test]
    fn process_gas_info_zeros_gas_left_when_exceeded() {
        // with_externally_used
        {
            let (env, _instance) = make_instance(100);
            let result = process_gas_info(&env, GasInfo::with_externally_used(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {:?}", err),
            }
            assert_eq!(env.get_gas_left(), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 120);
        }

        // with_cost
        {
            let (env, _instance) = make_instance(100);
            let result = process_gas_info(&env, GasInfo::with_cost(120));
            match result.unwrap_err() {
                VmError::GasDepletion { .. } => {}
                err => panic!("unexpected error: {:?}", err),
            }
            assert_eq!(env.get_gas_left(), 0);
            let gas_state = env.with_gas_state(|gas_state| gas_state.clone());
            assert_eq!(gas_state.gas_limit, 100);
            assert_eq!(gas_state.externally_used_gas, 0);
        }
    }

    #[test]
    fn process_gas_info_works_correctly_with_gas_consumption_in_wasmer() {
        let (env, _instance) = make_instance(100);
        assert_eq!(env.get_gas_left(), 100);

        // Some gas was consumed externally
        process_gas_info(&env, GasInfo::with_externally_used(50)).unwrap();
        assert_eq!(env.get_gas_left(), 50);
        process_gas_info(&env, GasInfo::with_externally_used(4)).unwrap();
        assert_eq!(env.get_gas_left(), 46);

        // Consume 20 gas directly in wasmer
        env.decrease_gas_left(20).unwrap();
        assert_eq!(env.get_gas_left(), 26);

        process_gas_info(&env, GasInfo::with_externally_used(6)).unwrap();
        assert_eq!(env.get_gas_left(), 20);
        process_gas_info(&env, GasInfo::with_externally_used(20)).unwrap();
        assert_eq!(env.get_gas_left(), 0);

        // Using one more unit of gas triggers a failure
        match process_gas_info(&env, GasInfo::with_externally_used(1)).unwrap_err() {
            VmError::GasDepletion { .. } => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        let result = env.call_function("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result[0]).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function_fails_for_missing_instance() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        // Clear context's wasmer_instance
        env.set_wasmer_instance(None);

        let res = env.call_function("allocate", &[]);
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function_fails_for_missing_function() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        let res = env.call_function("doesnt_exist", &[]);
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(msg, "Could not get export: Missing export doesnt_exist");
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function0_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        env.call_function0("interface_version_8", &[]).unwrap();
    }

    #[test]
    fn call_function0_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        let result = env.call_function0("allocate", &[10u32.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "allocate");
                assert_eq!(expected, 0);
                assert_eq!(actual, 1);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn call_function1_works() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn call_function1_errors_for_wrong_result_count() {
        let (env, _instance) = make_instance(TESTING_GAS_LIMIT);

        let result = env.call_function1("allocate", &[10u32.into()]).unwrap();
        let ptr = ref_to_u32(&result).unwrap();
        assert!(ptr > 0);

        let result = env.call_function1("deallocate", &[ptr.into()]);
        match result.unwrap_err() {
            VmError::ResultMismatch {
                function_name,
                expected,
                actual,
                ..
            } => {
                assert_eq!(function_name, "deallocate");
                assert_eq!(expected, 1);
                assert_eq!(actual, 0);
            }
            err => panic!("unexpected error: {:?}", err),
        }
    }
}
