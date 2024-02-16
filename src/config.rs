use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(short, long, default_value = "turbine.toml")]
    pub config_file: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub document_root: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            document_root: PathBuf::from("web_resources"),
        }
    }
}

impl Config {
    pub fn new(config_file: PathBuf) -> Result<Self> {
        if !config_file.exists() || config_file.ends_with("toml") {
            return Err(anyhow::anyhow!(
                "Config file does not exist or is not a toml file"
            ));
        }

        let content = std::fs::read_to_string(config_file)?;

        let config = toml::from_str(&content).unwrap_or_default();

        Ok(config)
    }
}
