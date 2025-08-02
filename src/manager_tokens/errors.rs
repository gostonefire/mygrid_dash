use std::fmt;
use std::fmt::Formatter;

/// Errors while managing tokens
///
#[derive(Debug)]
pub enum TokenError {
    InvalidJwt,
    FileIO(String),
    Request(String),
}
impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TokenError::InvalidJwt          => write!(f, "TokenError::InvalidJwt"),
            TokenError::FileIO(e)   => write!(f, "TokenError::File: {}", e),
            TokenError::Request(e)  => write!(f, "TokenError::Request: {}", e),
        }
    }
}
impl From<std::io::Error> for TokenError {
    fn from(e: std::io::Error) -> Self {
        TokenError::FileIO(e.to_string())
    }
}
impl From<serde_json::Error> for TokenError {
    fn from(e: serde_json::Error) -> Self {
        TokenError::FileIO(e.to_string())
    }
}
impl From<reqwest::Error> for TokenError {
    fn from(e: reqwest::Error) -> Self {
        TokenError::Request(e.to_string())
    }
}
impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(_: jsonwebtoken::errors::Error) -> Self { TokenError::InvalidJwt }
}
impl From<&str> for TokenError {
    fn from(e: &str) -> Self { TokenError::Request(e.to_string()) }
}
