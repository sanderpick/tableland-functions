use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::sync::Mutex;

use wasmer::{
    Exports, Function, FunctionEnv, FunctionEnvMut, Imports, Instance as WasmerInstance, Module,
    Store, Value,
};

use crate::backend::{Backend, BackendApi};
use crate::capabilities::required_capabilities_from_module;
use crate::conversion::{ref_to_u32, to_u32};
use crate::environment::Environment;
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::imports::{do_abort, do_debug, do_hello};
// #[cfg(feature = "iterator")]
// use crate::imports::{do_db_next, do_db_scan};
use crate::memory::{read_region, write_region};
use crate::size::Size;
use crate::wasm_backend::compile;
// use crate::wasm_backend::{compile, make_compile_time_store};

#[derive(Copy, Clone, Debug)]
pub struct GasReport {
    /// The original limit the instance was created with
    pub limit: u64,
    /// The remaining gas that can be spend
    pub remaining: u64,
    /// The amount of gas that was spend and metered externally in operations triggered by this instance
    pub used_externally: u64,
    /// The amount of gas that was spend and metered internally (i.e. by executing Wasm and calling
    /// API methods which are not metered externally)
    pub used_internally: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct InstanceOptions {
    pub gas_limit: u64,
    pub print_debug: bool,
}

pub struct Instance<A: BackendApi + 'static> {
    /// We put this instance in a box to maintain a constant memory address for the entire
    /// lifetime of the instance in the cache. This is needed e.g. when linking the wasmer
    /// instance to a context. See also https://github.com/CosmWasm/cosmwasm/pull/245.
    ///
    /// This instance should only be accessed via the Environment, which provides safe access.
    _inner: Box<WasmerInstance>,
    pub env: FunctionEnvMut<'static, Environment<A>>,
    // pub store: Store,
}

