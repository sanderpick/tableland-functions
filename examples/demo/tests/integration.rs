use tableland_std::Response;
use tableland_vm::{
    testing::{fetch, mock_env, mock_instance_with_gas_limit, MockApi},
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
    let res: Response = fetch(&mut instance, mock_env()).unwrap();
    assert_eq!(true, res.data.is_some());

    let report = instance.create_gas_report();
    println!("{:?}", report);
}
