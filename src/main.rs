trait Reader {
    fn read(&self) -> String;
}

struct Cursor {
    line: usize,
    column: usize,
}

struct SpanReader {
    file_path: String,
    start: Cursor,
    end: Cursor,
}

impl Reader for SpanReader {
    fn read(&self) -> String {
        use std::fs;

        // Load the entire file into memory and split into lines
        let content = fs::read_to_string(&self.file_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();

        // Validate that the requested span is within file bounds (1-based line numbers)
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
            )
        }

        let mut result = String::new();

        // Walk through each line in the span range (inclusive)
        for line_num in self.start.line..=self.end.line {
            let line = lines[line_num - 1]; // Convert from 1-based to 0-based indexing

            // Determine the column range to extract from this line:
            //
            // - First line: start from the specified start_column
            // - Middle line(s): start from column 0 (beginning of line) and end at the end of the line
            // - Last line: start from column 0 (beginning of line) end at the specified end_column

            let start_col = if line_num == self.start.line {
                self.start.column.saturating_sub(1) // Convert from 1-based to 0-based
            } else {
                0 // Start of line for continuation lines
            };
            let end_col = if line_num == self.end.line {
                std::cmp::min(self.end.column, line.len()) // Don't exceed line length
            } else {
                line.len() // Full line for continuation lines
            };

            // Extract the substring from this line if there's content to extract
            if start_col < line.len() {
                let slice = &line[start_col..end_col];
                result.push_str(slice);

                // Add newline character between lines (but not after the last line)
                if line_num < self.end.line {
                    result.push('\n');
                }
            }
        }

        result
    }
}

struct TomlReader {
    file_path: String,
    key_path: String,
}

#[derive(Clone)]
struct YamlReader {
    file_path: String,
    key_path: String,
}

impl Reader for YamlReader {
    fn read(&self) -> String {
        use saphyr::LoadableYamlNode;
        use std::fs;

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

impl Reader for TomlReader {
    fn read(&self) -> String {
        use std::fs;
        use toml::Value;

        // Read the TOML file content
        let content = fs::read_to_string(&self.file_path).unwrap_or_default();

        // Parse the TOML content
        let parsed: Value = content
            .parse()
            .unwrap_or(Value::Table(toml::map::Map::new()));

        // Split the key path by dots
        let path_parts: Vec<&str> = self.key_path.split('.').collect();

        // Navigate through the TOML structure to find the value
        let mut current_value = &parsed;
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
                    match current_value.get(key_name) {
                        Some(array_value) => {
                            if let Value::Array(arr) = array_value {
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
                        }
                        None => panic!("Key not found: {}", key_name),
                    }
                } else {
                    // Malformed array index notation
                    panic!("Malformed array index notation: {}", part);
                }
            } else {
                // Regular key access
                match current_value.get(part) {
                    Some(value) => current_value = value,
                    None => panic!("Key not found: {}", current_path),
                }
            }
        }

        // Convert the found value to a string
        match current_value {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Datetime(dt) => dt.to_string(),
            Value::Array(_) => panic!("Array access requires a specific index, e.g. 'key[0]'"),
            Value::Table(_) => "[Object]".to_string(),
        }
    }
}

struct QueryReader {
    file_path: String,
    query: String,
}

