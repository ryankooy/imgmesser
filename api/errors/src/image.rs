use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

#[derive(Debug)]
pub enum ImageError {
    MissingMultipartField,
    InvalidFileType,
    ReadFailure(String),
    S3OperationFailure(String),
    QueryFailure(String),
    NotFound,
    UserNotFound,
    TransformFailure(String),
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
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
            ImageError::ReadFailure(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error reading image: {}", e),
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
                    StatusCode::INTERNAL_SERVER_ERROR,
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
            ImageError::TransformFailure(e) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Image transformation error: {}", e),
                )
            }
        };

        let body = Json(serde_json::json!({ "error": error_message }));
        (status, body).into_response()
    }
}
