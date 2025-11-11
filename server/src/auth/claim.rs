use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::TypedHeader;
use chrono::{Duration, Utc};
use headers::{
    authorization::Bearer,
    Authorization,
};
use jsonwebtoken::{
    errors::Error as JwtError,
    Algorithm, Header, TokenData, Validation,
};
use serde::{Deserialize, Serialize};

use super::{
    error::AuthError,
    keys::get_keys,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    /// Subject (e.g., username)
    pub sub: String,

    /// Expiration time of JWT
    pub exp: usize,

    /// Time JWT was issued
    pub iat: usize,
}

impl Claims {
    fn new(username: &str, duration: Duration) -> Self {
        let now = Utc::now();
        let exp = (now + duration).timestamp() as usize;

        Self {
            sub: username.to_owned(),
            exp,
            iat: now.timestamp() as usize,
        }
    }

    /// Create claims for short-lived access token.
    pub(crate) fn new_for_access(username: &str) -> Self {
        Self::new(username, Duration::minutes(15))
    }

    /// Create claims for long-lived refresh token.
    pub(crate) fn new_for_refresh(username: &str) -> Self {
        Self::new(username, Duration::days(7))
    }
}

impl<S> FromRequestParts<S> for Claims where S: Send + Sync {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract token
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        // Decode user data
        let token_data = extract_token_data(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// Refresh a JWT access token.
pub fn refresh_access_token(refresh_token: &str) -> Result<String, JwtError> {
    let token_data = extract_token_data(refresh_token)?;
    create_access_token(&token_data.claims.sub)
}

/// Generate a JWT access token.
pub(crate) fn create_access_token(username: &str) -> Result<String, JwtError> {
    create_token(&Claims::new_for_access(username))
}

/// Generate a JWT refresh token.
pub(crate) fn create_refresh_token(username: &str) -> Result<String, JwtError> {
    create_token(&Claims::new_for_refresh(username))
}

fn create_token(claims: &Claims) -> Result<String, JwtError> {
    let keys = get_keys();
    jsonwebtoken::encode(&Header::default(), claims, &keys.encoding)
}

fn extract_token_data(token: &str) -> Result<TokenData<Claims>, JwtError> {
    let keys = get_keys();
    let validation = Validation::new(Algorithm::HS256);
    jsonwebtoken::decode::<Claims>(token, &keys.decoding, &validation)
}
