# CLAUDE.md - Layercake Tool Development Guide

## General guidance

- Do not have a sycophantic personality
- Use Australian English spelling

## Instructions

- docs/ARCHITECTURE.md contains the architecture, TODO.md contains the roadmap and priorities
- adr/ directory contains architecture decision records
- After all changes, ensure that the code compiles without errors and that all tests pass
- There are cargo tools available for analysis, testing, coverage reporting, dependency review, etc. use them as required and request installation of tools if they are not available
- The target platforms are Linux, macOS, and Windows. Ensure that the code is cross-platform compatible. Uses hybrid TLS: rustls for HTTP client operations, OpenSSL limited to git2 for HTTPS Git repository access
- When writing documentation, follow the guidelines in docs/writing-guide.md
- All items in a configuration file (example: config.yaml) are optional and must have a default value. This includes nested properties
- When writing files to docs/docs/reviews/ prefix each file with the date in format YYYY-MM-DD
- Do not break existing functionality when making changes

## Build/Test Commands
- Build: `cargo build`
- Run: `cargo run -- -p sample/ref/plan.yaml`
- Watch mode: `cargo run -- -p sample/ref/plan.yaml -w`
- Run tests: `cargo test`
- Run single test: `cargo test reference_exports`
- With log levels: `cargo run -- --log-level debug -p sample/ref/plan.yaml`

## Code Style Guidelines
- **Imports**: Group standard library, external crates, then internal modules
- **Formatting**: Use Rust standard formatting (rustfmt)
- **Error Handling**: Use `anyhow::Result` for most operations; propagate errors with `?`
- **Naming**: 
  - Use snake_case for variables, functions, modules 
  - Use CamelCase for types and enums
- **File Organization**: Keep related functionality in the same module
- **Documentation**: Document public APIs with /// comments
- **Type Safety**: Use strong typing and avoid unwraps when possible

## Project Structure
- CSV files define nodes, edges, and layers
- Plan files (YAML) define transformation and export steps
- Custom renderers use Handlebars templates
