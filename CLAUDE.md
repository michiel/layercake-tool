# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test Commands
- Rust tool: `cargo build`, `cargo test`, `cargo run -- -p sample/ref/plan.yaml`
- Run single test: `cargo test reference_exports`
- Watch mode: `cargo run -- -p sample/ref/plan.yaml -w`
- UI dev server: `cd ../layercake-ui && npm run dev`
- UI lint: `cd ../layercake-ui && npm run lint`

## Code Style Guidelines
- **Rust**: 
  - Use snake_case for functions/variables, CamelCase for types
  - Group imports: std, external crates, then internal modules
  - Use anyhow::Result, propagate errors with ?
  - Avoid unwraps when possible
- **TypeScript/React**:
  - Use strict typing with TypeScript
  - Follow ESLint recommended rules
  - Use functional components with hooks

## Project Structure
- Tool: CSV files define nodes/edges/layers, plan.yaml defines transformations
- UI: React/TypeScript with Apollo GraphQL client
- Renderer outputs: dot, gml, plantuml, csv, custom templates