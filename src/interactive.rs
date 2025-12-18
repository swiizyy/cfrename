//! Interactive CLI session management.
//!
//! This module handles the interactive command-line interface that guides
//! users through the document renaming process step by step.

use crate::config::{Config, DocumentType};
use crate::i18n::Messages;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, Input, Select};

/// Interactive session for guided document renaming.
///
/// Manages the step-by-step process of collecting information from the user:
/// category, document type, description, entity (if required), and date.
pub struct InteractiveSession {
    config: Config,
    messages: Messages,
}

impl InteractiveSession {
    /// Creates a new interactive session with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration defining categories, types, and options
    #[must_use]
    pub fn new(config: Config) -> Self {
        let messages = Messages::for_language(config.language);
        Self { config, messages }
    }

    /// Runs the interactive session and collects rename information.
    ///
    /// Guides the user through:
    /// 1. Category selection
    /// 2. Document type selection
    /// 3. Description selection
    /// 4. Entity selection (if required by document type)
    /// 5. Date input
    ///
    /// # Returns
    ///
    /// A `RenameRequest` containing all collected information
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - User cancels the interaction
    /// - Configuration is invalid (empty categories, etc.)
    /// - Date input is invalid
    pub fn run(&self) -> Result<RenameRequest> {
        println!("\n{}\n", self.messages.app_title);

        let category_key = self.select_category()?;
        let category = self.config.categories.get(&category_key)
            .context(self.messages.error_category_not_found)?;

        let type_key = self.select_type(&category_key)?;
        let doc_type = category.types.get(&type_key)
            .context(self.messages.error_type_not_found)?;

        let description = self.select_description(doc_type)?;

        let entity = if doc_type.require_entity {
            Some(self.select_entity(doc_type)?)
        } else {
            None
        };

        let date = self.input_date()?;

        Ok(RenameRequest {
            doc_type: type_key,
            entity,
            description,
            date,
            target_directory: category.target_directory.clone(),
        })
    }

    fn select_category(&self) -> Result<String> {
        let categories: Vec<(String, String)> = self.config.categories
            .iter()
            .map(|(key, cat)| (key.clone(), cat.name.clone()))
            .collect();

        if categories.is_empty() {
            anyhow::bail!(self.messages.error_no_categories);
        }

        let display: Vec<String> = categories.iter()
            .map(|(_, name)| name.clone())
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(self.messages.prompt_select_category)
            .items(&display)
            .default(0)
            .interact()?;

        Ok(categories[selection].0.clone())
    }

    fn select_type(&self, category_key: &str) -> Result<String> {
        let category = self.config.categories.get(category_key)
            .context(self.messages.error_category_not_found)?;

        let types: Vec<(String, String)> = category.types
            .iter()
            .map(|(key, doc_type)| (key.clone(), doc_type.name.clone()))
            .collect();

        if types.is_empty() {
            anyhow::bail!(self.messages.error_no_types);
        }

        let display: Vec<String> = types.iter()
            .map(|(_, name)| name.clone())
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(self.messages.prompt_select_type)
            .items(&display)
            .default(0)
            .interact()?;

        Ok(types[selection].0.clone())
    }

