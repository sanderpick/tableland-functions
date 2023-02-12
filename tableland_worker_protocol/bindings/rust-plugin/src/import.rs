use crate::types::*;

/// Logs a message to the (development) console.
#[fp_bindgen_support::fp_import_signature]
pub fn log(message: String);

/// Tableland query endpoint for plugins.
#[fp_bindgen_support::fp_import_signature]
pub async fn query(statement: String, options: ReadOptions) -> Result<serde_json::Value, Error>;
