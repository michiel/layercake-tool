#!/bin/bash
# Build script for Windows (MSI, NSIS installer)
set -e

echo "===================================="
echo "Building Layercake for Windows"
echo "===================================="

# Check if we're on Windows (Git Bash or WSL)
if [[ "$OSTYPE" != "msys" && "$OSTYPE" != "win32" && ! -f "/proc/sys/fs/binfmt_misc/WSLInterop" ]]; then
    echo "Error: This script must be run on Windows"
    exit 1
fi

# Navigate to project root
cd "$(dirname "$0")/.."

# Install dependencies if needed
echo "Checking dependencies..."
if [ ! -d "frontend/node_modules" ]; then
    echo "Installing frontend dependencies..."
    cd frontend && npm install && cd ..
fi

# Build frontend
echo "Building frontend..."
cd frontend && npm run build && cd ..

# Build Tauri app
echo "Building Tauri application..."
cd src-tauri
cargo tauri build

echo "===================================="
echo "Build complete!"
echo "===================================="
echo ""
echo "Output files:"
echo "  MSI:  src-tauri/target/release/bundle/msi/"
echo "  NSIS: src-tauri/target/release/bundle/nsis/"
echo ""
