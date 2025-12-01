use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use tracing::error;
use uuid::Uuid;

use crate::{
    db,
    models::{
        Image, ImageData, ImageList, UploadImage, UserInfo,
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
    async fn upload(&self, image: UploadImage, user: UserInfo) -> Result<()>;

    async fn get_one(
        &self,
        image_id_str: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>>;

    async fn get_all(
        &self,
        user: UserInfo,
        page: u32,
        limit: u32,
    ) -> Result<ImageList>;

    async fn delete(&self, image_id_str: &str, user: UserInfo) -> Result<()>;

    async fn revert(&self, image_id_str: &str, user: UserInfo) -> Result<()>;

    async fn unrevert(&self, image_id_str: &str, user: UserInfo) -> Result<()>;
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
        let image_path = get_object_path(&user.object_base_path, &image.name);
        let image_size = image.data.len();

        // Upload the image to the S3 bucket and get
        // the image's version id
        let output = s3::upload_object(
            &self.img_store_client,
            image.data,
            &image_path,
        )
        .await
        .map_err(|e| anyhow!("File upload error: {}", e))?;

        // Create a db record for the image
        if let Err(e) = db::insert_image(
            &self.db,
            &Uuid::now_v7(),
            &image.name,
            image.content_type,
            &user.username,
        )
        .await {
            error!("Image insert failed: {}", e);
        }

        if let Ok(Some(image_id)) = db::find_id_by_name(
            &self.db,
            &image.name,
            &user.username,
        )
        .await {
            let version_id = output.version_id().unwrap_or("");

            // Add the image object version in the db
            if let Err(e) = db::insert_image_version(
                &self.db,
                &image_id,
                version_id,
                image.dimensions,
                image_size,
            )
            .await {
                error!("Image version insert failed: {}", e);
            }
        }

        Ok(())
    }

    /// Get a single image object from S3.
    async fn get_one(
        &self,
        image_id_str: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>> {
        match get_metadata(&self.db, image_id_str, &user.username).await {
            Ok(image) => {
                // S3 object path of the image
                let image_path = get_object_path(
                    &user.object_base_path,
                    &image.name,
                );

                match s3::get_object(
                    &self.img_store_client, &image_path, &image.version,
                )
                .await {
                    Ok(output) => {
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
            Err(e) => error!("Error getting image metadata: {}", e),
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
        .map_err(|e| anyhow!("Error getting S3 object: {}", e))?;

        let objects = output.contents().to_vec();
        let total = objects.len();

        // Get image metadata from db
        let db_images = db::find_all(&self.db, &user.username).await?;

        // Build map of image metadata
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
//FIXME: do we need last_modified from S3?
//                            let last_modified = object
//                                .last_modified()
//                                .map(|dt| dt.to_string())
//                                .unwrap_or_else(|| "unknown".to_string());
//
//                            //TODO:REMOVE:
//                            println!();
//                            println!("S3 LAST MOD: {}", last_modified);
//                            println!("DB LAST MOD: {}", &image.last_modified);
//                            println!();

                            return Some(image.clone());
                        }
                    }
                }
                None
            })
            .collect();

        Ok(ImageList { images, total, has_more })
    }

    async fn delete(
        &self,
        image_id_str: &str,
        user: UserInfo,
    ) -> Result<()> {
        let image = get_metadata(&self.db, image_id_str, &user.username).await?;

        // Delete the image's metadata
        db::delete_image(&self.db, &image.id).await?;

        // S3 object path of the image
        let image_path = get_object_path(&user.object_base_path, &image.name);

        // Delete the S3 object
        s3::delete_object(&self.img_store_client, &image_path)
            .await
            .map_err(|e| anyhow!("Error deleting S3 object: {}", e))?;

        Ok(())
    }

    async fn revert(
        &self,
        image_id_str: &str,
        user: UserInfo,
    ) -> Result<()> {
        let image = get_metadata(&self.db, image_id_str, &user.username).await?;

        // Revert image to previous version
        let new_current_version = db::revert_image(&self.db, &image.id)
            .await?
            .ok_or(anyhow!("Error reverting image version"))?;

        if new_current_version == image.version {
            bail!("Failed to revert image");
        }

        Ok(())
    }

    async fn unrevert(
        &self,
        image_id_str: &str,
        user: UserInfo,
    ) -> Result<()> {
        let image = get_metadata(&self.db, image_id_str, &user.username).await?;

        // Restore image to newer version
        let new_current_version = db::unrevert_image(&self.db, &image.id)
            .await?
            .ok_or(anyhow!("Error restoring image to newer version"))?;

        if new_current_version == image.version {
            bail!("Failed to restore image to newer version");
        }

        Ok(())
    }
}

/// Get image metadata and return it or an error if not found.
async fn get_metadata(
    db: &PgPool,
    image_id_str: &str,
    username: &str,
) -> Result<Image> {
    let image_id = Uuid::parse_str(image_id_str)?;
    if let Some(image) = db::find_one(db, &image_id, username).await? {
        Ok(image)
    } else {
        bail!("Image not found");
    }
}

/// Get the S3 object path of the image.
fn get_object_path(base_path: &str, image_name: &str) -> String {
    format!("{}/{}", base_path, image_name)
}
