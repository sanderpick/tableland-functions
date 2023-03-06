use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum StoreError {
    #[error("VM error: {0}")]
    Vm(String),
    #[error("Function error: {0}")]
    Func(String),
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("IPFS error: {0}")]
    Ipfs(String),
    #[error("WASM cache error: {0}")]
    Cache(String),
    #[error("Tokie task join error: {0}")]
    TaskJoin(String),
}

impl StoreError {
    pub(crate) fn func_err(msg: impl Into<String>) -> Self {
        StoreError::Func(msg.into())
    }

    pub(crate) fn cache_err(msg: impl Into<String>) -> Self {
        StoreError::Cache(msg.into())
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::cache_err(e.to_string())
    }
}

impl From<reqwest::Error> for StoreError {
    fn from(e: reqwest::Error) -> Self {
        StoreError::Ipfs(e.to_string())
    }
}

impl From<tableland_vm::VmError> for StoreError {
    fn from(e: tableland_vm::VmError) -> Self {
        StoreError::Vm(e.to_string())
    }
}

impl From<tokio::task::JoinError> for StoreError {
    fn from(e: tokio::task::JoinError) -> Self {
        StoreError::TaskJoin(e.to_string())
    }
}
