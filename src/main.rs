// Import the readers module
mod readers;

// Re-export the readers module for easier access
pub use readers::{Accessor, Linker, QueryReader, Reader, SpanReader, TomlReader, YamlReader};

// Import the tests module directly in the file
#[cfg(test)]
mod tests;

fn main() {
    println!("Hello, world!")
}
