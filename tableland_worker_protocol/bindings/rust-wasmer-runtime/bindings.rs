use super::types::*;
use fp_bindgen_support::{
    common::{abi::WasmAbi, mem::FatPtr},
    host::{
        errors::{InvocationError, RuntimeError},
        mem::{
            deserialize_from_slice, export_to_guest, export_to_guest_raw, import_from_guest,
            import_from_guest_raw, serialize_to_vec,
        },
        r#async::{create_future_value, future::ModuleRawFuture, resolve_async_value},
        runtime::RuntimeInstanceData,
    },
};
use std::cell::RefCell;
use std::sync::Arc;
use wasmer::wasmparser::Operator;
use wasmer::CompilerConfig;
use wasmer::{imports, Function, ImportObject, Instance, Module, Store, WasmerEnv};
use wasmer_middlewares::{
    metering::{get_remaining_points, set_remaining_points, MeteringPoints},
    Metering,
};

#[derive(Clone)]
pub struct Runtime {
    instance: Instance,
    env: RuntimeInstanceData,
}

const COST_FUNCTION: fn(&Operator) -> u64 = |operator: &Operator| -> u64 {
    match operator {
        Operator::LocalGet { .. } | Operator::I32Const { .. } => 9,
        Operator::I32Add { .. } => 12,
        _ => 10,
    }
};

const INITIAL_GAS: u64 = 100_000_000_000;

impl Runtime {
    pub fn new(wasm_module: impl AsRef<[u8]>) -> Result<Self, RuntimeError> {
        let store = Self::default_store();
        let module = Module::new(&store, wasm_module)?;
        let mut env = RuntimeInstanceData::default();
        let import_object = create_import_object(module.store(), &env);
        let instance = Instance::new(&module, &import_object).unwrap();
        env.init_with_instance(&instance).unwrap();
        Ok(Self { instance, env })
    }

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    fn default_store() -> wasmer::Store {
        let metering = Arc::new(Metering::new(INITIAL_GAS, COST_FUNCTION));
        let mut compiler = wasmer::Cranelift::default();
        compiler.push_middleware(metering);
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    #[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
    fn default_store() -> wasmer::Store {
        let metering = Arc::new(Metering::new(INITIAL_GAS, COST_FUNCTION));
        let mut compiler = wasmer::Singlepass::default();
        compiler.push_middleware(metering);
        let engine = wasmer::Universal::new(compiler).engine();
        Store::new(&engine)
    }

    /// Fetch handler for the plugin.
    pub async fn fetch(
        &self,
        request: Request,
    ) -> (Result<Result<Response, Error>, InvocationError>, u64) {
        let request = serialize_to_vec(&request);
        let result = self.fetch_raw(request);
        let result = result.await;

        // Demo purposes only. Initial gas is very high to ensure we don't run out.
        // We're just collecting spent gas to show the caller.
        let gas_left = match get_remaining_points(&self.instance) {
            MeteringPoints::Remaining(p) => p,
            MeteringPoints::Exhausted => 0,
        };
        let gas_used = INITIAL_GAS - gas_left;
        set_remaining_points(&self.instance, INITIAL_GAS);

        let result = result.map(|ref data| deserialize_from_slice(data));
        (result, gas_used)
    }
    pub async fn fetch_raw(&self, request: Vec<u8>) -> Result<Vec<u8>, InvocationError> {
        let request = export_to_guest_raw(&self.env, request);
        let function = self
            .instance
            .exports
            .get_native_function::<FatPtr, FatPtr>("__fp_gen_fetch")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_fetch".to_owned()))?;
        let result = function.call(request.to_abi())?;
        let result = ModuleRawFuture::new(self.env.clone(), result).await;
        Ok(result)
    }

    /// Called on the plugin to give it a chance to initialize.
    pub fn init(&self) -> Result<(), InvocationError> {
        let result = self.init_raw();
        set_remaining_points(&self.instance, INITIAL_GAS);
        result
    }
    pub fn init_raw(&self) -> Result<(), InvocationError> {
        let function = self
            .instance
            .exports
            .get_native_function::<(), ()>("__fp_gen_init")
            .map_err(|_| InvocationError::FunctionNotExported("__fp_gen_init".to_owned()))?;
        let result = function.call()?;
        let result = WasmAbi::from_abi(result);
        Ok(result)
    }
}

fn create_import_object(store: &Store, env: &RuntimeInstanceData) -> ImportObject {
    imports! {
        "fp" => {
            "__fp_host_resolve_async_value" => Function::new_native_with_env(store, env.clone(), resolve_async_value),
            "__fp_gen_log" => Function::new_native_with_env(store, env.clone(), _log),
            "__fp_gen_query" => Function::new_native_with_env(store, env.clone(), _query),
        }
    }
}

pub fn _log(env: &RuntimeInstanceData, message: FatPtr) {
    let message = import_from_guest::<String>(env, message);
    let result = super::log(message);
}

pub fn _query(env: &RuntimeInstanceData, statement: FatPtr, options: FatPtr) -> FatPtr {
    let statement = import_from_guest::<String>(env, statement);
    let options = import_from_guest::<ReadOptions>(env, options);
    let result = super::query(statement, options);
    let env = env.clone();
    let async_ptr = create_future_value(&env);
    let handle = tokio::runtime::Handle::current();
    handle.spawn(async move {
        let result = result.await;
        let result_ptr = export_to_guest(&env, &result);
        env.guest_resolve_async_value(async_ptr, result_ptr);
    });
    async_ptr
}