impl<A> Instance<A>
where
    A: BackendApi + 'static, // 'static is needed here to allow copying API instances into closures
                             // S: Storage + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
                             // Q: Querier + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
{
    /// This is the only Instance constructor that can be called from outside of cosmwasm-vm,
    /// e.g. in test code that needs a customized variant of cosmwasm_vm::testing::mock_instance*.
    pub fn from_code(
        code: &[u8],
        backend: Backend<A>,
        options: InstanceOptions,
        memory_limit: Option<Size>,
    ) -> VmResult<Self> {
        let (module, store) = compile(code, memory_limit, &[])?;

        Instance::from_module(
            store,
            &module,
            backend,
            options.gas_limit,
            options.print_debug,
            None,
            None,
        )
    }

    pub(crate) fn from_module(
        mut store: Store,
        module: &Module,
        backend: Backend<A>,
        gas_limit: u64,
        print_debug: bool,
        extra_imports: Option<HashMap<&str, Exports>>,
        instantiation_lock: Option<&Mutex<()>>,
    ) -> VmResult<Self> {
        let env = FunctionEnv::new(
            &mut store,
            Environment::new(backend.api, gas_limit, print_debug),
        );

        let mut import_obj = Imports::new();
        let mut env_imports = Exports::new();

        env_imports.insert(
            "hello",
            Function::new_typed_with_env(&mut store, &env, do_hello),
        );

        // Allows the contract to emit debug logs that the host can either process or ignore.
        // This is never written to chain.
        // Takes a pointer argument of a memory region that must contain an UTF-8 encoded string.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "debug",
            Function::new_typed_with_env(&mut store, &env, do_debug),
        );

        // Aborts the contract execution with an error message provided by the contract.
        // Takes a pointer argument of a memory region that must contain an UTF-8 encoded string.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "abort",
            Function::new_typed_with_env(&mut store, &env, do_abort),
        );

        import_obj.register_namespace("env", env_imports);

        if let Some(extra_imports) = extra_imports {
            for (namespace, exports_obj) in extra_imports {
                import_obj.register_namespace(namespace, exports_obj);
            }
        }

        let wasmer_instance = Box::from(
            {
                let _lock = instantiation_lock.map(|l| l.lock().unwrap());
                WasmerInstance::new(&mut store, module, &import_obj)
            }
            .map_err(|original| {
                VmError::instantiation_err(format!("Error instantiating module: {:?}", original))
            })?,
        );

        let instance_ptr = NonNull::from(wasmer_instance.as_ref());
        let foo = env.as_ref(&store).clone();
        foo.set_wasmer_instance(Some(instance_ptr));
        foo.set_gas_left(&mut store, gas_limit);
        // env.as_ref(&store).move_in(backend.storage, backend.querier);
        let instance = Instance {
            _inner: wasmer_instance,
            env: env.into_mut(&mut store),
            // env: env.as_ref(&store).clone(),
            // store,
        };
        Ok(instance)
    }

    pub fn api(&self) -> &A {
        &self.env.data().api
    }

    // /// Decomposes this instance into its components.
    // /// External dependencies are returned for reuse, the rest is dropped.
    // pub fn recycle(self) -> Option<Backend<A>> {
    //     if let (Some(storage), Some(querier)) = self.env.move_out() {
    //         let api = self.env.api;
    //         Some(Backend {
    //             api,
    //             storage,
    //             querier,
    //         })
    //     } else {
    //         None
    //     }
    // }

    /// Returns the features required by this contract.
    ///
    /// This is not needed for production because we can do static analysis
    /// on the Wasm file before instatiation to obtain this information. It's
    /// only kept because it can be handy for integration testing.
    pub fn required_capabilities(&self) -> HashSet<String> {
        required_capabilities_from_module(self._inner.module())
    }

    /// Returns the size of the default memory in pages.
    /// This provides a rough idea of the peak memory consumption. Note that
    /// Wasm memory always grows in 64 KiB steps (pages) and can never shrink
    /// (https://github.com/WebAssembly/design/issues/1300#issuecomment-573867836).
    pub fn memory_pages(&self) -> usize {
        todo!()
    }

    /// Returns the currently remaining gas.
    pub fn get_gas_left(&mut self) -> u64 {
        let data = self.env.data().clone();
        data.get_gas_left(&mut self.env)
    }

    /// Creates and returns a gas report.
    /// This is a snapshot and multiple reports can be created during the lifetime of
    /// an instance.
    pub fn create_gas_report(&mut self) -> GasReport {
        let data = self.env.data().clone();
        let state = data.with_gas_state(|gas_state| gas_state.clone());
        let gas_left = data.get_gas_left(&mut self.env);
        GasReport {
            limit: state.gas_limit,
            remaining: gas_left,
            used_externally: state.externally_used_gas,
            // If externally_used_gas exceeds the gas limit, this will return 0.
            // no matter how much gas was used internally. But then we error with out of gas
            // anyways, and it does not matter much anymore where gas was spend.
            used_internally: state
                .gas_limit
                .saturating_sub(state.externally_used_gas)
                .saturating_sub(gas_left),
        }
    }

    // /// Sets the readonly storage flag on this instance. Since one instance can be used
    // /// for multiple calls in integration tests, this should be set to the desired value
    // /// right before every call.
    // pub fn set_storage_readonly(&mut self, new_value: bool) {
    //     self.env.set_storage_readonly(new_value);
    // }
    //
    // pub fn with_storage<F: FnOnce(&mut S) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
    //     self.env.with_storage_from_context::<F, T>(func)
    // }
    //
    // pub fn with_querier<F: FnOnce(&mut Q) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
    //     self.env.with_querier_from_context::<F, T>(func)
    // }

    /// Requests memory allocation by the instance and returns a pointer
    /// in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, size: usize) -> VmResult<u32> {
        let ret = self.call_function1("allocate", &[to_u32(size)?.into()])?;
        let ptr = ref_to_u32(&ret)?;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> VmResult<()> {
        self.call_function0("deallocate", &[ptr.into()])?;
        Ok(())
    }

    /// Copies all data described by the Region at the given pointer from Wasm to the caller.
    pub(crate) fn read_memory(&self, region_ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
        // let env = &self
        read_region(&self.env, region_ptr, max_length)
    }

    /// Copies data to the memory region that was created before using allocate.
    pub(crate) fn write_memory(&mut self, region_ptr: u32, data: &[u8]) -> VmResult<()> {
        write_region(&self.env, region_ptr, data)?;
        Ok(())
    }

    /// Calls a function exported by the instance.
    /// The function is expected to return no value. Otherwise this calls errors.
    pub(crate) fn call_function0(&mut self, name: &str, args: &[Value]) -> VmResult<()> {
        let data = self.env.data().clone();
        data.call_function0(&mut self.env, name, args)
    }

    /// Calls a function exported by the instance.
    /// The function is expected to return one value. Otherwise this calls errors.
    pub(crate) fn call_function1(&mut self, name: &str, args: &[Value]) -> VmResult<Value> {
        let data = self.env.data().clone();
        data.call_function1(&mut self.env, name, args)
    }
}

