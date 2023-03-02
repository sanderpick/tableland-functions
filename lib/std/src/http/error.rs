use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Internal error: {msg}")]
    InternalErr { msg: String },
    #[error("Bad encoding: {msg}")]
    BadEncoding { msg: String },
    #[error("Error parsing into type {target_type}: {msg}")]
    ParseErr { target_type: String, msg: String },
}

impl Error {
    pub fn internal_err(msg: impl Into<String>) -> Self {
        Error::InternalErr { msg: msg.into() }
    }

    pub fn bad_encoding(msg: impl Into<String>) -> Self {
        Error::BadEncoding { msg: msg.into() }
    }

    pub fn parse_err(target: impl Into<String>, msg: impl ToString) -> Self {
        Error::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
        }
    }
}

impl From<std::string::String> for Error {
    fn from(s: String) -> Self {
        Error::internal_err(s)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::internal_err(e.to_string())
    }
}
