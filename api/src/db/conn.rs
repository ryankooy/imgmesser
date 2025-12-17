use anyhow::Result;
use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};

use super::config::{get_config, Config};

pub async fn create_conn_pool() -> Result<PgPool> {
    let config: Config = get_config()?;

    let pool = PgPoolOptions::new()
        .max_connections(config.maxconns)
        .connect(config.url.as_str())
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
