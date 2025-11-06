use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::TypedHeader;
use headers::{
    authorization::Bearer,
    Authorization,
};
use jsonwebtoken::{Algorithm, Validation};
use serde::{Deserialize, Serialize};

use super::{
    error::AuthError,
    keys::get_keys,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub username: String,
    pub exp: usize,
}

//#[async_trait]
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

        let keys= get_keys();
        let validation = Validation::new(Algorithm::HS256);

        // Decode user data
        let token_data = jsonwebtoken::decode::<Claims>(
            bearer.token(),
            &keys.decoding,
            &validation,
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}
