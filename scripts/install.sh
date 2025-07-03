#!/bin/bash

# Layercake Installation Script
# This script downloads and installs the latest Layercake release for your platform

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="michiel/layercake-tool"
GITHUB_API="https://api.github.com/repos/${REPO}"
GITHUB_RELEASES="${GITHUB_API}/releases/latest"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="layercake"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect platform and architecture
detect_platform() {
    local os
    local arch
    
    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="linux";;
        Darwin*)    os="macos";;
        CYGWIN*|MINGW*|MSYS*) os="windows";;
        *)          
            log_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64";;
        aarch64|arm64)  arch="aarch64";;
        armv7l)         arch="armv7";;
        i386|i686)      arch="i686";;
        *)              
            log_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    PLATFORM="${os}"
    ARCH="${arch}"
    
    log_info "Detected platform: ${PLATFORM}-${ARCH}"
}

# Check if required tools are available
check_dependencies() {
    local missing_deps=()
    
    for cmd in curl jq tar; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Please install the missing dependencies and try again"
        
        # Provide installation hints based on platform
        case "$PLATFORM" in
            linux)
                log_info "On Ubuntu/Debian: sudo apt-get install curl jq tar"
                log_info "On CentOS/RHEL/Fedora: sudo yum install curl jq tar (or dnf)"
                ;;
            macos)
                log_info "On macOS: brew install curl jq"
                log_info "Or install using MacPorts: sudo port install curl jq"
                ;;
        esac
        
        exit 1
    fi
}

# Get the latest release information
get_latest_release() {
    log_info "Fetching latest release information..."
    
    if ! RELEASE_DATA=$(curl -s "$GITHUB_RELEASES"); then
        log_error "Failed to fetch release information from GitHub"
        exit 1
    fi
    
    if ! LATEST_VERSION=$(echo "$RELEASE_DATA" | jq -r '.tag_name'); then
        log_error "Failed to parse release version from GitHub API response"
        exit 1
    fi
    
    if [ "$LATEST_VERSION" = "null" ] || [ -z "$LATEST_VERSION" ]; then
        log_error "No releases found in the repository"
        exit 1
    fi
    
    log_info "Latest version: $LATEST_VERSION"
}

# Find the appropriate asset for the current platform
find_asset() {
    log_info "Looking for ${PLATFORM}-${ARCH} release asset..."
    
    # Common naming patterns for release assets
    local patterns=(
        "layercake-${LATEST_VERSION}-${PLATFORM}-${ARCH}.tar.gz"
        "layercake-${PLATFORM}-${ARCH}.tar.gz"
        "layercake-${LATEST_VERSION#v}-${PLATFORM}-${ARCH}.tar.gz"
        "layercake_${LATEST_VERSION#v}_${PLATFORM}_${ARCH}.tar.gz"
        "${PLATFORM}-${ARCH}.tar.gz"
    )
    
    # Windows uses .zip typically
    if [ "$PLATFORM" = "windows" ]; then
        patterns+=(
            "layercake-${LATEST_VERSION}-${PLATFORM}-${ARCH}.zip"
            "layercake-${PLATFORM}-${ARCH}.zip"
            "layercake-${LATEST_VERSION#v}-${PLATFORM}-${ARCH}.zip"
            "layercake_${LATEST_VERSION#v}_${PLATFORM}_${ARCH}.zip"
            "${PLATFORM}-${ARCH}.zip"
        )
    fi
    
    # Search for matching assets
    local assets
    assets=$(echo "$RELEASE_DATA" | jq -r '.assets[].name')
    
    for pattern in "${patterns[@]}"; do
        for asset in $assets; do
            if [[ "$asset" == *"$pattern"* ]] || [[ "$asset" == "$pattern" ]]; then
                ASSET_NAME="$asset"
                DOWNLOAD_URL=$(echo "$RELEASE_DATA" | jq -r ".assets[] | select(.name == \"$asset\") | .browser_download_url")
                log_success "Found release asset: $ASSET_NAME"
                return 0
            fi
        done
    done
    
    # If no exact match, try partial matching
    log_warning "Exact match not found, trying partial matching..."
    for asset in $assets; do
        if [[ "$asset" == *"$PLATFORM"* ]] && [[ "$asset" == *"$ARCH"* ]]; then
            ASSET_NAME="$asset"
            DOWNLOAD_URL=$(echo "$RELEASE_DATA" | jq -r ".assets[] | select(.name == \"$asset\") | .browser_download_url")
            log_warning "Using partial match: $ASSET_NAME"
            return 0
        fi
    done
    
    log_error "No suitable release asset found for ${PLATFORM}-${ARCH}"
    log_info "Available assets:"
    echo "$assets" | sed 's/^/  /'
    exit 1
}

