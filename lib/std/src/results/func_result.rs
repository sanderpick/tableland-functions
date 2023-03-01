use serde::{Deserialize, Serialize};
use std::fmt;

/// This is the final result type that is created and serialized in a function for
/// every fetch call. The VM then deserializes this type to distinguish
/// between successful and failed executions.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the Tableland VM needs to create the same JSON representation.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use tableland_std::{to_vec, FuncResult, Response};
/// let response: Response = Response::default();
/// let result: FuncResult<Response> = FuncResult::Ok(response);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#);
/// ```
///
/// Failure:
///
/// ```
/// # use tableland_std::{to_vec, FuncResult, Response};
/// let error_msg = String::from("Something went wrong");
/// let result: FuncResult<Response> = FuncResult::Err(error_msg);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"error":"Something went wrong"}"#);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FuncResult<S> {
    Ok(S),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl<S> FuncResult<S> {
    /// Converts a `FuncResult<S>` to a `Result<S, String>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<S, String> {
        Result::<S, String>::from(self)
    }

    pub fn unwrap(self) -> S {
        self.into_result().unwrap()
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, FuncResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, FuncResult::Err(_))
    }
}

impl<S: fmt::Debug> FuncResult<S> {
    pub fn unwrap_err(self) -> String {
        self.into_result().unwrap_err()
    }
}

impl<S, E: ToString> From<Result<S, E>> for FuncResult<S> {
    fn from(original: Result<S, E>) -> FuncResult<S> {
        match original {
            Ok(value) => FuncResult::Ok(value),
            Err(err) => FuncResult::Err(err.to_string()),
        }
    }
}

impl<S> From<FuncResult<S>> for Result<S, String> {
    fn from(original: FuncResult<S>) -> Result<S, String> {
        match original {
            FuncResult::Ok(value) => Ok(value),
            FuncResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec, Response, StdError, StdResult};

    #[test]
    fn contract_result_serialization_works() {
        let result = FuncResult::Ok(12);
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":12}");

        let result = FuncResult::Ok("foo");
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":\"foo\"}");

        let result: FuncResult<Response> = FuncResult::Ok(Response::default());
        assert_eq!(
            to_vec(&result).unwrap(),
            br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#
        );

        let result: FuncResult<Response> = FuncResult::Err("broken".to_string());
        assert_eq!(&to_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn contract_result_deserialization_works() {
        let result: FuncResult<u64> = from_slice(br#"{"ok":12}"#).unwrap();
        assert_eq!(result, FuncResult::Ok(12));

        let result: FuncResult<String> = from_slice(br#"{"ok":"foo"}"#).unwrap();
        assert_eq!(result, FuncResult::Ok("foo".to_string()));

        let result: FuncResult<Response> =
            from_slice(br#"{"ok":{"messages":[],"attributes":[],"events":[],"data":null}}"#)
                .unwrap();
        assert_eq!(result, FuncResult::Ok(Response::default()));

        let result: FuncResult<Response> = from_slice(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, FuncResult::Err("broken".to_string()));

        // ignores whitespace
        let result: FuncResult<u64> = from_slice(b" {\n\t  \"ok\": 5898\n}  ").unwrap();
        assert_eq!(result, FuncResult::Ok(5898));

        // fails for additional attributes
        let parse: StdResult<FuncResult<u64>> = from_slice(br#"{"unrelated":321,"ok":4554}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<FuncResult<u64>> = from_slice(br#"{"ok":4554,"unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<FuncResult<u64>> =
            from_slice(br#"{"ok":4554,"error":"What's up now?"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_from_core_result() {
        let original: Result<Response, StdError> = Ok(Response::default());
        let converted: FuncResult<Response> = original.into();
        assert_eq!(converted, FuncResult::Ok(Response::default()));

        let original: Result<Response, StdError> = Err(StdError::generic_err("broken"));
        let converted: FuncResult<Response> = original.into();
        assert_eq!(
            converted,
            FuncResult::Err("Generic error: broken".to_string())
        );
    }

    #[test]
    fn can_convert_to_core_result() {
        let original = FuncResult::Ok(Response::default());
        let converted: Result<Response, String> = original.into();
        assert_eq!(converted, Ok(Response::default()));

        let original = FuncResult::Err("went wrong".to_string());
        let converted: Result<Response, String> = original.into();
        assert_eq!(converted, Err("went wrong".to_string()));
    }
}