impl Reader for QueryReader {
    fn read(&self) -> String {
        use std::fs;

        // Read the file content
        let content = fs::read_to_string(&self.file_path).unwrap_or_else(|e| {
            panic!("Failed to read file {}: {}", self.file_path, e);
        });

        // Search for the query string
        if let Some(_) = content.find(&self.query) {
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

enum Accessor {
    Spans(SpanReader),
    Toml(TomlReader),
    Yaml(YamlReader),
    Query(QueryReader),
}

impl Accessor {
    fn read(&self) -> String {
        match self {
            Accessor::Spans(reader) => reader.read(),
            Accessor::Toml(reader) => reader.read(),
            Accessor::Yaml(reader) => reader.read(),
            Accessor::Query(reader) => reader.read(),
        }
    }
}

struct Linker {
    a: Accessor,
    b: Accessor,
}

impl Linker {
    fn check(&self) -> bool {
        self.a.read() == self.b.read()
    }
}

fn main() {
    println!("Hello, world!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_reader() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary file
        let file = NamedTempFile::new().unwrap();
        let file_path = file.path().to_str().unwrap().to_string();

        // Write some content to the file
        let content = "This is a test file with some specific content to search for.\nIt has multiple lines and the needle we're looking for is somewhere in here.\nThe needle is 'specific content' which should be found by our query reader.";
        fs::write(&file_path, content).unwrap();

        // Test finding a query that exists
        let query_reader = QueryReader {
            file_path: file_path.clone(),
            query: "specific content".to_string(),
        };

        assert_eq!(query_reader.read(), "specific content");

        // Test that a non-existent query causes a panic
        let non_existent_query = QueryReader {
            file_path: file_path.clone(),
            query: "this text doesn't exist".to_string(),
        };

        let result = std::panic::catch_unwind(|| non_existent_query.read());
        assert!(result.is_err(), "Expected panic when query is not found");

        // Test with a query at the beginning of the file
        let beginning_query = QueryReader {
            file_path: file_path.clone(),
            query: "This is".to_string(),
        };

        assert_eq!(beginning_query.read(), "This is");

        // Test with a query at the end of the file
        let end_query = QueryReader {
            file_path,
            query: "query reader".to_string(),
        };

        assert_eq!(end_query.read(), "query reader");
    }
    use std::fs;

    #[test]
    fn test_span_reader_multi_line() {
        let test_content = "line 1\nline 2\nline 3\nline 4";
        fs::write("test_multi.txt", test_content).unwrap();

        let span = SpanReader {
            file_path: "test_multi.txt".to_string(),
            start: Cursor { line: 2, column: 3 },
            end: Cursor { line: 3, column: 4 },
        };
        let result = span.read();

        assert_eq!(result, "ne 2\nline");

        fs::remove_file("test_multi.txt").unwrap();
    }

    #[test]
    fn test_span_reader_single_line() {
        let test_content = "line 1\nline 2\nline 3\nline 4";
        fs::write("test_single.txt", test_content).unwrap();

        let span = SpanReader {
            file_path: "test_single.txt".to_string(),
            start: Cursor { line: 2, column: 3 },
            end: Cursor { line: 2, column: 6 },
        };
        let result = span.read();

        assert_eq!(result, "ne 2");

        fs::remove_file("test_single.txt").unwrap();
    }

    #[test]
    fn test_linker_check_same_content() {
        let test_content = "line 1\nline 2\nline 3\nline 4";
        fs::write("test_linker_same.txt", test_content).unwrap();

        // Create two accessors that read the same content
        let span_a = SpanReader {
            file_path: "test_linker_same.txt".to_string(),
            start: Cursor { line: 2, column: 1 },
            end: Cursor { line: 2, column: 6 },
        };

        let span_b = SpanReader {
            file_path: "test_linker_same.txt".to_string(),
            start: Cursor { line: 2, column: 1 },
            end: Cursor { line: 2, column: 6 },
        };

        let linker = Linker {
            a: Accessor::Spans(span_a),
            b: Accessor::Spans(span_b),
        };

        assert!(linker.check());

        fs::remove_file("test_linker_same.txt").unwrap();
    }

    #[test]
    fn test_linker_check_different_content() {
        let test_content = "line 1\nline 2\nline 3\nline 4";
        fs::write("test_linker_diff.txt", test_content).unwrap();

        // Create two accessors that read different content
        let span_a = SpanReader {
            file_path: "test_linker_diff.txt".to_string(),
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 6 },
        };

        let span_b = SpanReader {
            file_path: "test_linker_diff.txt".to_string(),
            start: Cursor { line: 2, column: 1 },
            end: Cursor { line: 2, column: 6 },
        };

        let linker = Linker {
            a: Accessor::Spans(span_a),
            b: Accessor::Spans(span_b),
        };

        assert!(!linker.check());

        fs::remove_file("test_linker_diff.txt").unwrap();
    }

    #[test]
    fn test_linker_check_different_files() {
        // Create two different files with the same content in one section
        let test_content_a = "file A line 1\nfile A line 2\nfile A line 3";
        let test_content_b = "file B line 1\nfile A line 2\nfile B line 3";

        fs::write("test_linker_file_a.txt", test_content_a).unwrap();
        fs::write("test_linker_file_b.txt", test_content_b).unwrap();

        // Test case 1: Different files with same content in selected span
        let span_a = SpanReader {
            file_path: "test_linker_file_a.txt".to_string(),
            start: Cursor { line: 2, column: 1 },
            end: Cursor {
                line: 2,
                column: 12,
            },
        };

        let span_b = SpanReader {
            file_path: "test_linker_file_b.txt".to_string(),
            start: Cursor { line: 2, column: 1 },
            end: Cursor {
                line: 2,
                column: 12,
            },
        };

        let linker_same = Linker {
            a: Accessor::Spans(span_a),
            b: Accessor::Spans(span_b),
        };

        assert!(linker_same.check());

        // Test case 2: Different files with different content in selected span
        let span_c = SpanReader {
            file_path: "test_linker_file_a.txt".to_string(),
            start: Cursor { line: 1, column: 1 },
            end: Cursor {
                line: 1,
                column: 12,
            },
        };

        let span_d = SpanReader {
            file_path: "test_linker_file_b.txt".to_string(),
            start: Cursor { line: 1, column: 1 },
            end: Cursor {
                line: 1,
                column: 12,
            },
        };

        let linker_different = Linker {
            a: Accessor::Spans(span_c),
            b: Accessor::Spans(span_d),
        };

        assert!(!linker_different.check());

        // Clean up
        fs::remove_file("test_linker_file_a.txt").unwrap();
        fs::remove_file("test_linker_file_b.txt").unwrap();
    }

    #[test]
    fn test_toml_reader() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary TOML file
        let toml_file = NamedTempFile::new().unwrap();
        let toml_path = toml_file.path().to_str().unwrap().to_string();

        // Create test TOML content
        let test_toml = r#"[section]
key = "value"
number = 42
flag = true

[nested]
deep.key = "nested value"

[arrays]
numbers = [1, 2, 3, 4, 5]
"#;

        fs::write(&toml_path, test_toml).unwrap();

        // Test reading string value
        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "section.key".to_string(),
        };
        assert_eq!(reader.read(), "value");

        // Test reading numeric value
        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "section.number".to_string(),
        };
        assert_eq!(reader.read(), "42");

