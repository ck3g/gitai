use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;

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
        Commands::Commit => handle_commit(),
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

fn handle_commit() {
    match is_git_repository() {
        Ok(true) => println!("Success! That's a git repo"),
        Ok(false) => {
            eprintln!("Error: Not a git repository");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Not a git repository: {}", e);
            std::process::exit(1);
        }
    }

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    writeln!(temp_file, "Dummy commit message").expect("Failed to write temp file");
    writeln!(temp_file).expect("Failed to write to temp file");
    writeln!(temp_file, "This is a dummy commit message.").expect("Failed to write to temp file");
    writeln!(temp_file).expect("Failed to write to temp file");
    writeln!(temp_file, "# Edit this message as needed before committing")
        .expect("Failed to write to temp file");

    temp_file.flush().expect("Failed to flush temp file");

    let temp_path = temp_file.path().to_owned();

    run_git_commit(&temp_path);
}

fn run_git_commit(temp_path: &Path) {
    let status = Command::new("git")
        .arg("commit")
        .arg("-t")
        .arg(temp_path)
        .status();

    match status {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(e) => {
            eprintln!("Failed to run git commit: {}", e);
            std::process::exit(1);
        }
    }
}

fn store_api_key(api_key: &str, config_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(config_dir)?;

    let config_file = config_dir.join("config");
    fs::write(&config_file, api_key)?;

    Ok(config_file)
}

fn is_git_repository_at(path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .current_dir(path)
        .arg("rev-parse")
        .arg("--git-dir")
        .output()?;

    Ok(output.status.success())
}

fn is_git_repository() -> Result<bool, Box<dyn std::error::Error>> {
    is_git_repository_at(Path::new("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
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

    #[test]
    fn test_is_git_repository_in_actual_git_repo() -> Result<(), Box<dyn std::error::Error>> {
        // Use current project directory, which is a git repo
        assert!(is_git_repository_at(Path::new("."))?);

        Ok(())
    }

    #[test]
    fn test_is_git_repository_not_in_git_repo() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        assert!(!is_git_repository_at(temp_dir.path())?);

        Ok(())
    }
}
