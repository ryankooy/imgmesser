use async_trait::async_trait;
use sqlx::{Error as SqlxError, PgPool};
use uuid::Uuid;

use models::{User, UserInfo};

/// Result returning sqlx::Error on errors.
type Result<T> = anyhow::Result<T, SqlxError>;

#[derive(Clone)]
pub struct UserRepo {
    db: PgPool,
}

impl UserRepo {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
pub trait UserRepoOps: Send + Sync {
    async fn create(&self, user: &User) -> Result<UserInfo>;

    async fn authorize(&self, user: &User) -> Result<bool>;

    async fn find(&self, username: &str) -> Result<Option<UserInfo>>;
}

#[async_trait]
impl UserRepoOps for UserRepo {
    async fn create(&self, user: &User) -> Result<UserInfo> {
        let object_base_path = Uuid::now_v7().to_string();

        let user_info = sqlx::query_as::<_, UserInfo>(
            r#"
            INSERT INTO user_profile (username, password, object_base_path)
            VALUES ($1, crypt($2, gen_salt('md5')), $3)
            RETURNING username, object_base_path
            "#,
        )
        .bind(&user.username)
        .bind(&user.password)
        .bind(&object_base_path)
        .fetch_one(&self.db)
        .await?;

        Ok(user_info)
    }

    async fn authorize(&self, user: &User) -> Result<bool> {
        match sqlx::query!(
            r#"
            SELECT (password = crypt($2, password)) AS is_match
            FROM user_profile WHERE username = $1
            "#,
            &user.username,
            &user.password,
        )
        .fetch_one(&self.db)
        .await {
            Ok(result) => Ok(result.is_match.unwrap_or(false)),
            Err(e) => match e {
                SqlxError::RowNotFound => Ok(false),
                _ => Err(e),
            }
        }
    }

    async fn find(
        &self,
        username: &str,
    ) -> Result<Option<UserInfo>> {
        let user_info = sqlx::query_as::<_, UserInfo>(
            r#"
            SELECT username, object_base_path
            FROM user_profile WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.db)
        .await?;

        Ok(user_info)
    }
}
