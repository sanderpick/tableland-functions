use fp_bindgen::prelude::Serializable;

/// A [`Method`](https://developer.mozilla.org/en-US/docs/Web/API/Request/method) representation
/// used on Request objects.
/// Taken from https://github.com/cloudflare/workers-rs/blob/main/worker/src/http.rs.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Serializable)]
pub enum Method {
    Head,
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Options,
    Connect,
    Trace,
}

impl Method {
    pub fn all() -> Vec<Method> {
        vec![
            Method::Head,
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Patch,
            Method::Delete,
            Method::Options,
            Method::Connect,
            Method::Trace,
        ]
    }
}

impl From<String> for Method {
    fn from(m: String) -> Self {
        match m.to_ascii_uppercase().as_str() {
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            "OPTIONS" => Method::Options,
            "CONNECT" => Method::Connect,
            "TRACE" => Method::Trace,
            _ => Method::Get,
        }
    }
}

impl From<Method> for String {
    fn from(val: Method) -> Self {
        val.as_ref().to_string()
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &'static str {
        match self {
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
            Method::Options => "OPTIONS",
            Method::Connect => "CONNECT",
            Method::Trace => "TRACE",
            Method::Get => "GET",
        }
    }
}

impl ToString for Method {
    fn to_string(&self) -> String {
        (*self).clone().into()
    }
}

impl Default for Method {
    fn default() -> Self {
        Method::Get
    }
}

// use super::Body;
//
// #[derive(Serializable)]
// pub struct Request {
//     pub url: Uri,
//
//     /// HTTP method to use for the request.
//     pub method: Method,
//
//     /// HTTP headers to submit with the request.
//     pub headers: http::HeaderMap,
//
//     /// The body to submit with the request.
//     #[fp(skip_serializing_if = "Option::is_none")]
//     pub body: Option<Body>,
// }
//
// /// Represents an HTTP response we received.
// ///
// /// Please note we currently do not support streaming responses.
// #[derive(Serializable)]
// pub struct Response {
//     /// The response body. May be empty.
//     pub body: Body,
//
//     /// HTTP headers that were part of the response.
//     pub headers: http::HeaderMap,
//
//     /// HTTP status code.
//     pub status_code: u16,
// }
//
// /// Represents an error that occurred while attempting to submit the request.
// #[derive(Serializable)]
// #[fp(tag = "type", rename_all = "snake_case")]
// pub enum RequestError {
//     /// Used when we know we don't have an active network connection.
//     Offline,
//     NoRoute,
//     ConnectionRefused,
//     Timeout,
//     #[fp(rename_all = "snake_case")]
//     ServerError {
//         /// HTTP status code.
//         status_code: u16,
//         /// Response body.
//         response: Body,
//     },
//     /// Misc.
//     #[fp(rename = "other/misc")]
//     Other {
//         reason: String,
//     },
// }
//
// #[derive(Debug, Clone, Serializable)]
// pub struct QueryError;
