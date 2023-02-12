use tableland_client::{chains::ChainID, TablelandClient};
use types::*;
pub mod bindings;
pub mod types;

fn log(msg: String) {
    println!("Provider log: {}", msg);
}

async fn query(statement: String, options: ReadOptions) -> Result<serde_json::Value, Error> {
    let client = TablelandClient::new(ChainID::Local);
    let result = client.read(statement.as_str(), options).await;
    return match result {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Internal(e.to_string())),
    };
}
