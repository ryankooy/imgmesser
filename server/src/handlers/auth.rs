use axum::{
    body::Body,
    extract::{
        connect_info::ConnectInfo,
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::net::SocketAddr;
use tracing::{error, info};
use uuid::Uuid;

use crate::auth::{self, AuthPayload, Claims};
use crate::db::{self, User};
use crate::state::AppState;

trait ResponseBehavior {}

#[derive(Serialize)]
struct BasicResponse {
    success: bool,
    message: String,
}

impl ResponseBehavior for BasicResponse {}

fn success<T: ResponseBehavior>(resp: T) -> (StatusCode, Json<T>) {
    (StatusCode::OK, Json(resp))
}

fn failure<T: ResponseBehavior>(code: StatusCode, resp: T) -> (StatusCode, Json<T>) {
    (code, Json(resp))
}

/// Route for user registration.
pub async fn register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<User>,
) -> impl IntoResponse {
    info!("Client {addr} is attempting to register");

    let pool = state.db;
    let object_base_path = Uuid::now_v7().to_string();

    match db::insert_user(&pool, payload, object_base_path).await {
        Ok(rows_affected) => {
            if rows_affected == 1u64 {
                success(BasicResponse {
                    success: true,
                    message: "User registered successfully".to_string(),
                })
            } else {
                failure(
                    StatusCode::BAD_REQUEST,
                    BasicResponse {
                        success: false,
                        message: "User could not be registered".to_string(),
                    },
                )
            }
        }
        Err(e) => {
            error!("Failed to register user: {}", e);

            failure(
                StatusCode::INTERNAL_SERVER_ERROR,
                BasicResponse {
                    success: false,
                    message: "Failed to register user".to_string(),
                },
            )
        }
    }
}

/// Route for user login.
pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    json_payload: Json<AuthPayload>,
) -> impl IntoResponse {
    info!("Client {addr} is attempting to log in");

    let pool = state.db;

    match auth::authorize(&pool, json_payload).await {
        Ok(auth_body) => auth_body.into_response(),
        Err(e) => e.into_response(),
    }
}

/// Route for user logout.
pub async fn logout(
    State(state): State<AppState>,
    _claims: Claims,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    json_payload: Json<AuthPayload>,
) -> impl IntoResponse {
    info!("Client {addr} is attempting to log out");

    Response::builder()
        .body(Body::from("logged out"))
        .unwrap()
}
