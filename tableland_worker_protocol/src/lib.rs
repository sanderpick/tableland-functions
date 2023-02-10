mod error;
pub use error::*;
// mod headers;
// pub use headers::*;
pub mod http;
pub use http::*;
pub mod request;
pub use request::*;
pub mod response;
pub use response::*;

// #[doc(hidden)]
// use std::result::Result as StdResult;

// pub type Result<T> = StdResult<T, error::Error>;
