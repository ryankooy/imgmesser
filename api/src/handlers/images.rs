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
    errors::ImageError,
    models::{
        ContentType, Image, ImageData, ImageList,
        UploadImage, UserInfo,
    },
    schemas::{
        ImageRenameRequest,
        ImageUpdateResponse,
        PaginationParams,
    },
    state::AppState,
};

type Result<T> = anyhow::Result<T, ImageError>;

/// Route for retrieving images.
pub async fn get_all_images_metadata(
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
        .get_metadata_for_all(user, page, limit)
        .await?;

    Ok(Json(images))
}

/// Route for retrieving data for a specific image.
pub async fn get_image(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Response> {
    let image: ImageData = state
        .image_repo
        .get_one(&image_id, user)
        .await?
        .ok_or(ImageError::NotFound)?;

    let response = Response::builder()
        .header(header::CONTENT_TYPE, image.content_type)
        .header(
            header::CACHE_CONTROL,
            "no-store, no-cache, must-revalidate, proxy-revalidate",
        )
        .header(header::PRAGMA, "no-cache")
        .header(header::EXPIRES, "0")
        .body(Body::from(image.data))
        .unwrap();

    Ok(response)
}

/// Route for retrieving metadata for a specific image.
pub async fn get_image_metadata(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Json<Image>> {
    let image: Image = state
        .image_repo
        .get_metadata_for_one(&image_id, user)
        .await?
        .ok_or(ImageError::NotFound)?;

    Ok(Json(image))
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
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?
            .ok_or(ImageError::UserNotFound)?;

        // Upload the image to S3
        state
            .image_repo
            .upload(upload_image, user)
            .await?;

        return Ok(Response::default());
    }

    Err(ImageError::MissingMultipartField)
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
        .await?;

    Ok(Response::default())
}

/// Route for renaming an image.
pub async fn rename_image(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
    Json(payload): Json<ImageRenameRequest>,
) -> Result<Json<ImageUpdateResponse>> {
    let updated: bool = state
        .image_repo
        .rename(&image_id, &payload.image_name, user)
        .await?
        .is_some();

    Ok(Json(ImageUpdateResponse { updated }))
}

/// Route for reverting an image to its previous version.
pub async fn revert_image_version(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Json<ImageUpdateResponse>> {
    let updated: bool = state
        .image_repo
        .revert(&image_id, user)
        .await?
        .is_some();

    Ok(Json(ImageUpdateResponse { updated }))
}

/// Route for restoring an image back to its newer version.
pub async fn restore_image_version(
    State(state): State<AppState>,
    RequireAuth(user): RequireAuth,
    Path(image_id): Path<String>,
) -> Result<Json<ImageUpdateResponse>> {
    let updated: bool = state
        .image_repo
        .restore(&image_id, user)
        .await?
        .is_some();

    Ok(Json(ImageUpdateResponse { updated }))
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
