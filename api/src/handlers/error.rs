use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

#[derive(Debug)]
pub enum ImageError {
    UploadFailure,
    MissingMultipartField,
    InvalidFileType,
    S3OperationFailure,
    ObjectNotFound,
    QueryFailure,
    UserNotFound,
    ReadFailure,
    RevertFailure,
    RestoreFailure,
    DeleteFailure,
}

impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ImageError::UploadFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Image upload failed")
            }
            ImageError::MissingMultipartField => {
                (StatusCode::BAD_REQUEST, "Missing multipart field(s) in request")
            }
            ImageError::InvalidFileType => {
                (StatusCode::BAD_REQUEST, "Invalid file type; not an image file")
            }
            ImageError::ReadFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Error reading image")
            }
            ImageError::S3OperationFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "S3 operation failed")
            }
            ImageError::ObjectNotFound => {
                (StatusCode::NOT_FOUND, "Image not found")
            }
            ImageError::QueryFailure => {
                (StatusCode::UNAUTHORIZED, "Database operation failed")
            }
            ImageError::UserNotFound => {
                (StatusCode::UNAUTHORIZED, "User not found")
            }
            ImageError::RevertFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Image reversion failed")
            }
            ImageError::RestoreFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Image version restoration failed")
            }
            ImageError::DeleteFailure => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Image deletion failed")
            }
        };

        let body = Json(serde_json::json!({ "error": error_message }));
        (status, body).into_response()
    }
}
