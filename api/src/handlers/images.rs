use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        multipart::Field,
        Multipart, Path, Query, State,
    },
    http::header,
    response::{Json, Response},
};
use image::ImageReader;
use std::io::Cursor;
use std::net::SocketAddr;
use tracing::info;

use crate::{
    auth::middleware::RequireAuth,
    models::{
        ContentType, ImageData, ImageList, UploadImage, UserInfo,
    },
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
    Path(image_id): Path<String>,
) -> Result<Response> {
    let image: ImageData = state
        .image_repo
        .get_one(&image_id, user)
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

    let mut username = String::new();
    let mut image: Option<UploadImage> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ImageError::MissingMultipartField)?
    {
        match field.name().unwrap_or("") {
            "user" => {
                username = field
                    .text()
                    .await
                    .map_err(|_| ImageError::MissingMultipartField)?;
            }
            "file_path" => {
                let content_type = ContentType::from_str(
                    field.content_type().unwrap_or("unknown"),
                );
                if matches!(content_type, ContentType::UNKNOWN) {
                    return Err(ImageError::InvalidFileType);
                }

                let upload_image = parse_image_data(field, content_type)
                    .await
                    .map_err(|_| ImageError::ReadFailure)?;

                image = Some(upload_image);
            }
            _ => {},
        }
    }

    if !username.is_empty() && let Some(upload_image) = image {
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
            .upload(upload_image, user)
            //.upload(data, &file_name, user)
            .await
            .map_err(|_| ImageError::UploadFailure)?;

        return Ok(Response::default());
    }

    Err(ImageError::MissingMultipartField)
}

/// Route for reverting an image to its previous version.
pub async fn revert_image_version(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Response> {
    state
        .image_repo
        .revert(&image_id, user)
        .await
        .map_err(|_| ImageError::RevertFailure)?;

    Ok(Response::default())
}

/// Route for restoring an image back to its newer version.
pub async fn restore_image_version(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Response> {
    state
        .image_repo
        .unrevert(&image_id, user)
        .await
        .map_err(|_| ImageError::RestoreFailure)?;

    Ok(Response::default())
}

/// Route for deleting an image.
pub async fn delete_image(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Response> {
    state
        .image_repo
        .delete(&image_id, user)
        .await
        .map_err(|_| ImageError::DeleteFailure)?;

    Ok(Response::default())
}

/// Parse multipart image data.
async fn parse_image_data(
    field: Field<'_>,
    content_type: ContentType,
) -> anyhow::Result<UploadImage> {
    let name = field.file_name().unwrap_or("").to_string();
    let data = field.bytes().await?;

    let dimensions = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()?
        .into_dimensions()?;

    Ok(UploadImage { name, content_type, data, dimensions })
}
