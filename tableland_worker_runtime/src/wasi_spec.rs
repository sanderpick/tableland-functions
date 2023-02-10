use tableland_client::*;
use types::*;
pub mod bindings;
pub mod types;

fn log(msg: String) {
    println!("Provider log: {}", msg);
}

async fn query(statement: String) -> Result<serde_json::Value, Error> {
    let client = TablelandClient::new(chains::ChainID::Local);
    let result = client
        .read(statement.as_str(), ReadOptions::default())
        .await;
    return match result {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::Internal(e.to_string())),
    };
}
