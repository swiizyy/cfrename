//! Filename generation according to the cfrename naming convention.
//!
//! This module handles the construction of standardized filenames following
//! the formats:
//! - Standard: `YYYY-MM-DD_TYPE_DESCRIPTION.ext`
//! - Extended: `YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext`

use chrono::NaiveDate;
use std::path::Path;

/// Builder for creating standardized filenames.
///
/// Constructs filenames following the cfrename naming convention:
/// - Date is always formatted as YYYY-MM-DD
/// - Type and entity (if present) are converted to uppercase
/// - Description preserves original casing with spaces replaced by underscores
/// - Extension is preserved from the original file
pub struct FileNameBuilder {
    date: NaiveDate,
    doc_type: String,
    entity: Option<String>,
    description: String,
    extension: String,
}

impl FileNameBuilder {
    /// Creates a new filename builder with the specified components.
    ///
    /// # Arguments
    ///
    /// * `date` - Date for the document
    /// * `doc_type` - Document type (will be uppercased)
    /// * `entity` - Optional entity/organization name (will be uppercased)
    /// * `description` - Document description (spaces converted to underscores)
    /// * `extension` - File extension (preserved as-is)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let builder = FileNameBuilder::new(
    ///     NaiveDate::from_ymd(2024, 11, 15),
    ///     "TAX".to_string(),
    ///     None,
    ///     "Notice".to_string(),
    ///     "pdf".to_string(),
    /// );
    /// assert_eq!(builder.build(), "2024-11-15_TAX_Notice.pdf");
    /// ```
    #[must_use]
    pub fn new(
        date: NaiveDate,
        doc_type: String,
        entity: Option<String>,
        description: String,
        extension: String,
    ) -> Self {
        Self {
            date,
            doc_type,
            entity,
            description,
            extension,
        }
    }

    /// Builds the filename according to the naming convention.
    ///
    /// # Returns
    ///
    /// A standardized filename string.
    ///
    /// Format without entity: `YYYY-MM-DD_TYPE_DESCRIPTION.ext`
    /// Format with entity: `YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Without entity
    /// builder.build(); // "2024-11-15_TAX_Notice.pdf"
    ///
    /// // With entity
    /// builder.build(); // "2024-11-15_WORK_ACME_Contract_CDI.pdf"
    /// ```
    #[must_use]
    pub fn build(&self) -> String {
        let date_str = self.date.format("%Y-%m-%d").to_string();
        let doc_type = self.doc_type.to_uppercase();
        let description = self.description.replace(' ', "_");

        let filename = if let Some(entity) = &self.entity {
            let entity = entity.to_uppercase();
            format!("{date_str}_{doc_type}_{entity}_{description}")
        } else {
            format!("{date_str}_{doc_type}_{description}")
        };

        format!("{filename}.{}", self.extension)
    }
}

/// Extracts the file extension from a path.
///
/// # Arguments
///
/// * `file_path` - Path to extract extension from
///
/// # Returns
///
/// The file extension as a String. Returns "pdf" if no extension is found.
///
/// # Examples
///
/// ```ignore
/// let ext = extract_extension(Path::new("/path/to/file.pdf"));
/// assert_eq!(ext, "pdf");
///
/// let ext = extract_extension(Path::new("/path/to/file"));
/// assert_eq!(ext, "pdf"); // Default
/// ```
#[must_use]
pub fn extract_extension(file_path: &Path) -> String {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("pdf")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_format() {
        let builder = FileNameBuilder::new(
            NaiveDate::from_ymd_opt(2024, 11, 15).unwrap(),
            "IMPOTS".to_string(),
            None,
            "Avis".to_string(),
            "pdf".to_string(),
        );

        assert_eq!(builder.build(), "2024-11-15_IMPOTS_Avis.pdf");
    }

    #[test]
    fn test_extended_format_with_entity() {
        let builder = FileNameBuilder::new(
            NaiveDate::from_ymd_opt(2025, 10, 5).unwrap(),
            "TRAVAIL".to_string(),
            Some("ACME".to_string()),
            "Contrat CDI".to_string(),
            "pdf".to_string(),
        );

        assert_eq!(builder.build(), "2025-10-05_TRAVAIL_ACME_Contrat_CDI.pdf");
    }
}
