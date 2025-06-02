use crate::error::{AppError, Result};
use crate::readers::Reader;
use std::fs;

/// Reader for finding specific text in a file
#[derive(Clone, Debug)]
pub struct QueryReader {
    pub file_path: String,
    pub query: String,
}

impl Reader for QueryReader {
    fn read(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path).map_err(|e| AppError::FileOperation {
            path: self.file_path.clone(),
            source: e,
        })?;

        if content.contains(&self.query) {
            Ok(self.query.clone())
        } else {
            Err(AppError::QueryNotFound {
                query: self.query.clone(),
                file_path: self.file_path.clone(),
            })
        }
    }
}
