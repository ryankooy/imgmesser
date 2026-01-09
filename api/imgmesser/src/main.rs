//! ImgMesser Server

use anyhow::Result;
use axum::{
    http::{header, method::Method},
    routing::{get, post},
    Router,
};
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

use config;
use handlers::{
    current_user, login, logout, register, refresh,
    delete_image, get_all_images_metadata, get_image,
    get_image_metadata, rename_image, restore_image_version,
    revert_image_version, upload_image,
};
use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=info,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new().await?;
    let addresses = config::get_addresses().await?;

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(addresses.origin))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ORIGIN,
        ])
        .allow_credentials(true);

    let app = Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/user", get(current_user))
        .route("/images", get(get_all_images_metadata).post(upload_image))
        .route("/images/{id}", get(get_image))
        .route("/images/{id}/meta", get(get_image_metadata))
        .route("/images/{id}/delete", post(delete_image))
        .route("/images/{id}/rename", post(rename_image))
        .route("/images/{id}/revert", post(revert_image_version))
        .route("/images/{id}/restore", post(restore_image_version))
        // TODO: add handlers, etc. for this route:
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

    let listener = TcpListener::bind(&addresses.listener).await?;
    info!("Listening on {}...", &listener.local_addr()?);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
