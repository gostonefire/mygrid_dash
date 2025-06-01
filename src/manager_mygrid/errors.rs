use std::fmt::{Display, Formatter};

pub struct MyGridError(String);

impl Display for MyGridError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { 
        write!(f, "MyGridError: {}", self.0) 
    }
}
impl From<std::io::Error> for MyGridError {
    fn from(e: std::io::Error) -> Self { MyGridError(e.to_string()) }
}
impl From<serde_json::Error> for MyGridError {
    fn from(e: serde_json::Error) -> Self { MyGridError(e.to_string()) }
}
impl From<chrono::round::RoundingError> for MyGridError {
    fn from(e: chrono::round::RoundingError) -> Self { MyGridError(e.to_string()) }
}
impl From<glob::PatternError> for MyGridError {
    fn from(e: glob::PatternError) -> Self { MyGridError(e.to_string()) }
}