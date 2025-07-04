use crate::error::{AppError, Result};
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
    fn read(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path).map_err(|e| AppError::FileOperation {
            path: self.file_path.clone(),
            source: e,
        })?;

        let parsed: Value = content.parse().map_err(|e| AppError::Parse {
            file_type: "TOML".to_string(),
            path: self.file_path.clone(),
            source: anyhow::Error::from(e),
        })?;
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
                                let index =
                                    index_str.parse::<usize>().map_err(|e| AppError::Parse {
                                        file_type: "TOML".to_string(),
                                        path: self.file_path.clone(),
                                        source: anyhow::Error::from(e),
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
                            return Err(AppError::KeyNotFound {
                                key_path: key_name.to_string(),
                                file_path: self.file_path.clone(),
                            });
                        }
                    } else {
                        return Err(AppError::KeyNotFound {
                            key_path: current_path.clone(),
                            file_path: self.file_path.clone(),
                        });
                    }
                } else {
                    return Err(AppError::Parse {
                        file_type: "TOML".to_string(),
                        path: self.file_path.clone(),
                        source: anyhow::Error::msg(format!(
                            "Malformed array index notation '{part}'"
                        )),
                    });
                }
            } else if let Some(table) = current_value.as_table() {
                if let Some(value) = table.get(part) {
                    current_value = value;
                } else {
                    return Err(AppError::KeyNotFound {
                        key_path: current_path.clone(),
                        file_path: self.file_path.clone(),
                    });
                }
            } else {
                return Err(AppError::KeyNotFound {
                    key_path: current_path.clone(),
                    file_path: self.file_path.clone(),
                });
            }
        }

        match current_value {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Datetime(dt) => Ok(dt.to_string()),
            Value::Array(_) => Err(AppError::KeyNotFound {
                key_path: format!("{} (array requires index)", self.key_path),
                file_path: self.file_path.clone(),
            }),
            Value::Table(_) => Err(AppError::KeyNotFound {
                key_path: format!("{} (table requires key)", self.key_path),
                file_path: self.file_path.clone(),
            }),
        }
    }
}
