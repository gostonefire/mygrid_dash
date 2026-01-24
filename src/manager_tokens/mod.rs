use std::ops::Add;
use std::sync::Arc;
use chrono::{DateTime, TimeDelta, Utc};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use tracing::info;
use reqwest::{Response, Url};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use crate::initialization::Google;

#[derive(Deserialize)]
struct TokensResponse {
    access_token: String,
    expires_in: i64,
    id_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tokens {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
    pub email: String,
    pub authorized: bool,
}

impl Tokens {
    /// Creates a new Tokens instance given an OAuth2.0 code to be traded for tokens
    ///
    /// # Arguments
    ///
    /// * 'google_config' - configuration struct for Google
    /// * 'code' - code from an initiated OAuth2.0 code flow
    pub async fn from_code(google_config: &Arc<RwLock<Google>>, code: &str) -> Result<Self, TokenError> {
        let config = google_config.read().await;
        let body: [(&str, &str); 5] = [
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let client = reqwest::Client::new();
        let resp = client
            .post(&config.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&body)
            .send()
            .await?;

        let json = resp.text().await?;
        let import: TokensResponse = serde_json::from_str(&json)?;
        let email = validate_jwt(&*config, &import.id_token)?;
        
        let tokens = Tokens {
            access_token: import.access_token,
            expires_at: Utc::now().add(TimeDelta::seconds(import.expires_in)),
            authorized: config.users.contains(&email),
            email,
        };

        Ok(tokens)
    }

    /// Checks if the access token needs to be refreshed
    ///
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
    
    /// Checks if the authenticated user is authorized to use the application
    /// 
    pub fn is_authorized(&self) -> bool { self.authorized }
    
}

/// Validates the given JWT ID Token and returns the email claim
///
/// # Arguments
///
/// * 'config' - Google config
/// * 'jwt' - the JWT to validate and decode
fn validate_jwt(config: &Google, jwt: &str) -> Result<String, TokenError> {
    let jwks = config.jwks.as_ref().expect("should not be empty");
    let header = decode_header(jwt)?;

    let Some(kid) = header.kid else {
        return Err(TokenError::InvalidJwt);
    };

    let Some(jwk) = jwks.find(&kid) else {
        return Err(TokenError::InvalidJwt);
    };

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)?,
        _ => return Err(TokenError::InvalidJwt),
    };

    let validation = {
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[&config.client_id]);
        validation.validate_exp = false;
        validation
    };

    #[derive(Deserialize)]
    struct Claims {
        email: String,
    }
    let decoded_token = decode::<Claims>(jwt, &decoding_key, &validation)?;

    Ok(decoded_token.claims.email)
}

/// Updates the config with necessary data from Google Well Known
///
/// # Arguments
///
/// * 'google_config' - config to update
pub async fn google_base_data(google_config: Arc<RwLock<Google>>) -> Result<(), TokenError> {
    let utc_timestamp = Utc::now().timestamp();

    if utc_timestamp >= google_config.read().await.well_known_expire {
        info!("updating well known google urls");
        let mut config = google_config.write().await;
        
        #[derive(Deserialize)]
        struct Knowns {
            authorization_endpoint: String,
            token_endpoint: String,
            jwks_uri: String,
        }
        let resp = reqwest::get(&config.well_known).await?;
        let max_age = get_max_age(&resp)?;

        let json = resp.text().await?;
        let knowns: Knowns = serde_json::from_str(&json)?;

        config.jwks_uri = knowns.jwks_uri;
        config.auth_url = knowns.authorization_endpoint;
        config.token_url = knowns.token_endpoint;
        config.well_known_expire = utc_timestamp + max_age;
    }

    if utc_timestamp >= google_config.read().await.jwks_expire {
        info!("updating google jwks");
        let mut config = google_config.write().await;
        
        let resp = reqwest::get(&config.jwks_uri).await?;
        let max_age = get_max_age(&resp)?;
        
        let json = resp.text().await?;

        let jwks: JwkSet = serde_json::from_str(&json)?;

        config.jwks = Some(jwks);
        config.jwks_expire = utc_timestamp + max_age;
    }

    Ok(())
}

/// Builds an access request url and returns a url encoded version of it
///
/// # Arguments
///
/// * 'google_config' - configuration struct for Google
/// * 'state' - state
pub async fn build_access_request_url(google_config: &Arc<RwLock<Google>>, state: &str) -> String {
    let config = google_config.read().await;
    let params: [(&str, &str); 5] = [
        ("response_type", "code"),
        ("client_id", &config.client_id),
        ("scope", &config.scope),
        ("redirect_uri", &config.redirect_uri),
        ("state", state),
    ];

    let url = Url::parse_with_params(&config.auth_url, &params).unwrap();
    url.to_string()
}

/// Returns the cache control max-age value in seconds
/// 
/// # Arguments
/// 
/// * 'response' - the response objects from a request
fn get_max_age(response: &Response) -> Result<i64, TokenError> {
    // First get the max-age value
    let cache_control_header = response.headers().get("Cache-Control")
        .ok_or(TokenError::NoCacheControlHeaderError)?;
    let cache_value = cache_control_header.to_str()
        .map_err(|_| TokenError::InvalidCacheControlHeaderError)?;

    let s = cache_value.split(',').map(|s| s.trim()).collect::<Vec<&str>>();
    let s = s.into_iter().find(|s| s.starts_with("max-age")).ok_or(TokenError::NoMaxAgeError)?;
    let v = s.split('=').map(|s| s.trim()).last().ok_or(TokenError::InvalidMaxAgeError)?;
    let max_age = v.parse::<i64>().map_err(|_| TokenError::MaxAgeNotANumberError)?;

    // Then get the age value if present
    let age_header = response.headers().get("Age");
    let age = if let Some(age_header) = age_header {
        age_header.to_str().map_err(|_| TokenError::InvalidAgeError)?.parse::<i64>().map_err(|_| TokenError::AgeNotANumberError)?
    } else {
        0
    };

    Ok(max_age - age)
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("NoCacheControlHeaderError")]
    NoCacheControlHeaderError,
    #[error("InvalidCacheControlHeaderError")]
    InvalidCacheControlHeaderError,
    #[error("NoMaxAgeError")]
    NoMaxAgeError,
    #[error("InvalidMaxAgeError")]
    InvalidMaxAgeError,
    #[error("MaxAgeNotANumberError")]
    MaxAgeNotANumberError,
    #[error("InvalidAgeError")]
    InvalidAgeError,
    #[error("AgeNotANumberError")]
    AgeNotANumberError,
    #[error("InvalidJwt")]
    InvalidJwt,
    #[error("JwtDecodeError: {0}")]
    JwtDecodeError(#[from] jsonwebtoken::errors::Error),
    #[error("FileIOError: {0}")]
    FileIOError(#[from] std::io::Error),
    #[error("ReqwestError: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("JsonError: {0}")]
    JsonError(#[from] serde_json::Error),
}

/*
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
 */
