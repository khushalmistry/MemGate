//! MemGate CLI - Command-line interface for Memory I/O Simulation

use memgate::{MemGate, MemoryLayout};
use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// MemGate CLI - Memory I/O Simulation for IoT Emulation
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Load from template
    Template {
        /// Template name (e.g., STM32F103, ESP32)
        name: String,
        
        /// Allocate memory
        #[arg(short, long)]
        allocate: bool,
        
        /// Print memory map
        #[arg(short, long)]
        print: bool,
    },
    
    /// Load from configuration file
    Config {
        /// Configuration file path (TOML, YAML, or JSON)
        path: PathBuf,
        
        /// Allocate memory
        #[arg(short, long)]
        allocate: bool,
        
        /// Print memory map
        #[arg(short, long)]
        print: bool,
    },
    
    /// List available templates
    List,
    
    /// Show template details
    Show {
        /// Template name
        name: String,
    },
}

fn main() {
    let args = Args::parse();
    
    // Initialize logger
    env_logger::init();
    
    match args.command {
        Commands::Template { name, allocate, print } => {
            handle_template(&name, allocate, print);
        }
        
        Commands::Config { path, allocate, print } => {
            handle_config(&path, allocate, print);
        }
        
        Commands::List => {
            handle_list();
        }
        
        Commands::Show { name } => {
            handle_show(&name);
        }
    }
}

fn handle_template(name: &str, allocate: bool, print: bool) {
    println!("Loading template: {}", name);
    
    match MemGate::from_template(name) {
        Ok(mem) => {
            println!("✓ Template '{}' loaded successfully", name);
            
            if print {
                println!();
                mem.get_layout().print_map();
            }
            
            if allocate {
                let mut mem = mem;
                match mem.allocate() {
                    Ok(()) => {
                        println!("✓ Memory allocated successfully");
                        
                        if let Some(stats) = mem.stats() {
                            println!("\nAllocator Statistics:");
                            println!("  Total regions: {}", stats.total_regions);
                            println!("  Total size: {} bytes", format_size(stats.total_size));
                            println!("  Allocated: {}", stats.is_allocated);
                        }
                        
                        if print {
                            println!();
                            mem.print_map();
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Allocation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load template: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_config(path: &PathBuf, allocate: bool, print: bool) {
    println!("Loading configuration: {}", path.display());
    
    match MemGate::from_config(path) {
        Ok(mem) => {
            println!("✓ Configuration loaded successfully");
            
            if print {
                println!();
                mem.get_layout().print_map();
            }
            
            if allocate {
                let mut mem = mem;
                match mem.allocate() {
                    Ok(()) => {
                        println!("✓ Memory allocated successfully");
                        
                        if let Some(stats) = mem.stats() {
                            println!("\nAllocator Statistics:");
                            println!("  Total regions: {}", stats.total_regions);
                            println!("  Total size: {} bytes", format_size(stats.total_size));
                            println!("  Allocated: {}", stats.is_allocated);
                        }
                        
                        if print {
                            println!();
                            mem.print_map();
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Allocation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_list() {
    use memgate::layout::template::TemplateManager;
    
    println!("Available Templates:");
    println!("{:=<40}", "");
    println!("{:20} {}", "Name", "Description");
    println!("{:-<40}", "");
    
    for name in TemplateManager::list_templates() {
        match TemplateManager::get_template(name) {
            Ok(config) => {
                println!("{:20} {}", 
                    config.device.name,
                    config.device.description.unwrap_or_default()
                );
            }
            Err(_) => {
                println!("{:20} <error loading>", name);
            }
        }
    }
    
    println!("{:=<40}", "");
}

fn handle_show(name: &str) {
    use memgate::layout::template::TemplateManager;
    
    match TemplateManager::get_template(name) {
        Ok(config) => {
            match MemoryLayout::from_config(config) {
                Ok(layout) => {
                    println!("Template: {}", name);
                    println!();
                    layout.print_map();
                }
                Err(e) => {
                    eprintln!("✗ Failed to create layout: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("✗ Template not found: {}", e);
            std::process::exit(1);
        }
    }
}

/// Format size in human-readable format
fn format_size(size: usize) -> String {
    if size >= 1024 * 1024 * 1024 {
        format!("{}GB", size / (1024 * 1024 * 1024))
    } else if size >= 1024 * 1024 {
        format!("{}MB", size / (1024 * 1024))
    } else if size >= 1024 {
        format!("{}KB", size / 1024)
    } else {
        format!("{}B", size)
    }
}