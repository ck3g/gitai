use clap::{Parser, Subcommand};
use prompt::build_prompt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

mod api;
mod prompt;

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
    let config_dir = get_config_dir();

    match store_api_key(api_key, &config_dir) {
        Ok(path) => println!("API key saved to {:?}", path),
        Err(e) => eprintln!("Failed to save API key: {}", e),
    }
}

fn handle_commit() {
    match is_git_repository() {
        Ok(true) => {}
        Ok(false) => {
            eprintln!("Error: Not a git repository");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Not a git repository: {}", e);
            std::process::exit(1);
        }
    }

    let diff = get_staged_changes().expect("Failed to run git diff --cached");

    if diff.is_empty() {
        run_git_commit(None);
        return;
    }

    let config_dir = get_config_dir();
    let api_key = match read_api_key(&config_dir) {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let prompt = build_prompt(&diff);
    println!("Generating commit message...");

    let rt = Runtime::new().expect("Failed to create Tokio runtime");
    let commit_message = rt.block_on(async {
        match api::generate_commit_message(&api_key, &prompt).await {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error generating commit message: {}", e);
                std::process::exit(1);
            }
        }
    });

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    writeln!(temp_file, "{}", commit_message).expect("Failed to write temp file");
    writeln!(temp_file).expect("Failed to write to temp file");
    writeln!(temp_file, "# Edit this message as needed before committing")
        .expect("Failed to write to temp file");

    temp_file.flush().expect("Failed to flush temp file");

    let template_path = temp_file.path().to_owned();

    run_git_commit(Some(&template_path));
}

fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".gitai")
}

fn get_staged_changes() -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").arg("diff").arg("--cached").output()?;

    if !output.status.success() {
        return Err("Failed to get git diff".into());
    }

    let diff = String::from_utf8(output.stdout)?;

    Ok(diff)
}

fn run_git_commit(template_path: Option<&Path>) {
    let mut cmd = Command::new("git");
    cmd.arg("commit");

    if let Some(path) = template_path {
        cmd.arg("-t").arg(path);
    }

    match cmd.status() {
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

fn read_api_key(config_dir: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let config_file = config_dir.join("config");
    let file_exists = fs::exists(&config_file)?;
    if !file_exists {
        return Err("Config file not found. Please run 'gitai init' first.".into());
    }

    let file_content = fs::read_to_string(config_file)?;
    let api_key = file_content.trim().to_string();

    if api_key.is_empty() {
        return Err("API key is empty. Please run 'gitai init' first.".into());
    }

    Ok(api_key)
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

    #[test]
    fn test_read_api_key_success() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path();

        let config_file = config_dir.join("config");
        fs::write(&config_file, "test-api-key-123")?;

        let api_key = read_api_key(config_dir)?;
        assert_eq!(api_key, "test-api-key-123");

        Ok(())
    }

    #[test]
    fn test_read_api_key_with_whitespace() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path();

        let config_file = config_dir.join("config");
        fs::write(&config_file, "  test-api-key-123\n\n")?;

        let api_key = read_api_key(config_dir)?;
        assert_eq!(api_key, "test-api-key-123");

        Ok(())
    }

    #[test]
    fn test_read_api_key_file_not_found() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path();

        let result = read_api_key(config_dir);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Config file not found")
        );

        Ok(())
    }

    #[test]
    fn test_read_api_key_empty_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path();

        let config_file = config_dir.join("config");
        fs::write(&config_file, "")?;

        let result = read_api_key(config_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key is empty"));

        Ok(())
    }

    #[test]
    fn test_read_api_key_whitespace_only() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path();

        let config_file = config_dir.join("config");
        fs::write(&config_file, "  \n\n")?;

        let result = read_api_key(config_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("API key is empty"));

        Ok(())
    }
}