/// This exists only to be exported through `internals` for use by crates that are
/// part of Cosmwasm.
pub fn instance_from_module<A>(
    store: Store,
    module: &Module,
    backend: Backend<A>,
    gas_limit: u64,
    print_debug: bool,
    extra_imports: Option<HashMap<&str, Exports>>,
) -> VmResult<Instance<A>>
where
    A: BackendApi + 'static, // 'static is needed here to allow copying API instances into closures
                             // S: Storage + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
                             // Q: Querier + 'static,
{
    Instance::from_module(
        store,
        module,
        backend,
        gas_limit,
        print_debug,
        extra_imports,
        None,
    )
}

// #[cfg(test)]
// mod tests {
//     use std::sync::atomic::{AtomicBool, Ordering};
//     use std::sync::Arc;
//
//     use super::*;
//     use crate::backend::Storage;
//     use crate::calls::{call_execute, call_instantiate, call_query};
//     use crate::errors::VmError;
//     use crate::testing::{
//         mock_backend, mock_env, mock_info, mock_instance, mock_instance_options,
//         mock_instance_with_balances, mock_instance_with_failing_api, mock_instance_with_gas_limit,
//         mock_instance_with_options, MockInstanceOptions,
//     };
//     use cosmwasm_std::{
//         coin, coins, from_binary, AllBalanceResponse, BalanceResponse, BankQuery, Empty,
//         QueryRequest,
//     };
//     use wasmer::FunctionEnvMut;
//
//     const KIB: usize = 1024;
//     const MIB: usize = 1024 * 1024;
//     const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
//     static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
//
//     #[test]
//     fn required_capabilities_works() {
//         let backend = mock_backend(&[]);
//         let (instance_options, memory_limit) = mock_instance_options();
//         let instance =
//             Instance::from_code(CONTRACT, backend, instance_options, memory_limit).unwrap();
//         assert_eq!(instance.required_capabilities().len(), 0);
//     }
//
//     #[test]
//     fn required_capabilities_works_for_many_exports() {
//         let wasm = wat::parse_str(
//             r#"(module
//             (type (func))
//             (func (type 0) nop)
//             (export "requires_water" (func 0))
//             (export "requires_" (func 0))
//             (export "requires_nutrients" (func 0))
//             (export "require_milk" (func 0))
//             (export "REQUIRES_air" (func 0))
//             (export "requires_sun" (func 0))
//             )"#,
//         )
//         .unwrap();
//
//         let backend = mock_backend(&[]);
//         let (instance_options, memory_limit) = mock_instance_options();
//         let instance = Instance::from_code(&wasm, backend, instance_options, memory_limit).unwrap();
//         assert_eq!(instance.required_capabilities().len(), 3);
//         assert!(instance.required_capabilities().contains("nutrients"));
//         assert!(instance.required_capabilities().contains("sun"));
//         assert!(instance.required_capabilities().contains("water"));
//     }
//
//     #[test]
//     fn extra_imports_get_added() {
//         let wasm = wat::parse_str(
//             r#"(module
//             (import "foo" "bar" (func $bar))
//             (func (export "main") (call $bar))
//             )"#,
//         )
//         .unwrap();
//
//         let backend = mock_backend(&[]);
//         let (instance_options, memory_limit) = mock_instance_options();
//         let (module, store) = compile(&wasm, memory_limit, &[]).unwrap();
//
//         struct MyEnv {
//             // This can be mutated across threads safely. We initialize it as `false`
//             // and let our imported fn switch it to `true` to confirm it works.
//             called: Arc<AtomicBool>,
//         }
//
//         let my_env = MyEnv {
//             called: Arc::new(AtomicBool::new(false)),
//         };
//
//         let fun = Function::new_typed_with_env(
//             &mut store,
//             &FunctionEnv::new(&mut store, my_env),
//             |env: FunctionEnvMut<MyEnv>| {
//                 env.data().called.store(true, Ordering::Relaxed);
//             },
//         );
//         let mut exports = Exports::new();
//         exports.insert("bar", fun);
//         let mut extra_imports = HashMap::new();
//         extra_imports.insert("foo", exports);
//         let instance = Instance::from_module(
//             store,
//             &module,
//             backend,
//             instance_options.gas_limit,
//             false,
//             Some(extra_imports),
//             None,
//         )
//         .unwrap();
//
//         let main = instance._inner.exports.get_function("main").unwrap();
//         main.call(&mut store, &[]).unwrap();
//
//         assert!(my_env.called.load(Ordering::Relaxed));
//     }
//
//     #[test]
//     fn call_function0_works() {
//         let instance = mock_instance(CONTRACT, &[]);
//
//         instance
//             .call_function0("interface_version_8", &[])
//             .expect("error calling function");
//     }
//
//     #[test]
//     fn call_function1_works() {
//         let store = Store::default();
//
//         let instance = mock_instance(CONTRACT, &[]);
//
//         // can call function few times
//         let result = instance
//             .call_function1("allocate", &[0u32.into()])
//             .expect("error calling allocate");
//         assert_ne!(result.unwrap_i32(), 0);
//
//         let result = instance
//             .call_function1("allocate", &[1u32.into()])
//             .expect("error calling allocate");
//         assert_ne!(result.unwrap_i32(), 0);
//
//         let result = instance
//             .call_function1("allocate", &[33u32.into()])
//             .expect("error calling allocate");
//         assert_ne!(result.unwrap_i32(), 0);
//     }
//
//     #[test]
//     fn allocate_deallocate_works() {
//         let mut instance = mock_instance_with_options(
//             CONTRACT,
//             MockInstanceOptions {
//                 memory_limit: Some(Size::mebi(500)),
//                 ..Default::default()
//             },
//         );
//
//         let sizes: Vec<usize> = vec![
//             0,
//             4,
//             40,
//             400,
//             4 * KIB,
//             40 * KIB,
//             400 * KIB,
//             4 * MIB,
//             40 * MIB,
//             400 * MIB,
//         ];
//         for size in sizes.into_iter() {
//             let region_ptr = instance.allocate(size).expect("error allocating");
//             instance.deallocate(region_ptr).expect("error deallocating");
//         }
//     }
//
//     #[test]
//     fn write_and_read_memory_works() {
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         let sizes: Vec<usize> = vec![
//             0,
//             4,
//             40,
//             400,
//             4 * KIB,
//             40 * KIB,
//             400 * KIB,
//             4 * MIB,
//             // disabled for performance reasons, but pass as well
//             // 40 * MIB,
//             // 400 * MIB,
//         ];
//         for size in sizes.into_iter() {
//             let region_ptr = instance.allocate(size).expect("error allocating");
//             let original = vec![170u8; size];
//             instance
//                 .write_memory(region_ptr, &original)
//                 .expect("error writing");
//             let data = instance
//                 .read_memory(region_ptr, size)
//                 .expect("error reading");
//             assert_eq!(data, original);
//             instance.deallocate(region_ptr).expect("error deallocating");
//         }
//     }
//
//     #[test]
//     fn errors_in_imports() {
//         let store = Store::default();
//
//         // set up an instance that will experience an error in an import
//         let error_message = "Api failed intentionally";
//         let mut instance = mock_instance_with_failing_api(CONTRACT, &[], error_message);
//         let init_result = call_instantiate::<_, _, _, Empty>(
//             &mut instance,
//             &mock_env(),
//             &mock_info("someone", &[]),
//             b"{\"verifier\": \"some1\", \"beneficiary\": \"some2\"}",
//         );
//
//         match init_result.unwrap_err() {
//             VmError::RuntimeErr { msg, .. } => assert!(msg.contains(error_message)),
//             err => panic!("Unexpected error: {:?}", err),
//         }
//     }
//
//     #[test]
//     fn read_memory_errors_when_when_length_is_too_long() {
//         let length = 6;
//         let max_length = 5;
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // Allocate sets length to 0. Write some data to increase length.
//         let region_ptr = instance.allocate(length).expect("error allocating");
//         let data = vec![170u8; length];
//         instance
//             .write_memory(region_ptr, &data)
//             .expect("error writing");
//
//         let result = instance.read_memory(region_ptr, max_length);
//         match result.unwrap_err() {
//             VmError::CommunicationErr {
//                 source:
//                     CommunicationError::RegionLengthTooBig {
//                         length, max_length, ..
//                     },
//                 ..
//             } => {
//                 assert_eq!(length, 6);
//                 assert_eq!(max_length, 5);
//             }
//             err => panic!("unexpected error: {:?}", err),
//         };
//
//         instance.deallocate(region_ptr).expect("error deallocating");
//     }
//
//     #[test]
//     fn memory_pages_returns_min_memory_size_by_default() {
//         let store = Store::default();
//
//         // min: 0 pages, max: none
//         let wasm = wat::parse_str(
//             r#"(module
//                 (memory 0)
//                 (export "memory" (memory 0))
//
//                 (type (func))
//                 (func (type 0) nop)
//                 (export "interface_version_8" (func 0))
//                 (export "instantiate" (func 0))
//                 (export "allocate" (func 0))
//                 (export "deallocate" (func 0))
//             )"#,
//         )
//         .unwrap();
//         let instance = mock_instance(&wasm, &[]);
//         assert_eq!(instance.memory_pages(), 0);
//
//         // min: 3 pages, max: none
//         let wasm = wat::parse_str(
//             r#"(module
//                 (memory 3)
//                 (export "memory" (memory 0))
//
//                 (type (func))
//                 (func (type 0) nop)
//                 (export "interface_version_8" (func 0))
//                 (export "instantiate" (func 0))
//                 (export "allocate" (func 0))
//                 (export "deallocate" (func 0))
//             )"#,
//         )
//         .unwrap();
//         let instance = mock_instance(&wasm, &[]);
//         assert_eq!(instance.memory_pages(), 3);
//     }
//
//     #[test]
//     fn memory_pages_grows_with_usage() {
//         let store = Store::default();
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         assert_eq!(instance.memory_pages(), 17);
//
//         // 100 KiB require two more pages
//         let region_ptr = instance.allocate(100 * 1024).expect("error allocating");
//
//         assert_eq!(instance.memory_pages(), 19);
//
//         // Deallocating does not shrink memory
//         instance.deallocate(region_ptr).expect("error deallocating");
//         assert_eq!(instance.memory_pages(), 19);
//     }
//
//     #[test]
//     fn get_gas_left_works() {
//         let instance = mock_instance_with_gas_limit(CONTRACT, 123321);
//         let orig_gas = instance.get_gas_left();
//         assert_eq!(orig_gas, 123321);
//     }
//
//     #[test]
//     fn create_gas_report_works() {
//         const LIMIT: u64 = 700_000_000_000;
//         let mut instance = mock_instance_with_gas_limit(CONTRACT, LIMIT);
//
//         let report1 = instance.create_gas_report();
//         assert_eq!(report1.used_externally, 0);
//         assert_eq!(report1.used_internally, 0);
//         assert_eq!(report1.limit, LIMIT);
//         assert_eq!(report1.remaining, LIMIT);
//
//         // init contract
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         let report2 = instance.create_gas_report();
//         assert_eq!(report2.used_externally, 73);
//         assert_eq!(report2.used_internally, 5775750198);
//         assert_eq!(report2.limit, LIMIT);
//         assert_eq!(
//             report2.remaining,
//             LIMIT - report2.used_externally - report2.used_internally
//         );
//     }
//
//     #[test]
//     fn set_storage_readonly_works() {
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         assert!(instance.env.is_storage_readonly());
//
//         instance.set_storage_readonly(false);
//         assert!(!instance.env.is_storage_readonly());
//
//         instance.set_storage_readonly(false);
//         assert!(!instance.env.is_storage_readonly());
//
//         instance.set_storage_readonly(true);
//         assert!(instance.env.is_storage_readonly());
//     }
//
//     #[test]
//     fn with_storage_works() {
//         let store = Store::default();
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // initial check
//         instance
//             .with_storage(|store| {
//                 assert!(store.get(b"foo").0.unwrap().is_none());
//                 Ok(())
//             })
//             .unwrap();
//
//         // write some data
//         instance
//             .with_storage(|store| {
//                 store.set(b"foo", b"bar").0.unwrap();
//                 Ok(())
//             })
//             .unwrap();
//
//         // read some data
//         instance
//             .with_storage(|store| {
//                 assert_eq!(store.get(b"foo").0.unwrap(), Some(b"bar".to_vec()));
//                 Ok(())
//             })
//             .unwrap();
//     }
//
//     #[test]
//     #[should_panic]
//     fn with_storage_safe_for_panic() {
//         // this should fail with the assertion, but not cause a double-free crash (issue #59)
//         let store = Store::default();
//         let mut instance = mock_instance(CONTRACT, &[]);
//         instance
//             .with_storage::<_, ()>(|_store| panic!("trigger failure"))
//             .unwrap();
//     }
//
//     #[test]
//     fn with_querier_works_readonly() {
//         let store = Store::default();
//
//         let rich_addr = String::from("foobar");
//         let rich_balance = vec![coin(10000, "gold"), coin(8000, "silver")];
//         let mut instance = mock_instance_with_balances(CONTRACT, &[(&rich_addr, &rich_balance)]);
//
//         // query one
//         instance
//             .with_querier(|querier| {
//                 let response = querier
//                     .query::<Empty>(
//                         &QueryRequest::Bank(BankQuery::Balance {
//                             address: rich_addr.clone(),
//                             denom: "silver".to_string(),
//                         }),
//                         DEFAULT_QUERY_GAS_LIMIT,
//                     )
//                     .0
//                     .unwrap()
//                     .unwrap()
//                     .unwrap();
//                 let BalanceResponse { amount } = from_binary(&response).unwrap();
//                 assert_eq!(amount.amount.u128(), 8000);
//                 assert_eq!(amount.denom, "silver");
//                 Ok(())
//             })
//             .unwrap();
//
//         // query all
//         instance
//             .with_querier(|querier| {
//                 let response = querier
//                     .query::<Empty>(
//                         &QueryRequest::Bank(BankQuery::AllBalances {
//                             address: rich_addr.clone(),
//                         }),
//                         DEFAULT_QUERY_GAS_LIMIT,
//                     )
//                     .0
//                     .unwrap()
//                     .unwrap()
//                     .unwrap();
//                 let AllBalanceResponse { amount } = from_binary(&response).unwrap();
//                 assert_eq!(amount.len(), 2);
//                 assert_eq!(amount[0].amount.u128(), 10000);
//                 assert_eq!(amount[0].denom, "gold");
//                 assert_eq!(amount[1].amount.u128(), 8000);
//                 assert_eq!(amount[1].denom, "silver");
//
//                 Ok(())
//             })
//             .unwrap();
//     }
//
//     /// This is needed for writing intagration tests in which the balance of a contract changes over time
//     #[test]
//     fn with_querier_allows_updating_balances() {
//         let store = Store::default();
//
//         let rich_addr = String::from("foobar");
//         let rich_balance1 = vec![coin(10000, "gold"), coin(500, "silver")];
//         let rich_balance2 = vec![coin(10000, "gold"), coin(8000, "silver")];
//         let mut instance = mock_instance_with_balances(CONTRACT, &[(&rich_addr, &rich_balance1)]);
//
//         // Get initial state
//         instance
//             .with_querier(|querier| {
//                 let response = querier
//                     .query::<Empty>(
//                         &QueryRequest::Bank(BankQuery::Balance {
//                             address: rich_addr.clone(),
//                             denom: "silver".to_string(),
//                         }),
//                         DEFAULT_QUERY_GAS_LIMIT,
//                     )
//                     .0
//                     .unwrap()
//                     .unwrap()
//                     .unwrap();
//                 let BalanceResponse { amount } = from_binary(&response).unwrap();
//                 assert_eq!(amount.amount.u128(), 500);
//                 Ok(())
//             })
//             .unwrap();
//
//         // Update balance
//         instance
//             .with_querier(|querier| {
//                 querier.update_balance(&rich_addr, rich_balance2);
//                 Ok(())
//             })
//             .unwrap();
//
//         // Get updated state
//         instance
//             .with_querier(|querier| {
//                 let response = querier
//                     .query::<Empty>(
//                         &QueryRequest::Bank(BankQuery::Balance {
//                             address: rich_addr.clone(),
//                             denom: "silver".to_string(),
//                         }),
//                         DEFAULT_QUERY_GAS_LIMIT,
//                     )
//                     .0
//                     .unwrap()
//                     .unwrap()
//                     .unwrap();
//                 let BalanceResponse { amount } = from_binary(&response).unwrap();
//                 assert_eq!(amount.amount.u128(), 8000);
//                 Ok(())
//             })
//             .unwrap();
//     }
//
//     #[test]
//     fn contract_deducts_gas_init() {
//         let mut instance = mock_instance(CONTRACT, &[]);
//         let orig_gas = instance.get_gas_left();
//
//         // init contract
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         let init_used = orig_gas - instance.get_gas_left();
//         assert_eq!(init_used, 5775750271);
//     }
//
//     #[test]
//     fn contract_deducts_gas_execute() {
//         let mut store = Store::default();
//
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init contract
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         // run contract - just sanity check - results validate in contract unit tests
//         let gas_before_execute = instance.get_gas_left();
//         let info = mock_info("verifies", &coins(15, "earth"));
//         let msg = br#"{"release":{}}"#;
//         call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         let execute_used = gas_before_execute - instance.get_gas_left();
//         assert_eq!(execute_used, 8627053606);
//     }
//
//     #[test]
//     fn contract_enforces_gas_limit() {
//         let mut store = Store::default();
//
//         let mut instance = mock_instance_with_gas_limit(CONTRACT, 20_000);
//
//         // init contract
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         let res = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg);
//         assert!(res.is_err());
//     }
//
//     #[test]
//     fn query_works_with_gas_metering() {
//         let mut store = Store::default();
//
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init contract
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         let _res = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         // run contract - just sanity check - results validate in contract unit tests
//         let gas_before_query = instance.get_gas_left();
//         // we need to encode the key in base64
//         let msg = br#"{"verifier":{}}"#;
//         let res = call_query(&mut instance, &mock_env(), msg).unwrap();
//         let answer = res.unwrap();
//         assert_eq!(answer.as_slice(), b"{\"verifier\":\"verifies\"}");
//
//         let query_used = gas_before_query - instance.get_gas_left();
//         assert_eq!(query_used, 4438350006);
//     }
// }
