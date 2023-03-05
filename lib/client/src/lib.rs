extern crate core;

mod chains;
mod client;
mod errors;
pub mod testing;

pub use chains::{get_chain, ChainID};
pub use client::TablelandClient;
pub use errors::ClientError;

use serde_json::Value;
use tableland_client_types::ReadOptions;

pub trait Tableland: Clone + Send {
    fn new(chain_id: ChainID) -> Self;

    #[cfg(not(feature = "blocking"))]
    async fn read(
        &self,
        statement: &str,
        options: ReadOptions,
    ) -> Result<(Value, u64), ClientError>;

    /// Performs a Tableland read query.
    #[cfg(feature = "blocking")]
    fn read(&self, statement: &str, options: ReadOptions) -> Result<(Value, u64), ClientError>;
}
