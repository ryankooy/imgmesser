use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use tracing::error;
use uuid::Uuid;

use crate::{
    db,
    errors::ImageError,
    models::{
        ContentType, Image, ImageData, ImageInfo, ImageList,
        UploadImage, UserInfo,
    },
    s3,
};

type Result<T> = anyhow::Result<T, ImageError>;

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
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>>;

    async fn get_metadata_for_all(
        &self,
        user: UserInfo,
        page: u32,
        limit: u32,
    ) -> Result<ImageList>;

    async fn delete(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<()>;

    async fn revert(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<String>>;

    async fn restore(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<String>>;

    async fn rename(
        &self,
        image_id: &str,
        new_name: &str,
        user: UserInfo,
    ) -> Result<Option<String>>;
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
        let mut is_new: bool = true;

        // If an image by the given name exists for this user,
        // get the image id; otherwise, make a new one
        let image_id: Uuid = if let Ok(Some(image_id)) = db::find_image_id_by_name(
            &self.db,
            &image.name,
            &user.username,
        )
        .await {
            is_new = false;
            image_id
        } else {
            Uuid::now_v7()
        };

        let image_path = get_object_path(
            &user.object_base_path,
            &image_id,
            &image.name,
        );
        let image_size = image.data.len();

        // Upload the image to the S3 bucket and get
        // the image's version id
        let output = s3::upload_object(
            &self.img_store_client,
            image.data,
            &image_path,
        )
        .await
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

        // If it's a new image, create a db record for it
        if is_new && let Err(e) = db::insert_image(
            &self.db,
            &image_id,
            &image.name,
            image.content_type,
            &user.username,
        )
        .await {
            error!("Image insert failed: {}", e);
        }

        if let Some(version) = output.version_id() {
            // Add the image object version in the db
            if let Err(e) = db::insert_image_version(
                &self.db,
                &image_id,
                version,
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
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>> {
        let image = match get_image_info(&self.db, image_id, &user.username).await {
            Ok(img) => img,
            Err(e) => {
                error!("Error getting image metadata: {}", e);
                return Ok(None);
            }
        };

        // S3 object path of the image
        let image_path = get_object_path(
            &user.object_base_path,
            &image.id,
            &image.name,
        );

        let data = s3::get_object(
            &self.img_store_client, &image_path, &image.version,
        )
        .await
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?
        .body
        .collect()
        .await
        .map_err(|_| ImageError::ReadFailure)?
        .into_bytes();

        let content_type = ContentType::from_int(image.content_type)
            .to_string();

        Ok(Some(ImageData { content_type, data }))
    }

    /// Get metadata for all of a user's images.
    async fn get_metadata_for_all(
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
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

        let objects = output.contents().to_vec();
        let total = objects.len();

        // Get image metadata from db
        let db_images = db::find_all_images(&self.db, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Build map of image metadata
        let mut image_map: HashMap<String, Image> = HashMap::new();
        for image in db_images.iter() {
            let image_path = get_object_path(
                &user.object_base_path,
                &image.id,
                &image.name,
            );
            image_map.insert(image_path, image.clone());
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
                    if let Some(image) = image_map.get(key) {
                        return Some(image.clone());
                    }
                }
                None
            })
            .collect();

        Ok(ImageList { images, total, has_more })
    }

    async fn delete(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<()> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Delete the image's metadata
        db::delete_image(&self.db, &image.id)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // S3 object path of the image
        let image_path = get_object_path(
            &user.object_base_path,
            &image.id,
            &image.name,
        );

        // Delete the S3 object
        s3::delete_object(&self.img_store_client, &image_path)
            .await
            .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

        Ok(())
    }

    async fn revert(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<String>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Revert image to previous version
        let new_current_version = db::revert_image_version(&self.db, &image.id)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        if let Some(ref version) = new_current_version {
            if version == &image.version {
                return Ok(None);
            }
        }

        Ok(new_current_version)
    }

    async fn restore(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<String>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Restore image to newer version
        let new_current_version = db::restore_image_version(&self.db, &image.id)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        if let Some(ref version) = new_current_version {
            if version == &image.version {
                return Ok(None);
            }
        }

        Ok(new_current_version)
    }

    async fn rename(
        &self,
        image_id: &str,
        new_name: &str,
        user: UserInfo,
    ) -> Result<Option<String>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Update image's name
        let image_name = db::rename_image(&self.db, &image.id, new_name)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        Ok(image_name)
    }
}

/// Get image metadata and return it or an error if not found.
async fn get_image_info(
    db: &PgPool,
    image_id: &str,
    username: &str,
) -> anyhow::Result<ImageInfo> {
    let id = Uuid::parse_str(image_id)?;
    if let Some(image) = db::find_image(db, &id, username).await? {
        Ok(image)
    } else {
        anyhow::bail!("Image not found");
    }
}

/// Get the S3 object path of the image.
fn get_object_path(
    base_path: &str,
    image_id: &Uuid,
    image_name: &str,
) -> String {
    let extension = Path::new(&image_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("jpg");

    format!("{}/{}.{}", base_path, image_id, extension)
}
