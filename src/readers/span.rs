use crate::readers::Reader;
use anyhow::{Context, Result};

/// Represents a cursor position in a file
#[derive(Clone, Debug)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

/// Reader for specific spans of text in a file
#[derive(Clone, Debug)]
pub struct SpanReader {
    pub file_path: String,
    pub start: Cursor,
    pub end: Cursor,
}

impl Reader for SpanReader {
    fn read(&self) -> Result<String> {
        let content = std::fs::read_to_string(&self.file_path)
            .with_context(|| format!("Failed to read file: {}", self.file_path))?;
        let lines: Vec<&str> = content.lines().collect();

        if self.start.line == 0
            || self.end.line == 0
            || self.start.line > lines.len()
            || self.end.line > lines.len()
            || self.start.line > self.end.line
        {
            anyhow::bail!(
                "Invalid span in {}: start line {}, end line {}, total lines {}",
                self.file_path,
                self.start.line,
                self.end.line,
                lines.len()
            );
        }

        if self.start.line == self.end.line {
            let line = lines[self.start.line - 1];
            if self.start.column > line.len() || self.end.column > line.len() {
                anyhow::bail!(
                    "Invalid column span in {} at line {}: start column {}, end column {}, line length {}",
                    self.file_path,
                    self.start.line,
                    self.start.column,
                    self.end.column,
                    line.len()
                );
            }
            return Ok(line[self.start.column - 1..self.end.column].to_string());
        }

        let mut result = String::new();
        let first_line = lines[self.start.line - 1];
        if self.start.column <= first_line.len() {
            result.push_str(&first_line[self.start.column - 1..]);
        }
        result.push('\n');

        for line in lines
            .iter()
            .skip(self.start.line)
            .take(self.end.line - self.start.line - 1)
        {
            result.push_str(line);
            result.push('\n');
        }

        let last_line = lines[self.end.line - 1];
        if self.end.column <= last_line.len() {
            result.push_str(&last_line[..self.end.column]);
        } else {
            result.push_str(last_line);
        }

        Ok(result)
    }
}
