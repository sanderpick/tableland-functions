// use serde::de::DeserializeOwned;
use wasmer::Value;

use tableland_std::{ContractResult, Env, Response};
// #[cfg(feature = "stargate")]
// use cosmwasm_std::{
//     Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
//     IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
//     IbcReceiveResponse,
// };

use crate::backend::BackendApi;
use crate::conversion::ref_to_u32;
use crate::errors::{VmError, VmResult};
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

/// The limits in here protect the host from allocating an unreasonable amount of memory
/// and copying an unreasonable amount of data.
///
/// A JSON deserializer would want to set the limit to a much smaller value because
/// deserializing JSON is more expensive. As a consequence, any sane contract should hit
/// the deserializer limit before the read limit.
mod read_limits {
    /// A mibi (mega binary)
    const MI: usize = 1024 * 1024;
    pub const RESULT_QUERY: usize = 64 * MI;
}

/// The limits for the JSON deserialization.
///
/// Those limits are not used when the Rust JSON deserializer is bypassed by using the
/// public `call_*_raw` functions directly.
mod deserialization_limits {
    /// A kibi (kilo binary)
    const KI: usize = 1024;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 256 * KI;
}

pub fn call_fetch<A>(instance: &mut Instance<A>, env: &Env) -> VmResult<ContractResult<Response>>
where
    A: BackendApi + 'static,
{
    let env = to_vec(env)?;
    // let info = to_vec(info)?;
    let data = call_fetch_raw(instance, &env)?;
    let result: ContractResult<Response> = from_slice(&data, deserialization_limits::RESULT_QUERY)?;
    // Ensure query response is valid JSON
    // if let ContractResult::Ok(binary_response) = &result {
    //     serde_json::from_slice::<serde_json::Value>(binary_response.as_slice()).map_err(|e| {
    //         VmError::generic_err(format!("Query response must be valid JSON. {}", e))
    //     })?;
    // }

    Ok(result)
}

// /// Calls Wasm export "instantiate" and returns raw data from the contract.
// /// The result is length limited to prevent abuse but otherwise unchecked.
// pub fn call_instantiate_raw<A, S, Q>(
//     instance: &mut Instance<A, S, Q>,
//     env: &[u8],
//     info: &[u8],
//     msg: &[u8],
// ) -> VmResult<Vec<u8>>
// where
//     A: BackendApi + 'static,
//     S: Storage + 'static,
//     Q: Querier + 'static,
// {
//     instance.set_storage_readonly(false);
//     call_raw(
//         instance,
//         "instantiate",
//         &[env, info, msg],
//         read_limits::RESULT_INSTANTIATE,
//     )
// }

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_fetch_raw<A>(instance: &mut Instance<A>, env: &[u8]) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    // S: Storage + 'static,
    // Q: Querier + 'static,
{
    // instance.set_storage_readonly(true);
    call_raw(instance, "fetch", &[env], read_limits::RESULT_QUERY)
}

/// Calls a function with the given arguments.
/// The exported function must return exactly one result (an offset to the result Region).
pub(crate) fn call_raw<A>(
    instance: &mut Instance<A>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
{
    let mut arg_region_ptrs = Vec::<Value>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr.into());
    }
    let result = instance.call_function1(name, &arg_region_ptrs)?;
    let res_region_ptr = ref_to_u32(&result)?;
    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::testing::{mock_env, mock_info, mock_instance};
