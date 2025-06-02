//! Config module for loading and parsing configuration files.
//!
//! This module provides functionality to load configuration files in TOML format
//! and create appropriate reader instances based on the configuration.
//!
//! # Example
//!
//! ```
//! use clevis::Config;
//!
//! // Load the configuration file
//! let config = Config::load("config.toml").expect("Failed to load config");
//!
//! // Check if the values match for a specific link
//! let result = config.check("foo").expect("Failed to check link");
//! println!("Values match: {}", result);
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};
use crate::readers::{Accessor, Linker, SpanReader, TomlReader, span::Cursor};
use anyhow::Context;

/// Configuration structure that directly stores Linkers.
///
/// The `Config` struct represents a configuration loaded from a TOML file.
/// It contains a map of link keys to `Linker` objects, which are used to
/// compare values from different sources.
///
/// Each link in the configuration file must have an 'a' and 'b' section,
/// which define the two values to compare.
pub struct Config {
    /// Map of link keys to Linker objects
    pub links: HashMap<String, Linker>,
}

impl Config {
    /// Load a configuration from a file path.
    ///
    /// This function reads a TOML configuration file from the specified path,
    /// parses it, and creates a `Config` object with the appropriate linkers.
    /// If file paths in the config are not absolute, they are resolved relative
    /// to the directory containing the config file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Config` or an error.
    /// use clevis::Config;
    ///
    /// let config = Config::load("config.toml").expect("Failed to load config");
    /// ```
    pub fn load(path: &str) -> Result<Self> {
        // Get the absolute path to the config file
        let config_path = Path::new(path);
        let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));

        // Read the file
        let content = fs::read_to_string(path).map_err(|e| AppError::FileOperation {
            path: path.to_string(),
            source: e,
        })?;

        // Parse the TOML
        let parsed: toml::Value = content.parse().map_err(|e| AppError::Parse {
            file_type: "TOML".to_string(),
            path: path.to_string(),
            source: anyhow::Error::from(e),
        })?;

        // Convert to our Config structure with the config directory for resolving relative paths
        Self::parse_config(parsed, config_dir)
    }

    /// Resolve a file path relative to the config directory if it's not absolute
    fn resolve_path(file_path: &str, config_dir: &Path) -> String {
        let path = Path::new(file_path);
        if path.is_absolute() {
            file_path.to_string()
        } else {
            config_dir.join(path).to_string_lossy().into_owned()
        }
    }

    /// Parse a TOML value into a Config structure
    ///
    /// # Arguments
    ///
    /// * `value` - The parsed TOML value
    /// * `config_dir` - The directory containing the config file, used for resolving relative paths
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Config` or an error
    fn parse_config(value: toml::Value, config_dir: &Path) -> Result<Self> {
        let mut config = Config {
            links: HashMap::new(),
        };

        // Extract the links table
        if let Some(links) = value.get("links").and_then(|v| v.as_table()) {
            for (link_key, link_value) in links {
                if let Some(link_table) = link_value.as_table() {
                    // Each link should have exactly two entries: 'a' and 'b'
                    let a_value = link_table.get("a").ok_or_else(|| AppError::ConfigError {
                        message: format!("Missing 'a' in link '{}'", link_key),
                    })?;
                    let b_value = link_table.get("b").ok_or_else(|| AppError::ConfigError {
                        message: format!("Missing 'b' in link '{}'", link_key),
                    })?;

                    // Parse 'a' accessor
                    let a_kind = a_value
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .with_context(|| format!("Missing 'kind' in link '{}.a'", link_key))?;

                    let a_accessor = match a_kind {
                        "toml" => {
                            let raw_file_path = a_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.a'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let key_path = a_value
                                .get("key_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                    format!("Missing 'key_path' in link '{}.a'", link_key)
                                })?;

                            Accessor::Toml(TomlReader {
                                file_path,
                                key_path: key_path.to_string(),
                            })
                        }
                        "span" => {
                            let raw_file_path = a_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.a'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);

                            // Get start cursor
                            let start = a_value
                                .get("start")
                                .and_then(|v| v.as_table())
                                .with_context(|| {
                                    format!("Missing 'start' in link '{}.a'", link_key)
                                })?;
                            let start_line = start
                                .get("line")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'start.line' in link '{}.a'", link_key)
                                })?;
                            let start_column = start
                                .get("column")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'start.column' in link '{}.a'", link_key)
                                })?;

                            // Get end cursor
                            let end = a_value.get("end").and_then(|v| v.as_table()).with_context(
                                || format!("Missing 'end' in link '{}.a'", link_key),
                            )?;
                            let end_line =
                                end.get("line").and_then(|v| v.as_integer()).with_context(
                                    || format!("Missing 'end.line' in link '{}.a'", link_key),
                                )?;
                            let end_column = end
                                .get("column")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'end.column' in link '{}.a'", link_key)
                                })?;

                            Accessor::Spans(SpanReader {
                                file_path,
                                start: Cursor {
                                    line: start_line as usize,
                                    column: start_column as usize,
                                },
                                end: Cursor {
                                    line: end_line as usize,
                                    column: end_column as usize,
                                },
                            })
                        }
                        "yaml" => {
                            let raw_file_path = a_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.a'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let key_path = a_value
                                .get("key_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                    format!("Missing 'key_path' in link '{}.a'", link_key)
                                })?;

                            Accessor::Yaml(crate::readers::YamlReader {
                                file_path,
                                key_path: key_path.to_string(),
                            })
                        }
                        "query" => {
                            let raw_file_path = a_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.a'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let query =
                                a_value.get("query").and_then(|v| v.as_str()).with_context(
                                    || format!("Missing 'query' in link '{}.a'", link_key),
                                )?;

                            Accessor::Query(crate::readers::QueryReader {
                                file_path,
                                query: query.to_string(),
                            })
                        }
                        _ => {
                            return Err(AppError::ConfigError {
                                message: format!(
                                    "Unknown kind '{}' in link '{}.a'",
                                    a_kind, link_key
                                ),
                            });
                        }
                    };

                    // Parse 'b' accessor
                    let b_kind = b_value
                        .get("kind")
                        .and_then(|v| v.as_str())
                        .with_context(|| format!("Missing 'kind' in link '{}.b'", link_key))?;

                    let b_accessor = match b_kind {
                        "toml" => {
                            let raw_file_path = b_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.b'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let key_path = b_value
                                .get("key_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                    format!("Missing 'key_path' in link '{}.b'", link_key)
                                })?;

                            Accessor::Toml(TomlReader {
                                file_path,
                                key_path: key_path.to_string(),
                            })
                        }
                        "span" => {
                            let raw_file_path = b_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.b'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);

                            // Get start cursor
                            let start = b_value
                                .get("start")
                                .and_then(|v| v.as_table())
                                .with_context(|| {
                                    format!("Missing 'start' in link '{}.b'", link_key)
                                })?;
                            let start_line = start
                                .get("line")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'start.line' in link '{}.b'", link_key)
                                })?;
                            let start_column = start
                                .get("column")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'start.column' in link '{}.b'", link_key)
                                })?;

                            // Get end cursor
                            let end = b_value.get("end").and_then(|v| v.as_table()).with_context(
                                || format!("Missing 'end' in link '{}.b'", link_key),
                            )?;
                            let end_line =
                                end.get("line").and_then(|v| v.as_integer()).with_context(
                                    || format!("Missing 'end.line' in link '{}.b'", link_key),
                                )?;
                            let end_column = end
                                .get("column")
                                .and_then(|v| v.as_integer())
                                .with_context(|| {
                                    format!("Missing 'end.column' in link '{}.b'", link_key)
                                })?;

                            Accessor::Spans(SpanReader {
                                file_path,
                                start: Cursor {
                                    line: start_line as usize,
                                    column: start_column as usize,
                                },
                                end: Cursor {
                                    line: end_line as usize,
                                    column: end_column as usize,
                                },
                            })
                        }
                        "yaml" => {
                            let raw_file_path = b_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.b'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let key_path = b_value
                                .get("key_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                    format!("Missing 'key_path' in link '{}.b'", link_key)
                                })?;

                            Accessor::Yaml(crate::readers::YamlReader {
                                file_path,
                                key_path: key_path.to_string(),
                            })
                        }
                        "query" => {
                            let raw_file_path = b_value
                                .get("file_path")
                                .and_then(|v| v.as_str())
                                .with_context(|| {
                                format!("Missing 'file_path' in link '{}.b'", link_key)
                            })?;
                            let file_path = Self::resolve_path(raw_file_path, config_dir);
                            let query =
                                b_value.get("query").and_then(|v| v.as_str()).with_context(
                                    || format!("Missing 'query' in link '{}.b'", link_key),
                                )?;

                            Accessor::Query(crate::readers::QueryReader {
                                file_path,
                                query: query.to_string(),
                            })
                        }
                        _ => {
                            return Err(AppError::ConfigError {
                                message: format!(
                                    "Unknown kind '{}' in link '{}.b'",
                                    b_kind, link_key
                                ),
                            });
                        }
                    };

                    // Create the Linker directly
                    let linker = Linker {
                        a: a_accessor,
                        b: b_accessor,
                    };

                    config.links.insert(link_key.clone(), linker);
                }
            }
        }

        Ok(config)
    }

    /// Get a linker by key.
    ///
    /// This function retrieves a `Linker` object from the configuration by its key.
    ///
    /// # Arguments
    ///
    /// * `link_key` - The key of the link to retrieve
    ///
    /// # Returns
    ///
    /// A `Result` containing either a reference to the `Linker` or an error
    ///
    /// # Errors
    ///
    /// This function will return an error if the link key is not found in the configuration.
    ///
    /// # Example
    ///
    /// ```
    /// use clevis::Config;
    ///
    /// let config = Config::load("config.toml").expect("Failed to load config");
    /// let linker = config.get_linker("foo").expect("Failed to get linker");
    /// ```
    pub fn get_linker(&self, link_key: &str) -> Result<&Linker> {
        self.links
            .get(link_key)
            .ok_or_else(|| AppError::KeyNotFound {
                key_path: link_key.to_string(),
                file_path: "config".to_string(),
            })
    }

    /// Check if values match for a specific link.
    ///
    /// This function checks if the values from both accessors in a link match.
    ///
    /// # Arguments
    ///
    /// * `link_key` - The key of the link to check
    ///
    /// # Returns
    ///
    /// A `Result` containing either a boolean indicating whether the values match or an error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The link key is not found in the configuration
    /// - Either accessor fails to read its value
    ///
    /// # Example
    ///
    /// ```
    /// use clevis::Config;
    ///
    /// let config = Config::load("config.toml").expect("Failed to load config");
    /// let result = config.check("foo").expect("Failed to check link");
    /// println!("Values match: {}", result);
    /// ```
    pub fn check(&self, link_key: &str) -> Result<bool> {
        let linker = self.get_linker(link_key)?;
        linker.check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config() {
        // Create a temporary config file
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[links.foo.a]
kind = "toml"
file_path = "test.toml"
key_path = "test.key"
[links.foo.b]
kind = "span"
file_path = "test.txt"
[links.foo.b.start]
line = 1
column = 1
[links.foo.b.end]
line = 2
column = 10
"#
        )
        .unwrap();

        // Load the config
        let config = Config::load(file.path().to_str().unwrap()).unwrap();

        // Verify the config
        assert!(config.links.contains_key("foo"));
    }

    #[test]
    fn test_resolve_path() {
        // Test absolute path
        let config_dir = Path::new("/config/dir");
        let abs_path = if cfg!(windows) {
            "C:/path/to/file.txt"
        } else {
            "/path/to/file.txt"
        };
        let result = Config::resolve_path(abs_path, config_dir);
        assert_eq!(result, abs_path);

        // Test relative path
        let rel_path = "relative/path/file.txt";
        let expected = config_dir.join(rel_path).to_string_lossy().into_owned();
        let result = Config::resolve_path(rel_path, config_dir);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_config_with_multiple_links() {
        // Create a temporary config file with multiple links
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[links.link1.a]
kind = "toml"
file_path = "test1.toml"
key_path = "test.key1"
[links.link1.b]
kind = "toml"
file_path = "test2.toml"
key_path = "test.key2"

[links.link2.a]
kind = "span"
file_path = "test3.txt"
[links.link2.a.start]
line = 1
column = 1
[links.link2.a.end]
line = 2
column = 5
[links.link2.b]
kind = "span"
file_path = "test4.txt"
[links.link2.b.start]
line = 3
column = 1
[links.link2.b.end]
line = 4
column = 10
"#
        )
        .unwrap();

        // Load the config
        let config = Config::load(file.path().to_str().unwrap()).unwrap();

        // Verify the config has both links
        assert!(config.links.contains_key("link1"));
        assert!(config.links.contains_key("link2"));
        assert_eq!(config.links.len(), 2);
    }

    #[test]
    fn test_get_linker() {
        // Create a temporary config file
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[links.foo.a]
kind = "toml"
file_path = "test.toml"
key_path = "test.key"
[links.foo.b]
kind = "toml"
file_path = "test2.toml"
key_path = "test.key2"
"#
        )
        .unwrap();

        // Load the config
        let config = Config::load(file.path().to_str().unwrap()).unwrap();

        // Test getting an existing linker
        let linker = config.get_linker("foo");
        assert!(linker.is_ok());

        // Test getting a non-existent linker
        let linker = config.get_linker("bar");
        assert!(linker.is_err());
    }

    #[test]
    fn test_invalid_config() {
        // Create a temporary config file with invalid structure
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[links.invalid.a]
kind = "unknown"
file_path = "test.toml"
[links.invalid.b]
kind = "toml"
file_path = "test2.toml"
key_path = "test.key2"
"#
        )
        .unwrap();

        // Load the config - should fail due to unknown kind
        let config = Config::load(file.path().to_str().unwrap());
        assert!(config.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        // Create a temporary config file with missing required fields
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[links.missing.a]
kind = "toml"
# Missing file_path
key_path = "test.key"
[links.missing.b]
kind = "toml"
file_path = "test2.toml"
key_path = "test.key2"
"#
        )
        .unwrap();

        // Load the config - should fail due to missing file_path
        let config = Config::load(file.path().to_str().unwrap());
        assert!(config.is_err());
    }

    #[test]
    fn test_relative_path_resolution() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        // Create a temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().join("config");
        let data_dir = temp_dir.path().join("data");

        // Create the directories
        std::fs::create_dir(&config_dir).unwrap();
        std::fs::create_dir(&data_dir).unwrap();

        // Create a test file in the data directory
        let test_file_path = data_dir.join("test.toml");
        let mut test_file = File::create(&test_file_path).unwrap();
        writeln!(test_file, "[test]\nkey = \"test value\"").unwrap();

        // Create a config file in the config directory with a relative path to the test file
        let config_file_path = config_dir.join("config.toml");
        let mut config_file = File::create(&config_file_path).unwrap();

        // Use a relative path from the config directory to the data directory
        let relative_path = "../data/test.toml";
        write!(
            config_file,
            r#"
[links.test.a]
kind = "toml"
file_path = "{}"
key_path = "test.key"
[links.test.b]
kind = "toml"
file_path = "{}"
key_path = "test.key"
"#,
            relative_path, relative_path
        )
        .unwrap();

        // Load the config
        let config = Config::load(config_file_path.to_str().unwrap()).unwrap();

        // Get the linker
        let linker = config.get_linker("test").unwrap();

        // Verify that the file paths in the readers are resolved correctly
        if let Accessor::Toml(reader) = &linker.a {
            // The path should be resolved to the absolute path of the test file
            assert!(reader.file_path.contains("data/test.toml"));
        } else {
            panic!("Expected TomlReader");
        }
    }
}
