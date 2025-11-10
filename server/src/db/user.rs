use anyhow::Result;
use serde::Deserialize;
use sqlx::{
    postgres::PgQueryResult,
    PgPool,
};

#[derive(Deserialize)]
pub struct User {
    username: String,
    password: String,
}

/// Insert user into the database and return the
/// number of rows affected.
pub async fn insert_user(
    pool: &PgPool,
    user: User,
    object_base_path: String,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO user_profile (username, password, objectbasepath)
        VALUES ($1, crypt($2, gen_salt('md5')), $3);
        "#,
        user.username,
        user.password,
        object_base_path,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Check given user credentials against the database.
pub async fn validate_user(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<bool, sqlx::Error> {
    match sqlx::query!(
        r#"
        SELECT (password = crypt($2, password)) AS is_match
        FROM user_profile
        WHERE username = $1;
        "#,
        username,
        password,
    )
    .fetch_one(pool)
    .await {
        Ok(result) => Ok(result.is_match.unwrap_or(false)),
        Err(e) => match e {
            sqlx::Error::RowNotFound => Ok(false),
            _ => Err(e),
        }
    }
}

//TODO: make get_user_object_path query func
