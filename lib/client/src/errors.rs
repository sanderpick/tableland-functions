#[derive(Debug)]
pub enum ClientError {
    Request(reqwest::Error),
    NoContentLength,
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Request(e) => write!(f, "{}", e),
            ClientError::NoContentLength => write!(f, "No content-length header"),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(value: reqwest::Error) -> Self {
        ClientError::Request(value)
    }
}
