use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct UserInfo {
    pub username: String,
    pub object_base_path: String,
}
