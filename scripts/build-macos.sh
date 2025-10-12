#!/bin/bash
# Build script for macOS (DMG, App bundle)
set -e

echo "===================================="
echo "Building Layercake for macOS"
echo "===================================="

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo "Error: This script must be run on macOS"
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
echo "  App:  src-tauri/target/release/bundle/macos/"
echo "  DMG:  src-tauri/target/release/bundle/dmg/"
echo ""
echo "Note: To distribute on macOS, you'll need to:"
echo "  1. Sign the app with an Apple Developer certificate"
echo "  2. Notarize the app with Apple"
echo ""
