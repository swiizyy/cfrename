# cfrename - Usage Guide

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at: target/release/cfrename
```

## Configuration

Before using cfrename, you need to create a configuration file:

1. Copy the example configuration:
   ```bash
   mkdir -p ~/.config/cfrename
   cp config.example.toml ~/.config/cfrename/config.toml
   ```

2. Edit the configuration to match your needs:
   ```bash
   # Edit with your preferred editor
   nano ~/.config/cfrename/config.toml
   ```

## Usage

Basic usage:
```bash
cfrename <file>
```

With custom configuration:
```bash
cfrename <file> --config /path/to/config.toml
```

## Example Session

```bash
$ cfrename ~/Downloads/document.pdf

=== cfrename - File Renaming Tool ===

? Select category ›
  Administratif
❯ Professionnel
  Médical

? Select document type ›
❯ Travail
  Formation

? Select description ›
  Contrat CDI
❯ Contrat CDD
  Avenant
  Fiche de paie

? Select entity ›
  ACME
❯ TechCorp
  GlobalServices

? Enter date (YYYY-MM-DD) › 2024-11-15

=== Rename Preview ===
Source: /Users/user/Downloads/document.pdf
Target: /Users/user/Documents/Professionnel/2024-11-15_TRAVAIL_TECHCORP_Contrat_CDD.pdf

? Proceed with rename? › (y/N)
```

## Configuration Structure

The configuration file uses TOML format:

```toml
# Date input formats (optional)
# The first format is used as the default display
# Output filenames always use YYYY-MM-DD
date_formats = [
    "%Y-%m-%d",    # YYYY-MM-DD
    "%d/%m/%Y",    # DD/MM/YYYY
    "%d-%m-%Y",    # DD-MM-YYYY
]

[categories.category_key]
name = "Display Name"
target_directory = "~/path/to/directory"  # Optional

[categories.category_key.types.type_key]
name = "Type Display Name"
descriptions = ["Option1", "Option2", "Option3"]
require_entity = false  # or true
entities = ["Entity1", "Entity2"]  # Required if require_entity = true
```

See `config.example.toml` for a complete example.

## Output Format

Files are renamed according to these conventions:

### Without entity:
```
YYYY-MM-DD_TYPE_DESCRIPTION.ext
Example: 2024-11-15_IMPOTS_Avis.pdf
```

### With entity:
```
YYYY-MM-DD_TYPE_ENTITY_DESCRIPTION.ext
Example: 2024-11-15_TRAVAIL_ACME_Contrat_CDI.pdf
```

## Features

- ✓ Interactive guided navigation
- ✓ Keyboard-driven selection
- ✓ Strict validation (no free-form dangerous input)
- ✓ Configurable date input formats (output always YYYY-MM-DD)
- ✓ Preview before rename
- ✓ Confirmation required
- ✓ Automatic directory creation
- ✓ Configuration-driven behavior
- ✓ Support for conditional entity fields

## Date Format Configuration

You can configure which date formats are accepted for input while the output filename format remains fixed at YYYY-MM-DD:

```toml
date_formats = [
    "%Y-%m-%d",    # YYYY-MM-DD (default display format)
    "%d/%m/%Y",    # DD/MM/YYYY
    "%d-%m-%Y",    # DD-MM-YYYY
]
```

- The first format in the list is used as the default when prompting for dates
- Users can enter dates in any of the configured formats
- All output filenames will always use YYYY-MM-DD format regardless of input
- If omitted, defaults to: `%Y-%m-%d`, `%d/%m/%Y`, `%d-%m-%Y`

Common date format patterns:
- `%Y` - 4-digit year (e.g., 2024)
- `%m` - 2-digit month (01-12)
- `%d` - 2-digit day (01-31)
