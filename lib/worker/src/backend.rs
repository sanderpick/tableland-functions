use serde_json::Value;
use tableland_client::{chains::ChainID, TablelandClient};
use tableland_client_types::ReadOptions;
use tableland_vm::{BackendApi, BackendError, BackendResult, GasInfo};

/// Base gas per request
const GAS_COST_QUERY_FLAT: u64 = 100_000;
/// Gas per request byte
const GAS_COST_QUERY_REQUEST_MULTIPLIER: u64 = 0;
/// Gas per reponse byte
const GAS_COST_QUERY_RESPONSE_MULTIPLIER: u64 = 100;

#[derive(Clone)]
pub struct Api {
    client: TablelandClient,
}

impl Api {
    pub(crate) fn new() -> Self {
        let client = TablelandClient::new(ChainID::Local);
        Api { client }
    }
}

impl BackendApi for Api {
    fn read(&self, statement: &str, options: ReadOptions, gas_limit: u64) -> BackendResult<Value> {
        let mut gas_info = GasInfo::with_externally_used(
            GAS_COST_QUERY_FLAT + (GAS_COST_QUERY_REQUEST_MULTIPLIER * (statement.len() as u64)),
        );
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        let (val, len) = match self.client.read(statement, options) {
            Ok(res) => res,
            Err(e) => return (Err(BackendError::UserErr { msg: e.to_string() }), gas_info),
        };

        gas_info.externally_used += GAS_COST_QUERY_RESPONSE_MULTIPLIER * len;
        if gas_info.externally_used > gas_limit {
            return (Err(BackendError::out_of_gas()), gas_info);
        }

        (Ok(val), gas_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;

    #[test]
    fn read_works() {
        let api = Api::new();
        api.read(
            "select * from my_table",
            ReadOptions::default(),
            DEFAULT_QUERY_GAS_LIMIT,
        )
        .0
        .unwrap();
    }
}
