use fp_bindgen::prelude::Serializable;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
// use http::{Uri, HeaderMap};
use super::Method;
// use super::Result;

/// A [Request](https://developer.mozilla.org/en-US/docs/Web/API/Request) representation for
/// handling incoming and creating outbound HTTP requests.
/// Taken from https://github.com/cloudflare/workers-rs/blob/main/worker/src/request.rs.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "tableland_worker_protocol")]
pub struct Request {
    method: Method,
    path: String,
    // headers: Headers,
    body: Option<ByteBuf>,
}

impl Request {
    /// Construct a new `Request` with an HTTP Method.
    pub fn new(path: &str, method: Method, body: Option<ByteBuf>) -> Self {
        Request {
            method,
            path: path.to_string(),
            body,
        }
    }

    /// Get the `Headers` for this request.
    // pub fn headers(&self) -> &Headers {
    //     &self.headers
    // }

    /// The HTTP Method associated with this `Request`.
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// The URL Path of this `Request`.
    pub fn path(&self) -> String {
        self.path.clone()
    }

    pub fn body(&self) -> Option<ByteBuf> {
        self.body.clone()
    }
}
