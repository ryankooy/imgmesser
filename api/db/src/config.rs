use anyhow::{Context, Result};
use dotenv;
use std::env;

pub struct Config {
    pub url: String,
    pub maxconns: u32,
}

/// Capture database config values from environment variables.
pub fn get_config() -> Result<Config> {
    // Load environment variables
    dotenv::dotenv().ok();

    let url = env::var("DATABASE_URL")
        .context("Missing env variable: DATABASE_URL")?;

    let maxconns = env::var("MAX_CONNS")
        .context("Missing env variable: MAX_CONNS")?
        .parse::<u32>()
        .context("Failed to parse MAX_CONNS value as u32")?;

    Ok(Config { url, maxconns })
}
