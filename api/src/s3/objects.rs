use aws_sdk_s3::{
    operation::{
        delete_object::DeleteObjectOutput,
        get_object::GetObjectOutput,
        list_objects_v2::ListObjectsV2Output,
        put_object::PutObjectOutput,
    },
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;
use std::env;
use tracing::error;

use super::error::S3Error;

type Result<T> = anyhow::Result<T, S3Error>;

/// Upload object to S3 bucket.
pub async fn upload_object(
    client: &Client,
    data: Bytes,
    object_key: &str,
) -> Result<PutObjectOutput> {
    let result = client
        .put_object()
        .bucket(bucket_name())
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
    let object = client
        .get_object()
        .bucket(bucket_name())
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
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name())
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
    let object = client
        .delete_object()
        .bucket(bucket_name())
        .key(object_key.to_string())
        .send()
        .await?;

    Ok(object)
}

fn bucket_name() -> String {
    match env::var("S3_BUCKET_NAME") {
        Ok(name) => name,
        Err(_) => {
            error!("No env variable set for S3 bucket name");
            "imgmesser-storage".to_string()
        }
    }
}
