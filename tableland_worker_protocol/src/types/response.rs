use fp_bindgen::prelude::Serializable;
use serde_bytes::ByteBuf;
// use crate::cors::Cors;
use crate::types::error::Error;
// use crate::types::headers::Headers;
use super::Result;

#[derive(Debug, Serializable)]
pub enum ResponseBody {
    Empty,
    Body(ByteBuf),
}

const CONTENT_TYPE: &str = "content-type";

/// A [Response](https://developer.mozilla.org/en-US/docs/Web/API/Response) representation for
/// working with or returning a response to a `Request`.
#[derive(Debug, Serializable)]
#[fp(rust_module = "tableland_worker_protocol::prelude")]
pub struct Response {
    body: ResponseBody,
    // headers: Headers,
    status_code: u16,
}

impl Response {
    // Create a `Response` using `B` as the body encoded as JSON. Sets the associated
    // `Content-Type` header for the `Response` as `application/json`.
    // pub fn from_json<B: Serialize>(value: &B) -> Result<Self> {
    //     if let Ok(data) = serde_json::to_string(value) {
    //         let mut headers = Headers::new();
    //         headers.set(CONTENT_TYPE, "application/json")?;
    //
    //         return Ok(Self {
    //             body: ResponseBody::Body(data.into_bytes()),
    //             headers,
    //             status_code: 200,
    //             websocket: None,
    //         });
    //     }
    //
    //     Err(Error::Json(("Failed to encode data to json".into(), 500)))
    // }

    /// Create a `Response` using the body encoded as HTML. Sets the associated `Content-Type`
    /// header for the `Response` as `text/html`.
    pub fn from_html(html: impl AsRef<str>) -> Result<Self> {
        // let mut headers = Headers::new();
        // headers.set(CONTENT_TYPE, "text/html")?;

        let data = html.as_ref().as_bytes();
        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(data)),
            // headers,
            status_code: 200,
        })
    }

    /// Create a `Response` using unprocessed bytes provided. Sets the associated `Content-Type`
    /// header for the `Response` as `application/octet-stream`.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        // let mut headers = Headers::new();
        // headers.set(CONTENT_TYPE, "application/octet-stream")?;

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(bytes)),
            // headers,
            status_code: 200,
        })
    }

    /// Create a `Response` using a `ResponseBody` variant. Sets a status code of 200 and an empty
    /// set of Headers. Modify the Response with methods such as `with_status` and `with_headers`.
    pub fn from_body(body: ResponseBody) -> Result<Self> {
        Ok(Self {
            body,
            // headers: Headers::new(),
            status_code: 200,
        })
    }

    /// Create a `Response` using unprocessed text provided. Sets the associated `Content-Type`
    /// header for the `Response` as `text/plain`.
    pub fn ok(body: impl Into<String>) -> Result<Self> {
        // let mut headers = Headers::new();
        // headers.set(CONTENT_TYPE, "text/plain")?;

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(body.into().into_bytes())),
            // headers,
            status_code: 200,
        })
    }

    /// Create an empty `Response` with a 200 status code.
    pub fn empty() -> Result<Self> {
        Ok(Self {
            body: ResponseBody::Empty,
            // headers: Headers::new(),
            status_code: 200,
        })
    }

    /// A helper method to send an error message to a client. Will return `Err` if the status code
    /// provided is outside the valid HTTP error range of 400-599.
    pub fn error(msg: impl Into<String>, status: u16) -> Result<Self> {
        if !(400..=599).contains(&status) {
            return Err(Error::Internal(
                "error status codes must be in the 400-599 range! see https://developer.mozilla.org/en-US/docs/Web/HTTP/Status for more".into(),
            ));
        }

        Ok(Self {
            body: ResponseBody::Body(ByteBuf::from(msg.into().into_bytes())),
            // headers: Headers::new(),
            status_code: status,
        })
    }

    /// Get the HTTP Status code of this `Response`.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    // Access this response's body as plaintext.
    // pub async fn text(&mut self) -> Result<String> {
    //     match &self.body {
    //         ResponseBody::Body(bytes) => {
    //             Ok(String::from_utf8(bytes.clone()).map_err(|e| Error::from(e.to_string()))?)
    //         }
    //         ResponseBody::Empty => Ok(String::new()),
    //     }
    // }

    // Access this response's body encoded as JSON.
    // pub async fn json<B: DeserializeOwned>(&mut self) -> Result<B> {
    //     let content_type = self.headers().get(CONTENT_TYPE)?.unwrap_or_default();
    //     if !content_type.contains("application/json") {
    //         return Err(Error::BadEncoding);
    //     }
    //     serde_json::from_str(&self.text().await?).map_err(Error::from)
    // }

    /// Access this response's body encoded as raw bytes.
    pub async fn bytes(&mut self) -> Result<Vec<u8>> {
        match &self.body {
            ResponseBody::Body(bytes) => Ok(bytes.clone().into_vec()),
            ResponseBody::Empty => Ok(Vec::new()),
        }
    }

    // Set this response's `Headers`.
    // pub fn with_headers(mut self, headers: Headers) -> Self {
    //     self.headers = headers;
    //     self
    // }

    /// Set this response's status code.
    /// The Workers platform will reject HTTP status codes outside the range of 200..599 inclusive,
    /// and will throw a JavaScript `RangeError`, returning a response with an HTTP 500 status code.
    pub fn with_status(mut self, status_code: u16) -> Self {
        self.status_code = status_code;
        self
    }

    // Sets this response's cors headers from the `Cors` struct.
    // Example usage:
    // ```
    // use worker::*;
    // fn fetch() -> worker::Result<Response> {
    //     let cors = Cors::default();
    //     Response::empty()?
    //         .with_cors(&cors)
    // }
    // ```
    // pub fn with_cors(self, cors: &Cors) -> Result<Self> {
    //     let mut headers = self.headers.clone();
    //     cors.apply_headers(&mut headers)?;
    //     Ok(self.with_headers(headers))
    // }

    // Read the `Headers` on this response.
    // pub fn headers(&self) -> &Headers {
    //     &self.headers
    // }

    // Get a mutable reference to the `Headers` on this response.
    // pub fn headers_mut(&mut self) -> &mut Headers {
    //     &mut self.headers
    // }
}
