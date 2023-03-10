//! This file has some helpers for integration tests.
//! They should be imported via full path to ensure there is no confusion
//! use tableland_vm::testing::X
use std::collections::HashSet;

use crate::capabilities::capabilities_from_csv;
use crate::compatibility::check_wasm;
use crate::instance::{Instance, InstanceOptions};
use crate::size::Size;
use crate::{Backend, BackendApi};

use super::mock::MockApi;

/// This gas limit is used in integration tests and should be high enough to allow a reasonable
/// number of contract executions and queries on one instance. For this reason it is significatly
/// higher than the limit for a single execution that we have in the production setup.
const DEFAULT_GAS_LIMIT: u64 = 500_000_000_000; // ~0.5ms
const DEFAULT_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
const DEFAULT_PRINT_DEBUG: bool = true;

pub fn mock_instance(wasm: &[u8], data: Vec<u8>) -> Instance<MockApi> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            ..Default::default()
        },
        data,
    )
}

/// Creates an instance from the given Wasm bytecode.
/// The gas limit is measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
pub fn mock_instance_with_gas_limit(
    wasm: &[u8],
    gas_limit: u64,
    data: Vec<u8>,
) -> Instance<MockApi> {
    mock_instance_with_options(
        wasm,
        MockInstanceOptions {
            gas_limit,
            ..Default::default()
        },
        data,
    )
}

#[derive(Debug)]
pub struct MockInstanceOptions {
    /// Function capabilities (currently not used for Tableland Functions)
    pub available_capabilities: HashSet<String>,
    /// Gas limit measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    pub print_debug: bool,
    /// Memory limit in bytes. Use a value that is divisible by the Wasm page size 65536, e.g. full MiBs.
    pub memory_limit: Option<Size>,
}

impl MockInstanceOptions {
    fn default_capabilities() -> HashSet<String> {
        #[allow(unused_mut)]
        let mut out = capabilities_from_csv("");
        out
    }
}

impl Default for MockInstanceOptions {
    fn default() -> Self {
        Self {
            available_capabilities: Self::default_capabilities(),
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
            memory_limit: DEFAULT_MEMORY_LIMIT,
        }
    }
}

pub fn mock_instance_with_options(
    wasm: &[u8],
    options: MockInstanceOptions,
    data: Vec<u8>,
) -> Instance<MockApi> {
    check_wasm(wasm, &options.available_capabilities).unwrap();

    let backend = Backend {
        api: MockApi::new(data),
    };
    let memory_limit = options.memory_limit;
    let options = InstanceOptions {
        gas_limit: options.gas_limit,
        print_debug: options.print_debug,
    };
    Instance::from_code(wasm, backend, options, memory_limit).unwrap()
}

/// Creates InstanceOptions for testing
pub fn mock_instance_options() -> (InstanceOptions, Option<Size>) {
    (
        InstanceOptions {
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
        },
        DEFAULT_MEMORY_LIMIT,
    )
}

/// Runs a series of IO tests, hammering especially on allocate and deallocate.
/// This could be especially useful when run with some kind of leak detector.
pub fn test_io<A>(instance: &mut Instance<A>)
where
    A: BackendApi + 'static,
{
    let sizes: Vec<usize> = vec![0, 1, 3, 10, 200, 2000, 5 * 1024];
    let bytes: Vec<u8> = vec![0x00, 0xA5, 0xFF];

    for size in sizes.into_iter() {
        for byte in bytes.iter() {
            let original = vec![*byte; size];
            let wasm_ptr = instance
                .allocate(original.len())
                .expect("Could not allocate memory");
            instance
                .write_memory(wasm_ptr, &original)
                .expect("Could not write data");
            let wasm_data = instance.read_memory(wasm_ptr, size).expect("error reading");
            assert_eq!(
                original, wasm_data,
                "failed for size {}; expected: {:?}; actual: {:?}",
                size, original, wasm_data
            );
            instance
                .deallocate(wasm_ptr)
                .expect("Could not deallocate memory");
        }
    }
}
