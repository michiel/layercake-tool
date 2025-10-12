# Layercake Build Guide

This document describes how to build Layercake desktop application for different platforms.

## Prerequisites

### All Platforms

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Node.js** (18+) and npm
- **Tauri CLI**: Installed automatically via Cargo

### Platform-Specific

#### Linux
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  gtk3 \
  libappindicator-gtk3 \
  librsvg \
  libvips
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
- **Visual Studio** (2019 or later) with C++ build tools
- **WebView2**: Usually pre-installed on Windows 10/11

## Development Build

### Quick Start

```bash
# Install frontend dependencies
npm run frontend:install

# Run in development mode
npm run tauri:dev
```

This will:
1. Start the Vite dev server at `http://localhost:5173`
2. Start the embedded backend server on port `3030`
3. Launch the Tauri window pointed at the dev server

_Tip:_ Run `npm run frontend:dev` when you only need the standalone web app without the Tauri shell.

### Backend Only

```bash
# Build backend
npm run backend:build

# Run backend tests
npm run backend:test
```

### Frontend Only

```bash
# Development server (without Tauri)
npm run frontend:dev

# Production build
npm run frontend:build
```

## Production Build

### All Platforms (Quick)

```bash
# Build for current platform
npm run tauri:build
```

### Platform-Specific Builds

#### Linux

```bash
# Using build script
npm run tauri:build:linux

# Or manually
cd src-tauri
cargo tauri build
```

**Output**:
- AppImage: `src-tauri/target/release/bundle/appimage/`
- Deb package: `src-tauri/target/release/bundle/deb/`

**Supported Formats**:
- AppImage (universal)
- .deb (Debian/Ubuntu)

#### macOS

```bash
# Using build script
npm run tauri:build:macos

# Or manually
cd src-tauri
cargo tauri build
```

**Output**:
- App bundle: `src-tauri/target/release/bundle/macos/`
- DMG: `src-tauri/target/release/bundle/dmg/`

**Code Signing** (Optional but recommended for distribution):
```bash
# Set signing identity in tauri.conf.json
# Or use environment variables
export APPLE_CERTIFICATE="Developer ID Application: Your Name (TEAMID)"
export APPLE_CERTIFICATE_PASSWORD="your-password"
cargo tauri build
```

**Notarization** (Required for distribution):
```bash
# After building, notarize with Apple
xcrun notarytool submit \
  src-tauri/target/release/bundle/dmg/Layercake_0.1.7_aarch64.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAMID" \
  --password "app-specific-password" \
  --wait

# Staple the notarization
xcrun stapler staple \
  src-tauri/target/release/bundle/dmg/Layercake_0.1.7_aarch64.dmg
```

#### Windows

```bash
# Using build script (Git Bash or WSL)
npm run tauri:build:windows

# Or manually
cd src-tauri
cargo tauri build
```

**Output**:
- MSI installer: `src-tauri/target/release/bundle/msi/`
- NSIS installer: `src-tauri/target/release/bundle/nsis/` (if configured)

**Code Signing** (Optional):
1. Obtain a code signing certificate
2. Update `tauri.conf.json` with certificate thumbprint
3. Build as normal

## Build Configuration

### tauri.conf.json

Key configuration options:

```json
{
  "productName": "Layercake",
  "version": "0.1.7",
  "identifier": "com.layercake.app",
  "build": {
    "beforeDevCommand": "node ./scripts/run-frontend.js dev -- --host 127.0.0.1 --port 5173",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "node ./scripts/run-frontend.js build",
    "frontendDist": "../frontend/dist"
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [...],
    "category": "DeveloperTool"
  }
}
```

### Environment Variables

#### Development

- `RUST_LOG`: Set log level (e.g., `debug`, `info`, `warn`)
- `VITE_API_BASE_URL`: Override backend URL (default: `http://localhost:3030`)

#### Production

- `TAURI_SIGNING_PRIVATE_KEY`: Private key for update signing
- `TAURI_SIGNING_PUBLIC_KEY`: Public key for update verification
- `APPLE_ID` / `APPLE_TEAM_ID` / `APPLE_APP_SPECIFIC_PASSWORD`: Credentials for macOS signing and notarization pipelines
- `WINDOWS_CERT_THUMBPRINT` (or `WINDOWS_CERT_PATH` + `WINDOWS_CERT_PASSWORD`): Certificate configuration for Windows signing

## Troubleshooting

### Build Failures

**"Failed to bundle project"**
- Ensure all dependencies are installed
- Check that frontend build completed successfully
- Verify icons exist in `src-tauri/icons/`

**"cargo not found"**
```bash
# Add Cargo to PATH
source $HOME/.cargo/env
```

**"WebKit not found" (Linux)**
```bash
# Install WebKit development files
sudo apt install libwebkit2gtk-4.1-dev
```

### Runtime Issues

**"Failed to connect to backend"**
- Check that port 3030 is not in use
- Verify database path is writable
- Check logs in app data directory

**"Database locked"**
- Close all instances of the app
- Delete lock file if present
- Reinitialise database from settings

## CI/CD

GitHub Actions builds run via `.github/workflows/tauri-build.yml`.  
The workflow:
- spins up jobs on Ubuntu, macOS, and Windows runners
- installs Node 20, Rust stable, and the Tauri CLI
- installs Linux WebKit dependencies when needed
- runs `npm run frontend:install` followed by `npm run tauri:build`
- uploads the bundles from `src-tauri/target/release/bundle/` as artifacts

**Secrets** (optional but recommended for signed releases):
- `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_SPECIFIC_PASSWORD` for macOS notarization
- `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PUBLIC_KEY` for Tauri updater signing
- `WINDOWS_CERT_THUMBPRINT` (or `WINDOWS_CERT_PATH` + `WINDOWS_CERT_PASSWORD`) for Windows signing

Builds succeed without these secrets, but the resulting artifacts will be unsigned.

The automated release workflow (`.github/workflows/release.yml`) runs on tag creation and now produces both the Rust CLI binaries and the Tauri desktop bundles (AppImage/deb, MSI/EXE, DMG/zip). Frontend builds are triggered through `src-tauri/scripts/run-frontend.js`, which resolves the frontend path relative to the Tauri config so it works in CI environments. Ensure the frontend can build headlessly and that the icon assets are valid Windows resources before tagging.

## Bundle Size Optimization

Current bundle sizes (approximate):
- Linux AppImage: ~45MB
- macOS DMG: ~35MB
- Windows MSI: ~40MB

### Reducing Size

1. **Strip debug symbols**:
```toml
# Cargo.toml
[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
```

2. **Exclude unnecessary files**: Edit `src-tauri/.taurignore`

3. **Compress assets**: Use optimised icons and images

## Distribution

### Linux
- Distribute AppImage for universal compatibility
- Or publish to package repositories (AUR, PPA, Flathub)

### macOS
- Distribute signed and notarised DMG
- Consider Mac App Store for wider reach

### Windows
- Distribute signed MSI installer
- Consider Microsoft Store

## Support

For build issues:
1. Check this document
2. Search [Tauri documentation](https://tauri.app/)
3. Open issue on GitHub

## Quick Reference

```bash
# Development
npm run tauri:dev                    # Run dev build
npm run frontend:dev                 # Frontend only
npm run backend:test                 # Run tests

# Production
npm run tauri:build                  # Build for current OS
npm run tauri:build:linux           # Linux only
npm run tauri:build:macos           # macOS only
npm run tauri:build:windows         # Windows only

# Utilities
npm run install:all                  # Install all dependencies
npm run frontend:build               # Build frontend only
npm run backend:build                # Build backend only
```
