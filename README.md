# cfrename

`cfrename` is a CLI (command-line) tool designed to **standardize file renaming and organization** using a **strict naming convention** and **user-defined hierarchical configuration**.

The project now has a functional first version (MVP).

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)

---

## 🎯 Objective

The goal of `cfrename` is to provide a reliable and durable tool that:

- Produces consistent and unambiguous filenames
- Applies a unique and explicit naming convention
- Reduces human errors related to manual renaming
- Clearly separates business rules from implementation
- Builds a system that is understandable and maintainable over time

---

## 🧠 Project Philosophy

The project is built on a few simple principles:

- **Configuration describes the rules**
  Categories, types, descriptions, and constraints are never hardcoded.

- **No implicit assumptions**
  Every piece of necessary information is explicitly requested or defined.

- **Hierarchy before automation**
  Documents are classified according to a logical and navigable structure.

- **Durability**
  Generated filenames must remain readable and understandable years from now.

---

## 🏷️ Target Naming Convention

### Standard format

```
YYYY-MM-DD_TYPE_DESCRIPTION.ext
```

Examples:
```
2024-11-15_TAX_Notice.pdf
2023-06-01_BANK_Statement.pdf
```

### Extended format with entity

Some documents are linked to an external entity (company, bank, organization).

```
YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext
```

Examples:
```
2025-10-05_WORK_ACME_Contract_Permanent.pdf
2023-03-15_BANK_BNP_Statement.pdf
```

The inclusion of the entity depends on the document type and is defined by the configuration.

---

## 🗂️ Organizational Model

```
Category
 └── Type
      └── Description
```

Navigation is guided, without dangerous free-form input.

---

## 🧾 Configuration

The tool's behavior is entirely driven by an external configuration file stored in platform-specific standard directories:

- **Linux**: `~/.config/cfrename/config.toml`
- **macOS**: `~/Library/Application Support/io.swiizyy.cfrename/config.toml`
- **Windows**: `%APPDATA%\swiizyy\cfrename\config.toml`

The configuration defines:
- Language (English or French)
- Base path for document organization (optional)
- Categories
- Document types
- Allowed descriptions
- Target directories (can be relative to base path)
- Required fields (e.g., mandatory entity)
- Date input formats (output always uses YYYY-MM-DD)

---

## ⚙️ Features

- **Multilingual interface** - English and French support
- **Automatic configuration setup** - Creates default config on first run
- Interactive CLI
- Keyboard navigation
- Guided hierarchical selection
- Strict input validation
- Secure filename generation
- Pre-action confirmation
- Deterministic behavior
- Configurable date input formats (while output remains standardized)
- Automatic directory creation

---

## 🚀 Installation and Usage

See the [USAGE.md](USAGE.md) file for complete installation and usage instructions.

### Quick Start

```bash
# Build the project
cargo build --release

# On first run, cfrename will prompt you to create a default configuration
# Just run the tool and answer 'Yes' when prompted:
./target/release/cfrename <file>

# Or manually copy the example configuration
mkdir -p ~/.config/cfrename
cp config.example.toml ~/.config/cfrename/config.toml

# Edit the configuration to match your needs
# Set language (english/french), base_path, and customize categories
nano ~/.config/cfrename/config.toml
```

### Example Usage

```bash
# Rename a document interactively
cfrename ~/Downloads/document.pdf

# Use a custom configuration file
cfrename ~/Downloads/document.pdf --config /path/to/config.toml
```

---

## 📌 Status

Functional MVP with core features implemented:
- ✓ Multilingual support (English/French)
- ✓ Automatic configuration setup on first run
- ✓ Interactive CLI interface
- ✓ Guided keyboard navigation
- ✓ External configuration (TOML)
- ✓ Support for conditional entities
- ✓ Convention-compliant filename generation
- ✓ Preview and confirmation before action
- ✓ Automatic target directory creation
- ✓ Configurable date input formats
- ✓ Base path support for organized file structure

---

## 🛣️ Roadmap Summary

1. ✓ Document model and naming convention
2. ✓ External configuration
3. ✓ Guided CLI interface
4. ✓ Support for conditional entities
5. ✓ Security and confirmations
6. ✓ Automatic file organization
7. Ongoing: Robustness and maintainability improvements

---

## 📚 Documentation

- [USAGE.md](USAGE.md) - Detailed usage guide with examples
- [CLAUDE.md](CLAUDE.md) - Development guidelines for contributors
- [CHANGELOG.md](CHANGELOG.md) - Version history and changes
- [config.example.toml](config.example.toml) - Example configuration file

---

## 🤝 Contributing

This is a personal project, but contributions, suggestions, and feedback are welcome!

Please ensure that any contributions adhere to the project's core philosophy:
- Keep business rules in configuration, not code
- Maintain strict separation of concerns
- Avoid implicit assumptions
- Write clear, maintainable code

---

## 📜 License

MIT License - Free for personal and commercial use.

---

## 🔧 Development

```bash
# Build
cargo build

# Build release
cargo build --release

# Run
cargo run -- <file>

# Run with custom config
cargo run -- <file> --config config.example.toml

# Test
cargo test

# Test specific test
cargo test <test_name>

# Check without building
cargo check
```

---

## 💡 Design Principles

The codebase is organized into focused modules:

- **config.rs** - Configuration loading and parsing
- **interactive.rs** - CLI interaction layer
- **naming.rs** - Filename generation logic
- **operations.rs** - File system operations
- **main.rs** - Application entry point

All document categories, types, descriptions, and entities are configuration-driven. The implementation maintains strict separation between business rules and technical implementation.
