use http::header::{HeaderMap, HeaderValue};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_json::Value;

use super::{Error, Result};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ResponseBody {
    Empty,
    Body(ByteBuf),
}

const CONTENT_TYPE: &str = "content-type";

/// A [Response](https://developer.mozilla.org/en-US/docs/Web/API/Response) representation for
/// working with or returning a response to a `Request`.
/// Inspired by https://github.com/cloudflare/workers-rs/blob/main/worker/src/request.rs.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Response {
    body: ResponseBody,
    #[serde(
        deserialize_with = "super::serde::deserialize_header_map",
        serialize_with = "super::serde::serialize_header_map"
    )]
    headers: HeaderMap,
    status_code: u16,
}

impl Response {
    /// Create a `Response` using `B` as the body encoded as JSON. Sets the associated
    /// `Content-Type` header for the `Response` as `application/json`.
    pub fn from_json<B: Serialize>(value: &B) -> Result<Self> {
        if let Ok(data) = serde_json::to_string(value) {
            let mut headers = HeaderMap::new();
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );

            return Ok(Self {
                body: ResponseBody::Body(ByteBuf::from(data)),
                headers,
                status_code: 200,
            });
        }
        Err(Error::bad_encoding("Failed to encode data to json"))
    }

    /// Create a `Response` using the body encoded as HTML. Sets the associated `Content-Type`
    /// header for the `Response` as `text/html`.
    pub fn from_html(html: impl AsRef<str>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/html").unwrap());

        let data = html.as_ref().as_bytes();
        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(data)),
            headers,
            status_code: 200,
        })
    }

    /// Create a `Response` using unprocessed bytes provided. Sets the associated `Content-Type`
    /// header for the `Response` as `application/octet-stream`.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/octet-stream").unwrap(),
        );

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(bytes)),
            headers,
            status_code: 200,
        })
    }

    /// Create a `Response` using a `ResponseBody` variant. Sets a status code of 200 and an empty
    /// set of Headers. Modify the Response with methods such as `with_status` and `with_headers`.
    pub fn from_body(body: ResponseBody) -> Result<Self> {
        Ok(Self {
            body,
            headers: HeaderMap::new(),
            status_code: 200,
        })
    }

    /// Create a `Response` using unprocessed text provided. Sets the associated `Content-Type`
    /// header for the `Response` as `text/plain`.
    pub fn ok(body: impl Into<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_str("text/plain").unwrap());

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(body.into().into_bytes())),
            headers,
            status_code: 200,
        })
    }

    /// Create an empty `Response` with a 200 status code.
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            headers: HeaderMap::new(),
            status_code: 200,
        })
    }

    /// A helper method to send an error message to a client. Will return `Err` if the status code
    /// provided is outside the valid HTTP error range of 400-599.
    pub fn error(msg: impl Into<String>, status: u16) -> Result<Self> {
        if !(400..=599).contains(&status) {
            return Err(Error::internal_err(
                "error status codes must be in the 400-599 range! see https://developer.mozilla.org/en-US/docs/Web/HTTP/Status for more",
            ));
        }

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(msg.into().into_bytes())),
            headers: HeaderMap::new(),
            status_code: status,
        })
    }

    /// Get the HTTP Status code of this `Response`.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Access this response's body as plaintext.
    pub fn text(&mut self) -> Result<String> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(String::from_utf8(bytes.clone().into_vec())?),
            ResponseBody::Empty => Ok(String::new()),
        }
    }

    pub fn json2(&mut self) -> Result<Value> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(serde_json::from_slice(bytes.clone().as_ref())?),
            ResponseBody::Empty => Ok(Value::from("")),
        }
    }

    /// Access this response's body encoded as JSON.
    pub fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
        let content_type = self
            .headers()
            .get(CONTENT_TYPE)
            .ok_or(Error::bad_encoding("no content-type header"))?;
        if content_type.ne(&HeaderValue::from_str("application/json").unwrap()) {
            return Err(Error::bad_encoding("invalid content-type header"));
        }
        match &self.body {
            ResponseBody::Body(bytes) => Ok(serde_json::from_slice(bytes.clone().as_ref())?),
            ResponseBody::Empty => Ok(serde_json::from_slice(Vec::new().as_slice())?),
        }
    }

    /// Access this response's body encoded as raw bytes.
    pub fn bytes(&mut self) -> Result<Vec<u8>> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(bytes.clone().into_vec()),
            ResponseBody::Empty => Ok(Vec::new()),
        }
    }

    /// Set this response's `Headers`.
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Set this response's status code.
    /// The Workers platform will reject HTTP status codes outside the range of 200..599 inclusive,
    /// and will throw a JavaScript `RangeError`, returning a response with an HTTP 500 status code.
    pub fn with_status(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    /// Read the `Headers` on this response.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
}
