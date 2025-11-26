use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, FromRow)]
/// Image database values
pub struct Image {
    pub id: Uuid,
    pub name: String,
    pub extension: String,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub version: String,
    pub width: i32,
    pub height: i32,
}

/// Data for image yet to be uploaded
pub struct UploadImage {
    pub name: String,
    pub data: Bytes,
    pub dimensions: (u32, u32),
}

/// Image bytes and content type
pub struct ImageData {
    pub content_type: String,
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct ImageItem {
    pub id: Uuid,
    pub name: String,
    pub content_type: String,
    pub created_at: String,
    pub last_modified: String,
    pub version: String,
    pub width: i32,
    pub height: i32,
    pub size: i64,
}

#[derive(Debug, Serialize)]
pub struct ImageList {
    pub images: Vec<ImageItem>,
    pub total: usize,
    pub has_more: bool,
}
