mod errors;

use std::ops::Add;
use chrono::{DateTime, TimeDelta, Utc};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use crate::initialization::Google;
use crate::manager_tokens::errors::TokenError;

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
}

impl Tokens {
    /// Creates a new Tokens instance given an OAuth2.0 code to be traded for tokens
    ///
    /// # Arguments
    ///
    /// * 'config' - configuration struct for Google
    /// * 'code' - code from an initiated OAuth2.0 code flow
    pub async fn from_code(config: &Google, code: &str) -> Result<Self, TokenError> {
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

        let tokens = Tokens {
            access_token: import.access_token,
            expires_at: Utc::now().add(TimeDelta::seconds(import.expires_in)),
            email: validate_jwt(config, &import.id_token)? ,
        };

        Ok(tokens)
    }

    /// Checks if the access token needs to be refreshed
    ///
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
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
/// * 'config' - config to update
pub async fn google_base_data(config: &mut Google) -> Result<(), TokenError> {
    #[derive(Deserialize)]
    struct Knowns {
        authorization_endpoint: String,
        token_endpoint: String,
        jwks_uri: String,
    }
    let resp = reqwest::get(&config.well_known).await?;

    let json = resp.text().await?;
    let knowns: Knowns = serde_json::from_str(&json)?;

    let resp = reqwest::get(&knowns.jwks_uri).await?;
    let json = resp.text().await?;

    let jwks: JwkSet = serde_json::from_str(&json)?;

    config.jwks = Some(jwks);
    config.auth_url = knowns.authorization_endpoint;
    config.token_url = knowns.token_endpoint;

    Ok(())
}

/// Builds an access request url and returns a url encoded version of it
///
pub fn build_access_request_url(config: &Google, state: &str) -> String {
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
