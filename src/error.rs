use std::fmt;

#[derive(Debug)]
pub enum AppError {
    FileOperation {
        path: String,
        source: std::io::Error,
    },
    Parse {
        file_type: String,
        path: String,
        source: anyhow::Error,
    },
    KeyNotFound {
        key_path: String,
        file_path: String,
    },
    QueryNotFound {
        query: String,
        file_path: String,
    },
    ConfigError {
        message: String,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::FileOperation { path, source } => {
                write!(f, "File operation failed: {} ({})", path, source)
            }
            AppError::Parse {
                file_type,
                path,
                source,
            } => {
                write!(f, "Parse error in {}: {} ({})", file_type, path, source)
            }
            AppError::KeyNotFound {
                key_path,
                file_path,
            } => {
                write!(f, "Key '{}' not found in {}", key_path, file_path)
            }
            AppError::QueryNotFound { query, file_path } => {
                write!(f, "Query '{}' not found in {}", query, file_path)
            }
            AppError::ConfigError { message } => {
                write!(f, "Configuration error: {}", message)
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::FileOperation { source, .. } => Some(source),
            AppError::Parse { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::ConfigError {
            message: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
