use axum::{
    extract::{
        connect_info::ConnectInfo,
        State,
    },
    response::Json,
};
use std::net::SocketAddr;
use tracing::{error, info};

use crate::{
    auth::{
        error::AuthError,
        middleware::RequireAuth,
        jwt,
    },
    models::{User, UserInfo},
    schemas::{
        UserRequest, LoginResponse, UserResponse,
        RefreshTokenRequest, RefreshTokenResponse,
        LogoutRequest, LogoutResponse,
    },
    state::AppState,
};

/// Handler for user registration route.
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

    // Generate access and refresh tokens
    let (access_token, refresh_token) = create_tokens(
        &user.username,
        state,
    )
    .await?;

    Ok(Json(LoginResponse {
        user,
        access_token,
        refresh_token,
    }))
}

/// Handler for user login route.
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

    // Generate access and refresh tokens
    let (access_token, refresh_token) = create_tokens(
        &payload.username,
        state,
    )
    .await?;

    Ok(Json(LoginResponse {
        user,
        access_token,
        refresh_token,
    }))
}

/// Handler for current user identification route.
pub async fn current_user(
    RequireAuth(user): RequireAuth,
) -> Result<Json<UserResponse>, AuthError> {
    Ok(Json(UserResponse { user }))
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

/// Handler for token refresh route.
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, AuthError> {
    // Look up refresh token in database
    let refresh_token = state
        .refresh_token_repo
        .find_by_token(&payload.refresh_token)
        .await
        .map_err(|_| AuthError::QueryFailure)?
        .ok_or(AuthError::InvalidToken)?;

    // Check if token is expired
    if refresh_token.is_expired() {
        // Delete token
        state
            .refresh_token_repo
            .delete_token(&payload.refresh_token)
            .await
            .map_err(|_| AuthError::QueryFailure)?;

        // Reject request
        return Err(AuthError::InvalidToken);
    }

    // Check if token is already used
    if refresh_token.is_used {
        error!("Token reuse detected:");
        error!(
            "User {} originally used token {} at {:?}",
            &refresh_token.username,
            &payload.refresh_token,
            &refresh_token.used_at,
        );
        error!("Deleting all tokens for user {}", &refresh_token.username);

        // Delete all refresh tokens for this token's user
        state
            .refresh_token_repo
            .delete_all_user_tokens(&refresh_token.username)
            .await
            .map_err(|_| AuthError::QueryFailure)?;

        return Err(AuthError::InvalidToken);
    }

    // Mark token as used
    state
        .refresh_token_repo
        .mark_token_used(&payload.refresh_token)
        .await
        .map_err(|_| AuthError::QueryFailure)?;

    // Generate new access and refresh tokens
    let (access_token, new_refresh_token) = create_tokens(
        &refresh_token.username,
        state,
    )
    .await?;

    // Return both tokens
    Ok(Json(RefreshTokenResponse {
        access_token,
        refresh_token: new_refresh_token,
    }))
}

/// Generate and return access- and refresh- tokens.
async fn create_tokens(
    username: &str,
    state: AppState,
) -> Result<(String, String), AuthError> {
    // Generate JWT access token
    let access_token = jwt::create_access_token(username)
        .map_err(|_| AuthError::TokenCreationFailure)?;

    // Generate refresh token
    let refresh_token = jwt::create_refresh_token();

    // Save refresh token to database
    state
        .refresh_token_repo
        .create_token(username, &refresh_token)
        .await
        .map_err(|_| AuthError::RefreshTokenNotSaved)?;

    Ok((access_token, refresh_token))
}
