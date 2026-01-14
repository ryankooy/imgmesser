use anyhow::{anyhow, bail, Context, Result};
use aws_config::{
    meta::region::RegionProviderChain,
    BehaviorVersion,
};
use aws_sdk_ssm::Client;
use axum::http::HeaderValue;
use dotenv;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process::Command;
use std::str;

pub struct Addresses {
    pub listener: SocketAddr,
    pub origin: HeaderValue,
}

impl Addresses {
    pub fn new(origin_address: &str, listener_port: &str, is_prod: bool) -> Result<Self> {
        let origin = origin_address
            .parse::<HeaderValue>()
            .context("Failed to parse origin address as header value")?;

        let port = listener_port
            .parse::<u16>()
            .context("Failed to parse listener port")?;

        let listener = if is_prod {
            SocketAddr::from(([0, 0, 0, 0], port))
        } else {
            SocketAddr::from(([127, 0, 0, 1], port))
        };

        Ok(Self { listener, origin })
    }
}

pub struct DbConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DbConfig {
    pub fn new(database_url: &str, max_conns: &str) -> Result<Self> {
        let max_connections = max_conns
            .parse::<u32>()
            .context("Failed to parse max conns value as u32")?;

        Ok(Self {
            url: database_url.to_string(),
            max_connections,
        })
    }
}

/// Make a new AWS SSM client.
pub async fn get_ssm_client() -> Result<Client> {
    let region_provider = RegionProviderChain::default_provider()
        .or_else("us-east-1");

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(region_provider)
        .load()
        .await;

    Ok(Client::new(&config))
}

/// Get the value of a specific SSM parameter.
pub async fn get_ssm_param(client: &Client, name: &str, decrypt: bool) -> Result<Option<String>> {
    let param_name = format!("/imgmesser/{}", name);
    let response = client
        .get_parameter()
        .name(param_name)
        .with_decryption(decrypt)
        .send()
        .await?;

    if let Some(param) = response.parameter {
        return Ok(param.value);
    }

    Ok(None)
}

/// Get the server's listening IP address and accepted origin IP address.
pub async fn get_addresses() -> Result<Addresses> {
    match get_env().as_str() {
        "prod" => {
            let ssm_client = get_ssm_client().await?;

            let origin_address = get_ssm_param(&ssm_client, "origin-address", false)
                .await?
                .ok_or(anyhow!("Empty SSM parameter"))?;

            let listener_port = get_ssm_param(&ssm_client, "listener-port", false)
                .await?
                .ok_or(anyhow!("Empty SSM parameter"))?;

            let addresses = Addresses::new(&origin_address, &listener_port, true)?;
            Ok(addresses)
        }
        _ => {
            load_env()?;

            let origin_address = env::var("ORIGIN_ADDRESS")
                .context("Missing env variable: ORIGIN_ADDRESS")?;

            let listener_port = env::var("LISTENER_PORT")
                .context("Missing env variable: LISTENER_PORT")?;

            let addresses = Addresses::new(&origin_address, &listener_port, false)?;
            Ok(addresses)
        }
    }
}

/// Get the configured PostgreSQL database URL and maximum connections value.
pub async fn get_db_config() -> Result<DbConfig> {
    match get_env().as_str() {
        "prod" => {
            let ssm_client = get_ssm_client().await?;

            let database_url = get_ssm_param(&ssm_client, "database-url", true)
                .await?
                .ok_or(anyhow!("Empty SSM parameter"))?;

            let max_conns = get_ssm_param(&ssm_client, "max-conns", false)
                .await?
                .unwrap_or_else(|| "5".to_string());

            let db_config = DbConfig::new(&database_url, &max_conns)?;
            Ok(db_config)
        }
        _ => {
            load_env()?;

            let database_url = env::var("DATABASE_URL")
                .context("Missing env variable: DATABASE_URL")?;

            let max_conns = env::var("MAX_CONNS")
                .context("Missing env variable: MAX_CONNS")?;

            let db_config = DbConfig::new(&database_url, &max_conns)?;
            Ok(db_config)
        }
    }
}

/// Get the configured AWS S3 bucket name.
pub async fn get_s3_bucket_name() -> Result<String> {
    match get_env().as_str() {
        "prod" => {
            let ssm_client = get_ssm_client().await?;
            let bucket_name = get_ssm_param(&ssm_client, "s3-bucket-name", false)
                .await?
                .ok_or(anyhow!("Empty SSM parameter"))?;
            Ok(bucket_name)
        }
        _ => {
            load_env()?;
            let bucket_name = env::var("S3_BUCKET_NAME")
                .context("Missing env variable: S3_BUCKET_NAME")?;
            Ok(bucket_name)
        }
    }
}

/// Load environment variables.
fn load_env() -> Result<()> {
    // Locate workspace root
    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()?;

    let cargo_path = PathBuf::from(
        str::from_utf8(&output.stdout)?.trim(),
    );

    let mut env_path: PathBuf = cargo_path
        .parent()
        .ok_or(anyhow!("Failed to find workspace root"))?
        .to_path_buf();

    env_path.push(".env.dev");
    if !env_path.exists() {
        bail!("File {} does not exist", &env_path.display());
    }

    // Load env variables from file
    dotenv::from_path(&env_path).with_context(|| {
        format!("Failed to load {}", env_path.display())
    })?;

    Ok(())
}

fn get_env() -> String {
    env::var("ENV").unwrap_or("dev".to_string())
}
