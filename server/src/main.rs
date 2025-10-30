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

trait ResponseBehavior {
    fn new(message: String) -> Self;
    fn set_success_status(&mut self);
}

#[derive(Serialize, Deserialize)]
struct Response {
    success: bool,
    message: String,
}

impl ResponseBehavior for Response {
    fn new(message: String) -> Self {
        Self {
            success: false,
            message,
        }
    }

    fn set_success_status(&mut self) {
        self.success = true;
    }
}

#[derive(Serialize, Deserialize)]
struct UploadResponse {
    success: bool,
    message: String,
    filename: Option<String>,
}

impl UploadResponse {
    fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
    }
}

impl ResponseBehavior for UploadResponse {
    fn new(message: String) -> Self {
        Self {
            success: false,
            message,
            filename: None,
        }
    }

    fn set_success_status(&mut self) {
        self.success = true;
    }
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

    let mut resp = Response::new("GRAVEN IMAGES".to_string());
    resp.set_success_status();
    success(resp)
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

            let mut resp = Response::new(
                "Image retrieved successfully".to_string(),
            );
            resp.set_success_status();
            return success(resp);
        }
        Err(e) => {
            error!("File retrieval error: {}", e);

            // TODO: improve error handling here
            return failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                Response::new(format!("File retrieval error: {}", e)),
            );
        }
    }


    failure(
        StatusCode::NOT_FOUND,
        Response::new("Image not found".to_string()),
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
                    UploadResponse::new(
                        "Invalid file type; not an image file".to_string(),
                    ),
                );
            }

            let file_name = field.file_name().unwrap_or("unknown").to_string();
            let data: Bytes = field.bytes().await.unwrap();

            info!(
                "file_name: {}, content_type: {}, data_len: {} bytes",
                file_name, content_type, data.len(),
            );

            // TODO: create unique filename and save to db
            //let ext = PathBuf::from(&file_name)
            //    .extension()
            //    .and_then(|e| e.to_str())
            //    .unwrap_or("jpg");
            //let uniq_filename = format!("{}.{}", Uuid::new_v7(), ext);

            match s3::upload_object(&client, data, &file_name).await {
                Ok(output) => {
                    // TODO: save version id to db
                    if let Some(obj_version) = output.version_id() {
                        info!("{} version id: {}", file_name, obj_version);
                    } else {
                        info!("No object version id");
                    }

                    let mut resp = UploadResponse::new(
                        "Image uploaded successfully".to_string(),
                    );
                    resp.set_filename(file_name);
                    resp.set_success_status();

                    return success(resp);
                }
                Err(e) => {
                    error!("File upload error: {}", e);
                    return failure(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        UploadResponse::new(
                            format!("Failed to upload image: {}", e),
                        ),
                    );
                }
            }
        }
    }

    failure(
        StatusCode::BAD_REQUEST,
        UploadResponse::new(
            "No file_path field in request".to_string(),
        ),
    )
}
