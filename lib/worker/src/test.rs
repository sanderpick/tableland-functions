use serde_json::Value;
use tableland_vm::call_fetch;
use tableland_vm::testing::mock_get_request;

use crate::instance::instance_with_gas_limit;

const EXAMPLE_JSON: &[u8] =
    include_bytes!("../../../examples/json/target/wasm32-unknown-unknown/release/json.wasm");
const EXAMPLE_HTML: &[u8] =
    include_bytes!("../../../examples/html/target/wasm32-unknown-unknown/release/html.wasm");
const EXAMPLE_SVG: &[u8] =
    include_bytes!("../../../examples/svg/target/wasm32-unknown-unknown/release/svg.wasm");

#[test]
fn call_fetch_json_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut instance = instance_with_gas_limit(EXAMPLE_JSON, gas_limit);

    let mut res = call_fetch(&mut instance, &mock_get_request("/dog"))
        .unwrap()
        .unwrap();
    assert_eq!(res.status_code(), 200);

    let json = res.json::<Value>().unwrap();
    println!("{}", json);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}

#[test]
fn call_fetch_html_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut instance = instance_with_gas_limit(EXAMPLE_HTML, gas_limit);

    let mut res = call_fetch(&mut instance, &mock_get_request("/bird"))
        .unwrap()
        .unwrap();
    assert_eq!(res.status_code(), 200);

    let html = res.text().unwrap();
    println!("{}", html);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}

#[test]
fn call_fetch_svg_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut instance = instance_with_gas_limit(EXAMPLE_SVG, gas_limit);

    let mut res = call_fetch(&mut instance, &mock_get_request("/3"))
        .unwrap()
        .unwrap();
    assert_eq!(res.status_code(), 200);

    let svg = res.text().unwrap();
    println!("{}", svg);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
