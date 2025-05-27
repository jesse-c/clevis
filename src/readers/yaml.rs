use crate::readers::Reader;
use saphyr::LoadableYamlNode;
use std::fs;

/// Reader for YAML files
#[derive(Clone, Debug)]
pub struct YamlReader {
    pub file_path: String,
    pub key_path: String,
}

impl Reader for YamlReader {
    fn read(&self) -> String {
        // Read the YAML file content
        let content = fs::read_to_string(&self.file_path).unwrap_or_else(|e| {
            panic!("Failed to read YAML file {}: {}", self.file_path, e);
        });

        // Ensure the content has the YAML document marker
        let content_with_marker = if !content.trim_start().starts_with("---") {
            format!("---\n{}", content)
        } else {
            content
        };

        // Parse the YAML content
        let docs = match saphyr::Yaml::load_from_str(&content_with_marker) {
            Ok(docs) => docs,
            Err(e) => panic!("Failed to parse YAML content: {}", e),
        };

        // Get the first document
        if docs.is_empty() {
            panic!("No YAML documents found in the file: {}", self.file_path);
        }

        let doc = &docs[0]; // Get the first document

        // Split the key path by dots
        let path_parts: Vec<&str> = self.key_path.split('.').collect();

        // Navigate through the YAML structure to find the value
        let mut current_value = doc;
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
                    let array_value = &current_value[key_name];

                    // Check if it's an array
                    if let Some(arr) = array_value.as_sequence() {
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
                    // Malformed array index notation
                    panic!("Malformed array index notation: {}", part);
                }
            } else {
                // Regular key access
                current_value = &current_value[part];

                // Check if the key exists
                if current_value.is_badvalue() {
                    panic!("Key not found: {}", current_path);
                }
            }
        }

        // Convert the found value to a string
        if let Some(s) = current_value.as_str() {
            s.to_string()
        } else if let Some(i) = current_value.as_integer() {
            i.to_string()
        } else if let Some(f) = current_value.as_floating_point() {
            f.to_string()
        } else if let Some(b) = current_value.as_bool() {
            b.to_string()
        } else if current_value.is_sequence() {
            panic!("Array access requires a specific index, e.g. 'key[0]'")
        } else if current_value.is_mapping() {
            "[Object]".to_string()
        } else if current_value.is_null() {
            "null".to_string()
        } else {
            format!("{:?}", current_value)
        }
    }
}
