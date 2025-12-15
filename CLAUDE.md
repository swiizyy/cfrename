# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`cfrename` is a CLI tool for standardizing file renaming and organization using a strict naming convention and user-defined hierarchical configuration. The project is currently in early design and implementation phase.

**Language**: Rust
**Project Type**: Command-line interface (CLI)

## Core Philosophy

The project is built on these foundational principles:

1. **Configuration drives behavior** - Categories, types, descriptions, and constraints are never hardcoded
2. **No implicit assumptions** - Every required piece of information is explicitly requested or defined
3. **Hierarchy over automation** - Documents are classified according to a logical, navigable structure
4. **Durability** - Generated filenames must remain readable and understandable years later

## Naming Convention

The tool generates filenames following two formats:

### Standard format:
```
AAAA-MM-JJ_TYPE_DESCRIPTION.ext
```
Example: `2024-11-15_IMPOTS_Avis.pdf`

### Extended format with entity:
```
AAAA-MM-JJ_TYPE_ENTITE_DESCRIPTION.ext
```
Example: `2025-10-05_TRAVAIL_ACME_Contrat_CDI.pdf`

Entity inclusion depends on document type and is configuration-driven.

## Organizational Model

Documents are organized hierarchically:
```
Category
 └── Type
      └── Description
```

Navigation must be guided without dangerous free-form input.

## Configuration

The tool will be driven by an external configuration file:
```
~/.config/cfrename/config.toml
```

Configuration will define:
- Categories
- Document types
- Allowed descriptions
- Target directories
- Required fields (e.g., mandatory entity)

## Planned Features

- Interactive CLI
- Keyboard navigation
- Guided hierarchical selection
- Strict input validation
- Secure filename generation
- Pre-action confirmation
- Deterministic behavior

## Development Commands

- Build: `cargo build`
- Build release: `cargo build --release`
- Run: `cargo run -- <file>`
- Run with custom config: `cargo run -- <file> --config config.example.toml`
- Test: `cargo test`
- Test single test: `cargo test <test_name>`
- Check without building: `cargo check`

## Code Structure

The codebase is organized into focused modules:

### `src/config.rs`
Handles configuration loading and parsing from TOML files. Defines the hierarchical structure:
- `Config`: Root configuration containing categories and optional date input formats
- `Category`: Contains document types and optional target directory
- `DocumentType`: Defines descriptions, entity requirements, and available entities
- Supports configurable date input formats while output always uses YYYY-MM-DD
- Includes unit tests for configuration parsing and default values

### `src/naming.rs`
Implements the filename generation logic:
- `FileNameBuilder`: Constructs standardized filenames following the convention
- Handles both standard format (DATE_TYPE_DESCRIPTION) and extended format (DATE_TYPE_ENTITY_DESCRIPTION)
- Includes unit tests validating both formats

### `src/interactive.rs`
Provides the guided CLI interface:
- `InteractiveSession`: Manages the step-by-step selection process
- Uses dialoguer for interactive prompts with keyboard navigation
- Enforces selection from configured options (no free-form input for categories/types/descriptions)
- Supports multiple date input formats from configuration
- Tries parsing dates with each configured format until successful
- Returns a `RenameRequest` with all collected information

### `src/operations.rs`
Handles file system operations:
- `FileOperation`: Manages the rename operation with preview and confirmation
- `build_target_path`: Constructs target paths with support for target directories
- Creates directories as needed
- Validates source/target before execution

### `src/main.rs`
Application entry point that wires all modules together using clap for CLI argument parsing.

## Architecture Notes

The implementation maintains strict separation between:
- **Business rules** (defined in `config.toml`, never hardcoded)
- **CLI interaction layer** (`interactive.rs` - guided navigation, validation)
- **Naming logic** (`naming.rs` - deterministic filename generation)
- **File operations** (`operations.rs` - safe renaming and organization)

All document categories, types, descriptions, and entities are configuration-driven. When adding features, maintain this separation and avoid hardcoding any domain-specific logic.
