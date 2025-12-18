//! Path validation and security for file operations.
//!
//! This module provides robust path validation to prevent security vulnerabilities
//! including:
//! - Symbolic link detection and optional rejection
//! - Path traversal attacks (../ sequences)
//! - Windows reserved filename validation
//! - Forbidden character detection
//! - Path canonicalization for TOCTOU mitigation

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// A validated path with security metadata.
///
/// Stores the canonical (resolved) path along with metadata about whether
/// the original path was a symbolic link. This struct is created by
/// `PathValidator` after successful validation.
#[derive(Debug, Clone)]
pub struct ValidatedPath {
    /// The canonical (fully resolved) path
    pub canonical: PathBuf,

    /// Whether the original path was a symbolic link
    pub is_symlink: bool,

    /// The original path before canonicalization
    pub original: PathBuf,
}

impl ValidatedPath {
    /// Creates a new ValidatedPath.
    ///
    /// # Arguments
    ///
    /// * `canonical` - The canonical (resolved) path
    /// * `is_symlink` - Whether the original path was a symbolic link
    /// * `original` - The original path before resolution
    #[must_use]
    pub fn new(canonical: PathBuf, is_symlink: bool, original: PathBuf) -> Self {
        Self {
            canonical,
            is_symlink,
            original,
        }
    }
}

/// Path validator with configurable security policies.
///
/// Provides methods to validate file paths against various security threats
/// including symlinks, path traversal, and invalid filenames.
#[derive(Debug, Clone)]
pub struct PathValidator {
    /// Whether to allow following symbolic links
    pub allow_symlinks: bool,
}

impl PathValidator {
    /// Creates a new PathValidator with the specified symlink policy.
    ///
    /// # Arguments
    ///
    /// * `allow_symlinks` - If false, validation will reject symbolic links
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let validator = PathValidator::new(false); // Reject symlinks
    /// let validator = PathValidator::new(true);  // Allow symlinks
    /// ```
    #[must_use]
    pub fn new(allow_symlinks: bool) -> Self {
        Self { allow_symlinks }
    }

    /// Validates a source file path.
    ///
    /// Performs the following checks:
    /// 1. Detects if the path is a symbolic link (using symlink_metadata)
    /// 2. Canonicalizes the path to resolve all symbolic links and relative components
    /// 3. Verifies the resolved path points to a regular file
    /// 4. Rejects symbolic links if allow_symlinks is false
    ///
    /// # Arguments
    ///
    /// * `path` - The source file path to validate
    ///
    /// # Returns
    ///
    /// A `ValidatedPath` containing the canonical path and symlink status
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path doesn't exist
    /// - The path is a symlink and allow_symlinks is false
    /// - The path is not a regular file (e.g., it's a directory)
    /// - Canonicalization fails
    pub fn validate_source(&self, path: &Path) -> Result<ValidatedPath> {
        // Use symlink_metadata to detect symlinks without following them
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("Failed to read metadata for: {}", path.display()))?;

        let is_symlink = metadata.is_symlink();

        // Reject symlinks if policy doesn't allow them
        if is_symlink && !self.allow_symlinks {
            anyhow::bail!(
                "Symbolic links are not allowed by default. Use --follow-symlinks to allow.\nPath: {}",
                path.display()
            );
        }

        // Canonicalize to get the real path (follows symlinks)
        let canonical = fs::canonicalize(path)
            .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

        // Verify it's a regular file after resolution
        if !canonical.is_file() {
            anyhow::bail!("Path is not a regular file: {}", canonical.display());
        }

