use serde::{Deserialize, Serialize};

use crate::models::{User, UserInfo};

#[derive(Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
}

impl User {
    pub fn from_request(req: &UserRequest) -> Self {
        Self {
            username: req.username.clone(),
            password: req.password.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: UserInfo,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub user: UserInfo,
}
