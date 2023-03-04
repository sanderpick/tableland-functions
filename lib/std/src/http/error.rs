use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Internal error: {msg}")]
    InternalErr { msg: String },
    #[error("Bad encoding: {msg}")]
    BadEncoding { msg: String },
    #[error("Error parsing into type: {msg}")]
    ParseErr { msg: String },
}

impl Error {
    pub fn internal_err(msg: impl Into<String>) -> Self {
        Error::InternalErr { msg: msg.into() }
    }

    pub fn bad_encoding(msg: impl Into<String>) -> Self {
        Error::BadEncoding { msg: msg.into() }
    }

    pub fn parse_err(msg: impl ToString) -> Self {
        Error::ParseErr {
            msg: msg.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::parse_err(e.to_string())
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::bad_encoding(e.to_string())
    }
}
