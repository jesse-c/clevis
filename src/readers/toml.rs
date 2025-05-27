use crate::readers::Reader;
use std::fs;
use toml::Value;

/// Reader for TOML files
#[derive(Clone, Debug)]
pub struct TomlReader {
    pub file_path: String,
    pub key_path: String,
}

impl Reader for TomlReader {
    fn read(&self) -> String {
        // Read the TOML file content
        let content = fs::read_to_string(&self.file_path).unwrap_or_else(|e| {
            panic!("Failed to read TOML file {}: {}", self.file_path, e);
        });

        // Parse the TOML content
        let parsed: Value = content.parse().unwrap_or_else(|e| {
            panic!("Failed to parse TOML content: {}", e);
        });

        // Split the key path by dots
        let path_parts: Vec<&str> = self.key_path.split('.').collect();

        // Navigate through the TOML structure to find the value
        let mut current_value = &parsed;
        let mut current_path = String::new();

        for part in path_parts {
            // Update the current path for better error messages
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(part);

            // Check if this part has an array index notation like "key[0]"
            if let Some(bracket_pos) = part.find('[') {
                if part.ends_with(']') {
                    let key_name = &part[0..bracket_pos];
                    let index_str = &part[bracket_pos + 1..part.len() - 1];

                    // Get the array value using the key name
                    if let Some(table) = current_value.as_table() {
                        if let Some(array_value) = table.get(key_name) {
                            // Check if it's an array
                            if let Some(arr) = array_value.as_array() {
                                // Parse the index
                                match index_str.parse::<usize>() {
                                    Ok(index) => {
                                        if index < arr.len() {
                                            current_value = &arr[index];
                                        } else {
                                            panic!(
                                                "Array index out of bounds: {} has length {} but index is {}",
                                                key_name,
                                                arr.len(),
                                                index
                                            );
                                        }
                                    }
                                    Err(_) => panic!("Invalid array index: {}", index_str),
                                }
                            } else {
                                panic!("Value at '{}' is not an array", key_name);
                            }
                        } else {
                            panic!("Key not found: {}", key_name);
                        }
                    } else {
                        panic!(
                            "Expected a table at '{}', found something else",
                            current_path
                        );
                    }
                } else {
                    // Malformed array index notation
                    panic!("Malformed array index notation: {}", part);
                }
            } else {
                // Regular key access
                if let Some(table) = current_value.as_table() {
                    if let Some(value) = table.get(part) {
                        current_value = value;
                    } else {
                        panic!("Key not found: {}", current_path);
                    }
                } else {
                    panic!(
                        "Expected a table at '{}', found something else",
                        current_path
                    );
                }
            }
        }

        // Convert the found value to a string
        match current_value {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Datetime(dt) => dt.to_string(),
            Value::Array(_) => panic!("Array access requires a specific index, e.g. 'key[0]'"),
            Value::Table(_) => panic!("Table access requires a specific key"),
        }
    }
}
