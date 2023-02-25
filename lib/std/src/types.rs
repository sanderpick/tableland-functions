use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Env {
    pub block: BlockInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockInfo {
    /// The height of a block is the number of blocks preceding it in the blockchain.
    pub height: u64,
    pub chain_id: String,
}
