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

// impl From<url::ParseError> for Error {
//     fn from(e: url::ParseError) -> Self {
//         Self::RustError(e.to_string())
//     }
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             // Error::BadEncoding => write!(f, "content-type mismatch"),
//             // Error::Json((msg, status)) => write!(f, "{} (status: {})", msg, status),
//             Error::Internal(msg) => write!(f, "{}", msg),
//             // Error::BindingError(name) => write!(f, "no binding found for `{}`", name),
//             // Error::RouteNoDataError => write!(f, "route has no corresponding shared data"),
//             // Error::RustError(msg) => write!(f, "{}", msg),
//             // Error::SerdeJsonError(e) => write!(f, "Serde Error: {}", e),
//         }
//     }
// }
//
// impl std::error::Error for Error {
//     fn description(&self) -> &str {
//         match self {
//             Error::Internal(msg) => msg.as_str(),
//         }
//     }
//
//     fn cause(&self) -> Option<&dyn std::error::Error> {
//         match self {
//             Error::Internal(_) => None,
//         }
//     }
// }
//
// impl From<&str> for Error {
//     fn from(a: &str) -> Self {
//         Error::Internal(a.to_string())
//     }
// }
//
// impl From<String> for Error {
//     fn from(a: String) -> Self {
//         Error::Internal(a)
//     }
// }
//
// impl From<serde_json::Error> for Error {
//     fn from(e: serde_json::Error) -> Self {
//         Error::Internal(e.to_string())
//     }
// }
