//! ImgMesser Server

use anyhow::{Context, Result};
use axum::{
    http::{
        header, method::Method, HeaderValue,
    },
    routing::{get, post},
    Router,
};
use dotenv;
use std::env;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::info;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

use imgmesser_api::{
    handlers::{
        current_user, login, logout, register, refresh,
        delete_image, get_image, get_images, rename_image,
        restore_image_version, revert_image_version, upload_image,

    },
    state::AppState,
};
//use imgmesser_core::process_image;

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

    let state = AppState::new().await?;

    // Load environment variables
    dotenv::dotenv().ok();
    let origin_address = env::var("ORIGIN_ADDRESS")
        .context("Missing env variable: ORIGIN_ADDRESS")?;
    let listen_address = env::var("LISTEN_ADDRESS")
        .context("Missing env variable: LISTEN_ADDRESS")?;

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            origin_address.parse::<HeaderValue>()?,
        ))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::COOKIE,
            header::ORIGIN,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/user", get(current_user))
        .route("/images", get(get_images).post(upload_image))
        .route("/images/{id}", get(get_image))
        .route("/images/{id}/delete", post(delete_image))
        .route("/images/{id}/rename", post(rename_image))
        .route("/images/{id}/revert", post(revert_image_version))
        .route("/images/{id}/restore", post(restore_image_version))
        //.route("/images/{id}/transform", post(process_image))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(cors)
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = TcpListener::bind(&listen_address).await?;
    info!("Listening on {}...", listen_address);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
