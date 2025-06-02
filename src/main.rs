use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

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
    let config_dir = dirs::home_dir()
        .expect("Could not find home directory")
        .join(".gitai");

    match store_api_key(api_key, &config_dir) {
        Ok(path) => println!("API key saved to {:?}", path),
        Err(e) => eprintln!("Failed to save API key: {}", e),
    }
}

fn store_api_key(api_key: &str, config_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(config_dir)?;

    let config_file = config_dir.join("config");
    fs::write(&config_file, api_key)?;

    Ok(config_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_store_api_key() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().join(".gitai");

        let api_key = "test-api-key-123";
        let result_path = store_api_key(api_key, &config_dir)?;

        assert!(result_path.exists());

        let content = fs::read_to_string(&result_path)?;
        assert_eq!(content, api_key);

        Ok(())
    }
}