    fn select_description(&self, doc_type: &DocumentType) -> Result<String> {
        if doc_type.descriptions.is_empty() {
            anyhow::bail!(self.messages.error_no_descriptions);
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(self.messages.prompt_select_description)
            .items(&doc_type.descriptions)
            .default(0)
            .interact()?;

        Ok(doc_type.descriptions[selection].clone())
    }

    fn select_entity(&self, doc_type: &DocumentType) -> Result<String> {
        let entities = doc_type.entities.as_ref()
            .context(self.messages.error_no_entities)?;

        if entities.is_empty() {
            anyhow::bail!(self.messages.error_no_entities);
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(self.messages.prompt_select_entity)
            .items(entities)
            .default(0)
            .interact()?;

        Ok(entities[selection].clone())
    }

    fn input_date(&self) -> Result<NaiveDate> {
        let today = chrono::Local::now().date_naive();

        // Use the first format for the default display
        let default_format = self.config.date_formats.first()
            .map_or("%Y-%m-%d", std::string::String::as_str);
        let default_date = today.format(default_format).to_string();

        // Build prompt with all accepted formats
        let formats_display = self.config.date_formats.join(", ");
        let prompt = format!("{} ({formats_display})", self.messages.prompt_enter_date);

        let date_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(&prompt)
            .default(default_date)
            .interact_text()?;

        Self::parse_date_with_formats(&date_str, &self.config.date_formats)
            .with_context(|| format!("{} {formats_display}", self.messages.error_invalid_date))
    }

    /// Parses a date string using multiple format patterns.
    ///
    /// Tries each format in order until one succeeds.
    ///
    /// # Arguments
    ///
    /// * `date_str` - The date string to parse
    /// * `formats` - List of format patterns to try (e.g., "%Y-%m-%d", "%d/%m/%Y")
    ///
    /// # Returns
    ///
    /// The parsed date if any format matches
    ///
    /// # Errors
    ///
    /// Returns an error if none of the formats can parse the date string
    fn parse_date_with_formats(date_str: &str, formats: &[String]) -> Result<NaiveDate> {
        for format in formats {
            if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                return Ok(date);
            }
        }
        anyhow::bail!("Could not parse date '{}' with any of the provided formats", date_str)
    }
}

/// Request containing all information needed to rename a file.
///
/// This struct is returned by `InteractiveSession::run()` and contains
/// all the user-selected options for renaming a document.
pub struct RenameRequest {
    /// Document type key (used for filename generation)
    pub doc_type: String,

    /// Optional entity/organization name
    pub entity: Option<String>,

    /// Document description
    pub description: String,

    /// Document date
    pub date: NaiveDate,

    /// Target directory from category configuration
    pub target_directory: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::config::Category;
    use chrono::Datelike;

    fn create_test_config() -> Config {
        let mut categories = HashMap::new();

        let mut types = HashMap::new();
        types.insert(
            "TAX".to_string(),
            DocumentType {
                name: "Tax Document".to_string(),
                descriptions: vec!["Notice".to_string(), "Return".to_string()],
                require_entity: false,
                entities: None,
            },
        );
        types.insert(
            "WORK".to_string(),
            DocumentType {
                name: "Work Document".to_string(),
                descriptions: vec!["Contract".to_string(), "Pay Slip".to_string()],
                require_entity: true,
                entities: Some(vec!["ACME Corp".to_string(), "Tech Inc".to_string()]),
            },
        );

        categories.insert(
            "admin".to_string(),
            Category {
                name: "Administrative".to_string(),
                types,
                target_directory: Some("~/Documents/Admin".to_string()),
            },
        );

        Config {
            categories,
            date_formats: vec![
                "%Y-%m-%d".to_string(),
                "%d/%m/%Y".to_string(),
                "%d-%m-%Y".to_string(),
            ],
            base_path: None,
            language: crate::i18n::Language::English,
        }
    }

