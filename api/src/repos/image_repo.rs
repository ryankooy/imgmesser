use anyhow::{bail, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use bytes::Bytes;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::error;
use uuid::Uuid;

use crate::{
    models::{
        Image, ImageData, ImageItem, ImageList, UserInfo,
    },
    s3,
};

#[derive(Clone)]
pub struct ImageRepo {
    db: PgPool,
    img_store_client: S3Client,
}

impl ImageRepo {
    pub fn new(db: PgPool, img_store_client: S3Client) -> Self {
        Self { db, img_store_client }
    }
}

#[async_trait]
pub trait ImageRepoOps: Send + Sync {
    async fn upload(
        &self,
        data: Bytes,
        filename: &str,
        user: UserInfo,
    ) -> Result<()>;

    async fn get_one(
        &self,
        image_name: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>>;

    async fn get_all(
        &self,
        user: UserInfo,
        page: u32,
        limit: u32,
    ) -> Result<ImageList>;
}

#[async_trait]
impl ImageRepoOps for ImageRepo {
    /// Upload an image to S3 and save the image's
    /// metadata to the database.
    async fn upload(
        &self,
        data: Bytes,
        filename: &str,
        user: UserInfo,
    ) -> Result<()> {
        let path = PathBuf::from(&filename);
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");

        // The S3 object path of the image
        let image_path = get_object_path(&user.object_base_path, &filename);

        // Upload the image to the S3 bucket and get
        // the image's version id
        match s3::upload_object(
            &self.img_store_client,
            data,
            &image_path,
        )
        .await {
            Ok(output) => {
                let image_id = Uuid::now_v7();

                // Create a db record for the image
                insert_image(
                    &self.db, &image_id, &filename, &extension, &user.username,
                )
                .await?;

                if let Some(version) = output.version_id() {
                    // Add the image object version in the db
                    insert_image_version(&self.db, &image_id, version).await?;
                }
            }
            Err(e) => bail!("File upload error: {}", e),
        };

        Ok(())
    }

    /// Get a single image object from S3.
    async fn get_one(
        &self,
        image_name: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>> {
        if let Some(image) = find_one(&self.db, image_name, &user.username).await? {
            // S3 object path of the image
            let image_path = get_object_path(&user.object_base_path, image_name);

            match s3::get_object(&self.img_store_client, &image_path).await {
                Ok(output) => {
                    //TODO: do something with version id? compare?
                    if let Some(version) = output.version_id() {
                        //TODO:REMOVE:
                        println!();
                        println!("Object version id: {}", version);
                        println!("DB version id: {}", image.version);
                        println!();
                    }

                    let content_type = output
                        .content_type()
                        .unwrap_or("image/jpeg")
                        .to_string();

                    let data = output.body
                        .collect()
                        .await?
                        .into_bytes();

                    return Ok(Some(ImageData { content_type, data }));
                }
                Err(e) => error!("Error getting S3 object: {}", e),
            }
        }

        Ok(None)
    }

    /// Get metadata for all of a user's images.
    async fn get_all(
        &self,
        user: UserInfo,
        page: u32,
        limit: u32,
    ) -> Result<ImageList> {
        // List images in S3
        let output = s3::get_objects(
            &self.img_store_client,
            &user.object_base_path,
        )
        .await
        .map_err(|e| {
            anyhow::anyhow!("Error getting S3 object: {}", e)
        })?;

        let objects = output.contents().to_vec();
        let total = objects.len();

        // Get image metadata from db and build map
        let db_images: Vec<Image> = find_all(&self.db, &user.username).await?;
        let mut image_map: HashMap<&str, &Image> = HashMap::new();
        for image in db_images.iter() {
            image_map.insert(&image.name, image);
        }

        // Calculate pagination
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(total);
        let has_more = end < total;

        // Build image list
        let images: Vec<ImageItem> = objects[start..end]
            .iter()
            .filter_map(|object| {
                if let Some(key) = object.key() {
                    let path = Path::new(key);
                    if let Some(name) = path.file_name() {
                        let name = name.to_string_lossy().into_owned();
                        if let Some(db_image) = image_map.get(name.as_str()) {
                            return Some(ImageItem {
                                id: db_image.id,
                                name,
                                version: db_image.version.to_owned(),
                                size: object.size().unwrap_or(0),
                                last_modified: object.last_modified()
                                    .map(|dt| dt.to_string())
                                    .unwrap_or_else(|| {
                                        "unknown".to_string()
                                    }),
                                content_type: get_content_type(
                                    &db_image.extension,
                                )
                                .to_string(),
                            })
                        }
                    }
                }
                None
            })
            .collect();

        Ok(ImageList { images, total, has_more })
    }
}

/// Insert image record into the database.
async fn insert_image(
    db: &PgPool,
    id: &Uuid,
    name: &str,
    extension: &str,
    username: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO image (id, name, extension, username)
        VALUES ($1, $2, $3, $4)
        "#,
        id, name, extension, username,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Insert image version data into the database.
async fn insert_image_version(
    db: &PgPool,
    image_id: &Uuid,
    version: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        WITH update_latest AS (
            UPDATE image_version
            SET latest = FALSE
            WHERE image_id = $1
        )
        INSERT INTO image_version (image_id, version, latest)
        VALUES ($1, $2, TRUE)
        "#,
        image_id,
        version,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Retrieve datbase data for a single image.
async fn find_one(
    db: &PgPool,
    name: &str,
    username: &str,
) -> Result<Option<Image>> {
    let image = sqlx::query_as::<_, Image>(
        r#"
        SELECT i.id, i.name, i.extension, iv.version
        FROM image AS i
        LEFT JOIN image_version AS iv
            ON iv.image_id = i.id
        WHERE iv.latest
            AND i.name = $1
            AND i.username = $2
        "#,
    )
    .bind(name)
    .bind(username)
    .fetch_optional(db)
    .await?;

    Ok(image)
}

/// Retrieve database data for all of the given user's images.
async fn find_all(
    db: &PgPool,
    username: &str,
) -> Result<Vec<Image>> {
    let images = sqlx::query_as::<_, Image>(
        r#"
        SELECT i.id, i.name, i.extension, iv.version
        FROM image AS i
        LEFT JOIN image_version AS iv
            ON iv.image_id = i.id
        "#,
    )
    .bind(username)
    .fetch_all(db)
    .await?;

    Ok(images)
}

fn get_content_type(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn get_object_path(base_path: &str, image_name: &str) -> String {
    format!("{}/{}", base_path, image_name)
}
