# cfrename - Usage Guide

Complete guide for installing and using cfrename.

## Table of Contents

- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Example Sessions](#example-sessions)
- [Configuration Structure](#configuration-structure)
- [Output Format](#output-format)
- [Features](#features)
- [Troubleshooting](#troubleshooting)

---

## Installation

### Prerequisites

- Rust toolchain (1.70 or later)
- Cargo (comes with Rust)

### Build from source

```bash
# Clone the repository (if applicable)
git clone https://github.com/swiizyy/cfrename.git
cd cfrename

# Build the release version
cargo build --release

# The binary will be at: target/release/cfrename

# Optional: Install system-wide
cargo install --path .
```

After installation, the `cfrename` command will be available in your PATH.

---

## Configuration

Before using cfrename, you need to create a configuration file.

### 1. Copy the example configuration

```bash
# Create the config directory
mkdir -p ~/.config/cfrename

# Copy the example file
cp config.example.toml ~/.config/cfrename/config.toml
```

### 2. Edit the configuration

Open the configuration file with your preferred editor:

```bash
# Using nano
nano ~/.config/cfrename/config.toml

# Using vim
vim ~/.config/cfrename/config.toml

# Using VS Code
code ~/.config/cfrename/config.toml
```

### 3. Customize to your needs

Modify the categories, types, descriptions, and entities to match your document organization needs. See the [Configuration Structure](#configuration-structure) section for details.

---

## Usage

### Basic usage

```bash
cfrename <file>
```

### With custom configuration

```bash
cfrename <file> --config /path/to/config.toml
```

### Help

```bash
cfrename --help
```

---

## Example Sessions

### Example 1: Professional document with entity

Renaming a work contract:

```bash
$ cfrename ~/Downloads/contract.pdf

=== cfrename - File Renaming Tool ===

? Select category ›
  Administrative
❯ Professional
  Medical
  Personal

? Select document type ›
❯ Work
  Training
  Business

? Select description ›
  Permanent Contract
❯ Fixed-Term Contract
  Amendment
  Payslip

? Select entity ›
  ACME
❯ TechCorp
  GlobalServices

? Enter date (%Y-%m-%d, %d/%m/%Y, %d-%m-%Y) › 2024-11-15

=== Rename Preview ===
Source: /Users/user/Downloads/contract.pdf
Target: /Users/user/Documents/Professional/2024-11-15_WORK_TECHCORP_Fixed-Term_Contract.pdf

? Proceed with rename? › (y/N) y

✓ File renamed successfully!
  → /Users/user/Documents/Professional/2024-11-15_WORK_TECHCORP_Fixed-Term_Contract.pdf
```

### Example 2: Administrative document without entity

Renaming a tax document:

```bash
$ cfrename ~/Downloads/tax_notice.pdf

=== cfrename - File Renaming Tool ===

? Select category ›
❯ Administrative
  Professional
  Medical
  Personal

? Select document type ›
❯ Tax
  Bank
  Insurance
  Utilities

? Select description ›
❯ Notice
  Declaration
  Receipt

? Enter date (%Y-%m-%d, %d/%m/%Y, %d-%m-%Y) › 15/11/2024

=== Rename Preview ===
Source: /Users/user/Downloads/tax_notice.pdf
Target: /Users/user/Documents/Administrative/2024-11-15_TAX_Notice.pdf

? Proceed with rename? › (y/N) y

✓ File renamed successfully!
  → /Users/user/Documents/Administrative/2024-11-15_TAX_Notice.pdf
```

### Example 3: Medical document

Renaming a prescription:

```bash
$ cfrename ~/Desktop/prescription.pdf

=== cfrename - File Renaming Tool ===

? Select category ›
  Administrative
  Professional
❯ Medical
  Personal

? Select document type ›
❯ Prescription
  Lab
  Report

? Select description ›
❯ Consultation
  Renewal
  Specialist

? Enter date (%Y-%m-%d, %d/%m/%Y, %d-%m-%Y) › 2024-11-15

=== Rename Preview ===
Source: /Users/user/Desktop/prescription.pdf
Target: /Users/user/Documents/Medical/2024-11-15_PRESCRIPTION_Consultation.pdf

? Proceed with rename? › (y/N) y

✓ File renamed successfully!
  → /Users/user/Documents/Medical/2024-11-15_PRESCRIPTION_Consultation.pdf
```

### Example 4: Canceling an operation

You can cancel at any time:

```bash
$ cfrename ~/Downloads/document.pdf

=== cfrename - File Renaming Tool ===

? Select category ›
❯ Administrative
  Professional
  Medical
  Personal

? Select document type ›
❯ Tax
  Bank
  Insurance
  Utilities

? Select description ›
❯ Notice
  Declaration
  Receipt

? Enter date (%Y-%m-%d, %d/%m/%Y, %d-%m-%Y) › 2024-11-15

=== Rename Preview ===
Source: /Users/user/Downloads/document.pdf
Target: /Users/user/Documents/Administrative/2024-11-15_TAX_Notice.pdf

? Proceed with rename? › (y/N) n

Operation cancelled.
```

---

## Configuration Structure

The configuration file uses TOML format with the following structure:

### Base path (optional)

```toml
base_path = "~/Documents"
```

- Defines the root directory for document organization
- Supports tilde (`~`) expansion for home directory
- Category `target_directory` paths will be relative to this base path
- If a category has an absolute path, base_path is ignored for that category
- If omitted, category paths work as before (absolute or relative to source)

**Benefits:**
- Centralized configuration - change base path once instead of in every category
- Cleaner category definitions with relative paths
- Easy to move your entire document structure to a new location

### Date formats (optional)

```toml
date_formats = [
    "%Y-%m-%d",    # YYYY-MM-DD (ISO format)
    "%d/%m/%Y",    # DD/MM/YYYY (European format)
    "%d-%m-%Y",    # DD-MM-YYYY
]
```

- The first format is used as the default when prompting
- Users can enter dates in any of the configured formats
- All output filenames will always use YYYY-MM-DD format
- If omitted, defaults to: `%Y-%m-%d`, `%d/%m/%Y`, `%d-%m-%Y`

### Category definition

```toml
[categories.category_key]
name = "Display Name"
target_directory = "Administrative"  # Optional, can be relative or absolute
```

- `category_key`: Internal identifier (lowercase, no spaces)
- `name`: Display name shown in the CLI
- `target_directory`: Optional destination directory
  - If `base_path` is set and this is a relative path → combined with base_path
  - If this is an absolute path → used as-is (base_path ignored)
  - If omitted → files stay in source directory
  - Supports `~` expansion for home directory

### Document type definition

```toml
[categories.category_key.types.type_key]
name = "Type Display Name"
descriptions = ["Option1", "Option2", "Option3"]
require_entity = false  # or true
entities = ["Entity1", "Entity2"]  # Required if require_entity = true
```

- `type_key`: Internal identifier (lowercase, no spaces)
- `name`: Display name shown in the CLI
- `descriptions`: List of allowed description options
- `require_entity`: Whether an entity is mandatory for this type
- `entities`: List of available entities (only needed if `require_entity = true`)

### Complete example

```toml
# Global settings
base_path = "~/Documents"
date_formats = ["%Y-%m-%d", "%d/%m/%Y"]

[categories.administrative]
name = "Administrative"
# Relative path - will be combined with base_path
# Result: ~/Documents/Administrative
target_directory = "Administrative"

[categories.administrative.types.tax]
name = "Tax"
descriptions = ["Notice", "Declaration", "Receipt"]
require_entity = false

[categories.administrative.types.bank]
name = "Bank"
descriptions = ["Statement", "IBAN", "Contract"]
require_entity = true
entities = ["BNP", "Credit Agricole", "Societe Generale"]

[categories.archived]
name = "Archived"
# Absolute path - base_path is ignored for this category
target_directory = "/mnt/backup/documents/archived"

[categories.archived.types.old]
name = "Old Documents"
descriptions = ["Archive"]
require_entity = false
```

See `config.example.toml` for a complete configuration example.

---

## Output Format

Files are renamed according to these strict conventions:

### Without entity

```
YYYY-MM-DD_TYPE_DESCRIPTION.ext
```

Examples:
```
2024-11-15_TAX_Notice.pdf
2023-06-01_BANK_Statement.pdf
```

### With entity

```
YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext
```

Examples:
```
2024-11-15_WORK_ACME_Permanent_Contract.pdf
2023-03-15_BANK_BNP_Statement.pdf
```

### Naming rules

- Date is always in `YYYY-MM-DD` format (ISO 8601)
- TYPE is converted to uppercase
- ENTITY is converted to uppercase (if present)
- Description preserves the case from configuration
- Components are separated by underscores
- Original file extension is preserved
- Spaces in descriptions are replaced with underscores

---

## Features

### Interactive guided navigation

- ✓ Step-by-step selection process
- ✓ Keyboard navigation (arrow keys, Enter)
- ✓ No free-form text input for structured fields
- ✓ Clear prompts and visual feedback

### Strict validation

- ✓ Only configured options are selectable
- ✓ Date format validation
- ✓ File existence checks
- ✓ Duplicate detection

### Configurable date input formats

- ✓ Accept multiple date input formats
- ✓ Configurable format preferences
- ✓ Output always uses YYYY-MM-DD format
- ✓ Clear format guidance in prompts

### Preview and confirmation

- ✓ Show source and target paths before rename
- ✓ Explicit confirmation required
- ✓ Ability to cancel at any time

### Automatic directory creation

- ✓ Target directories are created automatically
- ✓ Parent directories are created as needed
- ✓ Safe directory path handling

### Configuration-driven behavior

- ✓ All categories, types, and descriptions from config
- ✓ No hardcoded business rules
- ✓ Entity requirements per document type
- ✓ Flexible target directory structure

---

## Troubleshooting

### Configuration file not found

**Error:**
```
Error: Configuration file not found.
Expected location: /Users/user/.config/cfrename/config.toml
```

**Solution:**
Copy the example configuration:
```bash
mkdir -p ~/.config/cfrename
cp config.example.toml ~/.config/cfrename/config.toml
```

### Invalid date format

**Error:**
```
Invalid date format. Expected one of: %Y-%m-%d, %d/%m/%Y, %d-%m-%Y
```

**Solution:**
Enter the date in one of the accepted formats. Check your configuration's `date_formats` setting.

### Target file already exists

**Error:**
```
Target file already exists: /path/to/target.pdf
```

**Solution:**
The tool won't overwrite existing files. Either:
- Rename or move the existing file
- Use a different date or description
- Choose a different target directory

### File does not exist

**Error:**
```
File does not exist: /path/to/file.pdf
```

**Solution:**
Check the file path. Use tab completion or copy the path from your file manager.

### Permission denied

**Error:**
```
Failed to create directory: /path/to/directory
```

**Solution:**
Ensure you have write permissions for the target directory. You may need to:
- Choose a different target directory in your configuration
- Change permissions: `chmod +w /path/to/directory`
- Run with appropriate permissions

### No categories defined

**Error:**
```
No categories defined in configuration
```

**Solution:**
Your configuration file is empty or malformed. Review the configuration structure and ensure at least one category is defined.

---

## Advanced Usage

### Multiple configurations

You can maintain multiple configuration files for different use cases:

```bash
# Work documents
cfrename document.pdf --config ~/.config/cfrename/work.toml

# Personal documents
cfrename document.pdf --config ~/.config/cfrename/personal.toml
```

### Shell aliases

Add aliases to your shell configuration for convenience:

```bash
# In ~/.bashrc or ~/.zshrc
alias cfr='cfrename'
alias cfr-work='cfrename --config ~/.config/cfrename/work.toml'
alias cfr-personal='cfrename --config ~/.config/cfrename/personal.toml'
```

### Batch processing

While cfrename is designed for interactive use, you can combine it with shell scripts:

```bash
#!/bin/bash
# Rename all PDFs in a directory (one by one, interactively)
for file in ~/Downloads/*.pdf; do
    cfrename "$file"
done
```

---

## Tips and Best Practices

### Organization

- Use meaningful category names that reflect your workflow
- Keep descriptions concise but clear
- Use entities consistently (same spelling, capitalization)

### Date formats

- Configure your preferred date format as the first option
- Include formats you commonly use from various sources
- Remember output is always YYYY-MM-DD regardless of input

### Target directories

- Use `base_path` to define your root documents folder once
- Keep category paths relative for easier maintenance
- Organize target directories by category
- Use home directory expansion (`~`) for portability
- Create a logical hierarchy that matches your needs
- Use absolute paths for special cases (like archived or backup locations)

### Maintenance

- Review and update your configuration periodically
- Remove unused categories, types, or entities
- Add new options as your needs evolve

---

## Common Date Format Patterns

Reference for configuring `date_formats`:

| Pattern | Example | Description |
|---------|---------|-------------|
| `%Y-%m-%d` | 2024-11-15 | ISO 8601 format (YYYY-MM-DD) |
| `%d/%m/%Y` | 15/11/2024 | European format (DD/MM/YYYY) |
| `%m/%d/%Y` | 11/15/2024 | US format (MM/DD/YYYY) |
| `%d-%m-%Y` | 15-11-2024 | Dashed European (DD-MM-YYYY) |
| `%Y%m%d` | 20241115 | Compact format (YYYYMMDD) |
| `%d.%m.%Y` | 15.11.2024 | Dotted European (DD.MM.YYYY) |

Where:
- `%Y` = 4-digit year (e.g., 2024)
- `%m` = 2-digit month (01-12)
- `%d` = 2-digit day (01-31)

---

## Getting Help

- Check the [README.md](README.md) for project overview
- Review [CLAUDE.md](CLAUDE.md) for development details
- See [CHANGELOG.md](CHANGELOG.md) for version history
- Report issues on the project repository

---

**Happy organizing!**