    #[test]
    fn test_parse_date_with_iso_format() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("2024-11-15", &formats);

        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_with_european_slash_format() {
        let formats = vec!["%d/%m/%Y".to_string()];
        let result = InteractiveSession::parse_date_with_formats("15/11/2024", &formats);

        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_with_european_dash_format() {
        let formats = vec!["%d-%m-%Y".to_string()];
        let result = InteractiveSession::parse_date_with_formats("15-11-2024", &formats);

        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_tries_multiple_formats() {
        let formats = vec![
            "%Y-%m-%d".to_string(),
            "%d/%m/%Y".to_string(),
            "%d-%m-%Y".to_string(),
        ];

        // Should succeed with second format
        let result = InteractiveSession::parse_date_with_formats("15/11/2024", &formats);
        assert!(result.is_ok());

        // Should succeed with first format
        let result = InteractiveSession::parse_date_with_formats("2024-11-15", &formats);
        assert!(result.is_ok());

        // Should succeed with third format
        let result = InteractiveSession::parse_date_with_formats("15-11-2024", &formats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_date_invalid_format() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("invalid-date", &formats);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Could not parse date"));
    }

    #[test]
    fn test_parse_date_no_matching_format() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("15/11/2024", &formats);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_empty_string() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("", &formats);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_with_invalid_day() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("2024-11-32", &formats);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_with_invalid_month() {
        let formats = vec!["%Y-%m-%d".to_string()];
        let result = InteractiveSession::parse_date_with_formats("2024-13-15", &formats);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_leap_year() {
        let formats = vec!["%Y-%m-%d".to_string()];

        // Valid leap year date
        let result = InteractiveSession::parse_date_with_formats("2024-02-29", &formats);
        assert!(result.is_ok());

        // Invalid leap year date
        let result = InteractiveSession::parse_date_with_formats("2023-02-29", &formats);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_date_with_whitespace() {
        let formats = vec!["%Y-%m-%d".to_string()];

        // chrono's parse_from_str actually accepts dates with leading whitespace
        let result = InteractiveSession::parse_date_with_formats(" 2024-11-15", &formats);
        assert!(result.is_ok());

        // But trailing whitespace makes it fail
        let result = InteractiveSession::parse_date_with_formats("2024-11-15 ", &formats);
        assert!(result.is_err());

        // Whitespace in the middle should fail
        let result = InteractiveSession::parse_date_with_formats("2024 -11-15", &formats);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_session_creates_correct_language() {
        let config = create_test_config();
        let session = InteractiveSession::new(config.clone());

        // Session should be created successfully
        // We can't directly test the messages field as it's private,
        // but we can verify the session is created without panicking
        assert_eq!(session.config.categories.len(), 1);
    }

    #[test]
    fn test_rename_request_structure() {
        let request = RenameRequest {
            doc_type: "TAX".to_string(),
            entity: None,
            description: "Notice".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 11, 15).unwrap(),
            target_directory: Some("~/Documents/Admin".to_string()),
        };

        assert_eq!(request.doc_type, "TAX");
        assert_eq!(request.entity, None);
        assert_eq!(request.description, "Notice");
        assert_eq!(request.date.year(), 2024);
        assert_eq!(request.target_directory, Some("~/Documents/Admin".to_string()));
    }

    #[test]
    fn test_rename_request_with_entity() {
        let request = RenameRequest {
            doc_type: "WORK".to_string(),
            entity: Some("ACME Corp".to_string()),
            description: "Contract".to_string(),
            date: NaiveDate::from_ymd_opt(2024, 11, 15).unwrap(),
            target_directory: Some("~/Documents/Work".to_string()),
        };

        assert_eq!(request.doc_type, "WORK");
        assert_eq!(request.entity, Some("ACME Corp".to_string()));
        assert_eq!(request.description, "Contract");
    }

    #[test]
    fn test_parse_date_with_custom_format() {
        let formats = vec!["%d.%m.%Y".to_string()];
        let result = InteractiveSession::parse_date_with_formats("15.11.2024", &formats);

        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 11);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_parse_date_format_priority() {
        // When multiple formats could match, the first one should be used
        let formats = vec![
            "%Y-%m-%d".to_string(),
            "%Y-%m-%d".to_string(), // Duplicate to test priority
        ];

        let result = InteractiveSession::parse_date_with_formats("2024-11-15", &formats);
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 11, 15).unwrap());
    }

    #[test]
    fn test_parse_date_with_short_year() {
        let formats = vec!["%d/%m/%y".to_string()];
        let result = InteractiveSession::parse_date_with_formats("15/11/24", &formats);

        assert!(result.is_ok());
        let date = result.unwrap();
        // chrono interprets 2-digit years as 1900-2099
        assert_eq!(date.year(), 2024);
    }

    #[test]
    fn test_parse_date_boundary_values() {
        let formats = vec!["%Y-%m-%d".to_string()];

        // First day of year
        let result = InteractiveSession::parse_date_with_formats("2024-01-01", &formats);
        assert!(result.is_ok());

        // Last day of year
        let result = InteractiveSession::parse_date_with_formats("2024-12-31", &formats);
        assert!(result.is_ok());

        // Invalid: day 0
        let result = InteractiveSession::parse_date_with_formats("2024-01-00", &formats);
        assert!(result.is_err());

        // Invalid: month 0
        let result = InteractiveSession::parse_date_with_formats("2024-00-01", &formats);
        assert!(result.is_err());
    }
}
