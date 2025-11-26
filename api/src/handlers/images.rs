use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        Multipart, Path, Query, State,
    },
    http::header,
    response::{Json, Response},
};
use bytes::Bytes;
use std::net::SocketAddr;
use tracing::info;

use crate::{
    auth::middleware::RequireAuth,
    models::{ImageData, ImageList, UserInfo},
    schemas::PaginationParams,
    state::AppState,
};

use super::error::ImageError;

type Result<T> = anyhow::Result<T, ImageError>;

/// Route for retrieving images.
pub async fn get_images(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    RequireAuth(user): RequireAuth,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ImageList>> {
    info!("Client {addr} requested images");

    let page = params.page.max(1);
    let limit = params.limit.max(1).min(100);

    let images: ImageList = state
        .image_repo
        .get_all(user, page, limit)
        .await
        .map_err(|_| ImageError::S3OperationFailure)?;

    Ok(Json(images))
}

/// Route for retrieving a specific image.
pub async fn get_image(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_name): Path<String>,
) -> Result<Response> {
    let image: ImageData = state
        .image_repo
        .get_one(&image_name, user)
        .await
        .map_err(|_| ImageError::S3OperationFailure)?
        .ok_or(ImageError::ObjectNotFound)?;

    let response = Response::builder()
        .header(header::CONTENT_TYPE, image.content_type)
        .header(header::CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(image.data))
        .unwrap();

    Ok(response)
}

/// Route for uploading an image.
pub async fn upload_image(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> Result<Response> {
    info!("Client {addr} added image");

    let mut username: String = String::new();
    let mut file_name: String = String::new();
    let mut data: Bytes = Bytes::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "user" => {
                username = field.text().await.unwrap();
            }
            "file_path" => {
                let content_type = field.content_type().unwrap().to_string();
                if !content_type.starts_with("image") {
                    return Err(ImageError::InvalidFileType);
                }

                file_name = field.file_name().unwrap_or("unknown").to_string();
                data = field.bytes().await.unwrap();
            }
            _ => {},
        }
    }

    if !(data.is_empty() || file_name.is_empty() || username.is_empty()) {
        // Find the user record
        let user: UserInfo = state
            .user_repo
            .find(&username)
            .await
            .map_err(|_| ImageError::QueryFailure)?
            .ok_or(ImageError::UserNotFound)?;

        // Upload the image to S3
        state
            .image_repo
            .upload(data, &file_name, user)
            .await
            .map_err(|_| ImageError::UploadFailure)?;

        return Ok(Response::default());
    }

    return Err(ImageError::MissingMultipartField);
}
