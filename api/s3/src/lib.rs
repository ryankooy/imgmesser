//! AWS S3 Operations

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

pub mod error;
pub mod objects;

pub use objects::{
    copy_object, delete_object, delete_object_version, delete_previous_versions,
    get_object, get_objects, upload_object,
};

/// Get AWS S3 client.
pub async fn get_client() -> Result<Client> {
    // Load AWS config from environment
    let config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;

    Ok(Client::new(&config))
}
