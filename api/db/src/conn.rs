use anyhow::Result;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

use config::DbConfig;

pub async fn create_conn_pool() -> Result<PgPool> {
    let config: DbConfig = config::get_db_config().await?;

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(config.url.as_str())
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
