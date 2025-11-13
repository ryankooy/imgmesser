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
    Algorithm, Header, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub(crate) fn new(username: &str) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::minutes(15)).timestamp() as usize;

        Self {
            sub: username.to_owned(),
            exp,
            iat: now.timestamp() as usize,
        }
    }
}

impl<S> FromRequestParts<S> for Claims where S: Send + Sync {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Extract token from auth header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::BadOrMissingHeader)?;

        // Extract claims from token data
        let token_data_claims = validate_token(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data_claims)
    }
}

/// Generate a JWT access token.
pub(crate) fn create_access_token(username: &str) -> Result<String, JwtError> {
    let keys = get_keys();

    jsonwebtoken::encode(
        &Header::default(),
        &Claims::new(username),
        &keys.encoding,
    )
}

pub(crate) fn validate_token(token: &str) -> Result<Claims, JwtError> {
    let keys = get_keys();

    jsonwebtoken::decode::<Claims>(
        token,
        &keys.decoding,
        &Validation::new(Algorithm::HS256),
    )
    .map(|token_data| token_data.claims)
}

/// Generate a refresh token.
pub fn create_refresh_token() -> String {
    Uuid::now_v7().simple().to_string()
}
