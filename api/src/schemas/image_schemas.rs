use serde::{Deserialize, Serialize};

use crate::models::Image;

#[derive(Debug, Deserialize)]
struct ImageRequest {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct ImageResponse {
    pub image: Image,
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
