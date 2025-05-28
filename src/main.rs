// Import modules
mod config;
mod readers;

// Re-export modules for easier access
pub use config::Config;
pub use readers::{Accessor, Linker, QueryReader, Reader, SpanReader, TomlReader, YamlReader};

// Import standard library modules
use std::process;

// Import clap for CLI
use clap::{Parser, Subcommand};

// Import the tests module directly in the file
#[cfg(test)]
mod tests;

/// A tool for checking links between different parts of your codebase
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "~/.config/clevis/config.toml")]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check if values match for a specific link or all links
    Check {
        /// Specific link key to check (if omitted, checks all links)
        link_key: Option<String>,

        /// Show values even when they match
        #[arg(short, long)]
        verbose: bool,
    },
    /// List all available links in the configuration
    List {},
    /// Show the values for a specific link
    Show {
        /// Link key to show
        link_key: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Expand the tilde in the config path if present
    let config_path = if cli.path.starts_with("~") {
        if let Some(home_dir) = dirs::home_dir() {
            let path_without_tilde = cli.path.strip_prefix("~").unwrap();
            home_dir
                .join(
                    path_without_tilde
                        .strip_prefix('/')
                        .unwrap_or(path_without_tilde),
                )
                .to_string_lossy()
                .into_owned()
        } else {
            eprintln!("Could not determine home directory");
            process::exit(1);
        }
    } else {
        cli.path.clone()
    };

    // Load the configuration file
    let config = match Config::load(&config_path) {
        Ok(config) => {
            println!("Loaded config '{}'", config_path);
            config
        }
        Err(e) => {
            println!("Failed to load config '{}': {}", config_path, e);
            process::exit(1);
        }
    };

    match &cli.command {
        Commands::Check { link_key, verbose } => {
            if let Some(link_key) = link_key {
                // Check a specific link
                match config.check(link_key) {
                    Ok(result) => {
                        if result {
                            println!("✓ Values match for '{}'", link_key);

                            // Show the values if verbose mode is enabled
                            if *verbose {
                                if let Ok(linker) = config.get_linker(link_key) {
                                    let a_value = linker.a.read();
                                    let b_value = linker.b.read();
                                    println!("  Value A: '{}'", a_value);
                                    println!("  Value B: '{}'", b_value);
                                }
                            }
                        } else {
                            println!("✗ Values do NOT match for '{}'", link_key);

                            // Show the values for better debugging
                            if let Ok(linker) = config.get_linker(link_key) {
                                let a_value = linker.a.read();
                                let b_value = linker.b.read();
                                println!("  Value A: '{}'", a_value);
                                println!("  Value B: '{}'", b_value);
                            }

                            process::exit(1);
                        }
                    }
                    Err(e) => {
                        println!("Error checking link '{}': {}", link_key, e);
                        process::exit(1);
                    }
                }
            } else {
                // Check all links
                let mut all_passed = true;
                let mut failed_links = Vec::new();

                if config.links.is_empty() {
                    println!("No links found in config file");
                    process::exit(0);
                }

                println!("Checking all links in {}:", config_path);

                for link_key in config.links.keys() {
                    match config.check(link_key) {
                        Ok(result) => {
                            if result {
                                println!("  ✓ '{}': Values match", link_key);

                                // Show the values if verbose mode is enabled
                                if *verbose {
                                    if let Ok(linker) = config.get_linker(link_key) {
                                        let a_value = linker.a.read();
                                        let b_value = linker.b.read();
                                        println!("    Value A: '{}'", a_value);
                                        println!("    Value B: '{}'", b_value);
                                    }
                                }
                            } else {
                                println!("  ✗ '{}': Values do NOT match", link_key);

                                // Show the values for better debugging
                                if let Ok(linker) = config.get_linker(link_key) {
                                    let a_value = linker.a.read();
                                    let b_value = linker.b.read();
                                    println!("    Value A: '{}'", a_value);
                                    println!("    Value B: '{}'", b_value);
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
                    process::exit(1);
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
                }
            }
        }
        Commands::Show { link_key } => {
            if let Ok(linker) = config.get_linker(link_key) {
                let a_value = linker.a.read();
                let b_value = linker.b.read();
                println!("Values for '{}':", link_key);
                println!("  Value A: '{}'", a_value);
                println!("  Value B: '{}'", b_value);
            } else {
                println!("Link '{}' not found in config", link_key);
                process::exit(1);
            }
        }
    }
}
