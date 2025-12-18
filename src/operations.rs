//! File system operations for renaming and organizing documents.
//!
//! This module handles safe file operations including:
//! - Preview of rename operations
//! - User confirmation
//! - Actual file renaming with validation
//! - Automatic directory creation
//! - Security validation against symlinks and path traversal

use crate::i18n::Messages;
use crate::path_validation::{PathValidator, ValidatedPath};
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a file rename operation with source and target paths.
///
/// Provides methods for previewing, confirming, and executing rename operations
/// with safety checks including symlink detection and path validation.
pub struct FileOperation {
    source: ValidatedPath,
    target: ValidatedPath,
    messages: Messages,
}

impl FileOperation {
    /// Creates a new file operation with path validation.
    ///
    /// Validates both source and target paths for security issues including
    /// symlinks, path traversal, and invalid filenames.
    ///
    /// # Arguments
    ///
    /// * `source` - Source file path
    /// * `target` - Target file path (may include new directory)
    /// * `messages` - Localized messages for output
    /// * `allow_symlinks` - Whether to allow following symbolic links
    ///
    /// # Returns
    ///
    /// A new `FileOperation` with validated paths
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Source path validation fails (doesn't exist, is not a file, is a symlink when not allowed)
    /// - Target path validation fails (invalid filename, forbidden characters)
    #[allow(clippy::large_types_passed_by_value)]
    pub fn new(source: PathBuf, target: PathBuf, messages: Messages, allow_symlinks: bool) -> Result<Self> {
        let validator = PathValidator::new(allow_symlinks);

        let source_validated = validator.validate_source(&source)?;
        let target_validated = validator.validate_target(&target)?;

        Ok(Self {
            source: source_validated,
            target: target_validated,
            messages,
        })
    }

    /// Displays a preview of the rename operation.
    ///
    /// Shows the canonical source and target paths to the user before confirmation.
    /// If the source is a symlink, displays both the original and resolved paths.
    pub fn preview(&self) {
        println!("\n{}", self.messages.preview_title);

        if self.source.is_symlink {
            println!("{} {} (symlink → {})",
                self.messages.preview_source,
                self.source.original.display(),
                self.source.canonical.display());
        } else {
            println!("{} {}", self.messages.preview_source, self.source.canonical.display());
        }

        println!("{} {}", self.messages.preview_target, self.target.canonical.display());
        println!();
    }

    /// Prompts the user to confirm the rename operation.
    ///
    /// # Returns
    ///
    /// `true` if user confirms, `false` if user cancels
    ///
    /// # Errors
    ///
    /// Returns an error if the confirmation dialog fails
    pub fn confirm(&self) -> Result<bool> {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(self.messages.prompt_confirm_rename)
            .default(false)
            .interact()?;

        Ok(confirmed)
    }

    /// Executes the file rename operation with TOCTOU mitigation.
    ///
    /// Uses canonical paths that were validated at construction time to minimize
    /// the window between validation and execution. Performs final checks immediately
    /// before the rename operation:
    /// 1. Verifies source file still exists (TOCTOU minimized)
    /// 2. Checks target doesn't already exist (TOCTOU minimized)
    /// 3. Creates target directory if needed
    /// 4. Performs atomic rename operation
    ///
    /// # Security Considerations
    ///
    /// While this implementation minimizes TOCTOU (time-of-check-time-of-use) race
    /// conditions by using canonical paths and performing checks immediately before
    /// the operation, complete elimination is not possible. The residual risk is
    /// accepted for typical CLI usage patterns. See SECURITY.md for details.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Source file disappeared between validation and execution
    /// - Target file already exists (race condition detected)
    /// - Directory creation fails
    /// - Rename operation fails
    pub fn execute(&self) -> Result<()> {
        let source_canonical = &self.source.canonical;
        let target_canonical = &self.target.canonical;

        // Final validation immediately before rename to minimize TOCTOU window
        // This check happens on the canonical path that was validated at construction
        if !source_canonical.exists() {
            anyhow::bail!(
                "Source file disappeared after validation: {}",
                source_canonical.display()
            );
        }

        if target_canonical.exists() {
            anyhow::bail!(
                "Target file already exists: {}",
                target_canonical.display()
            );
        }

        // Create parent directory if needed
        // Note: Race condition possible here (directory could become a symlink),
        // but risk is accepted for typical usage. See SECURITY.md.
        if let Some(parent) = target_canonical.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Perform atomic rename operation
        // fs::rename is atomic at the OS level, providing the best protection we can get
        fs::rename(source_canonical, target_canonical)
            .with_context(|| {
                format!(
                    "Failed to rename {} to {}",
                    source_canonical.display(),
                    target_canonical.display()
                )
            })?;

        println!("\n{}", self.messages.success_renamed);
        println!("  → {}", target_canonical.display());

        Ok(())
    }
}

/// Builds the target path for a renamed file.
///
/// # Arguments
///
/// * `source` - Source file path (used if no target directory specified)
/// * `new_filename` - New filename to use
/// * `target_directory` - Optional target directory (with tilde expansion)
///
/// # Returns
///
/// Complete target path combining directory and new filename
///
/// # Examples
///
/// ```ignore
/// // With target directory
/// let path = build_target_path(
///     Path::new("/tmp/file.pdf"),
///     "2024-11-15_TAX_Notice.pdf",
///     Some("~/Documents/Administrative")
/// );
/// // → ~/Documents/Administrative/2024-11-15_TAX_Notice.pdf
///
/// // Without target directory (stays in source directory)
/// let path = build_target_path(
///     Path::new("/tmp/file.pdf"),
///     "2024-11-15_TAX_Notice.pdf",
///     None
/// );
/// // → /tmp/2024-11-15_TAX_Notice.pdf
/// ```
#[must_use]
pub fn build_target_path(
    source: &Path,
    new_filename: &str,
    target_directory: Option<&str>,
) -> PathBuf {
    if let Some(target_dir) = target_directory {
        let expanded_dir = shellexpand::tilde(target_dir).to_string();
        PathBuf::from(expanded_dir).join(new_filename)
    } else {
        source.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(new_filename)
    }
}