        // Test reading boolean value
        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "section.flag".to_string(),
        };
        assert_eq!(reader.read(), "true");

        // Test reading nested value
        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "nested.deep.key".to_string(),
        };
        assert_eq!(reader.read(), "nested value");

        // Test reading array with index
        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "arrays.numbers[0]".to_string(),
        };
        assert_eq!(reader.read(), "1");

        let reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "arrays.numbers[4]".to_string(),
        };
        assert_eq!(reader.read(), "5");

        // Test that accessing an array without an index causes a panic
        let array_reader = TomlReader {
            file_path: toml_path,
            key_path: "arrays.numbers".to_string(),
        };
        let result = std::panic::catch_unwind(|| array_reader.read());
        assert!(
            result.is_err(),
            "Expected panic when accessing array without index"
        );

        // Temporary file will be automatically cleaned up when it goes out of scope
    }

    #[test]
    fn test_linker_with_toml_reader() {
        // Create two test TOML files with some matching and some different values
        let test_toml_a = r#"
            [server]
            host = "localhost"
            port = 8080

            [user]
            name = "John Doe"
            id = 12345
        "#;

        let test_toml_b = r#"
            [server]
            address = "localhost"
            port = 8080

            [profile]
            username = "John Doe"
            user_id = 67890
        "#;

        fs::write("test_toml_a.toml", test_toml_a).unwrap();
        fs::write("test_toml_b.toml", test_toml_b).unwrap();

        // Test 1: Compare values that are the same across different files and paths
        let toml_reader_a = TomlReader {
            file_path: "test_toml_a.toml".to_string(),
            key_path: "server.host".to_string(),
        };

        let toml_reader_b = TomlReader {
            file_path: "test_toml_b.toml".to_string(),
            key_path: "server.address".to_string(),
        };

        let linker_same = Linker {
            a: Accessor::Toml(toml_reader_a),
            b: Accessor::Toml(toml_reader_b),
        };

        assert!(linker_same.check());

        // Test 2: Compare values that are different
        let toml_reader_c = TomlReader {
            file_path: "test_toml_a.toml".to_string(),
            key_path: "user.id".to_string(),
        };

        let toml_reader_d = TomlReader {
            file_path: "test_toml_b.toml".to_string(),
            key_path: "profile.user_id".to_string(),
        };

        let linker_different = Linker {
            a: Accessor::Toml(toml_reader_c),
            b: Accessor::Toml(toml_reader_d),
        };

        assert!(!linker_different.check());

        // Clean up
        fs::remove_file("test_toml_a.toml").unwrap();
        fs::remove_file("test_toml_b.toml").unwrap();
    }

    #[test]
    fn test_yaml_reader() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create a temporary YAML file with proper indentation and YAML 1.2 format
        let yaml_file = NamedTempFile::new().unwrap();
        let yaml_path = yaml_file.path().to_str().unwrap().to_string();

        let test_yaml = "---\nserver:\n  host: localhost\n  port: 8080\nuser:\n  name: John Doe\n  active: true\n  scores:\n    - 10\n    - 20\n    - 30\n  personal:\n    email: john@example.com\n    contact:\n      phone: +1-555-555-5555\narrays:\n  strings:\n    - first\n    - second\n    - third\n  mixed:\n    - 42\n    - true\n    - text\n";

        fs::write(&yaml_path, test_yaml).unwrap();

        // Test reading string value
        let string_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.host".to_string(),
        };
        assert_eq!(string_reader.read(), "localhost");

        // Test reading numeric value
        let int_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.port".to_string(),
        };
        assert_eq!(int_reader.read(), "8080");

        // Test reading boolean value
        let bool_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.active".to_string(),
        };
        assert_eq!(bool_reader.read(), "true");

        // Test reading nested value
        let nested_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.personal.email".to_string(),
        };
        assert_eq!(nested_reader.read(), "john@example.com");

        // Test reading deeply nested value
        let deeply_nested_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.personal.contact.phone".to_string(),
        };
        assert_eq!(deeply_nested_reader.read(), "+1-555-555-5555");

        // Test array index access
        let array_index_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.scores[1]".to_string(),
        };
        assert_eq!(array_index_reader.read(), "20");

        // Test array of strings index access
        let string_array_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "arrays.strings[2]".to_string(),
        };
        assert_eq!(string_array_reader.read(), "third");

        // Test mixed array index access
        let mixed_array_reader_int = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "arrays.mixed[0]".to_string(),
        };
        assert_eq!(mixed_array_reader_int.read(), "42");

        let mixed_array_reader_bool = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "arrays.mixed[1]".to_string(),
        };
        assert_eq!(mixed_array_reader_bool.read(), "true");

        let mixed_array_reader_string = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "arrays.mixed[2]".to_string(),
        };
        assert_eq!(mixed_array_reader_string.read(), "text");

        // Test that accessing an array without an index causes a panic
        let array_reader = YamlReader {
            file_path: yaml_path,
            key_path: "user.scores".to_string(),
        };
        let result = std::panic::catch_unwind(|| array_reader.read());
        assert!(
            result.is_err(),
            "Expected panic when accessing array without index"
        );

        // The temporary file will be automatically cleaned up when it goes out of scope
    }

    #[test]
    fn test_linker_with_yaml() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files
        let yaml_file = NamedTempFile::new().unwrap();
        let toml_file = NamedTempFile::new().unwrap();

        let yaml_path = yaml_file.path().to_str().unwrap().to_string();
        let toml_path = toml_file.path().to_str().unwrap().to_string();

        // Create a YAML file and a TOML file with similar structure but different values
        let test_yaml = "---\nserver:\n  host: localhost\n  port: 8080\n\nuser:\n  id: 12345\n  name: John Doe\n";

        let test_toml = r#"[server]