# Download and extract the release
download_and_extract() {
    local temp_dir
    temp_dir=$(mktemp -d)
    local archive_path="${temp_dir}/${ASSET_NAME}"
    
    log_info "Downloading $ASSET_NAME..."
    
    if ! curl -L -o "$archive_path" "$DOWNLOAD_URL"; then
        log_error "Failed to download release archive"
        rm -rf "$temp_dir"
        exit 1
    fi
    
    log_info "Extracting archive..."
    
    # Extract based on file extension
    case "$ASSET_NAME" in
        *.tar.gz|*.tgz)
            if ! tar -xzf "$archive_path" -C "$temp_dir"; then
                log_error "Failed to extract tar.gz archive"
                rm -rf "$temp_dir"
                exit 1
            fi
            ;;
        *.zip)
            if ! unzip -q "$archive_path" -d "$temp_dir"; then
                log_error "Failed to extract zip archive"
                log_info "Make sure 'unzip' is installed on your system"
                rm -rf "$temp_dir"
                exit 1
            fi
            ;;
        *)
            log_error "Unsupported archive format: $ASSET_NAME"
            rm -rf "$temp_dir"
            exit 1
            ;;
    esac
    
    # Find the binary in the extracted files
    local binary_path
    binary_path=$(find "$temp_dir" -name "$BINARY_NAME" -type f -executable | head -1)
    
    if [ -z "$binary_path" ]; then
        # Try common variations
        for name in "layercake" "layercake.exe" "layercake-${PLATFORM}-${ARCH}" "layercake_${PLATFORM}_${ARCH}"; do
            binary_path=$(find "$temp_dir" -name "$name" -type f | head -1)
            if [ -n "$binary_path" ]; then
                break
            fi
        done
    fi
    
    if [ -z "$binary_path" ]; then
        log_error "Could not find layercake binary in extracted archive"
        log_info "Archive contents:"
        find "$temp_dir" -type f | sed 's/^/  /'
        rm -rf "$temp_dir"
        exit 1
    fi
    
    log_success "Found binary at: $binary_path"
    BINARY_PATH="$binary_path"
    TEMP_DIR="$temp_dir"
}

# Create installation directory and install binary
install_binary() {
    log_info "Installing layercake to $INSTALL_DIR..."
    
    # Create installation directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        log_info "Creating directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi
    
    # Copy binary to installation directory
    local target_path="${INSTALL_DIR}/${BINARY_NAME}"
    
    if ! cp "$BINARY_PATH" "$target_path"; then
        log_error "Failed to copy binary to $target_path"
        exit 1
    fi
    
    # Make sure it's executable
    chmod +x "$target_path"
    
    log_success "Layercake installed to $target_path"
    
    # Cleanup
    rm -rf "$TEMP_DIR"
}

# Check if the installation directory is in PATH
check_path() {
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        log_success "$INSTALL_DIR is already in your PATH"
        return 0
    fi
    
    log_warning "$INSTALL_DIR is not in your PATH"
    log_info "To add it to your PATH, add this line to your shell profile:"
    log_info "  export PATH=\"\$PATH:$INSTALL_DIR\""
    
    # Detect shell and provide specific instructions
    local shell_name
    shell_name=$(basename "$SHELL")
    
    case "$shell_name" in
        bash)
            local profile_file="$HOME/.bashrc"
            if [ -f "$HOME/.bash_profile" ]; then
                profile_file="$HOME/.bash_profile"
            fi
            log_info "For Bash, add the export line to: $profile_file"
            ;;
        zsh)
            log_info "For Zsh, add the export line to: $HOME/.zshrc"
            ;;
        fish)
            log_info "For Fish, run: fish_add_path $INSTALL_DIR"
            ;;
        *)
            log_info "For your shell ($shell_name), add the export line to your shell's configuration file"
            ;;
    esac
    
    log_info "Then restart your terminal or run: source <your-shell-config-file>"
    
    return 1
}

