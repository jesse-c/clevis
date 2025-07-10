# Clevis

A CLI tool for validating consistency between different parts of your codebase by comparing values across multiple file formats and locations.

Clevis is a tool that helps maintain consistency across your codebase by checking that values in different files match. It supports reading from TOML files, YAML files, and specific text spans, making it perfect for ensuring configuration values, version numbers, or other important data stays synchronized.

## Features

- **Multiple file format support**: TOML, YAML, and plain text files
- **Flexible value extraction**: Extract values using key paths, text spans, or custom queries
- **Batch validation**: Check all links at once or validate specific ones
- **Clear reporting**: Get detailed output showing which values match or differ
- **Relative path support**: Use paths relative to your configuration file
- **CI/CD integration**: Available as a GitHub Action for automated checks

## Supported Readers

- **TOML**: Read values using key paths (e.g., `package.version`)
- **YAML**: Read values using key paths (e.g., `metadata.name`)
- **Spans**: Read specific character ranges from any text file
- **Query**: Use custom queries to extract values from files

Generation, vs linking, is preferable. If you can't generate, then link. Linking infers having manually created something, and manual creation is generally more work intensive and error prone, as opposed to running generation—in the areas that this project covers, as opposed to things like code generation. 

## Installation

Download a GitHub release or build it yourself.

## Usage

### Local

Use it as `clevis` from the terminal.

### GitHub Action

There's a ready-made Action available at [https://github.com/jesse-c/clevis-action](https://github.com/jesse-c/clevis-action).

## Releases

Run manually the `CD` flow. Optionally specify a version.
