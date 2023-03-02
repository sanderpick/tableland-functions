use serde::{Deserialize, Serialize};

/// This is the final result type that is created and serialized in a function for
/// every fetch call. The VM then deserializes this type to distinguish
/// between successful and failed executions.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FuncResult<S> {
    Ok(S),
    /// An error type that every custom error created by developers can be converted to.
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

impl<S: std::fmt::Debug> FuncResult<S> {
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
