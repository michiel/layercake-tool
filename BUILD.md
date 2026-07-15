# Layercake Build Guide

This document describes how to build Layercake, which ships as a single native `layercake` binary per platform. The web UI is compiled into the binary at build time, so the frontend must be built before the Rust build.

## Prerequisites

### All Platforms

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **Node.js** (18+) and npm

### Platform-Specific

#### Linux
```bash
# Debian/Ubuntu
sudo apt update
sudo apt install build-essential \
  curl \
  wget \
  file \
  libssl-dev

# Fedora
sudo dnf install openssl-devel \
  curl \
  wget \
  file

# Arch
sudo pacman -S base-devel \
  curl \
  wget \
  file \
  openssl
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Windows
- **Visual Studio** (2019 or later) with C++ build tools

## Development Build

### Quick Start

```bash
# Install frontend dependencies
npm run frontend:install

# Run backend + Vite dev server together
./dev.sh
```

`./dev.sh` runs the Rust backend on port `3001` and the Vite dev server on `1422`, streaming logs to `backend.log` and `frontend.log`.

Alternatively, run the two pieces yourself: start the Vite dev server with `npm run frontend:dev`, and run the backend with `layercake serve` (or `cargo run -p layercake-cli -- serve`).

### Backend Only

```bash
# Build backend
npm run backend:build

# Run backend tests
npm run backend:test
```

### Frontend Only

```bash
# Development server
npm run frontend:dev

# Production build (also embedded into the binary)
npm run frontend:build
```

## Production Build

### Single Binary

The web UI is embedded into the binary via `include_dir!`, so the frontend build MUST precede the cargo build. The `build:binary` script does both in order:

```bash
# Builds the frontend, then the release binary
npm run build:binary
```

This is equivalent to running the two steps manually:

```bash
npm run frontend:build                     # produces frontend/dist
cargo build --release -p layercake-cli     # embeds frontend/dist into the binary
```

**Output**: `target/release/layercake`

Releases ship a single native binary per platform (linux-x86_64, windows-x86_64, macos-aarch64) through GitHub releases — there are no OS-native installers.

### Running the Binary

```bash
layercake serve --open
```

`--open` auto-launches the browser. By default the server binds to loopback (`127.0.0.1:3000`) for local-first use; pass `--host 0.0.0.0` to self-host or expose it on a network. Other flags: `--port`, `--database`, `--cors-origin`.

## Build Configuration

### Environment Variables

#### Development

- `RUST_LOG`: Set log level (e.g., `debug`, `info`, `warn`)
- `VITE_API_BASE_URL`: Override backend URL (default: `http://localhost:3001`)

## Troubleshooting

### Build Failures

**Frontend not embedded / stale UI**
- The web UI is compiled into the binary from `frontend/dist`. Run `npm run frontend:build` before the cargo build (or use `npm run build:binary`, which orders them correctly).

**"cargo not found"**
```bash
# Add Cargo to PATH
source $HOME/.cargo/env
```

### Runtime Issues

**"Failed to connect to backend"**
- Check that the chosen port (default `3000`) is not in use
- Verify the database path is writable

**"Database locked"**
- Close all running instances of the server
- Delete the lock file if present
- Reinitialise the database

## CI/CD

The automated release workflow (`.github/workflows/release.yml`) runs on tag creation and produces a single native `layercake` binary per platform (linux-x86_64, windows-x86_64, macos-aarch64), published to GitHub releases. Each job builds the frontend first (`npm run frontend:build`) so the web UI is embedded into the binary, then builds the release binary with cargo. Ensure the frontend can build headlessly before tagging.

## Binary Size Optimization

### Reducing Size

**Strip debug symbols**:
```toml
# Cargo.toml
[profile.release]
strip = true
opt-level = "z"
lto = true
codegen-units = 1
```

## Distribution

- Releases ship a single native binary per platform via GitHub releases (linux-x86_64, windows-x86_64, macos-aarch64).
- Users can also install the latest binary through the install scripts (`scripts/install.sh` / `scripts/install.ps1`); see the [README](README.md) for details.

## Support

For build issues:
1. Check this document
2. Open an issue on GitHub

## Quick Reference

```bash
# Development
./dev.sh                             # Backend + Vite dev server
npm run frontend:dev                 # Frontend only
npm run backend:test                 # Run tests

# Production
npm run build:binary                 # Build frontend, then release binary
layercake serve --open               # Run the binary, open the browser

# Utilities
npm run install:all                  # Install all dependencies
npm run frontend:build               # Build frontend only
npm run backend:build                # Build backend only
```
