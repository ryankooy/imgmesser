use anyhow::Result;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::TypedHeader;
use headers::{
    authorization::Bearer,
    Authorization,
};

use crate::{models::UserInfo, state::AppState};
use super::{error::AuthError, jwt};

/// Requires valid JWT for protected routes
pub struct RequireAuth(pub UserInfo);

impl<S> FromRequestParts<S> for RequireAuth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        // Extract token from auth header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::BadOrMissingHeader)?;

        // Extract claims from token data
        let claims = jwt::validate_token(bearer.token())
            .map_err(|_| AuthError::InvalidToken)?;

        // Retrieve user info using claims subject
        let user_info = app_state
            .user_repo
            .find(&claims.sub)
            .await
            .map_err(|_| AuthError::QueryFailure)?
            .ok_or(AuthError::UserNotFound)?;

        Ok(RequireAuth(user_info))
    }
}
