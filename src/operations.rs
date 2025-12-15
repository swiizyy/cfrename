use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileOperation {
    source: PathBuf,
    target: PathBuf,
}

impl FileOperation {
    pub fn new(source: PathBuf, target: PathBuf) -> Self {
        Self { source, target }
    }

    pub fn preview(&self) {
        println!("\n=== Rename Preview ===");
        println!("Source: {}", self.source.display());
        println!("Target: {}", self.target.display());
        println!();
    }

    pub fn confirm(&self) -> Result<bool> {
        let confirmed = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Proceed with rename?")
            .default(false)
            .interact()?;

        Ok(confirmed)
    }

    pub fn execute(&self) -> Result<()> {
        if !self.source.exists() {
            anyhow::bail!("Source file does not exist: {}", self.source.display());
        }

        if self.target.exists() {
            anyhow::bail!("Target file already exists: {}", self.target.display());
        }

        if let Some(parent) = self.target.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
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
