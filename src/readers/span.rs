use crate::readers::Reader;

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
    fn read(&self) -> String {
        use std::fs;

        // Load the entire file into memory and split into lines
        let content = fs::read_to_string(&self.file_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();

        // Check for out-of-bounds or invalid cursor positions
        if self.start.line == 0
            || self.end.line == 0
            || self.start.line > lines.len()
            || self.end.line > lines.len()
            || self.start.line > self.end.line
        {
            panic!(
                "out-of-bounds start and end lines: start={}, end={}, total_lines={}",
                self.start.line,
                self.end.line,
                lines.len()
            );
        }

        // Single line span
        if self.start.line == self.end.line {
            let line = lines[self.start.line - 1];

            // Check column bounds
            if self.start.column > line.len() || self.end.column > line.len() {
                panic!(
                    "out-of-bounds start and end columns: start={}, end={}, line_length={}",
                    self.start.column,
                    self.end.column,
                    line.len()
                );
            }

            // Extract the span from the single line
            return line[self.start.column - 1..self.end.column].to_string();
        }

        // Multi-line span
        let mut result = String::new();

        // First line (from start column to end of line)
        let first_line = lines[self.start.line - 1];
        if self.start.column <= first_line.len() {
            result.push_str(&first_line[self.start.column - 1..]);
        }
        result.push('\n');

        // Middle lines (entire lines)
        for line_num in self.start.line..self.end.line - 1 {
            result.push_str(lines[line_num]);
            result.push('\n');
        }

        // Last line (from start of line to end column)
        let last_line = lines[self.end.line - 1];
        if self.end.column <= last_line.len() {
            result.push_str(&last_line[..self.end.column]);
        } else {
            result.push_str(last_line);
        }

        result
    }
}
