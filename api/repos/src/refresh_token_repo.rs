use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Error as SqlxError, PgPool};

use models::RefreshToken;

#[derive(Clone)]
pub struct RefreshTokenRepo {
    db: PgPool,
}

impl RefreshTokenRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
pub trait RefreshTokenRepoOps: Send + Sync {
    async fn create_token(
        &self,
        username: &str,
        token: &str,
    ) -> Result<RefreshToken, SqlxError>;

    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<RefreshToken>, SqlxError>;

    async fn update_last_used(&self, token: &str) -> Result<(), SqlxError>;

    async fn delete_token(&self, token: &str) -> Result<(), SqlxError>;

    async fn delete_all_user_tokens(&self, username: &str) -> Result<(), SqlxError>;

    async fn mark_token_used(&self, token: &str) -> Result<(), SqlxError>;
}

#[async_trait]
impl RefreshTokenRepoOps for RefreshTokenRepo {
    async fn create_token(
        &self,
        username: &str,
        token: &str,
    ) -> Result<RefreshToken, SqlxError> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (username, token)
            VALUES ($1, $2)
            RETURNING id, username, token, expires_at, is_used,
                used_at, created_at, last_used_at
            "#,
        )
        .bind(username)
        .bind(token)
        .fetch_one(&self.db)
        .await?;

        Ok(refresh_token)
    }

    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<RefreshToken>, SqlxError> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, username, token, expires_at, is_used,
                used_at, created_at, last_used_at
            FROM refresh_tokens WHERE token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?;

        Ok(refresh_token)
    }

    async fn update_last_used(&self, token: &str) -> Result<(), SqlxError> {
        sqlx::query!(
            "UPDATE refresh_tokens SET last_used_at = $1 WHERE token = $2",
            Utc::now(),
            token,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn delete_token(&self, token: &str) -> Result<(), SqlxError> {
        sqlx::query!(
            "DELETE FROM refresh_tokens WHERE token = $1",
            token,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn delete_all_user_tokens(&self, username: &str) -> Result<(), SqlxError> {
        sqlx::query!(
            "DELETE FROM refresh_tokens WHERE username = $1",
            username,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn mark_token_used(&self, token: &str) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"
            UPDATE refresh_tokens
            SET is_used = TRUE, used_at = $1
            WHERE token = $2
            "#,
            Utc::now(),
            token,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}
