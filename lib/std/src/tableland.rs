use serde::{Deserialize, Serialize};
use tableland_client_types::ReadOptions;

/// An internal wrapper for Tableland read queries.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReadRequest {
    pub stm: String,
    pub opts: ReadOptions,
}
