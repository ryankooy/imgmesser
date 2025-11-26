use bytes::Bytes;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
pub struct Image {
    pub id: Uuid,
    pub name: String,
    pub extension: String,
    pub version: String,
}

pub struct ImageData {
    pub content_type: String,
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct ImageItem {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub size: i64,
    pub last_modified: String,
    pub content_type: String,
}

#[derive(Debug, Serialize)]
pub struct ImageList {
    pub images: Vec<ImageItem>,
    pub total: usize,
    pub has_more: bool,
}
