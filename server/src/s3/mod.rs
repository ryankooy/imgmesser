//! AWS S3 Operations

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    operation::get_object::GetObjectOutput,
    operation::put_object::PutObjectOutput,
    primitives::ByteStream,
    Client,
};
use bytes::Bytes;

pub mod error;

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
        .bucket("imgmesser-storage")
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
        .bucket("imgmesser-storage")
        .key(object_key.to_string())
        .send()
        .await?;

    Ok(object)
}
