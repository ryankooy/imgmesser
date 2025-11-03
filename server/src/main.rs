//! ImgMesser Server

use anyhow::{anyhow, bail, Context, Result};
use aws_sdk_s3::Client as S3Client;
use axum::{
    extract::{
        connect_info::ConnectInfo,
        Multipart, Path, State,
    },
    http::{header, StatusCode},
    response::{IntoResponse, Json},
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
        .route("/images/{id}", get(get_image))
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
struct Response {
    success: bool,
    message: String,
}

impl ResponseBehavior for Response {}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
    filename: Option<String>,
}

impl ResponseBehavior for UploadResponse {}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
    filename: Option<String>,
}

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
) -> impl IntoResponse {
    info!("Client {addr} requested images");

    // TODO: do stuff

    success(Response {
        success: true,
        message: "GRAVEN IMAGES".to_string(),
    })
}

/// Route for retrieving a specific image.
async fn get_image(
    Path(image_id): Path<String>,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client {addr} requested image {image_id}");

    let client = state.img_store_client;
    //let mut version_id = String::new();

    match s3::get_object(&client, &image_id).await {
        Ok(output) => {
            // TODO: return body to user

            if let Some(obj_version) = output.version_id() {
                info!("Object version id: {}", obj_version);

                // TODO: do something with version id
            } else {
                info!("No object version id");
            }

            return success(Response {
                success: true,
                message: "Image retrieved successfully".to_string(),
            });
        }
        Err(e) => {
            error!("File retrieval error: {}", e);

            // TODO: improve error handling here
            return failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                Response {
                    success: false,
                    message: format!("File retrieval error: {}", e),
                },
            );
        }
    }

    failure(
        StatusCode::NOT_FOUND,
        Response {
            success: false,
            message: "Image not found".to_string(),
        },
    )
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