        Ok(ValidatedPath::new(canonical, is_symlink, path.to_path_buf()))
    }

    /// Validates a target file path.
    ///
    /// Performs the following checks:
    /// 1. Extracts and validates the filename component
    /// 2. Ensures the parent directory exists or can be created
    /// 3. Canonicalizes the parent directory
    /// 4. Constructs the final target path from canonical parent + validated filename
    ///
    /// Note: This does NOT check if the target already exists - that check
    /// happens later in the file operation to minimize TOCTOU race conditions.
    ///
    /// # Arguments
    ///
    /// * `path` - The target file path to validate
    ///
    /// # Returns
    ///
    /// A `ValidatedPath` with the canonical target path
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path has no filename component
    /// - The filename contains forbidden characters or reserved names
    /// - The parent directory cannot be resolved
    pub fn validate_target(&self, path: &Path) -> Result<ValidatedPath> {
        // Extract and validate the filename
        let filename = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Target path has no filename component"))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Target filename is not valid UTF-8"))?;

        self.validate_filename(filename)?;

        // Get the parent directory
        let parent = path.parent().unwrap_or_else(|| Path::new("."));

        // Canonicalize parent if it exists, otherwise use as-is
        // (directory will be created later by the file operation)
        let canonical_parent = if parent.exists() {
            fs::canonicalize(parent)
                .with_context(|| format!("Failed to canonicalize parent directory: {}", parent.display()))?
        } else {
            // Parent doesn't exist yet - will be created later
            // Convert to absolute path for consistency
            parent.to_path_buf()
        };

        // Construct the final target path
        let canonical = canonical_parent.join(filename);

        Ok(ValidatedPath::new(canonical, false, path.to_path_buf()))
    }

    /// Validates a filename for security issues.
    ///
    /// Checks for:
    /// - Forbidden characters: \ / : * ? " < > |
    /// - Windows reserved names: CON, PRN, AUX, NUL, COM1-9, LPT1-9, CLOCK$
    /// - Excessive length (> 255 characters)
    /// - Empty filenames
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to validate (without directory path)
    ///
    /// # Errors
    ///
    /// Returns an error if the filename contains forbidden characters,
    /// is a reserved name, is too long, or is empty.
    pub fn validate_filename(&self, filename: &str) -> Result<()> {
        // Check for empty filename
        if filename.is_empty() {
            anyhow::bail!("Filename cannot be empty");
        }

        // Check length (max 255 chars for most filesystems)
        if filename.len() > 255 {
            anyhow::bail!("Filename too long (max 255 characters): {}", filename);
        }

        // Check for forbidden characters
        const FORBIDDEN_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
        if let Some(ch) = filename.chars().find(|c| FORBIDDEN_CHARS.contains(c)) {
            anyhow::bail!("Filename contains forbidden character '{}': {}", ch, filename);
        }

        // Check for Windows reserved names (case-insensitive)
        let name_upper = filename.to_uppercase();

        // Extract the name without extension for reserved name checking
        let base_name = if let Some(pos) = name_upper.rfind('.') {
            &name_upper[..pos]
        } else {
            &name_upper
        };

        // List of Windows reserved names
        const RESERVED_NAMES: &[&str] = &[
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
            "CLOCK$",
        ];

        if RESERVED_NAMES.contains(&base_name) {
            anyhow::bail!(
                "Filename uses Windows reserved name '{}': {}",
                base_name,
                filename
            );
        }

        Ok(())
    }

    /// Checks if a path is within an allowed base directory.
    ///
    /// This prevents path traversal attacks by ensuring the canonical
    /// path starts with the canonical base directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check
    /// * `base` - The base directory that should contain the path
    ///
    /// # Returns
    ///
    /// `true` if the path is within the base directory, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let validator = PathValidator::new(false);
    ///
    /// // Safe: /home/user/docs/file.txt is within /home/user
    /// assert!(validator.is_within_allowed_directory(
    ///     Path::new("/home/user/docs/file.txt"),
    ///     Path::new("/home/user")
    /// ));
    ///
    /// // Unsafe: /etc/passwd is not within /home/user
    /// assert!(!validator.is_within_allowed_directory(
    ///     Path::new("/etc/passwd"),
    ///     Path::new("/home/user")
    /// ));
    /// ```
    pub fn is_within_allowed_directory(&self, path: &Path, base: &Path) -> bool {
        // Canonicalize both paths to handle symlinks and relative components
        let canonical_path = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false,
        };

        let canonical_base = match fs::canonicalize(base) {
            Ok(b) => b,
            Err(_) => return false,
        };

        // Check if the path starts with the base
        canonical_path.starts_with(canonical_base)
    }

    /// Checks if a path string contains path traversal sequences.
    ///
    /// Detects patterns commonly used in path traversal attacks:
    /// - `../` (parent directory)
    /// - `..\\` (parent directory on Windows)
    /// - Path components that are exactly ".."
    ///
    /// # Arguments
    ///
    /// * `path_str` - The path string to check
    ///
    /// # Returns
    ///
    /// `true` if the path contains suspicious traversal patterns
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let validator = PathValidator::new(false);
    ///
    /// assert!(validator.contains_path_traversal("../etc/passwd"));
    /// assert!(validator.contains_path_traversal("docs/../../etc/passwd"));
    /// assert!(!validator.contains_path_traversal("docs/files"));
    /// ```
    pub fn contains_path_traversal(&self, path_str: &str) -> bool {
        // Check for obvious traversal patterns
        if path_str.contains("../") || path_str.contains("..\\") {
            return true;
        }

        // Check each path component
        let path = Path::new(path_str);
        for component in path.components() {
            if let std::path::Component::ParentDir = component {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Helper to create a temporary test file
    fn create_test_file(dir: &TempDir, name: &str) -> PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        file_path
    }

    /// Helper to create a temporary directory structure
    fn create_test_dir(dir: &TempDir, name: &str) -> PathBuf {
        let dir_path = dir.path().join(name);
        fs::create_dir(&dir_path).unwrap();
        dir_path
    }

    #[test]
    fn test_validate_source_regular_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt");

        let validator = PathValidator::new(false);
        let result = validator.validate_source(&test_file);

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert!(!validated.is_symlink);
        assert_eq!(validated.original, test_file);
    }

    #[test]
    fn test_validate_source_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent.txt");

        let validator = PathValidator::new(false);
        let result = validator.validate_source(&nonexistent);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to read metadata"));
    }

    #[test]
    fn test_validate_source_directory_rejected() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = create_test_dir(&temp_dir, "testdir");

        let validator = PathValidator::new(false);
        let result = validator.validate_source(&test_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a regular file"));
    }

    #[test]
    #[cfg(unix)]
    fn test_detect_symlink_source() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt");
        let symlink = temp_dir.path().join("link.txt");

        std::os::unix::fs::symlink(&test_file, &symlink).unwrap();

        let validator = PathValidator::new(true); // Allow symlinks
        let result = validator.validate_source(&symlink);

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert!(validated.is_symlink);
    }

    #[test]
    #[cfg(unix)]
    fn test_reject_symlink_by_default() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt");
        let symlink = temp_dir.path().join("link.txt");

        std::os::unix::fs::symlink(&test_file, &symlink).unwrap();

        let validator = PathValidator::new(false); // Reject symlinks
        let result = validator.validate_source(&symlink);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Symbolic links are not allowed"));
    }

    #[test]
    fn test_validate_target_with_valid_filename() {
        let temp_dir = TempDir::new().unwrap();
        let target = temp_dir.path().join("2024-11-15_TEST_Document.pdf");

        let validator = PathValidator::new(false);
        let result = validator.validate_target(&target);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_target_no_filename() {
        let validator = PathValidator::new(false);
        let result = validator.validate_target(Path::new("/"));

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no filename component"));
    }

    #[test]
    fn test_validate_filename_valid() {
        let validator = PathValidator::new(false);

        assert!(validator.validate_filename("2024-11-15_TEST_Document.pdf").is_ok());
        assert!(validator.validate_filename("simple.txt").is_ok());
        assert!(validator.validate_filename("file_with_underscores.doc").is_ok());
        assert!(validator.validate_filename("file-with-dashes.txt").is_ok());
    }

    #[test]
    fn test_validate_filename_invalid_chars() {
        let validator = PathValidator::new(false);

        // Test each forbidden character
        assert!(validator.validate_filename("file\\test.txt").is_err());
        assert!(validator.validate_filename("file/test.txt").is_err());
        assert!(validator.validate_filename("file:test.txt").is_err());
        assert!(validator.validate_filename("file*test.txt").is_err());
        assert!(validator.validate_filename("file?test.txt").is_err());
        assert!(validator.validate_filename("file\"test.txt").is_err());
        assert!(validator.validate_filename("file<test.txt").is_err());
        assert!(validator.validate_filename("file>test.txt").is_err());
        assert!(validator.validate_filename("file|test.txt").is_err());
    }

    #[test]
    fn test_reject_windows_reserved_names() {
        let validator = PathValidator::new(false);

        // Test basic reserved names
        assert!(validator.validate_filename("CON.txt").is_err());
        assert!(validator.validate_filename("PRN.pdf").is_err());
        assert!(validator.validate_filename("AUX.doc").is_err());
        assert!(validator.validate_filename("NUL.txt").is_err());

        // Test COM1-9
        assert!(validator.validate_filename("COM1.txt").is_err());
        assert!(validator.validate_filename("COM5.pdf").is_err());
        assert!(validator.validate_filename("COM9.doc").is_err());

        // Test LPT1-9
        assert!(validator.validate_filename("LPT1.txt").is_err());
        assert!(validator.validate_filename("LPT5.pdf").is_err());
        assert!(validator.validate_filename("LPT9.doc").is_err());

        // Test CLOCK$
        assert!(validator.validate_filename("CLOCK$.txt").is_err());

        // Test case-insensitivity
        assert!(validator.validate_filename("con.txt").is_err());
        assert!(validator.validate_filename("Con.txt").is_err());
        assert!(validator.validate_filename("CON.TXT").is_err());
    }

    #[test]
    fn test_allow_similar_to_reserved_names() {
        let validator = PathValidator::new(false);

        // These should be allowed (not exact matches)
        assert!(validator.validate_filename("CONTEXT.txt").is_ok());
        assert!(validator.validate_filename("PRINTER.pdf").is_ok());
        assert!(validator.validate_filename("COMMON.doc").is_ok());
        assert!(validator.validate_filename("LAPTOP.txt").is_ok());
    }

    #[test]
    fn test_validate_filename_empty() {
        let validator = PathValidator::new(false);
        assert!(validator.validate_filename("").is_err());
    }

    #[test]
    fn test_validate_filename_too_long() {
        let validator = PathValidator::new(false);
        let long_name = "a".repeat(256);

        assert!(validator.validate_filename(&long_name).is_err());

        // 255 chars should be ok
        let max_name = "a".repeat(255);
        assert!(validator.validate_filename(&max_name).is_ok());
    }

    #[test]
    fn test_canonical_path_resolution() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = create_test_file(&temp_dir, "test.txt");

        // Create a path with ../ components
        let subdir = create_test_dir(&temp_dir, "subdir");
        let relative_path = subdir.join("../test.txt");

        let validator = PathValidator::new(false);
        let result = validator.validate_source(&relative_path);

        assert!(result.is_ok());
        let validated = result.unwrap();
        // The canonical path should resolve the ../ and match the original file
        assert_eq!(
            validated.canonical,
            fs::canonicalize(&test_file).unwrap()
        );
    }

    #[test]
    fn test_is_within_allowed_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        let subdir = create_test_dir(&temp_dir, "subdir");
        let file_in_subdir = create_test_file(&temp_dir, "subdir/test.txt");

        let validator = PathValidator::new(false);

        // File within base directory
        assert!(validator.is_within_allowed_directory(&file_in_subdir, base_dir));

        // Directory within base directory
        assert!(validator.is_within_allowed_directory(&subdir, base_dir));

        // Base directory is within itself
        assert!(validator.is_within_allowed_directory(base_dir, base_dir));
    }

    #[test]
    fn test_is_not_within_allowed_directory() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        let file1 = create_test_file(&temp_dir1, "test1.txt");
        let base2 = temp_dir2.path();

        let validator = PathValidator::new(false);

        // File from different directory tree
        assert!(!validator.is_within_allowed_directory(&file1, base2));
    }

    #[test]
    #[cfg(unix)]
    fn test_path_traversal_with_symlink() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = create_test_dir(&temp_dir, "base");

        // Create a file outside the base directory
        let outside_file = create_test_file(&temp_dir, "outside.txt");

        // Create a symlink inside base that points outside
        let symlink = base_dir.join("link.txt");
        std::os::unix::fs::symlink(&outside_file, &symlink).unwrap();

        let validator = PathValidator::new(true); // Allow symlinks

        // The symlink itself is within the base
        // But when we validate it as a source, it should resolve to outside file
        let validated = validator.validate_source(&symlink).unwrap();

        // The canonical path should be the outside file
        assert_eq!(
            validated.canonical,
            fs::canonicalize(&outside_file).unwrap()
        );

        // The canonical path is NOT within base_dir
        assert!(!validator.is_within_allowed_directory(&validated.canonical, &base_dir));
    }

    #[test]
    fn test_contains_path_traversal_detected() {
        let validator = PathValidator::new(false);

        // Unix-style path traversal
        assert!(validator.contains_path_traversal("../etc/passwd"));
        assert!(validator.contains_path_traversal("docs/../../../etc/passwd"));
        assert!(validator.contains_path_traversal("./../../file.txt"));

        // Windows-style path traversal
        assert!(validator.contains_path_traversal("..\\windows\\system32"));
        assert!(validator.contains_path_traversal("docs\\..\\..\\file.txt"));

        // Mixed
        assert!(validator.contains_path_traversal("docs/../file.txt"));
    }

    #[test]
    fn test_contains_path_traversal_safe_paths() {
        let validator = PathValidator::new(false);

        // These are safe paths without traversal
        assert!(!validator.contains_path_traversal("docs/files"));
        assert!(!validator.contains_path_traversal("documents/2024/taxes.pdf"));
        assert!(!validator.contains_path_traversal("/absolute/path/file.txt"));
        assert!(!validator.contains_path_traversal("relative/path"));
        assert!(!validator.contains_path_traversal("file.txt"));

        // Paths that contain ".." in filenames (not as component)
        // These should still be detected by the component check
        assert!(!validator.contains_path_traversal("file..txt"));
        assert!(!validator.contains_path_traversal("my..file.pdf"));
    }
}
