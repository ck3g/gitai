# gitai - AI-Powered Git Commit Messages

A CLI tool that uses AI to generate meaningful git commit messages based on your staged changes.

⚠️ This is an unofficial project. It is not endorsed or supported by GitLab Inc.

## Features

- Analyzes your git diff to understand changes
- Generates clear, descriptive commit messages
- Opens in your configured git editor

## Installation

### macOS (Apple Silicon)

```bash
curl -fsSL https://gitlab.com/ck3g/gitai/-/raw/main/scripts/install.sh | sh
```

## Getting Started

### 1. One-time Setup

Before using gitai, you need to configure it with your API key. This is a **global configuration** that works across all your git repositories:

```bash
gitai init
```

This will prompt you for your Anthropic API key and store it securely in `~/.gitai/config`.

**Note:** Unlike `git init`, this command sets up gitai globally on your system, not per-repository. You only need to run it once, and you can run it from anywhere.

### 2. Using gitai

Once configured, use gitai in any git repository:

1. Make your code changes
2. Stage the files you want to commit:
   ```bash
   git add <files>
   ```
3. Instead of `git commit`, use:
   ```bash
   gitai commit
   ```

gitai will analyze your staged changes, generate a commit message suggestion, and open your git editor with the proposed message. You can then review, edit, and save to complete the commit.

## Commands

### `gitai init`
Initializes gitai with your API key. This is a one-time global setup that stores your configuration in your home directory.

- **What it does:** Prompts for and stores your LLM API key
- **Where to run:** Anywhere - it's a global configuration
- **When to run:** Once, before first use
- **Config location:** `~/.gitai/config`

### `gitai commit`
Analyzes staged git changes and suggests a commit message.

- **What it does:** Reads `git diff --cached`, sends to AI, opens editor with suggestion
- **Where to run:** Inside a git repository with staged changes
- **Prerequisites:** Must have run `gitai init` first

## Configuration

gitai stores its configuration in `~/.gitai/`:
- `config` - Contains your API key

## Requirements

- Git
- An Anthropic API key (get one at [console.anthropic.com](https://console.anthropic.com))

## Roadmap

- [ ] Support for multiple LLM providers (OpenAI, etc.)
- [ ] Custom commit message format rules per project
- [ ] Configuration for commit message style preferences
- [ ] Integration with conventional commits format

## Contributing

This is a learning project for Rust. Contributions and feedback are welcome!

## License

MIT License - see [LICENSE](LICENSE) file for details.