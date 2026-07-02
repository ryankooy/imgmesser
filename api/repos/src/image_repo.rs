use async_trait::async_trait;
use aws_sdk_s3::Client as S3Client;
use bytes::Bytes;
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use tracing::error;
use uuid::Uuid;

use db;
use errors::ImageError;
use models::{
    ContentType, Image, ImageData, ImageInfo, ImageList, ImageVersion,
    Transformations, UploadImage, UserInfo,
};
use s3;

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
    async fn upload(&self, images: Vec<UploadImage>, user: UserInfo) -> Result<()>;

    async fn get_one(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<ImageData>>;

    async fn get_metadata_for_one(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<Image>>;

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

    async fn delete_current_version(
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

    async fn transform(
        &self,
        image_id: &str,
        specs: &Transformations,
        user: UserInfo,
    ) -> Result<Option<ImageVersion>>;

    async fn update(
        &self,
        image_id: &str,
        version: &str,
        user: UserInfo,
    ) -> Result<Option<String>>;

    async fn save_copy(
        &self,
        image_id: &str,
        copy_name: &str,
        user: UserInfo,
    ) -> Result<Option<String>>;
}

#[async_trait]
impl ImageRepoOps for ImageRepo {
    /// Upload image(s) to S3 and store metadata in the database.
    async fn upload(
        &self,
        images: Vec<UploadImage>,
        user: UserInfo,
    ) -> Result<()> {
        for image in images {
            upload_image(&self.db, &self.img_store_client, image, &user)
                .await?;
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
        .map_err(|e| ImageError::ReadFailure(e.to_string()))?
        .into_bytes();

        let content_type = ContentType::from_int(image.content_type)
            .to_string();

        Ok(Some(ImageData { content_type, data }))
    }

    /// Get a single image object from S3 along with extra
    /// version info from the database.
    async fn get_metadata_for_one(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<Option<Image>> {
        let id = match Uuid::parse_str(image_id) {
            Ok(id) => id,
            Err(e) => {
                error!("Error parsing image id: {}", e);
                return Ok(None);
            }
        };

        let image: Option<Image> = match db::find_image_with_version_info(
            &self.db,
            &id,
            &user.username,
        )
        .await {
            Ok(img) => img,
            Err(e) => {
                error!("Error getting image and version metadata: {}", e);
                return Ok(None);
            }
        };

        Ok(image)
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

        let mut objects = output.contents().to_vec();

        // Sort the objects in descending order by time modified
        objects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

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
        let start_idx = ((page - 1) * limit) as usize;
        let end_idx = start_idx + limit as usize;
        let mut match_count: usize = 0;

        // Build list of image metadata, selecting only images matching
        // S3 objects and that are within pagination range
        let images: Vec<Image> = objects
            .iter()
            .filter_map(|object| {
                if let Some(key) = object.key() {
                    if let Some(image) = image_map.get(key) {
                        match_count += 1;

                        if match_count > start_idx && match_count <= end_idx {
                            return Some(image.clone());
                        }
                    }
                }
                None
            })
            .collect();

        let total: usize = match_count;
        let has_more = end_idx.min(total) < total;

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

    async fn delete_current_version(
        &self,
        image_id: &str,
        user: UserInfo,
    ) -> Result<()> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // Delete the image's current version
        if let Some(version) = db::delete_current_version(&self.db, &image.id)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?
        {
            // S3 object path of the image
            let image_path = get_object_path(
                &user.object_base_path,
                &image.id,
                &image.name,
            );

            // Delete specified version of the S3 object
            s3::delete_object_version(&self.img_store_client, &image_path, &version)
                .await
                .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;
        }

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

    async fn transform(
        &self,
        image_id: &str,
        specs: &Transformations,
        user: UserInfo,
    ) -> Result<Option<ImageVersion>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // S3 object path of the image
        let image_path = get_object_path(
            &user.object_base_path,
            &image.id,
            &image.name,
        );

        let data: Bytes = s3::get_object(
            &self.img_store_client, &image_path, &image.version,
        )
        .await
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?
        .body
        .collect()
        .await
        .map_err(|e| ImageError::ReadFailure(e.to_string()))?
        .into_bytes();

        let byte_slice: &[u8] = data.as_ref();
        let content_type = ContentType::from_int(image.content_type);

        let transformed_image = transform::transform_image(
            byte_slice,
            &content_type,
            specs
        )
        .map_err(|e| ImageError::TransformFailure(e.to_string()))?;

        match transformed_image {
            Some(image_data) => {
                let data = Bytes::from(image_data);

                if let Some(version) = update_object_version(
                    &self.db,
                    &self.img_store_client,
                    &image,
                    image_path,
                    data,
                    &user.username,
                )
                .await? {
                    Ok(Some(ImageVersion { version }))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        image_id: &str,
        version: &str,
        user: UserInfo,
    ) -> Result<Option<String>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // S3 object path of the image
        let image_path = get_object_path(
            &user.object_base_path,
            &image.id,
            &image.name,
        );

        let new_current_version = db::set_image_version(
            &self.db,
            &image.id,
            version,
            &user.username,
        )
        .await
        .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        if let Some(ref new_version) = new_current_version {
            if new_version != version {
                return Ok(None);
            }
        }

        // Delete previous object versions
        if let Some(output) = s3::delete_previous_versions(
            &self.img_store_client,
            &image_path,
            version,
        )
        .await
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?
        {
            for i in output.errors() {
                eprintln!("error: {:?}", i);
            }
        }

        Ok(new_current_version)
    }

    async fn save_copy(
        &self,
        image_id: &str,
        copy_name: &str,
        user: UserInfo,
    ) -> Result<Option<String>> {
        let image = get_image_info(&self.db, image_id, &user.username)
            .await
            .map_err(|e| ImageError::QueryFailure(e.to_string()))?;

        // S3 object path of the image
        let image_path = get_object_path(
            &user.object_base_path,
            &image.id,
            &image.name,
        );

        let image_copy_id = Uuid::now_v7();
        let image_copy_path = get_object_path(
            &user.object_base_path,
            &image_copy_id,
            copy_name,
        );

        let output = s3::copy_object(
            &self.img_store_client,
            &image_path,
            &image.version,
            &image_copy_path,
        )
        .await
        .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

        if let Err(e) = db::insert_image(
            &self.db,
            &image_copy_id,
            &copy_name,
            ContentType::from_int(image.content_type),
            &user.username,
        )
        .await {
            error!("Image insert failed: {}", e);
        }

        if let Some(image_copy_version) = output.version_id() {
            // Add the image object version in the db
            if let Err(e) = db::insert_image_copy_version(
                &self.db,
                &image.id,
                &image.version,
                &image_copy_id,
                image_copy_version,
            )
            .await {
                error!("Image version insert failed: {}", e);
            }

            return Ok(Some(image_copy_version.to_string()));
        }

        Ok(None)
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

/// Upload an image to S3 and store metadata in the database.
async fn upload_image(
    db: &PgPool,
    s3_client: &S3Client,
    image: UploadImage,
    user: &UserInfo,
) -> Result<()> {
    let mut is_new: bool = true;

    // If an image by the given name exists for this user,
    // get the image id; otherwise, make a new one
    let image_id: Uuid = if let Ok(Some(image_id)) = db::find_image_id_by_name(
        db,
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
        s3_client,
        image.data,
        &image_path,
    )
    .await
    .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

    // If it's a new image, create a db record for it
    if is_new && let Err(e) = db::insert_image(
        db,
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
            db,
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

/// Using the given image bytes, upload a new version of an existing image
/// to S3 and store metadata in the database.
async fn update_object_version(
    db: &PgPool,
    s3_client: &S3Client,
    image_info: &ImageInfo,
    image_path: String,
    data: Bytes,
    username: &str,
) -> Result<Option<String>> {
    let image: Option<Image> = match db::find_image_with_version_info(
        &db,
        &image_info.id,
        username,
    )
    .await {
        Ok(img) => img,
        Err(e) => {
            error!("Error getting image and version metadata: {}", e);
            None
        }
    };

    let image_size = data.len();
    let dimensions: (u32, u32) = transform::get_dimensions(&data)
        .map_err(|e| ImageError::TransformFailure(e.to_string()))?;

    let output = s3::upload_object(
        s3_client,
        data,
        &image_path,
    )
    .await
    .map_err(|e| ImageError::S3OperationFailure(e.to_string()))?;

    if let Some(image_details) = image {
        if let Some(version) = output.version_id() {
            // Add the image object version in the db
            if let Err(e) = db::insert_image_version(
                db,
                &image_details.id,
                version,
                dimensions,
                image_size,
            )
            .await {
                error!("Image version insert failed: {}", e);
            }

            return Ok(Some(version.to_string()))
        }
    }

    Ok(None)
}
