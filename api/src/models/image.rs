use bytes::Bytes;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::PgRow,
    Error, FromRow, Row,
};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Image database values
pub struct Image {
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

impl<'a> FromRow<'a, PgRow> for Image {
    fn from_row(row: &'a PgRow) -> Result<Self, Error> {
        let content_type_int: i32 = row.try_get("content_type")?;
        let content_type = ContentType::from_int(content_type_int);
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let last_modified: DateTime<Utc> = row.try_get("last_modified")?;

        let image = Image {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            content_type: content_type.to_string(),
            created_at: created_at.to_string(),
            last_modified: last_modified.to_string(),
            version: row.try_get("version")?,
            width: row.try_get("width")?,
            height: row.try_get("height")?,
            size: row.try_get("size")?,
        };

        Ok(image)
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct ImageVersion {
    pub image_id: Uuid,
    pub version: String,
    pub ts: DateTime<Utc>,
}

/// Data for image yet to be uploaded
pub struct UploadImage {
    pub name: String,
    pub content_type: ContentType,
    pub data: Bytes,
    pub dimensions: (u32, u32),
}

/// Image bytes and content type
pub struct ImageData {
    pub content_type: String,
    pub data: Bytes,
}

#[derive(Debug, Serialize)]
pub struct ImageList {
    pub images: Vec<Image>,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Clone)]
pub enum ContentType {
    UNKNOWN,
    JPEG,
    PNG,
    GIF,
    WEBP,
    BMP,
}

impl ContentType {
    pub fn from_str(content_type: &str) -> Self {
        match content_type.strip_prefix("image/").unwrap_or(content_type) {
            "jpg" | "jpeg" => ContentType::JPEG,
            "png" => ContentType::PNG,
            "gif" => ContentType::GIF,
            "webp" => ContentType::WEBP,
            "bmp" => ContentType::BMP,
            _ => ContentType::UNKNOWN,
        }
    }

    pub fn from_int(content_type: i32) -> Self {
        if let Ok(ct) = content_type.try_into() {
            ct
        } else {
            ContentType::UNKNOWN
        }
    }
}

impl TryFrom<i32> for ContentType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == ContentType::JPEG as i32 => Ok(ContentType::JPEG),
            x if x == ContentType::PNG as i32 => Ok(ContentType::PNG),
            x if x == ContentType::GIF as i32 => Ok(ContentType::GIF),
            x if x == ContentType::WEBP as i32 => Ok(ContentType::WEBP),
            x if x == ContentType::BMP as i32 => Ok(ContentType::BMP),
            x if x == ContentType::UNKNOWN as i32 => Ok(ContentType::UNKNOWN),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ContentType::JPEG => write!(f, "image/jpeg"),
            ContentType::PNG => write!(f, "image/png"),
            ContentType::GIF => write!(f, "image/gif"),
            ContentType::WEBP => write!(f, "image/webp"),
            ContentType::BMP => write!(f, "image/bmp"),
            ContentType::UNKNOWN => write!(f, "application/octet-stream"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_enum_from_extension_str() {
        assert_eq!(ContentType::from_str("jpg"), ContentType::JPEG);
    }

    #[test]
    fn test_content_type_enum_from_content_type_str() {
        assert_eq!(ContentType::from_str("image/jpeg"), ContentType::JPEG);
    }

    #[test]
    fn test_content_type_enum_from_unmatched_str() {
        assert_eq!(ContentType::from_str("text/html"), ContentType::UNKNOWN);
    }

    #[test]
    fn test_content_type_enum_from_extension() {
        assert_eq!(ContentType::from_str("jpg"), ContentType::JPEG);
    }

    #[test]
    fn test_content_type_enum_to_string() {
        assert_eq!(ContentType::JPEG.to_string(), "image/jpeg");
    }
}
