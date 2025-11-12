use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub url: String,
    pub maxconns: u32,
}

/// Read values from config file.
pub fn get_config() -> Result<Config> {
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "db.toml"]
        .iter()
        .collect();

    let config_str = std::fs::read_to_string(&path)
        .context("Failed to read db config file")?;

    let config: Config = toml::from_str(&config_str)
        .context("Failed to parse db config")?;

    Ok(config)
}
