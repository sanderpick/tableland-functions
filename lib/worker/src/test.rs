use serde_json::{to_string, Value};
use tableland_vm::testing::mock_get_request;
use tableland_vm::{call_fetch, Instance};

use crate::backend::Api;
use crate::instance::instance_with_gas_limit;

const WASM: &[u8] =
    include_bytes!("../../../examples/json/target/wasm32-unknown-unknown/release/json.wasm");

fn create_function() -> Instance<Api> {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    instance_with_gas_limit(WASM, gas_limit)
}

#[test]
fn call_fetch_works() {
    let mut instance = create_function();
    let mut res = call_fetch(&mut instance, &mock_get_request("/dog"))
        .unwrap()
        .unwrap();
    assert_eq!(res.status_code(), 200);

    let json = res.json::<Value>().unwrap();
    println!("{}", json);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
