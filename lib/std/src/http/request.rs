use http::{header::HeaderMap, method::Method, uri::Uri};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

/// A [Request](https://developer.mozilla.org/en-US/docs/Web/API/Request) representation for
/// handling incoming and creating outbound HTTP requests.
/// Inspired by https://github.com/cloudflare/workers-rs/blob/main/worker/src/request.rs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Request {
    /// Target function content identifier.
    id: String,
    #[serde(
        deserialize_with = "super::serde::deserialize_http_method",
        serialize_with = "super::serde::serialize_http_method"
    )]
    method: Method,
    #[serde(
        deserialize_with = "super::serde::deserialize_uri",
        serialize_with = "super::serde::serialize_uri"
    )]
    uri: Uri,
    #[serde(
        deserialize_with = "super::serde::deserialize_header_map",
        serialize_with = "super::serde::serialize_header_map"
    )]
    headers: HeaderMap,
    body: Option<ByteBuf>,
}

impl Request {
    /// Construct a new `Request` with an HTTP Method.
    pub fn new(
        id: String,
        uri: Uri,
        method: Method,
        headers: HeaderMap,
        body: Option<ByteBuf>,
    ) -> Self {
        Request {
            id,
            uri,
            method,
            headers,
            body,
        }
    }

    /// Get the target function content identifier.
    pub fn id(&self) -> String {
        self.id.clone()
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

    /// The Body of the Request.
    pub fn body(&self) -> Option<ByteBuf> {
        self.body.clone()
    }
}
