// Import modules
mod config;
mod error;
mod readers;

// Re-export modules for easier access
pub use config::Config;
pub use error::{AppError, Result};
pub use readers::{Accessor, Linker, QueryReader, Reader, SpanReader, TomlReader, YamlReader};

// Import standard library modules
use std::process;

// Import clap for CLI
use clap::{Parser, Subcommand};

// Import anyhow for error handling
use anyhow::Context;

// Import the tests module directly in the file
#[cfg(test)]
mod tests;

/// A tool for checking links between different parts of your codebase
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "./clevis.toml")]
    path: String,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if values match for a specific link or all links
    Check {
        /// Specific link key to check (if omitted, checks all links)
        link_key: Option<String>,
    },
    /// List all available links in the configuration
    List {},
    /// Show the values for a specific link
    Show {
        /// Link key to show
        link_key: String,
    },
}

fn expand_path(path: &str) -> String {
    if let Some(path_without_tilde) = path.strip_prefix("~") {
        if let Some(home_dir) = dirs::home_dir() {
            home_dir
                .join(
                    path_without_tilde
                        .strip_prefix('/')
                        .unwrap_or(path_without_tilde),
                )
                .to_string_lossy()
                .into_owned()
        } else {
            // Fallback to original path if home dir unavailable
            path.to_string()
        }
    } else {
        path.to_string()
    }
}

fn show_values(a: &Accessor, b: &Accessor) -> anyhow::Result<()> {
    match (a.read(), b.read()) {
        (Ok(a_value), Ok(b_value)) => {
            println!("  Value A: '{}'", a_value);
            println!("  Value B: '{}'", b_value);
        }
        (Err(e), _) => println!("  Error reading A: {}", e),
        (_, Err(e)) => println!("  Error reading B: {}", e),
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);

        // Handle different error types with specific exit codes
        let exit_code = match e.downcast_ref::<AppError>() {
            Some(AppError::FileOperation { .. }) => 2,
            Some(AppError::Parse { .. }) => 3,
            Some(AppError::KeyNotFound { .. }) | Some(AppError::QueryNotFound { .. }) => 4,
            Some(AppError::ConfigError { .. }) => 5,
            _ => 1,
        };

        process::exit(exit_code);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config_path = expand_path(&cli.path);

    // Load the configuration file
    let config = Config::load(&config_path)
        .with_context(|| format!("Failed to load configuration from: {}", config_path))?;

    if cli.verbose {
        println!("Loaded config: {}", config_path);
    }

    match &cli.command {
        Commands::Check { link_key } => {
            if let Some(link_key) = link_key {
                // Check a specific link
                let result = config
                    .check(link_key)
                    .with_context(|| format!("Failed to check link: {}", link_key))?;

                if result {
                    println!("✓ Values match for '{}'", link_key);

                    // Show the values if verbose mode is enabled
                    if cli.verbose {
                        let linker = config.get_linker(link_key)?;
                        show_values(&linker.a, &linker.b)?;
                    }
                } else {
                    println!("✗ Values do NOT match for '{}'", link_key);

                    // Show the values for better debugging
                    let linker = config.get_linker(link_key)?;
                    show_values(&linker.a, &linker.b)?;

                    anyhow::bail!("Values do not match for link: {}", link_key);
                }
            } else {
                // Check all links
                let mut all_passed = true;
                let mut failed_links = Vec::new();

                if config.links.is_empty() {
                    println!("No links found in config file");
                    return Ok(());
                }

                println!("Checking all links in {}:", config_path);

                for link_key in config.links.keys() {
                    match config.check(link_key) {
                        Ok(result) => {
                            if result {
                                println!("  ✓ '{}': Values match", link_key);

                                // Show the values if verbose mode is enabled
                                if cli.verbose {
                                    if let Ok(linker) = config.get_linker(link_key) {
                                        match (linker.a.read(), linker.b.read()) {
                                            (Ok(a_value), Ok(b_value)) => {
                                                println!("    Value A: '{}'", a_value);
                                                println!("    Value B: '{}'", b_value);
                                            }
                                            (Err(e), _) => println!("    Error reading A: {}", e),
                                            (_, Err(e)) => println!("    Error reading B: {}", e),
                                        }
                                    }
                                }
                            } else {
                                println!("  ✗ '{}': Values do NOT match", link_key);

                                // Show the values for better debugging
                                if let Ok(linker) = config.get_linker(link_key) {
                                    match (linker.a.read(), linker.b.read()) {
                                        (Ok(a_value), Ok(b_value)) => {
                                            println!("    Value A: '{}'", a_value);
                                            println!("    Value B: '{}'", b_value);
                                        }
                                        (Err(e), _) => println!("    Error reading A: {}", e),
                                        (_, Err(e)) => println!("    Error reading B: {}", e),
                                    }
                                }

                                all_passed = false;
                                failed_links.push(link_key.clone());
                            }
                        }
                        Err(e) => {
                            println!("  ✗ '{}': Error: {}", link_key, e);
                            all_passed = false;
                            failed_links.push(link_key.clone());
                        }
                    }
                }

                if all_passed {
                    println!("\nAll links passed!");
                } else {
                    println!("\nFailed links: {}", failed_links.join(", "));
                    anyhow::bail!("Some links failed");
                }
            }
        }
        Commands::List {} => {
            println!("Links in {}:", config_path);

            if config.links.is_empty() {
                println!("  No links found");
            } else {
                for link_key in config.links.keys() {
                    println!("  {}", link_key);

                    if cli.verbose {
                        if let Ok(linker) = config.get_linker(link_key) {
                            match (linker.a.read(), linker.b.read()) {
                                (Ok(a_value), Ok(b_value)) => {
                                    println!("    Value A: '{}'", a_value);
                                    println!("    Value B: '{}'", b_value);
                                }
                                (Err(e), _) => println!("    Error reading A: {}", e),
                                (_, Err(e)) => println!("    Error reading B: {}", e),
                            }
                        }
                    }
                }
            }
        }
        Commands::Show { link_key } => {
            if let Ok(linker) = config.get_linker(link_key) {
                println!("Values for '{}':", link_key);
                match (linker.a.read(), linker.b.read()) {
                    (Ok(a_value), Ok(b_value)) => {
                        println!("  Value A: '{}'", a_value);
                        println!("  Value B: '{}'", b_value);
                    }
                    (Err(e), _) => println!("  Error reading A: {}", e),
                    (_, Err(e)) => println!("  Error reading B: {}", e),
                }

                if cli.verbose {
                    match config.links.get(link_key) {
                        Some(link_config) => {
                            println!("  A source: {:?}", link_config.a);
                            println!("  B source: {:?}", link_config.b);

                            // Show if values match
                            let match_result = config.check(link_key).unwrap_or(false);
                            if match_result {
                                println!("  Match status: ✓ Values match");
                            } else {
                                println!("  Match status: ✗ Values do NOT match");
                            }
                        }
                        None => println!("  No detailed configuration found"),
                    }
                }
            } else {
                anyhow::bail!("Link '{}' not found in config", link_key);
            }
        }
    }

    Ok(())
}
