#!/bin/sh
# detect_system.sh - Detects OS and architecture for gitai installation
# This script uses POSIX sh for maximum compatibility

set -e  # Exit on error

# Function to print messages
info() {
    printf '%s\n' "$1"
}

error() {
    printf 'Error: %s\n' "$1" >&2
    exit 1
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

# Main detection logic
main() {
    info "Detecting system information..."
    
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    info "Operating System: $OS"
    info "Architecture: $ARCH"
    
    # Construct the target triple (Rust convention) for building
    case "$OS-$ARCH" in
        linux-x86_64)
            TARGET="x86_64-unknown-linux-gnu"
            PLATFORM="linux-amd64"
            ;;
        linux-aarch64)
            TARGET="aarch64-unknown-linux-gnu"
            PLATFORM="linux-arm64"
            ;;
        darwin-x86_64)
            TARGET="x86_64-apple-darwin"
            PLATFORM="darwin-amd64"
            ;;
        darwin-aarch64)
            TARGET="aarch64-apple-darwin"
            PLATFORM="darwin-arm64"
            ;;
        windows-x86_64)
            TARGET="x86_64-pc-windows-msvc"
            PLATFORM="windows-amd64"
            ;;
        *)
            error "No prebuilt binary available for $OS-$ARCH"
            ;;
    esac
    
    info "Target triple: $TARGET"
    info "Platform: $PLATFORM"
    info "Archive name would be: gitai-0.1.0-$PLATFORM.tar.gz"
}

# Run the main function
main "$@"