host = "127.0.0.1"
port = 8080

[user]
id = 67890
name = "John Doe"
"#;

        fs::write(&yaml_path, test_yaml).unwrap();
        fs::write(&toml_path, test_toml).unwrap();

        // Test 1: Compare values that are the same across different formats
        let yaml_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.name".to_string(),
        };

        let toml_reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "user.name".to_string(),
        };

        let linker_same = Linker {
            a: Accessor::Yaml(yaml_reader),
            b: Accessor::Toml(toml_reader),
        };

        assert!(
            linker_same.check(),
            "Same values across formats should match"
        );

        // Test 2: Compare values that are different across formats
        let yaml_reader_diff = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.host".to_string(),
        };

        let toml_reader_diff = TomlReader {
            file_path: toml_path.clone(),
            key_path: "server.host".to_string(),
        };

        let linker_different = Linker {
            a: Accessor::Yaml(yaml_reader_diff),
            b: Accessor::Toml(toml_reader_diff),
        };

        assert!(
            !linker_different.check(),
            "Different values should not match"
        );

        // Test 3: Compare numeric values that are the same
        let yaml_reader_num = YamlReader {
            file_path: yaml_path,
            key_path: "server.port".to_string(),
        };

        let toml_reader_num = TomlReader {
            file_path: toml_path,
            key_path: "server.port".to_string(),
        };

        let linker_num = Linker {
            a: Accessor::Yaml(yaml_reader_num),
            b: Accessor::Toml(toml_reader_num),
        };

        assert!(linker_num.check(), "Same numeric values should match");

        // Temporary files will be automatically cleaned up when they go out of scope
    }

    #[test]
    fn test_linker_with_toml_and_span() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files
        let toml_file = NamedTempFile::new().unwrap();
        let string_file = NamedTempFile::new().unwrap();
        let number_file = NamedTempFile::new().unwrap();
        let boolean_file = NamedTempFile::new().unwrap();
        let array_file = NamedTempFile::new().unwrap();
        let nested_file = NamedTempFile::new().unwrap();
        let non_match_file = NamedTempFile::new().unwrap();

        let toml_path = toml_file.path().to_str().unwrap().to_string();
        let string_path = string_file.path().to_str().unwrap().to_string();
        let number_path = number_file.path().to_str().unwrap().to_string();
        let boolean_path = boolean_file.path().to_str().unwrap().to_string();
        let array_path = array_file.path().to_str().unwrap().to_string();
        let nested_path = nested_file.path().to_str().unwrap().to_string();
        let non_match_path = non_match_file.path().to_str().unwrap().to_string();

        // Create a TOML file with various data types
        let test_toml = r#"[config]
