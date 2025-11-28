use anyhow::{bail, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use tracing::error;
use uuid::Uuid;

use crate::{
    models::{
        ContentType, Image, ImageData, ImageList,
        UploadImage, UserInfo,
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
        image: UploadImage,
        user: UserInfo,
    ) -> Result<()>;

    async fn get_one(
        &self,
        image_id_string: &str,
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
        image: UploadImage,
        user: UserInfo,
    ) -> Result<()> {
        // The S3 object path of the image
        let image_path = get_object_path(&user.object_base_path, &image.name);

        let image_size = image.data.len();

        // Upload the image to the S3 bucket and get
        // the image's version id
        match s3::upload_object(
            &self.img_store_client,
            image.data,
            &image_path,
        )
        .await {
            Ok(output) => {
                // Create a db record for the image
                if let Err(e) = insert_image(
                    &self.db,
                    &Uuid::now_v7(),
                    &image.name,
                    image.content_type,
                    &user.username,
                )
                .await {
                    error!("Image insert failed: {}", e);
                }

                if let Some(version) = output.version_id() {
                    if let Ok(Some(image_id)) = find_id_by_name(
                        &self.db,
                        &image.name,
                        &user.username,
                    )
                    .await {
                        // Add the image object version in the db
                        if let Err(e) = insert_image_version(
                            &self.db, &image_id, image.dimensions, version, image_size,
                        )
                        .await {
                            error!("Image version insert failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => bail!("File upload error: {}", e),
        };

        Ok(())
    }

    /// Get a single image object from S3.
    async fn get_one(
        &self,
        image_id_string: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>> {
        let image_id = Uuid::parse_str(image_id_string)?;

        if let Some(image) = find_one(&self.db, &image_id, &user.username).await? {
            // S3 object path of the image
            let image_path = get_object_path(&user.object_base_path, &image.name);

            match s3::get_object(&self.img_store_client, &image_path).await {
                Ok(output) => {
                    //TODO: do something with version id? compare?
                    //if let Some(version) = output.version_id() {
                    //    println!("Object version id: {}", version);
                    //    println!("DB version id: {}", image.version);
                    //}

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
        let db_images = find_all(&self.db, &user.username).await?;
        let mut image_map: HashMap<&str, Image> = HashMap::new();
        for image in db_images.iter() {
            image_map.insert(&image.name, image.clone());
        }

        // Calculate pagination
        let start = ((page - 1) * limit) as usize;
        let end = (start + limit as usize).min(total);
        let has_more = end < total;

        // Build image list
        let images: Vec<Image> = objects[start..end]
            .iter()
            .filter_map(|object| {
                if let Some(key) = object.key() {
                    let path = Path::new(key);
                    if let Some(name) = path.file_name() {
                        let name = name.to_string_lossy().into_owned();
                        if let Some(image) = image_map.get(name.as_str()) {
                            let last_modified = object
                                .last_modified()
                                .map(|dt| dt.to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            //TODO:REMOVE:
                            println!();
                            println!("S3 LAST MOD: {}", last_modified);
                            println!("DB LAST MOD: {}", &image.last_modified);
                            println!();

                            return Some(image.clone());
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
    content_type: ContentType,
    username: &str,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO image (id, name, content_type, username)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (name, username) DO NOTHING
        "#,
        id, name, content_type as i32, username,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Insert image version data into the database.
async fn insert_image_version(
    db: &PgPool,
    image_id: &Uuid,
    dimensions: (u32, u32),
    version: &str,
    size: usize,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO image_version (
            image_id, version, current, width, height, size
        )
        VALUES ($1, $2, TRUE, $3, $4, $5)
        "#,
        image_id,
        version,
        dimensions.0 as i32,
        dimensions.1 as i32,
        size as i64,
    )
    .execute(db)
    .await?;

    // Unset the `current` flag for any old versions
    sqlx::query!(
        r#"
        UPDATE image_version SET current = FALSE
        WHERE image_id = $1 AND version <> $2
        "#,
        image_id, version,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Delete an image.
async fn delete_image(
    db: &PgPool,
    image_id: &Uuid,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM image_version WHERE image_id = $1",
        image_id,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Delete the specified version of an image.
async fn delete_image_version(
    db: &PgPool,
    image_id: &Uuid,
    version: &str,
) -> Result<()> {
    sqlx::query!(
        "DELETE FROM image_version WHERE image_id = $1 AND version = $2",
        image_id,
        version,
    )
    .execute(db)
    .await?;

    // Set the `current` flag for the next most recent version
    sqlx::query!(
        r#"
        UPDATE image_version SET current = TRUE
        WHERE version = (
            SELECT version FROM image_version
            WHERE image_id = $1 AND version <> $2
            ORDER BY ts DESC LIMIT 1
        )
        "#,
        image_id, version,
    )
    .execute(db)
    .await?;

    Ok(())
}

/// Retrieve datbase data for a single image.
async fn find_one(
    db: &PgPool,
    id: &Uuid,
    username: &str,
) -> Result<Option<Image>> {
    let image = sqlx::query_as::<_, Image>(
        r#"
        SELECT i.id, i.name, i.content_type, i.created_at,
            iv.ts AS last_modified, iv.version,
            iv.width, iv.height, iv.size
        FROM image AS i
        LEFT JOIN image_version AS iv
            ON iv.image_id = i.id
        WHERE iv.current
            AND i.id = $1
            AND i.username = $2
        "#,
    )
    .bind(id)
    .bind(username)
    .fetch_optional(db)
    .await?;

    Ok(image)
}

/// Retrieve datbase data for a single image.
async fn find_id_by_name(
    db: &PgPool,
    name: &str,
    username: &str,
) -> Result<Option<Uuid>> {
    let image_id: Option<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM image WHERE name = $1 and username = $2",
        name,
        username,
    )
    .fetch_optional(db)
    .await?;

    Ok(image_id)
}

/// Retrieve database data for all of the given user's images.
async fn find_all(
    db: &PgPool,
    username: &str,
) -> Result<Vec<Image>> {
    let images = sqlx::query_as::<_, Image>(
        r#"
        SELECT i.id, i.name, i.content_type, i.created_at,
            iv.ts AS last_modified, iv.version,
            iv.width, iv.height, iv.size
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

fn get_object_path(base_path: &str, image_name: &str) -> String {
    format!("{}/{}", base_path, image_name)
}
