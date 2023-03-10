use std::collections::HashSet;
use tableland_client::TablelandClient;
use tableland_vm::{capabilities_from_csv, check_wasm, Backend, Instance, InstanceOptions, Size};

use crate::backend::Api;

const DEFAULT_GAS_LIMIT: u64 = 2_000_000_000_000;
const DEFAULT_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));
const DEFAULT_PRINT_DEBUG: bool = true;

#[derive(Debug)]
pub struct ApiInstanceOptions {
    pub available_capabilities: HashSet<String>,
    /// Gas limit measured in [CosmWasm gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: u64,
    pub print_debug: bool,
    /// Memory limit in bytes. Use a value that is divisible by the Wasm page size 65536, e.g. full MiBs.
    pub memory_limit: Option<Size>,
}

impl ApiInstanceOptions {
    fn default_capabilities() -> HashSet<String> {
        #[allow(unused_mut)]
        let mut out = capabilities_from_csv("");
        out
    }
}

impl Default for ApiInstanceOptions {
    fn default() -> Self {
        Self {
            available_capabilities: Self::default_capabilities(),
            gas_limit: DEFAULT_GAS_LIMIT,
            print_debug: DEFAULT_PRINT_DEBUG,
            memory_limit: DEFAULT_MEMORY_LIMIT,
        }
    }
}

pub fn instance_with_options(
    wasm: &[u8],
    options: ApiInstanceOptions,
    client: TablelandClient,
) -> Instance<Api<TablelandClient>> {
    check_wasm(wasm, &options.available_capabilities).unwrap();

    let backend = Backend {
        api: Api::new(client),
    };
    let memory_limit = options.memory_limit;
    let options = InstanceOptions {
        gas_limit: options.gas_limit,
        print_debug: options.print_debug,
    };
    // todo: catch error, could be bad wasm
    Instance::from_code(wasm, backend, options, memory_limit).unwrap()
}
