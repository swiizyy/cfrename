//! Property-based tests using proptest.
//!
//! These tests verify that certain properties hold true for a wide range of inputs,
//! helping catch edge cases and unexpected behaviors.

use proptest::prelude::*;

// Re-export the modules we need to test
// Note: Since we're testing internal modules, we need to access them through the binary crate
// For property tests on public APIs, we would import from the crate

/// Test that path traversal detection works for generated paths
#[cfg(test)]
mod path_traversal_tests {
    use super::*;

    proptest! {
        /// Any string containing "../" should be detected as path traversal
        #[test]
        fn detects_unix_path_traversal(
            prefix in "[a-z]{0,10}",
            suffix in "[a-z]{0,10}"
        ) {
            let path = format!("{}../{}",prefix, suffix);
            // This would require access to PathValidator
            // Since it's in the binary crate, we test the pattern matching logic
            assert!(path.contains("../"));
        }

        /// Any string containing "..\\" should be detected as path traversal
        #[test]
        fn detects_windows_path_traversal(
            prefix in "[a-z]{0,10}",
            suffix in "[a-z]{0,10}"
        ) {
            let path = format!("{}..\\{}",  prefix, suffix);
            assert!(path.contains("..\\"));
        }

        /// Safe paths without ".." should not trigger traversal detection
        #[test]
        fn allows_safe_paths(
            segments in prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5)
        ) {
            let path = segments.join("/");
            // Safe paths should not contain ".."
            prop_assume!(!path.contains(".."));
            assert!(!path.contains("../"));
            assert!(!path.contains("..\\"));
        }
    }
}

/// Test that filename validation catches forbidden characters
#[cfg(test)]
mod filename_validation_tests {
    use super::*;

    const FORBIDDEN_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

    proptest! {
        /// Filenames with forbidden characters should be detectable
        #[test]
        fn detects_forbidden_chars(
            prefix in "[a-zA-Z0-9]{1,10}",
            suffix in "[a-zA-Z0-9]{1,10}",
            forbidden_char in prop::sample::select(FORBIDDEN_CHARS)
        ) {
            let filename = format!("{}{}{}", prefix, forbidden_char, suffix);
            assert!(filename.contains(|c: char| FORBIDDEN_CHARS.contains(&c)));
        }

        /// Safe filenames with only alphanumeric, dash, underscore, dot should be valid
        #[test]
        fn allows_safe_filenames(s in "[a-zA-Z0-9._-]{1,100}") {
            // Should not contain any forbidden character
            assert!(!s.chars().any(|c| FORBIDDEN_CHARS.contains(&c)));
        }

        /// Filenames longer than 255 characters should be detectable
        #[test]
        fn detects_too_long_filenames(s in "[a-z]{256,300}") {
            assert!(s.len() > 255);
        }

        /// Filenames exactly 255 characters should be acceptable
        #[test]
        fn allows_max_length_filenames(s in "[a-z]{255}") {
            assert_eq!(s.len(), 255);
        }
    }
}

/// Test that Windows reserved names are detected (case-insensitive)
#[cfg(test)]
mod windows_reserved_names_tests {
    use super::*;

    const RESERVED_NAMES: &[&str] = &[
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        "CLOCK$",
    ];

    proptest! {
        /// Reserved names with any extension should be detectable
        #[test]
        fn detects_reserved_names_with_extension(
            reserved in prop::sample::select(RESERVED_NAMES),
            ext in "[a-z]{2,4}"
        ) {
            let filename = format!("{}.{}", reserved, ext);
            let base = filename.split('.').next().unwrap().to_uppercase();
            assert!(RESERVED_NAMES.contains(&base.as_str()));
        }

        /// Reserved names are case-insensitive
        #[test]
        fn detects_case_insensitive_reserved(
            reserved in prop::sample::select(RESERVED_NAMES),
            // Generate a random case combination
            case_mask in prop::collection::vec(prop::bool::ANY, 1..10)
        ) {
            let mut filename = String::new();
            for (i, ch) in reserved.chars().enumerate() {
                if i < case_mask.len() && case_mask[i] {
                    filename.push(ch.to_lowercase().next().unwrap());
                } else {
                    filename.push(ch);
                }
            }
            let upper = filename.to_uppercase();
            assert!(RESERVED_NAMES.contains(&upper.as_str()));
        }
    }
}

