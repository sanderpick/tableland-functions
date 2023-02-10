use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "tableland_worker_protocol")]
pub enum Error {
    // BadEncoding,
    // Json((String, u16)),
    #[error("internal error: {0}")]
    Internal(String),
    // BindingError(String),
    // RouteNoDataError,
    // RustError(String),
    // SerdeJsonError(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Internal(e.to_string())
    }
}
