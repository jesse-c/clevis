use crate::readers::Reader;
use std::fs;

/// Reader for finding specific text in a file
#[derive(Clone, Debug)]
pub struct QueryReader {
    pub file_path: String,
    pub query: String,
}

impl Reader for QueryReader {
    fn read(&self) -> String {
        // Read the file content
        let content = fs::read_to_string(&self.file_path).unwrap_or_else(|e| {
            panic!("Failed to read file {}: {}", self.file_path, e);
        });

        // Search for the query string
        if content.contains(&self.query) {
            // Return the query string itself
            self.query.clone()
        } else {
            panic!(
                "Query '{}' not found in file {}",
                self.query, self.file_path
            );
        }
    }
}
