use tableland_std::{from_binary, from_slice, Response};
use tableland_vm::{
    testing::{fetch, mock_env, mock_instance_with_gas_limit, MockApi},
    Instance,
};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/demo.wasm");

/// Instantiates a contract with no elements
fn create_contract() -> Instance<MockApi> {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut deps = mock_instance_with_gas_limit(WASM, gas_limit);
    // let creator = String::from("creator");
    // let info = mock_info(&creator, &[]);
    let res: Response = fetch(&mut deps, mock_env()).unwrap();
    assert_eq!(0, res.data.is_none());
    deps
}

// fn get_count(deps: &mut Instance<MockApi, MockStorage, MockQuerier>) -> u32 {
//     let data = query(deps, mock_env(), QueryMsg::Count {}).unwrap();
//     let res: CountResponse = from_binary(&data).unwrap();
//     res.count
// }
//
// fn get_sum(deps: &mut Instance<MockApi, MockStorage, MockQuerier>) -> i32 {
//     let data = query(deps, mock_env(), QueryMsg::Sum {}).unwrap();
//     let res: SumResponse = from_binary(&data).unwrap();
//     res.sum
// }

#[test]
fn basic_fetch() {
    let (mut deps, _) = create_contract();
    // assert_eq!(get_count(&mut deps), 0);
    // assert_eq!(get_sum(&mut deps), 0);
}
