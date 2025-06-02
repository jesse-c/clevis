use crate::readers::Reader;
use anyhow::{Context, Result};
use std::fs;

/// Reader for finding specific text in a file
#[derive(Clone, Debug)]
pub struct QueryReader {
    pub file_path: String,
    pub query: String,
}

impl Reader for QueryReader {
    fn read(&self) -> Result<String> {
        let content = fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read file: {}", self.file_path))?;

        if content.contains(&self.query) {
            Ok(self.query.clone())
        } else {
            anyhow::bail!("Query '{}' not found in file: {}", self.query, self.file_path)
        }
    }
}
