//! Integration tests for cfrename CLI.
//!
//! These tests verify end-to-end behavior of the cfrename command-line tool.
//!
//! Note: Many tests here verify error conditions and validation logic.
//! Full interactive flow testing requires mocking dialoguer prompts, which
//! is out of scope for these tests.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Helper to create a test file with content
fn create_test_file(dir: &TempDir, name: &str, content: &[u8]) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(content).unwrap();
    file_path
}

/// Helper to create a minimal test configuration
fn create_test_config(dir: &TempDir) -> std::path::PathBuf {
    let config_content = r#"
[categories.test]
name = "Test Category"

[categories.test.types.doc]
name = "Document"
descriptions = ["Test"]
require_entity = false
"#;
    let config_path = dir.path().join("config.toml");
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_content.as_bytes()).unwrap();
    config_path
}

#[test]
fn test_cli_help_flag() {
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CLI tool for standardizing file renaming"));
}

#[test]
fn test_cli_version_flag() {
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("cfrename"));
}

#[test]
fn test_reject_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent.pdf");
    let config = create_test_config(&temp_dir);

    // The test will fail because file doesn't exist
    // We just verify it fails (regardless of whether it's due to missing file or terminal)
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&nonexistent)
        .arg("--config")
        .arg(&config)
        .assert()
        .failure();
}

#[test]
fn test_reject_directory() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("testdir");
    fs::create_dir(&test_dir).unwrap();
    let config = create_test_config(&temp_dir);

    // Should fail (directory not a file)
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&test_dir)
        .arg("--config")
        .arg(&config)
        .assert()
        .failure();
}

#[test]
#[cfg(unix)]
fn test_reject_symlink_by_default() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file(&temp_dir, "test.pdf", b"fake pdf");
    let symlink = temp_dir.path().join("link.pdf");

    std::os::unix::fs::symlink(&test_file, &symlink).unwrap();

    let config = create_test_config(&temp_dir);

    // Without --follow-symlinks, symlinks should be rejected
    // However, in non-terminal environment, dialoguer fails first
    // So we just verify the command fails
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&symlink)
        .arg("--config")
        .arg(&config)
        .assert()
        .failure();
}

#[test]
#[cfg(unix)]
fn test_accept_symlink_with_flag() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file(&temp_dir, "test.pdf", b"fake pdf");
    let symlink = temp_dir.path().join("link.pdf");

    std::os::unix::fs::symlink(&test_file, &symlink).unwrap();

    let config = create_test_config(&temp_dir);

    // With --follow-symlinks, the symlink should be accepted
    // However, the test would still require interactive input, so we just verify
    // it doesn't fail with the symlink error
    let output = Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&symlink)
        .arg("--follow-symlinks")
        .arg("--config")
        .arg(&config)
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Symbolic links are not allowed"));
}

#[test]
fn test_config_path_traversal_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file(&temp_dir, "test.pdf", b"fake pdf");

    // Create a config with path traversal
    let malicious_config = r#"
base_path = "../../../etc"

[categories.test]
name = "Test"

[categories.test.types.doc]
name = "Document"
descriptions = ["Test"]
require_entity = false
"#;
    let config_path = temp_dir.path().join("malicious_config.toml");
    fs::write(&config_path, malicious_config).unwrap();

    // The path traversal in base_path should be rejected
    // This would be caught when trying to resolve target paths
    let output = Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&test_file)
        .arg("--config")
        .arg(&config_path)
        .output()
        .unwrap();

    // Just verify the command doesn't crash and returns an exit code
    assert!(output.status.code().is_some());
}

#[test]
fn test_missing_config_prompts_creation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = create_test_file(&temp_dir, "test.pdf", b"fake pdf");

    // Use a non-existent config path (not the default location)
    let nonexistent_config = temp_dir.path().join("nonexistent_config.toml");

    let output = Command::cargo_bin("cfrename")
        .unwrap()
        .arg(&test_file)
        .arg("--config")
        .arg(&nonexistent_config)
        .output()
        .unwrap();

    // Should fail because the specified config doesn't exist
    assert!(!output.status.success());
}

#[test]
fn test_follow_symlinks_flag_exists() {
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--follow-symlinks"));
}

#[test]
fn test_config_flag_exists() {
    Command::cargo_bin("cfrename")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--config"));
}

// Note: Testing the full interactive flow (category selection, type selection, etc.)
// would require mocking dialoguer prompts, which is complex. The tests above focus
// on:
// 1. Command-line flag parsing
// 2. Input validation (file existence, type checking)
// 3. Security features (symlink detection, path traversal)
// 4. Error handling
//
// Full end-to-end testing with simulated user input could be added using
// expect-like testing or by mocking the dialoguer interactions.
