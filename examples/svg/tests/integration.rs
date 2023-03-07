use serde_json::Value;
use tableland_vm::testing::{fetch, mock_get_request, mock_instance_with_gas_limit};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/svg.wasm");
const MOCK_QUERY_RESPONSE_METADATA: &[u8] = include_bytes!("../testdata/response1.json");
const MOCK_QUERY_RESPONSE_IMAGE: &[u8] = include_bytes!("../testdata/response2.json");

#[test]
fn call_fetch_metadata_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut instance =
        mock_instance_with_gas_limit(WASM, gas_limit, MOCK_QUERY_RESPONSE_METADATA.to_vec());

    let mut res = fetch(&mut instance, mock_get_request("/3")).unwrap();
    assert_eq!(res.status_code(), 200);

    let json = res.json::<Value>().unwrap();
    println!("{}", json);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}

#[test]
fn call_fetch_image_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut instance =
        mock_instance_with_gas_limit(WASM, gas_limit, MOCK_QUERY_RESPONSE_IMAGE.to_vec());

    let mut res = fetch(&mut instance, mock_get_request("/3/image")).unwrap();
    assert_eq!(res.status_code(), 200);

    let svg = res.text().unwrap();
    println!("{}", svg);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
