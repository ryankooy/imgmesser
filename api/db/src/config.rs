use anyhow::{Context, Result};
use dotenv;

use config;

pub struct Config {
    pub url: String,
    pub maxconns: u32,
}

/// Capture database config values from environment variables.
pub fn get_config() -> Result<Config> {
    config::load_config()?;

    let url = dotenv::var("DATABASE_URL")
        .context("Missing env variable: DATABASE_URL")?;

    let maxconns = dotenv::var("MAX_CONNS")
        .context("Missing env variable: MAX_CONNS")?
        .parse::<u32>()
        .context("Failed to parse MAX_CONNS value as u32")?;

    Ok(Config { url, maxconns })
}
