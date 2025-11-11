use anyhow::Result;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;

use crate::db::user::validate_user;

use super::{claim, error::AuthError};

#[derive(Debug, Serialize)]
pub struct AuthBody {
    access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    client_id: String,
    client_secret: String,
}

/// Authorize a user.
pub async fn authorize(
    pool: &PgPool,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Check if given credentials match those of
    // existing user account
    match validate_user(pool, &payload.client_id, &payload.client_secret).await {
        Ok(is_match) => if !is_match {
            return Err(AuthError::WrongCredentials);
        }
        Err(e) => {
            error!("Failed to validate user: {}", e);
            return Err(AuthError::UserValidationFailure);
        }
    }

    // Create auth token
    let token = claim::create_access_token(&payload.client_id)
        .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthBody::new(token)))
}
