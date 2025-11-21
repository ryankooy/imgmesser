//! ImgMesser Server

use anyhow::Result;
use axum::{
    http::{
        header, method::Method, HeaderValue,
    },
    routing::{get, post},
    Router,
};
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
        add_image, get_image, get_images,
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

    // Configure CORS
    let origin_address = "http://127.0.0.1:5173";
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
        .route("/images", get(get_images).post(add_image))
        .route("/images/{id}", get(get_image))
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
