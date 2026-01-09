//! AWS S3 Operations

use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_s3::Client;

pub mod error;
pub mod objects;

pub use objects::{
    delete_object, get_env_object, get_object,
    get_objects, upload_object,
};

/// Get AWS S3 client.
pub async fn get_client() -> Result<Client> {
    // Load AWS config from environment
    let config = aws_config::defaults(BehaviorVersion::latest())
        .load()
        .await;

    Ok(Client::new(&config))
}
