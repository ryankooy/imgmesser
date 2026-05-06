use aws_sdk_s3::{
    operation::{
        delete_object::DeleteObjectOutput,
        delete_objects::DeleteObjectsOutput,
        get_object::GetObjectOutput,
        list_objects_v2::ListObjectsV2Output,
        put_object::PutObjectOutput,
    },
    primitives::ByteStream,
    types::{ObjectIdentifier, Delete},
    Client,
};
use bytes::Bytes;
use tracing::error;

use config;
use super::error::S3Error;

type Result<T> = anyhow::Result<T, S3Error>;

/// Upload object to S3 bucket.
pub async fn upload_object(
    client: &Client,
    data: Bytes,
    object_key: &str,
) -> Result<PutObjectOutput> {
    let bucket_name = get_bucket_name().await;
    let result = client
        .put_object()
        .bucket(bucket_name)
        .key(object_key.to_string())
        .body(ByteStream::from(data))
        .send()
        .await
        .map_err(S3Error::from)?;

    Ok(result)
}

/// Retrieve specific version of object from S3 bucket.
pub async fn get_object(
    client: &Client,
    object_key: &str,
    version_id: &str,
) -> Result<GetObjectOutput> {
    let bucket_name = get_bucket_name().await;
    let object = client
        .get_object()
        .bucket(bucket_name)
        .key(object_key.to_string())
        .version_id(version_id.to_string())
        .send()
        .await?;

    Ok(object)
}

/// Retrieve all objects from S3 bucket.
pub async fn get_objects(
    client: &Client,
    prefix: &str,
) -> Result<ListObjectsV2Output> {
    let bucket_name = get_bucket_name().await;
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .prefix(prefix)
        .send()
        .await?;

    Ok(objects)
}

/// Delete object from S3 bucket.
pub async fn delete_object(
    client: &Client,
    object_key: &str,
) -> Result<DeleteObjectOutput> {
    let bucket_name = get_bucket_name().await;
    let object = client
        .delete_object()
        .bucket(bucket_name)
        .key(object_key.to_string())
        .send()
        .await?;

    Ok(object)
}

/// Delete object from S3 bucket.
pub async fn delete_previous_versions(
    client: &Client,
    object_key: &str,
    current_version_id: &str,
) -> Result<Option<DeleteObjectsOutput>> {
    //let bucket_name = get_bucket_name().await;

    // Fetch object's versions and delete markers
    let output = client
        .list_object_versions()
        .bucket(get_bucket_name().await)
        .prefix(object_key.to_string())
        .send()
        .await?;

    let mut to_delete = Vec::new();

    // Identify non-current versions
    for version in output.versions() {
        if let Some(version_id) = version.version_id() {
            if version_id != current_version_id {
                if let Ok(object_id) = ObjectIdentifier::builder()
                    .set_key(version.key().map(|s| s.to_string()))
                    .set_version_id(Some(version_id.to_string()))
                    .build()
                {
                    to_delete.push(object_id);
                }
            }
        }
    }

    // Identify old delete markers
    //if let Some(markers) = output.delete_markers() {
    for marker in output.delete_markers() {
        if !marker.is_latest().unwrap_or(true) {
            if let Ok(object_id) = ObjectIdentifier::builder()
                .set_key(marker.key().map(|s| s.to_string()))
                .set_version_id(marker.version_id.clone())
                .build()
            {
                to_delete.push(object_id);
            }
        }
    }

    if !to_delete.is_empty() {
        if let Ok(delete_request) = Delete::builder()
            .set_objects(Some(to_delete))
            .build()
        {
            // Do the deletes
            let delete_output = client
                .delete_objects()
                .bucket(get_bucket_name().await)
                .delete(delete_request)
                .send()
                .await?;

            return Ok(Some(delete_output));
        }
    }

    Ok(None)
}

async fn get_bucket_name() -> String {
    match config::get_s3_bucket_name().await {
        Ok(name) => name,
        Err(e) => {
            error!("No env variable set for S3 bucket name: {}", e);
            "unknown".to_string()
        }
    }
}
