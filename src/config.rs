use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub categories: HashMap<String, Category>,
    #[serde(default = "default_date_formats")]
    pub date_formats: Vec<String>,
}

fn default_date_formats() -> Vec<String> {
    vec![
        "%Y-%m-%d".to_string(),
        "%d/%m/%Y".to_string(),
        "%d-%m-%Y".to_string(),
    ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_date_formats() {
        let formats = default_date_formats();
        assert_eq!(formats.len(), 3);
        assert_eq!(formats[0], "%Y-%m-%d");
        assert_eq!(formats[1], "%d/%m/%Y");
        assert_eq!(formats[2], "%d-%m-%Y");
    }

    #[test]
    fn test_config_with_custom_date_formats() {
        let toml = r#"
            date_formats = ["%d.%m.%Y", "%Y-%m-%d"]

            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.date_formats.len(), 2);
        assert_eq!(config.date_formats[0], "%d.%m.%Y");
        assert_eq!(config.date_formats[1], "%Y-%m-%d");
    }

    #[test]
    fn test_config_without_date_formats_uses_defaults() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.date_formats.len(), 3);
        assert_eq!(config.date_formats[0], "%Y-%m-%d");
    }
}