api_key = "abc123"
timeout = 30
debug = true

[nested]
level1.level2.value = "nested value"

[arrays]
numbers = [1, 2, 3, 4, 5]
"#;

        // Create text files with content that matches and doesn't match TOML values
        let matching_text = "abc123";
        let non_matching_text = "xyz789";
        let number_text = "30";
        let boolean_text = "true";
        let array_text = "3";
        let nested_text = "nested value";

        fs::write(&toml_path, test_toml).unwrap();
        fs::write(&string_path, matching_text).unwrap();
        fs::write(&non_match_path, non_matching_text).unwrap();
        fs::write(&number_path, number_text).unwrap();
        fs::write(&boolean_path, boolean_text).unwrap();
        fs::write(&array_path, array_text).unwrap();
        fs::write(&nested_path, nested_text).unwrap();

        // Test 1: Compare string value
        let toml_reader_string = TomlReader {
            file_path: toml_path.clone(),
            key_path: "config.api_key".to_string(),
        };

        let span_reader_string = SpanReader {
            file_path: string_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 6 },
        };

        let linker_string = Linker {
            a: Accessor::Toml(toml_reader_string),
            b: Accessor::Spans(span_reader_string),
        };

        assert!(
            linker_string.check(),
            "String value comparison should match"
        );

        // Test 2: Compare numeric value
        let toml_reader_number = TomlReader {
            file_path: toml_path.clone(),
            key_path: "config.timeout".to_string(),
        };

        let span_reader_number = SpanReader {
            file_path: number_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 2 },
        };

        let linker_number = Linker {
            a: Accessor::Toml(toml_reader_number),
            b: Accessor::Spans(span_reader_number),
        };

        assert!(
            linker_number.check(),
            "Numeric value comparison should match"
        );

        // Test 3: Compare boolean value
        let toml_reader_bool = TomlReader {
            file_path: toml_path.clone(),
            key_path: "config.debug".to_string(),
        };

        let span_reader_bool = SpanReader {
            file_path: boolean_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 4 },
        };

        let linker_bool = Linker {
            a: Accessor::Toml(toml_reader_bool),
            b: Accessor::Spans(span_reader_bool),
        };

        assert!(linker_bool.check(), "Boolean value comparison should match");

        // Test 4: Compare array value with index
        let toml_reader_array = TomlReader {
            file_path: toml_path.clone(),
            key_path: "arrays.numbers[2]".to_string(),
        };

        let span_reader_array = SpanReader {
            file_path: array_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 1 },
        };

        let linker_array = Linker {
            a: Accessor::Toml(toml_reader_array),
            b: Accessor::Spans(span_reader_array),
        };

        assert!(linker_array.check(), "Array value comparison should match");

        // Test 5: Compare nested value
        let toml_reader_nested = TomlReader {
            file_path: toml_path.clone(),
            key_path: "nested.level1.level2.value".to_string(),
        };

        let span_reader_nested = SpanReader {
            file_path: nested_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor {
                line: 1,
                column: 12,
            },
        };

        let linker_nested = Linker {
            a: Accessor::Toml(toml_reader_nested),
            b: Accessor::Spans(span_reader_nested),
        };

        assert!(
            linker_nested.check(),
            "Nested value comparison should match"
        );

        // Test 6: Compare non-matching values
        let toml_reader_non_match = TomlReader {
            file_path: toml_path,
            key_path: "config.api_key".to_string(),
        };

        let span_reader_non_match = SpanReader {
            file_path: non_match_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 6 },
        };

        let linker_non_match = Linker {
            a: Accessor::Toml(toml_reader_non_match),
            b: Accessor::Spans(span_reader_non_match),
        };

        assert!(
            !linker_non_match.check(),
            "Different values should not match"
        );

        // Temporary files will be automatically cleaned up when they go out of scope
    }

    #[test]
    fn test_linker_with_toml_and_yaml() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files
        let toml_file = NamedTempFile::new().unwrap();
        let yaml_file = NamedTempFile::new().unwrap();
        let yaml_diff_file = NamedTempFile::new().unwrap();

        let toml_path = toml_file.path().to_str().unwrap().to_string();
        let yaml_path = yaml_file.path().to_str().unwrap().to_string();
        let yaml_diff_path = yaml_diff_file.path().to_str().unwrap().to_string();

        // Create a test TOML file with proper formatting
        let test_toml = r#"[server]
