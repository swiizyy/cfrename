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

        // Try to parse with each configured format
        for format in &self.config.date_formats {
            if let Ok(date) = NaiveDate::parse_from_str(&date_str, format) {
                return Ok(date);
            }
        }

        anyhow::bail!(
            "{} {formats_display}",
            self.messages.error_invalid_date
        )
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
