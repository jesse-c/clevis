use crate::readers::Reader;
use anyhow::{Context, Result};
use saphyr::LoadableYamlNode;
use std::fs;

/// Reader for YAML files
#[derive(Clone, Debug)]
pub struct YamlReader {
    pub file_path: String,
    pub key_path: String,
}

impl Reader for YamlReader {
    fn read(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read YAML file: {}", self.file_path))?;

        let content_with_marker = if !content.trim_start().starts_with("---") {
            format!("---\n{}", content)
        } else {
            content
        };

        let docs = saphyr::Yaml::load_from_str(&content_with_marker)
            .with_context(|| format!("Failed to parse YAML content in: {}", self.file_path))?;

        if docs.is_empty() {
            anyhow::bail!("No YAML documents found in file: {}", self.file_path);
        }

        let doc = &docs[0];
        let path_parts: Vec<&str> = self.key_path.split('.').collect();

        let mut current_value = doc;
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

                    let array_value = &current_value[key_name];
                    if let Some(arr) = array_value.as_sequence() {
                        let index = index_str
                            .parse::<usize>()
                            .with_context(|| format!("Invalid array index '{}' in YAML file: {}", index_str, self.file_path))?;
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
                        anyhow::bail!("Value at '{}' is not an array in YAML file: {}", key_name, self.file_path);
                    }
                } else {
                    anyhow::bail!("Malformed array index notation '{}' in YAML file: {}", part, self.file_path);
                }
            } else {
                current_value = &current_value[part];
                if current_value.is_badvalue() {
                    anyhow::bail!("Key '{}' not found in YAML file: {}", current_path, self.file_path);
                }
            }
        }

        if let Some(s) = current_value.as_str() {
            Ok(s.to_string())
        } else if let Some(i) = current_value.as_integer() {
            Ok(i.to_string())
        } else if let Some(f) = current_value.as_floating_point() {
            Ok(f.to_string())
        } else if let Some(b) = current_value.as_bool() {
            Ok(b.to_string())
        } else if current_value.is_sequence() {
            anyhow::bail!("Array access requires a specific index, e.g. 'key[0]' in YAML file: {}", self.file_path)
        } else if current_value.is_mapping() {
            Ok("[Object]".to_string())
        } else if current_value.is_null() {
            Ok("null".to_string())
        } else {
            Ok(format!("{:?}", current_value))
        }
    }
}
