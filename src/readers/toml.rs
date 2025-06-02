use crate::readers::Reader;
use anyhow::{Context, Result};
use std::fs;
use toml::Value;

/// Reader for TOML files
#[derive(Clone, Debug)]
pub struct TomlReader {
    pub file_path: String,
    pub key_path: String,
}

impl Reader for TomlReader {
    fn read(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read TOML file: {}", self.file_path))?;

        let parsed: Value = content.parse()
            .with_context(|| format!("Failed to parse TOML content in: {}", self.file_path))?;
        let path_parts: Vec<&str> = self.key_path.split('.').collect();

        let mut current_value = &parsed;
        let mut current_path = String::new();

        for part in path_parts {
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(part);

            if let Some(bracket_pos) = part.find('[') {
                if part.ends_with(']') {
                    let key_name = &part[0..bracket_pos];
                    let index_str = &part[bracket_pos + 1..part.len() - 1];

                    if let Some(table) = current_value.as_table() {
                        if let Some(array_value) = table.get(key_name) {
                            if let Some(arr) = array_value.as_array() {
                                let index = index_str
                                    .parse::<usize>()
                                    .with_context(|| format!("Invalid array index '{}' in TOML file: {}", index_str, self.file_path))?;
                                if index < arr.len() {
                                    current_value = &arr[index];
                                } else {
                                    anyhow::bail!(
                                        "Array index out of bounds in {}: key '{}' has length {} but index is {}",
                                        self.file_path,
                                        key_name,
                                        arr.len(),
                                        index
                                    );
                                }
                            } else {
                                anyhow::bail!("Value at '{}' is not an array in TOML file: {}", key_name, self.file_path);
                            }
                        } else {
                            anyhow::bail!("Key '{}' not found in TOML file: {}", key_name, self.file_path);
                        }
                    } else {
                        anyhow::bail!("Expected a table at '{}', found something else in TOML file: {}", current_path, self.file_path);
                    }
                } else {
                    anyhow::bail!("Malformed array index notation '{}' in TOML file: {}", part, self.file_path);
                }
            } else {
                if let Some(table) = current_value.as_table() {
                    if let Some(value) = table.get(part) {
                        current_value = value;
                    } else {
                        anyhow::bail!("Key '{}' not found in TOML file: {}", current_path, self.file_path);
                    }
                } else {
                    anyhow::bail!("Expected a table at '{}', found something else in TOML file: {}", current_path, self.file_path);
                }
            }
        }

        match current_value {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Datetime(dt) => Ok(dt.to_string()),
            Value::Array(_) => anyhow::bail!("Array access requires a specific index, e.g. 'key[0]' in TOML file: {}", self.file_path),
            Value::Table(_) => anyhow::bail!("Table access requires a specific key in TOML file: {}", self.file_path),
        }
    }
}
