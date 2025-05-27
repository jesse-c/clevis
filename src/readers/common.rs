use crate::readers::{QueryReader, SpanReader, TomlReader, YamlReader};

/// Common trait for all reader types
pub trait Reader {
    fn read(&self) -> String;
}

/// Accessor enum to handle different reader types
pub enum Accessor {
    Spans(SpanReader),
    Toml(TomlReader),
    Yaml(YamlReader),
    Query(QueryReader),
}

impl Accessor {
    pub fn read(&self) -> String {
        match self {
            Accessor::Spans(reader) => reader.read(),
            Accessor::Toml(reader) => reader.read(),
            Accessor::Yaml(reader) => reader.read(),
            Accessor::Query(reader) => reader.read(),
        }
    }
}

/// Linker struct to compare values from different readers
pub struct Linker {
    pub a: Accessor,
    pub b: Accessor,
}

impl Linker {
    pub fn check(&self) -> bool {
        self.a.read() == self.b.read()
    }
}
