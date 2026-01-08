use anyhow::{anyhow, bail, Context, Result};
use dotenv;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str;

/// Loads environment variables.
pub fn load_config() -> Result<()> {
    let env_file: String = match env::var("ENV") {
        Ok(ext) => format!(".env.{}", ext),
        Err(_) => String::from(".env.dev"),
    };

    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()?;

    let cargo_path = PathBuf::from(str::from_utf8(&output.stdout)?.trim());
    let mut env_path: PathBuf = cargo_path
        .parent()
        .ok_or(anyhow!("Failed to find workspace root"))?
        .to_path_buf();

    env_path.push(&env_file);

    if !env_path.exists() {
        bail!("File {} does not exist", &env_path.display());
    }

    dotenv::from_path(&env_path)
        .with_context(|| format!("Failed to load {}", env_path.display()))?;

    Ok(())
}
