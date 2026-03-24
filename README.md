# Clevis

A CLI tool for validating consistency between different parts of your codebase by comparing values across multiple file formats and locations.

Clevis helps you maintain consistency in your projects by "linking" values that should match across different files. Instead of manually checking that version numbers, configuration values, or other data stays synchronized, Clevis automates these checks to catch inconsistencies early.

## Features

- **Multiple file format support**: TOML, YAML, and plain text files
- **Flexible value extraction**: Extract values using key paths, text spans, or custom queries
- **Batch validation**: Check all links at once or validate specific ones
- **Clear reporting**: Get detailed output showing which values match or differ
- **Relative path support**: Use paths relative to your configuration file
- **CI/CD integration**: Available as a GitHub Action for automated checks

## Configuration

Create a `clevis.toml` file in your project root. Each link defines two accessors (`a` and `b`) whose extracted values must match.

```toml
[links.<link_name>.a]
kind = "<reader>"
file_path = "<path>"
# reader-specific fields

[links.<link_name>.b]
kind = "<reader>"
file_path = "<path>"
# reader-specific fields
```

File paths can be absolute or relative to the `clevis.toml` file.

### Readers

#### TOML

Extracts a value from a TOML file using a dot-separated key path. Supports array indexing (e.g., `items[0].name`).

```toml
[links.version.a]
kind = "toml"
file_path = "Cargo.toml"
key_path = "package.version"
```

#### YAML

Extracts a value from a YAML file using a dot-separated key path. Supports array indexing (e.g., `spec.containers[0].image`).

```toml
[links.version.b]
kind = "yaml"
file_path = "Chart.yaml"
key_path = "appVersion"
```

#### Span

Extracts a slice of text from any file by specifying start and end positions (1-indexed line and column numbers).

```toml
[links.version.a]
kind = "span"
file_path = "README.md"
[links.version.a.start]
line = 5
column = 10
[links.version.a.end]
line = 5
column = 15
```

#### Query

Checks that an exact string exists in a file and returns it. Useful for verifying a value appears in an unstructured file.

```toml
[links.base_image.a]
kind = "query"
file_path = "Dockerfile"
query = "FROM ubuntu:24.04"
```

### Example

```toml
# Ensure Cargo.toml and the GitHub Action workflow reference the same version
[links.cargo_version.a]
kind = "toml"
file_path = "Cargo.toml"
key_path = "package.version"

[links.cargo_version.b]
kind = "span"
file_path = ".github/workflows/release.yml"
[links.cargo_version.b.start]
line = 12
column = 14
[links.cargo_version.b.end]
line = 12
column = 19
```

## Supported Readers

- **TOML**: Read values using key paths (e.g., `package.version`)
- **YAML**: Read values using key paths (e.g., `metadata.name`)
- **Spans**: Read specific character ranges from any text file
- **Query**: Use custom queries to extract values from files

Generation, vs linking, is preferable. If you can't generate, then link. Linking infers having manually created something, and manual creation is generally more work intensive and error prone, as opposed to running generation—in the areas that this project covers, as opposed to things like code generation. 

## Installation

Download a GitHub release or build it yourself.

### Build

1. `cargo build --release`
2. `cp target/release/clevis ~/.local/usr/bin`

Or:

1 `cargo install --locked --path .`

## Usage

### Local

Use it as `clevis` from the terminal.

### GitHub Action

There's a ready-made Action available at [https://github.com/jesse-c/clevis-action](https://github.com/jesse-c/clevis-action).

## Releases

Run manually the `CD` flow. Optionally specify a version.
