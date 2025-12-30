use serde::Serialize;

use models::UserInfo;

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
