use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    pub name: String,
    pub types: HashMap<String, DocumentType>,
    pub target_directory: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DocumentType {
    pub name: String,
    pub descriptions: Vec<String>,
    pub require_entity: bool,
    pub entities: Option<Vec<String>>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    pub fn default_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("", "", "cfrename")
            .context("Failed to determine config directory")?
            .config_dir()
            .to_path_buf();

        Ok(config_dir.join("config.toml"))
    }

    pub fn load_default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load(&path)
    }
}
