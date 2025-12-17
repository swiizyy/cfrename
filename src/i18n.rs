//! Internationalization (i18n) support for cfrename.
//!
//! Provides localized messages for English and French.

use serde::Deserialize;

/// Supported languages for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Language {
    /// English language
    #[serde(alias = "en")]
    #[default]
    English,
    /// French language
    #[serde(alias = "fr")]
    French,
}


impl Language {
    /// Returns the language code (e.g., "en", "fr").
    #[must_use]
    #[allow(dead_code)]
    pub fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }

    /// Parses a language from a string code.
    ///
    /// Accepts: "en", "english", "fr", "french" (case-insensitive)
    #[must_use]
    #[allow(dead_code)]
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en" | "english" => Some(Self::English),
            "fr" | "french" | "français" | "francais" => Some(Self::French),
            _ => None,
        }
    }
}

/// Localized messages for the application.
///
/// Contains all user-facing strings that can be translated.
#[derive(Debug, Clone, Copy)]
pub struct Messages {
    // Application header
    pub app_title: &'static str,

    // Prompts
    pub prompt_select_category: &'static str,
    pub prompt_select_type: &'static str,
    pub prompt_select_description: &'static str,
    pub prompt_select_entity: &'static str,
    pub prompt_enter_date: &'static str,
    pub prompt_confirm_rename: &'static str,

    // Preview section
    pub preview_title: &'static str,
    pub preview_source: &'static str,
    pub preview_target: &'static str,

    // Success/completion messages
    pub success_renamed: &'static str,
    pub operation_cancelled: &'static str,

    // Error messages
    pub error_config_not_found: &'static str,
    pub error_expected_location: &'static str,
    pub error_create_config: &'static str,

    // Config creation messages
    pub prompt_create_default_config: &'static str,
    pub success_config_created: &'static str,
    pub error_file_not_exist: &'static str,
    pub error_not_a_file: &'static str,
    pub error_invalid_date: &'static str,
    pub error_no_categories: &'static str,
    pub error_no_types: &'static str,
    pub error_no_descriptions: &'static str,
    pub error_no_entities: &'static str,
    pub error_category_not_found: &'static str,
    pub error_type_not_found: &'static str,
}

impl Messages {
    /// Returns messages for the specified language.
    #[must_use]
    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::English => Self::english(),
            Language::French => Self::french(),
        }
    }

    /// English messages.
    #[must_use]
    fn english() -> Self {
        Self {
            app_title: "=== cfrename - File Renaming Tool ===",
            prompt_select_category: "Select category",
            prompt_select_type: "Select document type",
            prompt_select_description: "Select description",
            prompt_select_entity: "Select entity",
            prompt_enter_date: "Enter date",
            prompt_confirm_rename: "Proceed with rename?",
            preview_title: "=== Rename Preview ===",
            preview_source: "Source:",
            preview_target: "Target:",
            success_renamed: "✓ File renamed successfully!",
            operation_cancelled: "Operation cancelled.",
            error_config_not_found: "Error: Configuration file not found.",
            error_expected_location: "Expected location:",
            error_create_config: "Please create a configuration file. See the example configuration.",
            prompt_create_default_config: "Would you like to create a default configuration file?",
            success_config_created: "✓ Default configuration file created successfully!",
            error_file_not_exist: "File does not exist:",
            error_not_a_file: "Path is not a file:",
            error_invalid_date: "Invalid date format. Expected one of:",
            error_no_categories: "No categories defined in configuration",
            error_no_types: "No document types defined for this category",
            error_no_descriptions: "No descriptions defined for this document type",
            error_no_entities: "No entities defined for this document type",
            error_category_not_found: "Selected category not found",
            error_type_not_found: "Selected type not found",
        }
    }

    /// French messages.
    #[must_use]
    fn french() -> Self {
        Self {
            app_title: "=== cfrename - Outil de Renommage de Fichiers ===",
            prompt_select_category: "Sélectionnez une catégorie",
            prompt_select_type: "Sélectionnez le type de document",
            prompt_select_description: "Sélectionnez une description",
            prompt_select_entity: "Sélectionnez une entité",
            prompt_enter_date: "Entrez la date",
            prompt_confirm_rename: "Procéder au renommage ?",
            preview_title: "=== Aperçu du Renommage ===",
            preview_source: "Source :",
            preview_target: "Cible :",
            success_renamed: "✓ Fichier renommé avec succès !",
            operation_cancelled: "Opération annulée.",
            error_config_not_found: "Erreur : Fichier de configuration introuvable.",
            error_expected_location: "Emplacement attendu :",
            error_create_config: "Veuillez créer un fichier de configuration. Consultez l'exemple de configuration.",
            prompt_create_default_config: "Voulez-vous créer un fichier de configuration par défaut ?",
            success_config_created: "✓ Fichier de configuration par défaut créé avec succès !",
            error_file_not_exist: "Le fichier n'existe pas :",
            error_not_a_file: "Le chemin n'est pas un fichier :",
            error_invalid_date: "Format de date invalide. Formats attendus :",
            error_no_categories: "Aucune catégorie définie dans la configuration",
            error_no_types: "Aucun type de document défini pour cette catégorie",
            error_no_descriptions: "Aucune description définie pour ce type de document",
            error_no_entities: "Aucune entité définie pour ce type de document",
            error_category_not_found: "Catégorie sélectionnée introuvable",
            error_type_not_found: "Type sélectionné introuvable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_code() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::French.code(), "fr");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("EN"), Some(Language::English));
        assert_eq!(Language::from_code("english"), Some(Language::English));
        assert_eq!(Language::from_code("English"), Some(Language::English));

        assert_eq!(Language::from_code("fr"), Some(Language::French));
        assert_eq!(Language::from_code("FR"), Some(Language::French));
        assert_eq!(Language::from_code("french"), Some(Language::French));
        assert_eq!(Language::from_code("français"), Some(Language::French));
        assert_eq!(Language::from_code("francais"), Some(Language::French));

        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_default_language() {
        assert_eq!(Language::default(), Language::English);
    }

    #[test]
    fn test_messages_english() {
        let msg = Messages::for_language(Language::English);
        assert!(msg.app_title.contains("File Renaming Tool"));
        assert_eq!(msg.prompt_select_category, "Select category");
    }

    #[test]
    fn test_messages_french() {
        let msg = Messages::for_language(Language::French);
        assert!(msg.app_title.contains("Renommage de Fichiers"));
        assert_eq!(msg.prompt_select_category, "Sélectionnez une catégorie");
    }
}
