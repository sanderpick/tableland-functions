use serde_json::{from_slice, to_string, Value};
use tableland_std::{BlockInfo, Env};
use tableland_vm::{call_fetch, Instance};

use crate::backend::Api;
use crate::instance::instance_with_gas_limit;

const WASM: &[u8] =
    include_bytes!("../../../examples/demo/target/wasm32-unknown-unknown/release/demo.wasm");

fn create_function() -> Instance<Api> {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    instance_with_gas_limit(WASM, gas_limit)
}

fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            chain_id: "yoyo".to_string(),
        },
    }
}

#[test]
fn call_fetch_works() {
    let mut instance = create_function();
    let res = call_fetch(&mut instance, &mock_env()).unwrap().unwrap();
    assert_eq!(true, res.data.is_some());

    let data = res.data.unwrap().into_vec();
    let json = from_slice::<Value>(data.as_slice()).unwrap();
    println!("{}", to_string(&json).unwrap());

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
