use chrono::NaiveDate;
use std::path::Path;

pub struct FileNameBuilder {
    date: NaiveDate,
    doc_type: String,
    entity: Option<String>,
    description: String,
    extension: String,
}

impl FileNameBuilder {
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

    pub fn build(&self) -> String {
        let date_str = self.date.format("%Y-%m-%d").to_string();
        let doc_type = self.doc_type.to_uppercase();
        let description = self.description.replace(" ", "_");

        let filename = if let Some(entity) = &self.entity {
            let entity = entity.to_uppercase();
            format!("{}_{}_{}_{}", date_str, doc_type, entity, description)
        } else {
            format!("{}_{}_{}", date_str, doc_type, description)
        };

        format!("{}.{}", filename, self.extension)
    }
}

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
    use chrono::NaiveDate;

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
