use serde_json::Value;
use tableland_vm::{
    testing::{fetch, mock_get_request, mock_instance_with_gas_limit, MockApi},
    Instance,
};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/json.wasm");
const MOCK_QUERY_RESPONSE: &[u8] = include_bytes!("../testdata/response.json");

fn create_function() -> Instance<MockApi> {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    mock_instance_with_gas_limit(WASM, gas_limit, MOCK_QUERY_RESPONSE.to_vec())
}

#[test]
fn call_fetch_works() {
    let mut instance = create_function();
    let mut res = fetch(&mut instance, mock_get_request("/dog")).unwrap();
    assert_eq!(res.status_code(), 200);

    let json = res.json::<Value>().unwrap();
    println!("{}", json);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
