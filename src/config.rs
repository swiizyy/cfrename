//! Configuration management for cfrename.
//!
//! This module handles loading and parsing configuration files that define
//! the document organization structure, including categories, types,
//! descriptions, entities, and date formats.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for cfrename.
///
/// The configuration defines how documents should be organized, including
/// categories, document types, allowed descriptions, and target directories.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Map of category identifiers to their definitions
    pub categories: HashMap<String, Category>,

    /// Accepted date input formats for interactive prompts
    /// Output filenames always use YYYY-MM-DD regardless of input format
    #[serde(default = "default_date_formats")]
    pub date_formats: Vec<String>,

    /// Optional base path for document organization
    /// If set, relative category paths will be combined with this base path
    pub base_path: Option<String>,
}

/// Returns the default date formats if none are specified in configuration.
///
/// Default formats are:
/// - `%Y-%m-%d` (YYYY-MM-DD, ISO 8601)
/// - `%d/%m/%Y` (DD/MM/YYYY, European format)
/// - `%d-%m-%Y` (DD-MM-YYYY)
fn default_date_formats() -> Vec<String> {
    vec![
        "%Y-%m-%d".to_string(),
        "%d/%m/%Y".to_string(),
        "%d-%m-%Y".to_string(),
    ]
}

/// A category of documents (e.g., Administrative, Professional, Medical).
///
/// Categories group related document types and can have a target directory
/// where renamed files will be moved.
#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    /// Display name for this category
    pub name: String,

    /// Map of document type identifiers to their definitions
    pub types: HashMap<String, DocumentType>,

    /// Optional target directory for this category
    /// Can be absolute or relative (relative paths are combined with `base_path` if set)
    pub target_directory: Option<String>,
}

/// A type of document within a category (e.g., Tax, Bank, Work).
///
/// Document types define allowed descriptions and whether an entity
/// (organization/company name) is required.
#[derive(Debug, Deserialize, Clone)]
pub struct DocumentType {
    /// Display name for this document type
    pub name: String,

    /// List of allowed descriptions for this document type
    pub descriptions: Vec<String>,

    /// Whether an entity (organization/company name) is required
    pub require_entity: bool,

    /// List of available entities if `require_entity` is true
    pub entities: Option<Vec<String>>,
}

impl Config {
    /// Loads configuration from a specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file (TOML format)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The file contains invalid TOML syntax
    /// - Required configuration fields are missing
    pub fn load(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    /// Returns the default configuration file path.
    ///
    /// On Unix systems: `~/.config/cfrename/config.toml`
    /// On Windows: `%APPDATA%\cfrename\config.toml`
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be determined
    /// (e.g., HOME environment variable not set)
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = directories::ProjectDirs::from("", "", "cfrename")
            .context("Failed to determine config directory")?
            .config_dir()
            .to_path_buf();

        Ok(config_dir.join("config.toml"))
    }

    /// Loads configuration from the default location.
    ///
    /// This is a convenience method that combines `default_path()` and `load()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The default path cannot be determined
    /// - The configuration file doesn't exist or cannot be read
    /// - The configuration is invalid
    pub fn load_default() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load(&path)
    }

    /// Validates the configuration for common errors.
    ///
    /// Checks:
    /// - At least one category is defined
    /// - Each category has at least one document type
    /// - Each document type has at least one description
    /// - If `require_entity` is true, entities list is provided and non-empty
    /// - Date formats are valid
    ///
    /// # Errors
    ///
    /// Returns an error describing the validation problem if any check fails
    pub fn validate(&self) -> Result<()> {
        if self.categories.is_empty() {
            anyhow::bail!("Configuration must define at least one category");
        }

        for (cat_key, category) in &self.categories {
            if category.types.is_empty() {
                anyhow::bail!("Category '{cat_key}' must have at least one document type");
            }

            for (type_key, doc_type) in &category.types {
                if doc_type.descriptions.is_empty() {
                    anyhow::bail!(
                        "Document type '{cat_key}.{type_key}' must have at least one description"
                    );
                }

                if doc_type.require_entity {
                    match &doc_type.entities {
                        None => anyhow::bail!(
                            "Document type '{cat_key}.{type_key}' requires entity but no entities are defined"
                        ),
                        Some(entities) if entities.is_empty() => anyhow::bail!(
                            "Document type '{cat_key}.{type_key}' requires entity but entities list is empty"
                        ),
                        Some(_) => {} // Valid
                    }
                }
            }
        }

        // Validate date formats
        if self.date_formats.is_empty() {
            anyhow::bail!("At least one date format must be specified");
        }

        Ok(())
    }

    /// Resolves a target directory path, considering the `base_path` if set.
    ///
    /// Path resolution logic:
    /// - If `base_path` is set and `target_directory` is relative → combined
    /// - If `target_directory` is absolute → used as-is (`base_path` ignored)
    /// - If only `base_path` is set → `base_path` used
    /// - If only `target_directory` is set → `target_directory` used
    /// - If neither is set → `None`
    ///
    /// Tilde (`~`) expansion is supported for home directories.
    ///
    /// # Arguments
    ///
    /// * `target_directory` - Optional target directory from category configuration
    ///
    /// # Returns
    ///
    /// The resolved absolute path as a String, or `None` if no path is configured
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // base_path = "~/Documents", target = "Administrative"
    /// // → Some("~/Documents/Administrative")
    ///
    /// // base_path = "~/Documents", target = "/mnt/backup"
    /// // → Some("/mnt/backup")
    ///
    /// // base_path = None, target = "Documents/Test"
    /// // → Some("Documents/Test")
    /// ```
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

    #[test]
    fn test_validate_empty_categories() {
        let toml = r#"
            [categories]
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one category"));
    }

    #[test]
    fn test_validate_category_without_types() {
        let toml = r#"
            [categories.test]
            name = "Test"
        "#;

        let config: Result<Config, _> = toml::from_str(toml);
        assert!(config.is_err()); // Will fail to parse without types
    }

    #[test]
    fn test_validate_type_without_descriptions() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = []
            require_entity = false
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least one description"));
    }

    #[test]
    fn test_validate_require_entity_without_entities() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = true
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no entities are defined"));
    }

    #[test]
    fn test_validate_require_entity_with_empty_entities() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test"]
            require_entity = true
            entities = []
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("entities list is empty"));
    }

    #[test]
    fn test_validate_valid_configuration() {
        let toml = r#"
            [categories.test]
            name = "Test"

            [categories.test.types.doc]
            name = "Document"
            descriptions = ["Test", "Other"]
            require_entity = true
            entities = ["Entity1", "Entity2"]
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        let result = config.validate();
        assert!(result.is_ok());
    }
}
