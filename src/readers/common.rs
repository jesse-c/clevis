use crate::error::Result;
use crate::readers::{QueryReader, SpanReader, TomlReader, YamlReader};

/// Common trait for all reader types
pub trait Reader {
    /// Read the content and return a string result or an error
    fn read(&self) -> Result<String>;
}

/// Accessor enum to handle different reader types
#[derive(Debug)]
pub enum Accessor {
    Spans(SpanReader),
    Toml(TomlReader),
    Yaml(YamlReader),
    Query(QueryReader),
}

impl Accessor {
    /// Read through the accessor, returning the content or an error
    pub fn read(&self) -> Result<String> {
        match self {
            Accessor::Spans(reader) => reader.read(),
            Accessor::Toml(reader) => reader.read(),
            Accessor::Yaml(reader) => reader.read(),
            Accessor::Query(reader) => reader.read(),
        }
    }
}

/// Linker struct to compare values from different readers
#[derive(Debug)]
pub struct Linker {
    pub a: Accessor,
    pub b: Accessor,
}

impl Linker {
    /// Compare the results of two readers, returning Ok(true) if equal,
    /// Ok(false) if not equal, or an error if either reader fails.
    pub fn check(&self) -> Result<bool> {
        Ok(self.a.read()? == self.b.read()?)
    }
}
