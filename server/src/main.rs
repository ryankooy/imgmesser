use anyhow::{anyhow, bail, Context, Result};
use aws_config::{
    profile::{
        ProfileFileCredentialsProvider,
        ProfileFileRegionProvider,
    },
    BehaviorVersion,
};
use aws_sdk_s3::{
    error::ProvideErrorMetadata,
    operation::put_object::{
        PutObjectError,
        PutObjectOutput,
    },
    primitives::ByteStream,
    Client as S3Client
};
use axum::{
    extract::{
        connect_info::ConnectInfo,
        Multipart, Path,
    },
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use futures_util::stream::StreamExt;
use serde_json::json;
use std::net::SocketAddr;
use tokio::{
    fs::File,
    io::AsyncReadExt,
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

mod error;
use error::S3Error;

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

    let app = Router::new()
        .route("/images", get(get_images))
        .route("/images", post(add_image))
        .route("/images/{id}", get(get_image))
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client {addr} requested images");

    Json(json!({ "images": "NUH UH UH" })).into_response()
}

/// Route for retrieving a specific image.
async fn get_image(
    Path(image_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("Client {addr} requested image {image_id}");

    Json(json!({ "image": "A GRAVEN IMAGE" })).into_response()
}

/// Route for uploading an image.
async fn add_image(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let s3_client = get_s3_client().await.unwrap();
    let object_key = Uuid::now_v7();

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "file_path" {
            let data: Bytes = field.bytes().await.unwrap();
            println!("Length of {} is {} bytes", name, data.len());

            match upload_image_data(&s3_client, data, &object_key).await {
                Ok(output) => {
                    if let Some(version_id) = output.version_id {
                        info!("Version id: {}", version_id);
                        return Json(json!({ "version_id": version_id }))
                            .into_response();
                    } else {
                        info!("No version id");
                    }
                }
                Err(e) => {
                    error!("File upload error: {}", e);
                    return Json(json!({ "error": e.to_string() })).into_response();
                }
            }
        }
    }

    Json(json!({ "whoa": "watch it" })).into_response()
}

/// Get AWS S3 client.
async fn get_s3_client() -> Result<S3Client> {
    // Load AWS config from environment
    let config = aws_config::defaults(BehaviorVersion::latest())
        .profile_name("imgmesser-client")
        .load()
        .await;

    Ok(S3Client::new(&config))
}

/// Upload image data to AWS S3 bucket.
async fn upload_image_data(
    client: &S3Client,
    data: Bytes,
    object_key: &Uuid,
) -> Result<PutObjectOutput, S3Error> {
    let body = ByteStream::from(data);

    let result = client.put_object()
        .bucket("imgmesser-storage")
        .key(object_key.to_string())
        .body(body)
        .send()
        .await
        .map_err(S3Error::from)?;

    Ok(result)
}
