//! ImgMesser Server

use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_s3::Client as S3Client;
use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        Multipart, Path, Query, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::{
    net::TcpListener,
};
use tower_http::{
    cors::{CorsLayer, Any},
    trace::{DefaultMakeSpan, TraceLayer},
};
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

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let img_store_client = s3::get_client().await?;
    let state = AppState { img_store_client };

    let app = Router::new()
        .route("/images", get(get_images).post(add_image))
        .route("/images/:id", get(get_image))
        .with_state(state)
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listen_address = "127.0.0.1:3000";
    let listener = TcpListener::bind(listen_address).await?;
    info!("Listening on {}...", listen_address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

trait ResponseBehavior {}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
    filename: Option<String>,
}

impl ResponseBehavior for UploadResponse {}

#[derive(Deserialize)]
struct PaginationParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 10 }

#[derive(Serialize)]
struct ImageItem {
    key: String,
    size: i64,
    last_modified: String,
    content_type: String,
}

#[derive(Serialize)]
struct ImagesResponse {
    success: bool,
    images: Vec<ImageItem>,
    page: u32,
    limit: u32,
    total: usize,
    has_more: bool,
}

impl ResponseBehavior for ImagesResponse {}

fn success<T: ResponseBehavior>(resp: T) -> (StatusCode, Json<T>) {
    (StatusCode::OK, Json(resp))
}

fn failure<T: ResponseBehavior>(code: StatusCode, resp: T) -> (StatusCode, Json<T>) {
    (code, Json(resp))
}

/// Route for retrieving images.
async fn get_images(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Query(params): Query<PaginationParams>,
) -> impl IntoResponse {
    info!("Client {addr} requested images");

    let page = params.page.max(1);
    let limit = params.limit.max(1).min(100);
    let client = state.img_store_client;

    match s3::get_objects(&client).await {
        Ok(output) => {
            let objects = output.contents().to_vec();
            let total = objects.len();

            // Calculate pagination
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(total);
            let has_more = end < total;

            // Build image list
            let images: Vec<ImageItem> = objects[start..end]
                .iter()
                .filter_map(|obj| {
                    obj.key().map(|key| {
                        let path = PathBuf::from(key);
                        let extension = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("jpg");

                        ImageItem {
                            key: key.to_string(),
                            size: obj.size().unwrap_or(0),
                            last_modified: obj.last_modified()
                                .map(|dt| dt.to_string())
                                .unwrap_or_else(|| "unknown".to_string()),
                            content_type: get_content_type(extension).to_string(),
                        }
                    })
                })
                .collect();

            success(ImagesResponse {
                success: true,
                images,
                page,
                limit,
                total,
                has_more,
            })
        }
        Err(e) => {
            error!("Error listing S3 objects: {}", e);
            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                ImagesResponse {
                    success: false,
                    images: vec![],
                    page,
                    limit,
                    total: 0,
                    has_more: false,
                },
            )
        }
    }
}

/// Route for retrieving a specific image.
async fn get_image(
    State(state): State<AppState>,
    Path(image_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    info!("Client {addr} requested image {image_id}");
    let client = state.img_store_client;

    match s3::get_object(&client, &image_id).await {
        Ok(output) => {
            if let Some(obj_version) = output.version_id() {
                info!("Object version id: {}", obj_version);

                // TODO: do something with version id
            } else {
                info!("No object version id");
            }

            let content_type = output
                .content_type()
                .unwrap_or("image/jpeg")
                .to_string();

            let body = output.body;

            match body.collect().await {
                Ok(data) => {
                    let bytes = data.into_bytes();

                    Response::builder()
                        .header(header::CONTENT_TYPE, content_type)
                        .header(header::CACHE_CONTROL, "public, max-age=31536000")
                        .body(Body::from(bytes))
                        .unwrap()
                }
                Err(e) => {
                    error!("Error reading image data: {}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Failed to read image"))
                        .unwrap()
                }
            }
        }
        Err(e) => {
            error!("Error fetching image from S3: {}", e);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Image not found"))
                .unwrap()
        }
    }
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
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file_path" {
            let content_type = field.content_type().unwrap().to_string();
            if !content_type.starts_with("image") {
                return failure(
                    StatusCode::BAD_REQUEST,
                    UploadResponse {
                        success: false,
                        message: "Invalid file type; not an image file".to_string(),
                        filename: None,
                    },
                );
            }

            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let data: Bytes = field.bytes().await.unwrap();

            // TODO: create unique filename and save to db
            let image_id = Uuid::now_v7();
            let file_path = PathBuf::from(&file_name);
            let extension = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg");
            let uniq_filename = format!("{}.{}", image_id, extension);

            match s3::upload_object(&client, data, &uniq_filename).await {
                Ok(output) => {
                    // TODO: save version id to db
                    if let Some(obj_version) = output.version_id() {
                        info!("{} version id: {}", &uniq_filename, obj_version);
                    } else {
                        info!("No object version id");
                    }

                    return success(UploadResponse {
                        success: true,
                        message: "Image uploaded successfully".to_string(),
                        filename: Some(uniq_filename),
                    });
                }
                Err(e) => {
                    error!("File upload error: {}", e);
                    return failure(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        UploadResponse {
                            success: false,
                            message: format!("Failed to upload image: {}", e),
                            filename: None,
                        },
                    );
                }
            }
        }
    }

    failure(
        StatusCode::BAD_REQUEST,
        UploadResponse {
            success: false,
            message: "No file_path field in request".to_string(),
            filename: None,
        },
    )
}

fn get_content_type(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}
