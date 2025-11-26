//! AWS S3 Operations

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    operation::{
        get_object::GetObjectOutput,
        list_objects_v2::ListObjectsV2Output,
        put_object::PutObjectOutput,
    },
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;

pub mod error;

const S3_BUCKET_NAME: &str = "imgmesser-storage";

/// Get AWS S3 client.
pub async fn get_client() -> Result<Client> {
    // Load AWS config from environment
    let config = aws_config::defaults(BehaviorVersion::latest())
        .profile_name("imgmesser-client")
        .load()
        .await;

    Ok(Client::new(&config))
}

/// Upload object to S3 bucket.
pub async fn upload_object(
    client: &Client,
    data: Bytes,
    object_key: &str,
) -> Result<PutObjectOutput, error::S3Error> {
    let result = client
        .put_object()
        .bucket(S3_BUCKET_NAME)
        .key(object_key.to_string())
        .body(ByteStream::from(data))
        .send()
        .await
        .map_err(error::S3Error::from)?;

    Ok(result)
}

/// Retrieve object from S3 bucket.
pub async fn get_object(
    client: &Client,
    object_key: &str,
) -> Result<GetObjectOutput, error::S3Error> {
    let object = client
        .get_object()
        .bucket(S3_BUCKET_NAME)
        .key(object_key.to_string())
        .send()
        .await?;

    Ok(object)
}

/// Retrieve all objects from S3 bucket.
pub async fn get_objects(
    client: &Client,
    prefix: &str,
) -> Result<ListObjectsV2Output, error::S3Error> {
    let objects = client
        .list_objects_v2()
        .bucket(S3_BUCKET_NAME)
        .prefix(prefix)
        .send()
        .await?;

    Ok(objects)
}