host = "localhost"
port = 8080

[user]
name = "John Doe"
id = 12345
scores = [10, 20, 30]
"#;

        // Create a test YAML file with matching values and proper formatting
        // Include the YAML document marker (---) for saphyr compatibility
        let test_yaml = "---\nserver:\n  host: localhost\n  port: 8080\nuser:\n  name: John Doe\n  id: 12345\n  scores:\n    - 10\n    - 20\n    - 30\n";

        fs::write(&toml_path, test_toml).unwrap();
        fs::write(&yaml_path, test_yaml).unwrap();

        // Test 1: Compare string values
        let toml_reader_string = TomlReader {
            file_path: toml_path.clone(),
            key_path: "server.host".to_string(),
        };

        let yaml_reader_string = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.host".to_string(),
        };

        let linker_string = Linker {
            a: Accessor::Toml(toml_reader_string),
            b: Accessor::Yaml(yaml_reader_string),
        };

        assert!(
            linker_string.check(),
            "String value comparison should match"
        );

        // Test 2: Compare numeric values
        let toml_reader_number = TomlReader {
            file_path: toml_path.clone(),
            key_path: "server.port".to_string(),
        };

        let yaml_reader_number = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.port".to_string(),
        };

        let linker_number = Linker {
            a: Accessor::Toml(toml_reader_number),
            b: Accessor::Yaml(yaml_reader_number),
        };

        assert!(
            linker_number.check(),
            "Numeric value comparison should match"
        );

        // Test 3: Compare array values with index
        let toml_reader_array = TomlReader {
            file_path: toml_path.clone(),
            key_path: "user.scores[1]".to_string(),
        };

        let yaml_reader_array = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "user.scores[1]".to_string(),
        };

        let linker_array = Linker {
            a: Accessor::Toml(toml_reader_array),
            b: Accessor::Yaml(yaml_reader_array),
        };

        assert!(linker_array.check(), "Array value comparison should match");

        // Test 4: Create a YAML file with different values
        let test_yaml_different = "---\nserver:\n  host: different-host\n  port: 9090\n";

        fs::write(&yaml_diff_path, test_yaml_different).unwrap();

        let toml_reader_diff = TomlReader {
            file_path: toml_path,
            key_path: "server.host".to_string(),
        };

        let yaml_reader_diff = YamlReader {
            file_path: yaml_diff_path,
            key_path: "server.host".to_string(),
        };

        let linker_diff = Linker {
            a: Accessor::Toml(toml_reader_diff),
            b: Accessor::Yaml(yaml_reader_diff),
        };

        assert!(!linker_diff.check(), "Different values should not match");

        // Temporary files will be automatically cleaned up when they go out of scope
    }

    #[test]
    fn test_linker_with_span_and_yaml() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files
        let yaml_file = NamedTempFile::new().unwrap();
        let span_match_file = NamedTempFile::new().unwrap();
        let span_diff_file = NamedTempFile::new().unwrap();

        let yaml_path = yaml_file.path().to_str().unwrap().to_string();
        let span_match_path = span_match_file.path().to_str().unwrap().to_string();
        let span_diff_path = span_diff_file.path().to_str().unwrap().to_string();

        // Create a simple test for span and YAML comparison
        // Create a test YAML file with proper indentation and YAML document marker
        let test_yaml = "---\nserver:\n  host: localhost\n  port: 8080\n";

        fs::write(&yaml_path, test_yaml).unwrap();

        // Test 1: Create a span with matching content
        fs::write(&span_match_path, "localhost").unwrap();

        let span_reader = SpanReader {
            file_path: span_match_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 9 }, // "localhost" is 9 characters
        };

        let yaml_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "server.host".to_string(),
        };

        let linker = Linker {
            a: Accessor::Spans(span_reader),
            b: Accessor::Yaml(yaml_reader),
        };

        assert!(linker.check(), "Matching values should compare equal");

        // Test 2: Create a span with different content
        fs::write(&span_diff_path, "different").unwrap();

        let span_reader_diff = SpanReader {
            file_path: span_diff_path,
            start: Cursor { line: 1, column: 1 },
            end: Cursor { line: 1, column: 9 }, // Just use the first 9 characters
        };

        let yaml_reader_diff = YamlReader {
            file_path: yaml_path,
            key_path: "server.host".to_string(),
        };

        let linker_diff = Linker {
            a: Accessor::Spans(span_reader_diff),
            b: Accessor::Yaml(yaml_reader_diff),
        };

        assert!(
            !linker_diff.check(),
            "Different values should not compare equal"
        );

        // Temporary files will be automatically cleaned up when they go out of scope
    }

    #[test]
    fn test_linker_with_query() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files
        let content_file = NamedTempFile::new().unwrap();
        let toml_file = NamedTempFile::new().unwrap();
        let yaml_file = NamedTempFile::new().unwrap();

        let content_path = content_file.path().to_str().unwrap().to_string();
        let toml_path = toml_file.path().to_str().unwrap().to_string();
        let yaml_path = yaml_file.path().to_str().unwrap().to_string();

        // Create a file with some content to search in
        let file_content =
            String::from("This is a test file that contains some configuration values.\n")
                + "The API key is abc123 and should be kept secret.\n"
                + "The server host is localhost and the port is 8080.\n"
                + "Debug mode is enabled (true).";

        fs::write(&content_path, file_content).unwrap();

        // Create a TOML file with matching values
        let toml_content = r#"[config]
