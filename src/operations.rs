//! File system operations for renaming and organizing documents.
//!
//! This module handles safe file operations including:
//! - Preview of rename operations
//! - User confirmation
//! - Actual file renaming with validation
//! - Automatic directory creation

use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a file rename operation with source and target paths.
///
/// Provides methods for previewing, confirming, and executing rename operations
/// with safety checks.
pub struct FileOperation {
    source: PathBuf,
    target: PathBuf,
}

impl FileOperation {
    /// Creates a new file operation.
    ///
    /// # Arguments
    ///
    /// * `source` - Source file path
    /// * `target` - Target file path (may include new directory)
    #[must_use]
    pub fn new(source: PathBuf, target: PathBuf) -> Self {
        Self { source, target }
    }

    /// Displays a preview of the rename operation.
    ///
    /// Shows the source and target paths to the user before confirmation.
    pub fn preview(&self) {
        println!("\n=== Rename Preview ===");
        println!("Source: {}", self.source.display());
        println!("Target: {}", self.target.display());
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
    pub fn confirm() -> Result<bool> {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with rename?")
            .default(false)
            .interact()?;

        Ok(confirmed)
    }

    /// Executes the file rename operation.
    ///
    /// Performs validation and executes the rename:
    /// 1. Verifies source file exists
    /// 2. Checks target doesn't already exist
    /// 3. Creates target directory if needed
    /// 4. Renames the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Source file doesn't exist
    /// - Target file already exists
    /// - Directory creation fails
    /// - Rename operation fails
    pub fn execute(&self) -> Result<()> {
        if !self.source.exists() {
            anyhow::bail!("Source file does not exist: {}", self.source.display());
        }

        if self.target.exists() {
            anyhow::bail!("Target file already exists: {}", self.target.display());
        }

        if let Some(parent) = self.target.parent()
            && !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

        fs::rename(&self.source, &self.target)
            .with_context(|| {
                format!(
                    "Failed to rename {} to {}",
                    self.source.display(),
                    self.target.display()
                )
            })?;

        println!("\n✓ File renamed successfully!");
        println!("  → {}", self.target.display());

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
