use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Response {
    /// The binary payload to include in the response.
    pub data: Option<ByteBuf>,
}

impl Default for Response {
    fn default() -> Self {
        Response { data: None }
    }
}

impl Response {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the binary data included in the response.
    pub fn set_data(mut self, data: impl Into<ByteBuf>) -> Self {
        self.data = Some(data.into());
        self
    }
}