api_key = "abc123"
host = "localhost"
port = 8080
debug = true
"#;

        fs::write(&toml_path, toml_content).unwrap();

        // Create a YAML file with matching values
        let yaml_content =
            "---\nconfig:\n  api_key: abc123\n  host: localhost\n  port: 8080\n  debug: true\n";

        fs::write(&yaml_path, yaml_content).unwrap();

        // Test 1: Compare query result with TOML value (matching)
        let query_reader = QueryReader {
            file_path: content_path.clone(),
            query: "abc123".to_string(),
        };

        let toml_reader = TomlReader {
            file_path: toml_path.clone(),
            key_path: "config.api_key".to_string(),
        };

        let linker = Linker {
            a: Accessor::Query(query_reader),
            b: Accessor::Toml(toml_reader),
        };

        assert!(
            linker.check(),
            "Matching query and TOML value should compare equal"
        );

        // Test 2: Compare query result with YAML value (matching)
        let query_reader = QueryReader {
            file_path: content_path.clone(),
            query: "localhost".to_string(),
        };

        let yaml_reader = YamlReader {
            file_path: yaml_path.clone(),
            key_path: "config.host".to_string(),
        };

        let linker = Linker {
            a: Accessor::Query(query_reader),
            b: Accessor::Yaml(yaml_reader),
        };

        assert!(
            linker.check(),
            "Matching query and YAML value should compare equal"
        );

        // Test 3: Compare query result with TOML value (non-matching)
        let query_reader = QueryReader {
            file_path: content_path,
            query: "8080".to_string(),
        };

        let toml_reader = TomlReader {
            file_path: toml_path,
            key_path: "config.api_key".to_string(),
        };

        let linker = Linker {
            a: Accessor::Query(query_reader),
            b: Accessor::Toml(toml_reader),
        };

        assert!(
            !linker.check(),
            "Non-matching query and TOML value should not compare equal"
        );

        // Temporary files will be automatically cleaned up when they go out of scope
    }

    #[test]
    fn test_linker_with_query_and_toml() {
        use std::fs;
        use tempfile::NamedTempFile;

        // Create temporary files for the test
        let text_file = NamedTempFile::new().unwrap();
        let config_file = NamedTempFile::new().unwrap();

        let text_path = text_file.path().to_str().unwrap().to_string();
        let config_path = config_file.path().to_str().unwrap().to_string();

        // Create a text file with various configuration values embedded in prose
        let text_content = String::from(
            "\
# Project Configuration

This document contains the configuration settings for our project.

## API Settings

The production API endpoint is https://api.example.com/v2 and should be used for all requests.
For development, use http://localhost:3000 instead.

## Database Settings

Database connection string: postgresql://user:password@db.example.com:5432/mydb
Max connections: 100
Timeout: 30 seconds

## Feature Flags

- Enable caching: true
- Debug mode: false
- Beta features: true

Last updated: 2025-05-01
",
        );

        // Create a TOML configuration file with the same values
        let config_content = r#"[api]
production_url = "https://api.example.com/v2"
development_url = "http://localhost:3000"

[database]
connection_string = "postgresql://user:password@db.example.com:5432/mydb"
max_connections = 100
timeout = 30

[features]
enable_caching = true
debug_mode = false
beta_features = true

[metadata]
last_updated = "2025-05-01"
"#;

        fs::write(&text_path, text_content).unwrap();
        fs::write(&config_path, config_content).unwrap();

        // Test 1: API production URL
        let query_api_prod = QueryReader {
            file_path: text_path.clone(),
            query: "https://api.example.com/v2".to_string(),
        };

        let toml_api_prod = TomlReader {
            file_path: config_path.clone(),
            key_path: "api.production_url".to_string(),
        };

        let linker_api = Linker {
            a: Accessor::Query(query_api_prod),
            b: Accessor::Toml(toml_api_prod),
        };

        assert!(
            linker_api.check(),
            "API production URL should match between text and config"
        );

        // Test 2: Database connection string
        let query_db = QueryReader {
            file_path: text_path.clone(),
            query: "postgresql://user:password@db.example.com:5432/mydb".to_string(),
        };

        let toml_db = TomlReader {
            file_path: config_path.clone(),
            key_path: "database.connection_string".to_string(),
        };

        let linker_db = Linker {
            a: Accessor::Query(query_db),
            b: Accessor::Toml(toml_db),
        };

        assert!(
            linker_db.check(),
            "Database connection string should match between text and config"
        );

        // Test 3: Numeric value (timeout)
        let query_timeout = QueryReader {
            file_path: text_path.clone(),
            query: "30".to_string(),
        };

        let toml_timeout = TomlReader {
            file_path: config_path.clone(),
            key_path: "database.timeout".to_string(),
        };

        let linker_timeout = Linker {
            a: Accessor::Query(query_timeout),
            b: Accessor::Toml(toml_timeout),
        };

        assert!(
            linker_timeout.check(),
            "Timeout value should match between text and config"
        );

        // Test 4: Boolean value (debug mode)
        let query_debug = QueryReader {
            file_path: text_path.clone(),
            query: "false".to_string(),
        };

        let toml_debug = TomlReader {
            file_path: config_path.clone(),
            key_path: "features.debug_mode".to_string(),
        };

        let linker_debug = Linker {
            a: Accessor::Query(query_debug),
            b: Accessor::Toml(toml_debug),
        };

        assert!(
            linker_debug.check(),
            "Debug mode value should match between text and config"
        );

        // Test 5: Non-matching value (intentionally wrong)
        let query_wrong = QueryReader {
            file_path: text_path,
            query: "true".to_string(),
        };

        let toml_wrong = TomlReader {
            file_path: config_path,
            key_path: "features.debug_mode".to_string(),
        };

        let linker_wrong = Linker {
            a: Accessor::Query(query_wrong),
            b: Accessor::Toml(toml_wrong),
        };

        assert!(
            !linker_wrong.check(),
            "Mismatched values should not compare equal"
        );
    }
}