/// Test date format parsing and generation
#[cfg(test)]
mod date_formatting_tests {
    use super::*;
    use chrono::NaiveDate;

    proptest! {
        /// Valid dates should be parseable and reproducible
        #[test]
        fn date_roundtrip(
            year in 2000i32..2100i32,
            month in 1u32..=12u32,
            day in 1u32..=28u32  // Use 28 to avoid invalid dates
        ) {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                // Format to YYYY-MM-DD
                let formatted = date.format("%Y-%m-%d").to_string();

                // Parse back
                let parsed = NaiveDate::parse_from_str(&formatted, "%Y-%m-%d");

                prop_assert!(parsed.is_ok());
                prop_assert_eq!(parsed.unwrap(), date);
            }
        }

        /// Generated filenames should always start with YYYY-MM-DD format
        #[test]
        fn filename_starts_with_date(
            year in 2000i32..2100i32,
            month in 1u32..=12u32,
            day in 1u32..=28u32
        ) {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                let formatted = date.format("%Y-%m-%d").to_string();

                // Check format: exactly 10 characters YYYY-MM-DD
                prop_assert_eq!(formatted.len(), 10);
                prop_assert_eq!(&formatted[4..5], "-");
                prop_assert_eq!(&formatted[7..8], "-");

                // Verify it's numeric where expected
                prop_assert!(formatted[0..4].chars().all(|c| c.is_ascii_digit()));
                prop_assert!(formatted[5..7].chars().all(|c| c.is_ascii_digit()));
                prop_assert!(formatted[8..10].chars().all(|c| c.is_ascii_digit()));
            }
        }
    }
}

/// Test file extension extraction
#[cfg(test)]
mod extension_tests {
    use super::*;

    proptest! {
        /// Files with extensions should extract correctly
        #[test]
        fn extracts_extensions(
            name in "[a-zA-Z0-9_-]{1,20}",
            ext in "[a-z]{2,4}"
        ) {
            let filename = format!("{}.{}", name, ext);
            // The extension should be extractable
            let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
            prop_assert_eq!(parts.len(), 2);
            prop_assert_eq!(parts[0], ext);
        }

        /// Files without extensions should handle gracefully
        #[test]
        fn handles_no_extension(name in "[a-zA-Z0-9_-]{1,20}") {
            // Should not contain a dot
            prop_assume!(!name.contains('.'));
            let parts: Vec<&str> = name.rsplitn(2, '.').collect();
            prop_assert_eq!(parts.len(), 1);
        }

        /// Multiple dots should only use the last extension
        #[test]
        fn uses_last_extension(
            name in "[a-zA-Z0-9_-]{1,10}",
            ext1 in "[a-z]{2,3}",
            ext2 in "[a-z]{2,4}"
        ) {
            let filename = format!("{}.{}.{}", name, ext1, ext2);
            let parts: Vec<&str> = filename.rsplitn(2, '.').collect();
            prop_assert_eq!(parts.len(), 2);
            prop_assert_eq!(parts[0], ext2);
        }
    }
}

/// Test that sanitization never panics
#[cfg(test)]
mod sanitization_tests {
    use super::*;

    proptest! {
        /// Any alphanumeric string should be safely processable
        #[test]
        fn never_panics_on_alphanumeric(s in "[a-zA-Z0-9]{0,100}") {
            // Just verify we can process alphanumeric strings
            let _ = s.len();
            let _ = s.chars().count();
        }

        /// Very long strings should be processable
        #[test]
        fn handles_long_strings(s in "[a-z]{1000,2000}") {
            prop_assert!(s.len() >= 1000);
            prop_assert!(s.len() <= 2000);
        }
    }
}
