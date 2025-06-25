#!/bin/sh
# install.sh - Install script for gitai
# This script uses POSIX sh for maximum compatibility

set -e  # Exit on error

# Configuration
GITLAB_PROJECT_ID="70251003"  # Project ID for ck3g/gitai
# Or use namespace/project format:
GITLAB_PROJECT_PATH="ck3g/gitai"  # Your GitLab username/project
GITLAB_INSTANCE="https://gitlab.com"  # Change if using self-hosted GitLab
GITLAB_API="$GITLAB_INSTANCE/api/v4"

# Function to print messages
info() {
    printf '%s\n' "$1" >&2  # Send to stderr so it doesn't interfere with function returns
}

error() {
    printf 'Error: %s\n' "$1" >&2
    exit 1
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect the operating system
detect_os() {
    os=$(uname -s)
    case "$os" in
        Linux*)
            echo "linux"
            ;;
        Darwin*)
            echo "darwin"
            ;;
        MINGW* | MSYS* | CYGWIN*)
            echo "windows"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac
}

# Detect the CPU architecture
detect_arch() {
    arch=$(uname -m)
    case "$arch" in
        x86_64 | amd64)
            echo "x86_64"
            ;;
        aarch64 | arm64)
            echo "aarch64"
            ;;
        armv7l)
            echo "armv7"
            ;;
        i386 | i686)
            echo "i686"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac
}

# Get the platform string for downloads
get_platform() {
    local os=$1
    local arch=$2
    
    case "$os-$arch" in
        linux-x86_64)
            echo "linux-amd64"
            ;;
        linux-aarch64)
            echo "linux-arm64"
            ;;
        darwin-x86_64)
            echo "darwin-amd64"
            ;;
        darwin-aarch64)
            echo "darwin-arm64"
            ;;
        windows-x86_64)
            echo "windows-amd64"
            ;;
        *)
            error "No prebuilt binary available for $os-$arch"
            ;;
    esac
}

# Get the latest release version from GitLab
get_latest_version() {
    # Use project ID for more reliable API calls
    local api_url="$GITLAB_API/projects/$GITLAB_PROJECT_ID/releases"
    
    info "Checking latest version from GitLab..."
    
    # Check if we have curl or wget
    if command_exists curl; then
        response=$(curl -s "$api_url")
    elif command_exists wget; then
        response=$(wget -qO- "$api_url")
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    # Check if we got an actual error (GitLab returns error at root level)
    if echo "$response" | grep -q '^{"error"'; then
        error_msg=$(echo "$response" | grep -o '"error":"[^"]*"' | cut -d'"' -f4)
        error "GitLab API error: $error_msg"
    fi
    
    # Check for message field at root level (another error format)
    if echo "$response" | grep -q '^{"message"'; then
        error_msg=$(echo "$response" | grep -o '"message":"[^"]*"' | cut -d'"' -f4)
        error "GitLab API error: $error_msg"
    fi
    
    # Extract the first (latest) release tag
    # GitLab returns releases in descending order by default
    version=$(echo "$response" | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    if [ -z "$version" ]; then
        info "No releases found. For testing, using version 0.1.0"
        version="0.1.0"
    else
        info "Found version: $version"
    fi
    
    # Remove 'v' prefix if present
    version=${version#v}
    
    # Return only the version string (no info messages)
    echo "$version"
}

# Main installation logic
main() {
    info "Installing gitai..."
    
    # Detect system
    OS=$(detect_os)
    ARCH=$(detect_arch)
    PLATFORM=$(get_platform "$OS" "$ARCH")
    
    info "Detected: $OS/$ARCH ($PLATFORM)"
    
    # Get latest version (store it once!)
    VERSION=$(get_latest_version)
    info "Latest version: $VERSION"
    
    # Construct download URL
    if [ "$OS" = "windows" ]; then
        ARCHIVE_NAME="gitai-$VERSION-$PLATFORM.zip"
    else
        ARCHIVE_NAME="gitai-$VERSION-$PLATFORM.tar.gz"
    fi
    
    # GitLab release assets can be accessed via:
    # 1. Direct link if you know the URL
    # 2. Generic package registry
    # 3. Release assets API
    
    # For now, we'll use the GitLab Package Registry URL pattern
    # Files are uploaded to: /packages/generic/gitai/VERSION/FILENAME
    DOWNLOAD_URL="$GITLAB_API/projects/$GITLAB_PROJECT_ID/packages/generic/gitai/$VERSION/$ARCHIVE_NAME"
    
    info "Download URL: $DOWNLOAD_URL"
    
    # Create temporary directory
    info "Creating temporary directory..."
    TEMP_DIR=$(mktemp -d) || error "Failed to create temp directory"
    trap "rm -rf '$TEMP_DIR'" EXIT  # Clean up on exit
    
    # Download the archive
    info "Downloading $ARCHIVE_NAME..."
    if command_exists curl; then
        curl -sSLf "$DOWNLOAD_URL" -o "$TEMP_DIR/$ARCHIVE_NAME" || {
            info ""
            info "Download failed. This is expected if the release doesn't exist yet."
            info "Once you create a release and upload the asset, the download will work."
            info ""
            info "For testing the rest of the installation flow, you can:"
            info "1. Build the binary: cargo build --release --target aarch64-apple-darwin"
            info "2. Create a test archive: tar czf gitai-0.1.0-darwin-arm64.tar.gz -C target/aarch64-apple-darwin/release gitai"
            info "3. Run the installer with a local file"
            exit 0
        }
    elif command_exists wget; then
        wget -q "$DOWNLOAD_URL" -O "$TEMP_DIR/$ARCHIVE_NAME" || {
            info "Download failed. This is expected if the release doesn't exist yet."
            exit 0
        }
    fi
    
    # Extract the binary
    info "Extracting binary..."
    cd "$TEMP_DIR" || error "Failed to change to temp directory"
    
    if [ "$OS" = "windows" ]; then
        # For Windows, we'd need unzip
        unzip -q "$ARCHIVE_NAME" || error "Failed to extract archive"
    else
        tar -xzf "$ARCHIVE_NAME" || error "Failed to extract archive"
    fi
    
    # Verify the binary exists
    if [ ! -f "gitai" ] && [ ! -f "gitai.exe" ]; then
        error "Binary not found in archive. Expected 'gitai' or 'gitai.exe'"
    fi
    
    # Determine installation directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    else
        INSTALL_DIR="$HOME/bin"
        mkdir -p "$INSTALL_DIR"
    fi
    
    info "Installing to $INSTALL_DIR..."
    
    # Install the binary
    if [ "$OS" = "windows" ]; then
        BINARY_NAME="gitai.exe"
    else
        BINARY_NAME="gitai"
    fi
    
    # Use install command for proper permissions
    install -m 755 "$BINARY_NAME" "$INSTALL_DIR/" || error "Failed to install binary"
    
    # Verify installation
    if command_exists gitai; then
        info "✅ gitai has been installed successfully!"
        info "Version: $(gitai --version 2>/dev/null || echo 'version command not implemented yet')"
    else
        info "⚠️  gitai was installed to $INSTALL_DIR but is not in your PATH"
        info ""
        info "Add this to your shell configuration file (.bashrc, .zshrc, etc.):"
        info "  export PATH=\"$INSTALL_DIR:\$PATH\""
        info ""
        info "Then reload your shell or run:"
        info "  source ~/.bashrc  # or ~/.zshrc"
    fi
}

# Run the main function
main "$@"