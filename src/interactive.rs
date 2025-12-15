use crate::config::{Config, DocumentType};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use dialoguer::{theme::ColorfulTheme, Input, Select};

pub struct InteractiveSession {
    config: Config,
}

impl InteractiveSession {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&self) -> Result<RenameRequest> {
        println!("\n=== cfrename - File Renaming Tool ===\n");

        let category_key = self.select_category()?;
        let category = self.config.categories.get(&category_key)
            .context("Selected category not found")?;

        let type_key = self.select_type(category_key.clone())?;
        let doc_type = category.types.get(&type_key)
            .context("Selected type not found")?;

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
            anyhow::bail!("No categories defined in configuration");
        }

        let display: Vec<String> = categories.iter()
            .map(|(_, name)| name.clone())
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select category")
            .items(&display)
            .default(0)
            .interact()?;

        Ok(categories[selection].0.clone())
    }

    fn select_type(&self, category_key: String) -> Result<String> {
        let category = self.config.categories.get(&category_key)
            .context("Category not found")?;

        let types: Vec<(String, String)> = category.types
            .iter()
            .map(|(key, doc_type)| (key.clone(), doc_type.name.clone()))
            .collect();

        if types.is_empty() {
            anyhow::bail!("No document types defined for this category");
        }

        let display: Vec<String> = types.iter()
            .map(|(_, name)| name.clone())
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select document type")
            .items(&display)
            .default(0)
            .interact()?;

        Ok(types[selection].0.clone())
    }

    fn select_description(&self, doc_type: &DocumentType) -> Result<String> {
        if doc_type.descriptions.is_empty() {
            anyhow::bail!("No descriptions defined for this document type");
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select description")
            .items(&doc_type.descriptions)
            .default(0)
            .interact()?;

        Ok(doc_type.descriptions[selection].clone())
    }

    fn select_entity(&self, doc_type: &DocumentType) -> Result<String> {
        let entities = doc_type.entities.as_ref()
            .context("No entities defined for this document type")?;

        if entities.is_empty() {
            anyhow::bail!("No entities defined for this document type");
        }

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select entity")
            .items(entities)
            .default(0)
            .interact()?;

        Ok(entities[selection].clone())
    }

    fn input_date(&self) -> Result<NaiveDate> {
        let today = chrono::Local::now().date_naive();

        // Use the first format for the default display
        let default_format = self.config.date_formats.first()
            .map(|s| s.as_str())
            .unwrap_or("%Y-%m-%d");
        let default_date = today.format(default_format).to_string();

        // Build prompt with all accepted formats
        let formats_display = self.config.date_formats.join(", ");
        let prompt = format!("Enter date ({})", formats_display);

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
            "Invalid date format. Expected one of: {}",
            formats_display
        )
    }
}

pub struct RenameRequest {
    pub doc_type: String,
    pub entity: Option<String>,
    pub description: String,
    pub date: NaiveDate,
    pub target_directory: Option<String>,
}
