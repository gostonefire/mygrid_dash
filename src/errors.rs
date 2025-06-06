use std::fmt;
use std::fmt::Formatter;
use log4rs::config::runtime::ConfigErrors;
use log::SetLoggerError;
use crate::manager_fox_cloud::errors::FoxError;
use crate::manager_mygrid::errors::MyGridError;

/// Error representing an unrecoverable error that will halt the application
///
#[derive(Debug)]
pub struct UnrecoverableError(pub String);
impl fmt::Display for UnrecoverableError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "UnrecoverableError: {}", self.0)
    }
}
impl From<ConfigError> for UnrecoverableError {
    fn from(e: ConfigError) -> Self {
        UnrecoverableError(e.to_string())
    }
}
impl From<std::io::Error> for UnrecoverableError {
    fn from(e: std::io::Error) -> Self {
        UnrecoverableError(e.to_string())
    }
}
impl From<rustls_pki_types::pem::Error> for UnrecoverableError {
    fn from(e: rustls_pki_types::pem::Error) -> Self { UnrecoverableError(e.to_string()) }
}
impl From<rustls::Error> for UnrecoverableError {
    fn from(e: rustls::Error) -> Self { UnrecoverableError(e.to_string()) }
}


/// Errors while managing configuration
///
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ConfigError: {}", self.0)
    }
}
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<SetLoggerError> for ConfigError {
    fn from(e: SetLoggerError) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<ConfigErrors> for ConfigError {
    fn from(e: ConfigErrors) -> Self {
        ConfigError(e.to_string())
    }
}
impl From<&str> for ConfigError {
    fn from(e: &str) -> Self { ConfigError(e.to_string()) }
}

pub struct DispatcherError(pub String);
impl fmt::Display for DispatcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DispatcherError: {}", self.0)
    }
}
impl From<MyGridError> for DispatcherError {
    fn from(e: MyGridError) -> Self { DispatcherError(e.to_string()) }
}
impl<T> From<tokio::sync::mpsc::error::SendError<T>> for DispatcherError {
    fn from(e: tokio::sync::mpsc::error::SendError<T>) -> Self {
        DispatcherError(e.to_string())
    }
}
impl From<&str> for DispatcherError {
    fn from(e: &str) -> Self { DispatcherError(e.to_string()) }
}
impl From<FoxError> for DispatcherError {
    fn from(e: FoxError) -> Self { DispatcherError(e.to_string()) } 
}
impl From<chrono::format::ParseError> for DispatcherError {
    fn from(e: chrono::format::ParseError) -> Self { DispatcherError(e.to_string()) }
}
impl From<chrono::round::RoundingError> for DispatcherError {
    fn from(e: chrono::round::RoundingError) -> Self { DispatcherError(e.to_string()) }
}
impl From<serde_json::Error> for DispatcherError {
    fn from(e: serde_json::Error) -> Self { DispatcherError(e.to_string()) }
}

pub struct WeatherError(pub String);
impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "WeatherError: {}", self.0)
    }
}
impl From<&str> for WeatherError {
    fn from(e: &str) -> Self { WeatherError(e.to_string()) }
}