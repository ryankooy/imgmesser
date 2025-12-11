use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ImageRenameRequest {
    pub image_name: String,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 10 }

#[derive(Serialize)]
pub struct ImageUpdateResponse {
    pub updated: bool,
}
