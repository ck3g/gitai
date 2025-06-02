use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gitai")]
#[command(version, about= "AI-powered git commit messages", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize gitai with your API key
    Init,
    /// Generate a commit message based on staged changes
    Commit,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => handle_init(),
        Commands::Commit => println!("commit command coming soon"),
    }
}

fn handle_init() {
    print!("Enter your Anthropic API key: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    let api_key = input.trim();

    match store_api_key(api_key) {
        Ok(path) => println!("API key saved to {:?}", path),
        Err(e) => eprintln!("Failed to save API key: {}", e),
    }
}

fn store_api_key(api_key: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = dirs::home_dir()
        .ok_or("Could not find home directory")?
        .join(".gitai");

    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("config");
    fs::write(&config_file, api_key)?;

    Ok(config_file)
}
