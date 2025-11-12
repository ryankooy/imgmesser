use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AuthError {
    MissingCredentials,
    InvalidCredentials,
    InvalidToken,
    TokenCreationFailure,
    RefreshTokenNotSaved,
    InvalidUserInput,
    UsernameTaken,
    UserCreationFailure,
    UserNotFound,
    BadOrMissingHeader,
    QueryFailure,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials")
            }
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token")
            }
            AuthError::TokenCreationFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate access token")
            }
            AuthError::RefreshTokenNotSaved => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to store refresh token")
            }
            AuthError::InvalidUserInput => {
                (StatusCode::BAD_REQUEST, "Invalid user info")
            }
            AuthError::UsernameTaken => {
                (StatusCode::CONFLICT, "Username already in use")
            }
            AuthError::UserCreationFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user profile")
            }
            AuthError::UserNotFound => {
                (StatusCode::UNAUTHORIZED, "User not found")
            }
            AuthError::BadOrMissingHeader => {
                (StatusCode::BAD_REQUEST, "Authorization header missing or malformed")
            }
            AuthError::QueryFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Error querying data")
            }
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
