use serde_json::{from_slice, to_string, Value};
use tableland_vm::{
    testing::{fetch, mock_instance_with_gas_limit, mock_request, MockApi},
    Instance,
};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/demo.wasm");

fn create_function() -> Instance<MockApi> {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    mock_instance_with_gas_limit(WASM, gas_limit)
}

#[test]
fn call_fetch_works() {
    let mut instance = create_function();
    let res = fetch(&mut instance, mock_request()).unwrap();
    assert_eq!(true, res.data.is_some());

    let data = res.data.unwrap().into_vec();
    let json = from_slice::<Value>(data.as_slice()).unwrap();
    println!("{}", to_string(&json).unwrap());

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
