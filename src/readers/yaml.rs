use crate::error::{AppError, Result};
use crate::readers::Reader;
use anyhow::Context;
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
        let content = fs::read_to_string(&self.file_path).map_err(|e| AppError::FileOperation {
            path: self.file_path.clone(),
            source: e,
        })?;

        let content_with_marker = if !content.trim_start().starts_with("---") {
            format!("---\n{content}")
        } else {
            content
        };

        let docs =
            saphyr::Yaml::load_from_str(&content_with_marker).map_err(|e| AppError::Parse {
                file_type: "YAML".to_string(),
                path: self.file_path.clone(),
                source: anyhow::Error::from(e),
            })?;

        if docs.is_empty() {
            return Err(AppError::Parse {
                file_type: "YAML".to_string(),
                path: self.file_path.clone(),
                source: anyhow::Error::msg("No YAML documents found"),
            });
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
                        let index = index_str.parse::<usize>().with_context(|| {
                            format!(
                                "Invalid array index '{index_str}' in YAML file: {}",
                                self.file_path
                            )
                        })?;
                        if index < arr.len() {
                            current_value = &arr[index];
                        } else {
                            return Err(AppError::KeyNotFound {
                                key_path: format!("{key_name}[{index}]"),
                                file_path: self.file_path.clone(),
                            });
                        }
                    } else {
                        return Err(AppError::KeyNotFound {
                            key_path: key_name.to_string(),
                            file_path: self.file_path.clone(),
                        });
                    }
                } else {
                    return Err(AppError::Parse {
                        file_type: "YAML".to_string(),
                        path: self.file_path.clone(),
                        source: anyhow::Error::msg(format!(
                            "Malformed array index notation '{part}'"
                        )),
                    });
                }
            } else {
                current_value = &current_value[part];
                if current_value.is_badvalue() {
                    return Err(AppError::KeyNotFound {
                        key_path: current_path.clone(),
                        file_path: self.file_path.clone(),
                    });
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
            Err(AppError::KeyNotFound {
                key_path: format!("{} (array requires index)", self.key_path),
                file_path: self.file_path.clone(),
            })
        } else if current_value.is_mapping() {
            Ok("[Object]".to_string())
        } else if current_value.is_null() {
            Ok("null".to_string())
        } else {
            Ok(format!("{current_value:?}"))
        }
    }
}