# Verify installation
verify_installation() {
    local binary_path="${INSTALL_DIR}/${BINARY_NAME}"
    
    if [ ! -f "$binary_path" ]; then
        log_error "Installation verification failed: binary not found at $binary_path"
        return 1
    fi
    
    if [ ! -x "$binary_path" ]; then
        log_error "Installation verification failed: binary is not executable"
        return 1
    fi
    
    log_info "Testing layercake installation..."
    
    # Test if the binary runs and shows version/help
    if "$binary_path" --version >/dev/null 2>&1 || "$binary_path" --help >/dev/null 2>&1; then
        log_success "Layercake installation verified successfully!"
        
        # Show version if available
        if version_output=$("$binary_path" --version 2>/dev/null); then
            log_info "Installed version: $version_output"
        fi
        
        return 0
    else
        log_warning "Binary installed but failed to run properly"
        log_info "This might be due to missing system dependencies"
        return 1
    fi
}

# Show usage information
show_usage() {
    log_info "Layercake has been installed! Here's how to use it:"
    echo
    echo "  Process a plan file:"
    echo "    layercake -p <plan.yaml>"
    echo
    echo "  Watch mode for development:"
    echo "    layercake -p <plan.yaml> -w"
    echo
    echo "  Get help:"
    echo "    layercake --help"
    echo
    echo "  Check version:"
    echo "    layercake --version"
    echo
    log_info "For more information, visit: https://github.com/${REPO}"
}

# Main installation function
main() {
    log_info "🎂 Layercake Installation Script"
    log_info "================================="
    echo
    
    # Check if we're running as root (not recommended)
    if [ "$EUID" -eq 0 ]; then
        log_warning "Running as root is not recommended"
        log_warning "This will install layercake system-wide instead of user-local"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled"
            exit 0
        fi
        INSTALL_DIR="/usr/local/bin"
    fi
    
    # Perform installation steps
    detect_platform
    check_dependencies
    get_latest_release
    find_asset
    download_and_extract
    install_binary
    
    # Check PATH and verify installation
    local path_ok=0
    check_path || path_ok=1
    
    echo
    if verify_installation; then
        echo
        log_success "🎉 Layercake installation completed successfully!"
        
        if [ $path_ok -eq 0 ]; then
            show_usage
        else
            log_info "Add $INSTALL_DIR to your PATH to use layercake from anywhere"
        fi
    else
        log_error "Installation completed but verification failed"
        log_info "The binary is installed at ${INSTALL_DIR}/${BINARY_NAME}"
        exit 1
    fi
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Layercake Installation Script"
        echo
        echo "Usage: $0 [OPTIONS]"
        echo
        echo "This script downloads and installs the latest Layercake release"
        echo "from GitHub for your platform."
        echo
        echo "Options:"
        echo "  --help, -h    Show this help message"
        echo "  --version     Show script version"
        echo
        echo "Environment Variables:"
        echo "  LAYERCAKE_INSTALL_DIR    Custom installation directory (default: ~/.local/bin)"
        echo
        echo "Examples:"
        echo "  $0                               # Install latest version"
        echo "  LAYERCAKE_INSTALL_DIR=/opt/bin $0  # Install to custom directory"
        echo
        exit 0
        ;;
    --version)
        echo "Layercake Installation Script v1.0.0"
        exit 0
        ;;
    "")
        # No arguments, proceed with installation
        ;;
    *)
        log_error "Unknown option: $1"
        log_info "Use --help for usage information"
        exit 1
        ;;
esac

# Allow custom installation directory
if [ -n "${LAYERCAKE_INSTALL_DIR:-}" ]; then
    INSTALL_DIR="$LAYERCAKE_INSTALL_DIR"
    log_info "Using custom installation directory: $INSTALL_DIR"
fi

# Run the main installation
main