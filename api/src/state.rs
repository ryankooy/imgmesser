use anyhow::Result;
use aws_sdk_s3::Client as S3Client;
use axum::extract::FromRef;
use sqlx::{Error as SqlxError, PgPool};
use std::sync::Arc;

use crate::{
    db,
    repos::{
        RefreshTokenRepo, RefreshTokenRepoOps, UserRepo, UserRepoOps,
    },
    s3,
};

#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: PgPool,

    /// AWS S3 client
    pub img_store_client: S3Client,

    /// Refresh token repository
    pub refresh_token_repo: Arc<dyn RefreshTokenRepoOps>,

    /// User repository
    pub user_repo: Arc<dyn UserRepoOps>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        let db = db::create_conn_pool().await?;
        //FIXME: sqlx::migrate!("./migrations").run(&db).await?;

        let img_store_client = s3::get_client().await?;
        let refresh_token_repo: Arc<dyn RefreshTokenRepoOps> = Arc::new(
            RefreshTokenRepo::new(db.clone()),
        );
        let user_repo: Arc<dyn UserRepoOps> = Arc::new(
            UserRepo::new(db.clone()),
        );

        Ok(Self {
            db,
            img_store_client,
            refresh_token_repo,
            user_repo,
        })
    }
}
