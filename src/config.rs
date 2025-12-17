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
    pub base_path: Option<String>,
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

    /// Resolves a target directory, considering the base_path if set.
    ///
    /// If base_path is set and target_directory is a relative path,
    /// they are combined. If target_directory is absolute or base_path
    /// is not set, target_directory is returned as-is.
    pub fn resolve_target_path(&self, target_directory: Option<&str>) -> Option<String> {
        match (self.base_path.as_ref(), target_directory) {
            (Some(base), Some(target)) => {
                // Expand tilde in base_path
                let expanded_base = shellexpand::tilde(base).to_string();
                let base_path = PathBuf::from(&expanded_base);

                // Expand tilde in target if present
                let expanded_target = shellexpand::tilde(target).to_string();
                let target_path = PathBuf::from(&expanded_target);

                // If target is absolute, use it as-is
                if target_path.is_absolute() {
                    Some(expanded_target)
                } else {
                    // Combine base_path with relative target
                    Some(base_path.join(target_path).to_string_lossy().to_string())
                }
            }
            (Some(base), None) => {
                // Only base_path is set
                Some(shellexpand::tilde(base).to_string())
            }
            (None, Some(target)) => {
                // Only target is set
                Some(shellexpand::tilde(target).to_string())
            }
            (None, None) => None,
        }
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

    #[test]
    fn test_resolve_target_path_with_base_and_relative_target() {
        let toml = r#"
            base_path = "/home/user/Documents"

            [categories.test]
            name = "Test"
            target_directory = "Administrative"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.resolve_target_path(Some("Administrative"));
        assert_eq!(result, Some("/home/user/Documents/Administrative".to_string()));
    }

    #[test]
    fn test_resolve_target_path_with_base_and_absolute_target() {
        let toml = r#"
            base_path = "/home/user/Documents"

            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.resolve_target_path(Some("/absolute/path"));
        assert_eq!(result, Some("/absolute/path".to_string()));
    }

    #[test]
    fn test_resolve_target_path_with_base_only() {
        let toml = r#"
            base_path = "/home/user/Documents"

            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.resolve_target_path(None);
        assert_eq!(result, Some("/home/user/Documents".to_string()));
    }

    #[test]
    fn test_resolve_target_path_without_base() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.resolve_target_path(Some("Documents/Test"));
        assert_eq!(result, Some("Documents/Test".to_string()));
    }

    #[test]
    fn test_resolve_target_path_with_neither() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.resolve_target_path(None);
        assert_eq!(result, None);
    }
}