//     use cosmwasm_std::{coins, Empty};
//     use wasmer::Store;
//
//     static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
//
//     #[test]
//     fn call_instantiate_works() {
//         let store = Store::default();
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//     }
//
//     #[test]
//     fn call_execute_works() {
//         let store = Store::default();
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         // execute
//         let info = mock_info("verifies", &coins(15, "earth"));
//         let msg = br#"{"release":{}}"#;
//         call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//     }
//
//     #[test]
//     fn call_migrate_works() {
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         // change the verifier via migrate
//         let msg = br#"{"verifier": "someone else"}"#;
//         let _res = call_migrate::<_, _, _, Empty>(&mut instance, &mock_env(), msg);
//
//         // query the new_verifier with verifier
//         let msg = br#"{"verifier":{}}"#;
//         let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
//         let query_response = contract_result.unwrap();
//         assert_eq!(
//             query_response.as_slice(),
//             b"{\"verifier\":\"someone else\"}"
//         );
//     }
//
//     #[test]
//     fn call_query_works() {
//         let mut instance = mock_instance(CONTRACT, &[]);
//
//         // init
//         let info = mock_info("creator", &coins(1000, "earth"));
//         let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
//         call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
//             .unwrap()
//             .unwrap();
//
//         // query
//         let msg = br#"{"verifier":{}}"#;
//         let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
//         let query_response = contract_result.unwrap();
//         assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");
//     }
//
//     #[cfg(feature = "stargate")]
//     mod ibc {
//         use super::*;
//         use crate::calls::{call_instantiate, call_reply};
//         use crate::testing::{
//             mock_env, mock_info, mock_instance, MockApi, MockQuerier, MockStorage,
//         };
//         use cosmwasm_std::testing::{
//             mock_ibc_channel_close_init, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
//             mock_ibc_packet_ack, mock_ibc_packet_recv, mock_ibc_packet_timeout, mock_wasmd_attr,
//         };
//         use cosmwasm_std::{
//             Empty, Event, IbcAcknowledgement, IbcOrder, Reply, ReplyOn, SubMsgResponse,
//             SubMsgResult,
//         };
//         static CONTRACT: &[u8] = include_bytes!("../testdata/ibc_reflect.wasm");
//         const IBC_VERSION: &str = "ibc-reflect-v1";
//         fn setup(
//             instance: &mut Instance<MockApi, MockStorage, MockQuerier>,
//             channel_id: &str,
//             account: &str,
//         ) {
//             // init
//             let info = mock_info("creator", &[]);
//             let msg = br#"{"reflect_code_id":77}"#;
//             call_instantiate::<_, _, _, Empty>(&mut store, instance, &mock_env(), &info, msg)
//                 .unwrap()
//                 .unwrap();
//             // first we try to open with a valid handshake
//             let handshake_open =
//                 mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_VERSION);
//             call_ibc_channel_open(instance, &mock_env(), &handshake_open)
//                 .unwrap()
//                 .unwrap();
//             // then we connect (with counter-party version set)
//             let handshake_connect =
//                 mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_VERSION);
//             let res: IbcBasicResponse = call_ibc_channel_connect::<_, _, _, Empty>(
//                 instance,
//                 &mock_env(),
//                 &handshake_connect,
//             )
//             .unwrap()
//             .unwrap();
//             assert_eq!(1, res.messages.len());
//             assert_eq!(
//                 res.events,
//                 [Event::new("ibc").add_attribute("channel", "connect")]
//             );
//             assert_eq!(ReplyOn::Success, res.messages[0].reply_on);
//             let id = res.messages[0].id;
//             let event = Event::new("instantiate").add_attributes(vec![
//                 // We have to force this one to avoid the debug assertion against _
//                 mock_wasmd_attr("_contract_address", account),
//             ]);
//             // which creates a reflect account. here we get the callback
//             let response = Reply {
//                 id,
//                 result: SubMsgResult::Ok(SubMsgResponse {
//                     events: vec![event],
//                     data: None,
//                 }),
//             };
//             call_reply::<_, _, _, Empty>(&mut store, instance, &mock_env(), &response).unwrap();
//         }
//         const CHANNEL_ID: &str = "channel-123";
//         const ACCOUNT: &str = "account-456";
//         #[test]
//         fn call_ibc_channel_open_and_connect_works() {
//             let mut instance = mock_instance(CONTRACT, &[]);
//             setup(&mut instance, CHANNEL_ID, ACCOUNT);
//         }
//         #[test]
//         fn call_ibc_channel_close_works() {
//             let mut instance = mock_instance(CONTRACT, &[]);
//             setup(&mut instance, CHANNEL_ID, ACCOUNT);
//             let handshake_close =
//                 mock_ibc_channel_close_init(CHANNEL_ID, IbcOrder::Ordered, IBC_VERSION);
//             call_ibc_channel_close::<_, _, _, Empty>(
//                 &mut store,
//                 &mut instance,
//                 &mock_env(),
//                 &handshake_close,
//             )
//             .unwrap()
//             .unwrap();
//         }
//         #[test]
//         fn call_ibc_packet_ack_works() {
//             let mut instance = mock_instance(CONTRACT, &[]);
//             setup(&mut instance, CHANNEL_ID, ACCOUNT);
//             let ack = IbcAcknowledgement::new(br#"{}"#);
//             let msg = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#, ack).unwrap();
//             call_ibc_packet_ack::<_, _, _, Empty>(&mut store, &mut instance, &mock_env(), &msg)
//                 .unwrap()
//                 .unwrap();
//         }
//         #[test]
//         fn call_ibc_packet_timeout_works() {
//             let mut instance = mock_instance(CONTRACT, &[]);
//             setup(&mut instance, CHANNEL_ID, ACCOUNT);
//             let msg = mock_ibc_packet_timeout(CHANNEL_ID, br#"{}"#).unwrap();
//             call_ibc_packet_timeout::<_, _, _, Empty>(&mut store, &mut instance, &mock_env(), &msg)
//                 .unwrap()
//                 .unwrap();
//         }
//         #[test]
//         fn call_ibc_packet_receive_works() {
//             let mut instance = mock_instance(CONTRACT, &[]);
//             setup(&mut instance, CHANNEL_ID, ACCOUNT);
//             let who_am_i = br#"{"who_am_i":{}}"#;
//             let msg = mock_ibc_packet_recv(CHANNEL_ID, who_am_i).unwrap();
//             call_ibc_packet_receive::<_, _, _, Empty>(&mut store, &mut instance, &mock_env(), &msg)
//                 .unwrap()
//                 .unwrap();
//         }
//     }
// }
