use fp_bindgen::prelude::Serializable;
use http::{HeaderMap, Method, Uri};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

/// A [Request](https://developer.mozilla.org/en-US/docs/Web/API/Request) representation for
/// handling incoming and creating outbound HTTP requests.
/// Taken from https://github.com/cloudflare/workers-rs/blob/main/worker/src/request.rs.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Serializable)]
#[fp(rust_module = "tableland_worker_protocol")]
pub struct Request {
    #[serde(
        deserialize_with = "fp_bindgen_support::http::deserialize_http_method",
        serialize_with = "fp_bindgen_support::http::serialize_http_method"
    )]
    method: Method,
    #[serde(
        deserialize_with = "fp_bindgen_support::http::deserialize_uri",
        serialize_with = "fp_bindgen_support::http::serialize_uri"
    )]
    uri: Uri,
    #[serde(
        deserialize_with = "fp_bindgen_support::http::deserialize_header_map",
        serialize_with = "fp_bindgen_support::http::serialize_header_map"
    )]
    headers: HeaderMap,
    body: Option<ByteBuf>,
}

impl Request {
    /// Construct a new `Request` with an HTTP Method.
    pub fn new(uri: Uri, method: Method, headers: HeaderMap, body: Option<ByteBuf>) -> Self {
        Request {
            uri,
            method,
            headers,
            body,
        }
    }

    /// Get the `Headers` for this request.
    pub fn headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    /// The HTTP Method associated with this `Request`.
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// The URI of this `Request`.
    pub fn uri(&self) -> Uri {
        self.uri.clone()
    }

    pub fn body(&self) -> Option<ByteBuf> {
        self.body.clone()
    }
}
