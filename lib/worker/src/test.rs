use serde_json::{Error, Value};
use tableland_std::{BlockInfo, Env, Response};
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
    let res: Response = call_fetch(&mut instance, &mock_env()).unwrap().unwrap();
    assert_eq!(true, res.data.is_some());

    let json: Result<Value, Error> = serde_json::from_slice(res.data.unwrap().as_slice());
    println!("{}", serde_json::to_string_pretty(&json.unwrap()).unwrap());

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
