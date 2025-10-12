#!/bin/bash
# Build script for Linux (AppImage, Deb, RPM)
set -e

echo "===================================="
echo "Building Layercake for Linux"
echo "===================================="

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: This script must be run on Linux"
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
echo "  AppImage: src-tauri/target/release/bundle/appimage/"
echo "  Deb:      src-tauri/target/release/bundle/deb/"
echo ""
