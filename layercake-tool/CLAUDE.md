# CLAUDE.md - Layercake Tool Development Guide

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