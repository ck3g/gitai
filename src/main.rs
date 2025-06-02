use clap::{Parser, Subcommand};

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
        Commands::Init => println!("init command coming soon"),
        Commands::Commit => println!("commit command coming soon"),
    }
}
