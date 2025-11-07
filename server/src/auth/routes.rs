use anyhow::Result;
use axum::Json;
use chrono::{naive::Days, Utc};
use jsonwebtoken::Header;
use serde::{Deserialize, Serialize};

use super::{
    claim::Claims,
    error::AuthError,
    keys::get_keys,
};

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

//TODO: pass db data to this func
pub fn authorize(
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    if payload.client_id.is_empty() || payload.client_secret.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    //TODO: GET VALS FROM DB VIA ARGS
    if &payload.client_id != "CLIENT_ID" || &payload.client_secret != "CLIENT_SECRET" {
        return Err(AuthError::WrongCredentials);
    }

    // Create expiry timestamp
    let exp = (Utc::now().naive_utc() + Days::new(1))
        .and_utc()
        .timestamp() as usize;

    let claims = Claims {
        username: payload.client_id,
        exp,
    };

    let keys = get_keys();

    // Create auth token
    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &keys.encoding,
    )
    .map_err(|_| AuthError::TokenCreation)?;

    Ok(Json(AuthBody::new(token)))
}
