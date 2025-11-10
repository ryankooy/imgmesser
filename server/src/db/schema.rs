use anyhow::Result;
use sqlx::PgPool;

/// Create database tables if they don't exist.
pub async fn create_schema(pool: &PgPool) -> Result<()> {
    sqlx::query!(r#"
        CREATE TABLE IF NOT EXISTS user_profile (
            username text,
            password varchar(100) NOT NULL,
            objectbasepath text NOT NULL,
            PRIMARY KEY (username)
        );
    "#)
    .execute(pool).await?;

    sqlx::query!(r#"
        CREATE TABLE IF NOT EXISTS image (
            id uuid,
            name text,
            extension text,
            username text NOT NULL,
            PRIMARY KEY (id),
            FOREIGN KEY (username) REFERENCES user_profile (username)
        );
    "#)
    .execute(pool).await?;

    sqlx::query!(r#"
        CREATE TABLE IF NOT EXISTS image_version (
            imageid uuid,
            version text,
            ts timestamptz NOT NULL,
            latest boolean NOT NULL,
            PRIMARY KEY (imageid, version),
            FOREIGN KEY (imageid) REFERENCES image (id)
        );
    "#)
    .execute(pool).await?;

    sqlx::query!("CREATE EXTENSION IF NOT EXISTS pgcrypto;")
        .execute(pool).await?;

    Ok(())
}
