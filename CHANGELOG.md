# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2025-12-15

### Changed
- Updated license field in Cargo.toml to "MIT"

## [0.2.0] - 2025-12-15

### Added
- Configurable date input formats support
    - Users can now specify accepted date formats in configuration file
    - Default formats: `%Y-%m-%d`, `%d/%m/%Y`, `%d-%m-%Y`, `%Y%m%d`
    - Interactive prompt shows the first configured format as default
    - Parser attempts each configured format until successful match`
- Unit tests for date format parsing and default values
- Documentation updates for new date format feature`

### Changed
- Updated `CLAUDE.md` with date format configuration details
- Enhanced `USAGE.md` with date format examples
- Expanded `config.example.toml` with date format configuration

## [0.1.1] - 2025-12-15

### Changed
- Updated dependency versions to latest stable releases

## [0.1.0] - 2025-12-15

### Added
- Initial MVP release of cfrename CLI tool
- Core filename standardization functionality
    - Standard format: `YYYY-MM-DD_TYPE_DESCRIPTION.ext`
    - Extended format: `YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext`
- Configuration system via TOML files
    - Hierarchical document organization (Category → Type → Description)
    - Configurable categories, types, and descriptions
    - Entity requirements per document type
    - Target directory configuration
- Interactive CLI interface
    - Guided step-by-step selection process
    - Keyboard navigation using dialoguer
    - Strict validation (no free-form input for structured fields)
    - Pre-action confirmation
- File operations module
    - Safe rename operations with preview
    - Automatic directory creation
    - Source/target validation
- Comprehensive documentation
    - README with project overview
    - USAGE guide with examples
    - CLAUDE.md with development guidelines
    - Example configuration file
- MIT License
- Project metadata and repository setup

### Technical Details
- Built with Rust 2024 edition
- Dependencies: clap, dialoguer, serde, toml, chrono, anyhow, directories, shellexpand
- Modular architecture with separation of concerns
    - `config.rs`: Configuration loading and parsing
    - `interactive.rs`: CLI interaction layer
    - `naming.rs`: Filename generation logic
    - `operations.rs`: File system operations
    - `main.rs`: Application entry point

