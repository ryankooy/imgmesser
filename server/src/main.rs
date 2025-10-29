use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_s3::Client as S3Client;
use axum::{
    extract::{
        connect_info::ConnectInfo,
        Multipart, Path, State,
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use serde_json::json;
use std::net::SocketAddr;
use tokio::{
    net::TcpListener,
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{error, info};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use uuid::Uuid;

mod s3;

//use imgmesser_core::process_image;

#[derive(Clone)]
struct AppState {
    /// AWS S3 client
    img_store_client: S3Client,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();


    let img_store_client = s3::get_client().await?;
    let state = AppState { img_store_client };

    let app = Router::new()
        .route("/images", get(get_images).post(add_image))
        .route("/images/{id}", get(get_image))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// Route for retrieving images.
async fn get_images(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client {addr} requested images");
    Json(json!({ "result": "GRAVEN IMAGES" })).into_response()
}

/// Route for retrieving a specific image.
async fn get_image(
    Path(image_id): Path<String>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client {addr} requested image {image_id}");

    let client = state.img_store_client;
    let mut version_id = String::new();

    match s3::get_object(&client, &image_id).await {
        Ok(output) => {
            // TODO: return body to user

            if let Some(obj_version) = output.version_id() {
                info!("Object version id: {}", obj_version);
                return Json(json!({ "version_id": obj_version }))
                    .into_response();
            } else {
                info!("No object version id");
                return Response::default();
            }
        }
        Err(e) => {
            error!("File retrieval error: {}", e);

            // TODO: improve error handling here
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Upload error: {}", e),
            )
            .into_response();
        }
    }

    (
        StatusCode::NOT_FOUND,
        "Image not found".to_string(),
    )
    .into_response()
}

/// Route for uploading an image.
async fn add_image(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    info!("Client {addr} added image");

    let client = state.img_store_client;

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap().to_string();

        if field_name == "file_path" {
            let content_type = field.content_type().unwrap().to_string();
            if !content_type.starts_with("image") {
                return (
                    StatusCode::BAD_REQUEST,
                    "Not an image file".to_string(),
                )
                .into_response();
            }

            let file_name = field.file_name().unwrap().to_string();
            let data: Bytes = field.bytes().await.unwrap();

            info!(
                "file_name: {}, content_type: {}, data_len: {} bytes",
                file_name, content_type, data.len(),
            );

            match s3::upload_object(&client, data, &file_name).await {
                Ok(output) => {
                    if let Some(obj_version) = output.version_id() {
                        info!("{} version id: {}", file_name, obj_version);
                    } else {
                        info!("No object version id");
                    }

                    return Response::default();
                }
                Err(e) => {
                    error!("File upload error: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Upload error: {}", e),
                    )
                    .into_response();
                }
            }
        }
    }

    (
        StatusCode::BAD_REQUEST,
        "No image to upload".to_string(),
    )
    .into_response()
}
