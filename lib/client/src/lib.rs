mod chains;
mod client;
mod errors;
pub mod testing;

pub use chains::{get_chain, Chain, ChainID};
pub use client::TablelandClient;
pub use errors::ClientError;

use async_trait::async_trait;
use serde_json::Value;
use tableland_client_types::ReadOptions;

#[async_trait]
pub trait Tableland: Clone + Send {
    /// Creates a new Tableland client.
    fn new(chain_id: ChainID) -> Self;

    /// Performs a Tableland read query.
    #[cfg(not(feature = "blocking"))]
    async fn read(
        &self,
        statement: &str,
        options: ReadOptions,
    ) -> Result<(Value, u64), ClientError>;

    /// Performs a blocking Tableland read query.
    #[cfg(feature = "blocking")]
    fn read(&self, statement: &str, options: ReadOptions) -> Result<(Value, u64), ClientError>;

    /// Returns the clien't Chain.
    fn chain(&self) -> Chain;
}
