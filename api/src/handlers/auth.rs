use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::net::SocketAddr;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    auth::{
        error::AuthError,
        middleware::RequireAuth,
        jwt, Claims,
    },
    models::{User, UserInfo},
    schemas::{
        UserRequest, LoginResponse, UserResponse,
        RefreshTokenRequest, RefreshTokenResponse,
        LogoutRequest, LogoutResponse,
    },
    state::AppState,
};

/// Route for user registration.
pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<UserRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    info!("Client {addr} is attempting to register");

    // Check if username is already taken
    if state
        .user_repo
        .find(&payload.username)
        .await
        .map_err(|_| AuthError::QueryFailure)?
        .is_some()
    {
        return Err(AuthError::UsernameTaken);
    }

    // Create user
    let user: UserInfo = state
        .user_repo
        .create(User::from_request(&payload))
        .await
        .map_err(|_| AuthError::UserCreationFailure)?;

    user_tokens_response(user, state).await
}

/// Route for user login.
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<UserRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    info!("Client {addr} is attempting to log in");

    // Verify user credentials
    let pw_valid = state
        .user_repo
        .authorize(User::from_request(&payload))
        .await
        .map_err(|_| AuthError::QueryFailure)?;

    if !pw_valid {
        return Err(AuthError::InvalidCredentials);
    }

    // Retrieve user info
    let user: UserInfo = state
        .user_repo
        .find(&payload.username)
        .await
        .map_err(|_| AuthError::QueryFailure)?
        .ok_or(AuthError::UserNotFound)?;

    user_tokens_response(user, state).await
}

/// Identify the current user.
pub async fn current_user(
    RequireAuth(user): RequireAuth,
) -> Result<Json<UserResponse>, AuthError> {
    Ok(Json(UserResponse { user }))
}

async fn user_tokens_response(
    user: UserInfo,
    state: AppState,
) -> Result<Json<LoginResponse>, AuthError> {
    // Generate JWT access token
    let access_token = jwt::create_access_token(&user.username)
        .map_err(|_| AuthError::TokenCreationFailure)?;

    // Generate refresh token
    let refresh_token = jwt::create_refresh_token();

    // Save refresh token to database
    state
        .refresh_token_repo
        .create_token(&user.username, &refresh_token)
        .await
        .map_err(|_| AuthError::RefreshTokenNotSaved)?;

    Ok(Json(LoginResponse {
        user,
        access_token,
        refresh_token,
    }))
}

/// Route for user logout.
pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, AuthError> {
    info!("Client {addr} is attempting to log out");

    // Delete given refresh token from database
    state
        .refresh_token_repo
        .delete_token(&payload.refresh_token)
        .await
        .map_err(|_| AuthError::QueryFailure)?;

    Ok(Json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    }))
}

// TODO: ADD TOKEN REFRESH HANDLER

