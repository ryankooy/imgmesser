use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

#[derive(Debug)]
pub enum ImageError {
    UploadFailure,
    MissingMultipartField,
    InvalidFileType,
    ReadFailure,
    S3OperationFailure(String),
    QueryFailure(String),
    NotFound,
    UserNotFound,
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ImageError::UploadFailure => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Image upload failed".to_string(),
                )
            }
            ImageError::MissingMultipartField => {
                (
                    StatusCode::BAD_REQUEST,
                    "Missing multipart field(s) in request".to_string(),
                )
            }
            ImageError::InvalidFileType => {
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid file type; not an image file".to_string(),
                )
            }
            ImageError::ReadFailure => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error reading image".to_string(),
                )
            }
            ImageError::S3OperationFailure(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("S3 operation failed: {}", e),
                )
            }
            ImageError::QueryFailure(e) => {
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Database operation error: {}", e),
                )
            }
            ImageError::NotFound => {
                (
                    StatusCode::NOT_FOUND,
                    "Image not found".to_string(),
                )
            }
            ImageError::UserNotFound => {
                (
                    StatusCode::UNAUTHORIZED,
                    "User not found".to_string(),
                )
            }
        };

        let body = Json(serde_json::json!({ "error": error_message }));
        (status, body).into_response()
    }
}
