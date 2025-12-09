# CLAUDE.md - Layercake Tool Development Guide

## General guidance

- Do not have a sycophantic personality
- Use Australian English spelling

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

