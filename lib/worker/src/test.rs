use serde_json::Value;
use tableland_client::{testing::MockClient, ChainID, Tableland};
use tableland_vm::{
    call_fetch, check_wasm, testing::mock_get_request, Backend, Instance, InstanceOptions,
};

use crate::backend::Api;
use crate::instance::ApiInstanceOptions;

const EXAMPLE_JSON_WASM: &[u8] =
    include_bytes!("../../../examples/json/target/wasm32-unknown-unknown/release/json.wasm");
const EXAMPLE_HTML_WASM: &[u8] =
    include_bytes!("../../../examples/html/target/wasm32-unknown-unknown/release/html.wasm");
const EXAMPLE_SVG_WASM: &[u8] =
    include_bytes!("../../../examples/svg/target/wasm32-unknown-unknown/release/svg.wasm");

const EXAMPLE_JSON_QUERY_RESPONSE: &[u8] =
    include_bytes!("../../../examples/json/testdata/response.json");
const EXAMPLE_HTML_QUERY_RESPONSE: &[u8] =
    include_bytes!("../../../examples/html/testdata/response.json");
const EXAMPLE_SVG_QUERY_RESPONSE: &[u8] =
    include_bytes!("../../../examples/svg/testdata/response.json");

fn instance_with_gas_limit(
    wasm: &[u8],
    gas_limit: u64,
    client: MockClient,
) -> Instance<Api<MockClient>> {
    let options = ApiInstanceOptions {
        gas_limit,
        ..Default::default()
    };

    check_wasm(wasm, &options.available_capabilities).unwrap();

    let backend = Backend {
        api: Api::new(client),
    };
    let memory_limit = options.memory_limit;
    let options = InstanceOptions {
        gas_limit: options.gas_limit,
        print_debug: options.print_debug,
    };
    Instance::from_code(wasm, backend, options, memory_limit).unwrap()
}

#[test]
fn call_fetch_json_works() {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut client = MockClient::new(ChainID::Local);
    client.respond_with(EXAMPLE_JSON_QUERY_RESPONSE.to_vec());
    let mut instance = instance_with_gas_limit(EXAMPLE_JSON_WASM, gas_limit, client);

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
    let mut client = MockClient::new(ChainID::Local);
    client.respond_with(EXAMPLE_HTML_QUERY_RESPONSE.to_vec());
    let mut instance = instance_with_gas_limit(EXAMPLE_HTML_WASM, gas_limit, client);

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
    let mut client = MockClient::new(ChainID::Local);
    client.respond_with(EXAMPLE_SVG_QUERY_RESPONSE.to_vec());
    let mut instance = instance_with_gas_limit(EXAMPLE_SVG_WASM, gas_limit, client);

    let mut res = call_fetch(&mut instance, &mock_get_request("/3"))
        .unwrap()
        .unwrap();
    assert_eq!(res.status_code(), 200);

    let svg = res.text().unwrap();
    println!("{}", svg);

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
