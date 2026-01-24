use std::{env, fs};
use std::path::PathBuf;
use chrono::{DateTime, Local};
use jsonwebtoken::jwk::JwkSet;
use serde::{Deserialize, Deserializer};
use serde::de;
use thiserror::Error;
use tracing::level_filters::LevelFilter;
use crate::logging::setup_logger;

#[derive(Deserialize, Clone)]
pub struct Google {
    pub redirect_uri: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
    pub scope: String,
    #[serde(default)]
    pub jwks_uri: String,
    pub jwks: Option<JwkSet>,
    #[serde(default)]
    pub jwks_expire: i64,
    #[serde(default)]
    pub auth_url: String,
    #[serde(default)]
    pub token_url: String,
    pub well_known: String,
    #[serde(default)]
    pub well_known_expire: i64,
    #[serde(default)]
    pub users: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct WebServerParameters {
    pub bind_address: String,
    pub bind_port: u16,
}

#[derive(Deserialize, Clone)]
pub struct FoxESS {
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub inverter_sn: String,
}

#[derive(Deserialize, Clone)]
pub struct MyGrid {
    pub schedule_path: String,
    pub base_data_path: String,
}

#[derive(Deserialize, Clone)]
pub struct Weather {
    pub host: String,
    pub sensor: String,
}

#[derive(Deserialize, Clone)]
pub struct General {
    pub debug_run_time: Option<DateTime<Local>>,
    pub log_path: String,
    pub log_level: LogLevel,
    pub log_to_stdout: bool,
    #[serde(default)]
    pub version: String,
}

#[derive(Clone, Copy, Debug)]
pub struct LogLevel(pub LevelFilter);

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let normalized = raw.trim().to_ascii_lowercase();
        let level = match normalized.as_str() {
            "off" => LevelFilter::OFF,
            "error" => LevelFilter::ERROR,
            "warn" | "warning" => LevelFilter::WARN,
            "info" => LevelFilter::INFO,
            "debug" => LevelFilter::DEBUG,
            "trace" => LevelFilter::TRACE,
            _ => {
                return Err(de::Error::custom(format!(
                    "invalid log_level '{raw}', expected off|error|warn|info|debug|trace"
                )));
            }
        };

        Ok(LogLevel(level))
    }
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub google: Google,
    pub web_server: WebServerParameters,
    pub fox_ess: FoxESS,
    pub mygrid: MyGrid,
    pub weather: Weather,
    pub general: General,
}

/// Returns a configuration struct for the application and starts logging
///
pub fn config() -> Result<Config, ConfigError> {
    let args: Vec<String> = env::args().collect();
    let config_path = args.iter()
        .find(|p| p.starts_with("--config="))
        .ok_or(ConfigError::InvalidConfigParameterError)?;
    let config_path = config_path
        .split_once('=')
        .ok_or(ConfigError::InvalidConfigParameterError)?
        .1;
    
    let mut config = load_config(&config_path)?;
    config.general.version = env!("CARGO_PKG_VERSION").to_string();
    config.google.client_id = read_credential("google_client_id")?;
    config.google.client_secret = read_credential("google_client_secret")?;
    config.fox_ess.api_key = read_credential("fox_ess_api_key")?;
    config.fox_ess.inverter_sn = read_credential("fox_ess_inverter_sn")?;
    config.google.users = read_credential("google_users")?
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();

    setup_logger(&config.general.log_path, config.general.log_level.0, config.general.log_to_stdout)?;

    Ok(config)
}

/// Loads the configuration file and returns a struct with all configuration items
///
/// # Arguments
///
/// * 'config_path' - path to the config file
fn load_config(config_path: &str) -> Result<Config, ConfigError> {

    let toml = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&toml)?;

    Ok(config)
}

/// Reads a credential from the file system supported by the credstore and
/// given from systemd
///
/// # Arguments
///
/// * 'name' - name of the credential to read
fn read_credential(name: &str) -> Result<String, ConfigError> {
    let dir = env::var("CREDENTIALS_DIRECTORY")?;
    let mut p = PathBuf::from(dir);
    p.push(name);
    let bytes = fs::read(p)?;
    Ok(String::from_utf8(bytes)?.trim_end().to_string())
}

/// Errors while managing configuration
///
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TomlParseError: {0}")]
    TomlParseError(#[from] toml::de::Error),
    #[error("LoggerSetupError: {0}")]
    LoggerSetupError(#[from] tracing::subscriber::SetGlobalDefaultError),
    #[error("Invalid log_path: expected a file path")]
    InvalidLogPathError,
    #[error("StringConversionError: {0}")]
    StringConversionError(#[from] alloc::string::FromUtf8Error),
    #[error("EnvVarError: {0}")]
    EnvVarError(#[from] env::VarError),
    #[error("Invalid --config=<config_path> argument")]
    InvalidConfigParameterError,
    #[error("TracingTryInitError: {0}")]
    TracingTryInitError(#[from] tracing_subscriber::util::TryInitError),
}
