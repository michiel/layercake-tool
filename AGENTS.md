# Repository Guidelines

## Project Structure & Module Organization
- `layercake-core/` holds the Rust workspace crate for the CLI, API layers, and pipeline modules under `src/`.
- `src-tauri/` wraps the backend into a Tauri shell and owns desktop configuration (`tauri.conf.json`, icons).
- `frontend/` contains the Vite + React/TypeScript UI; shared assets live under `frontend/src`.
- `external-modules/` packages optional integrations such as `axum-mcp`; reuse them instead of duplicating code.
- `sample/`, `resources/`, and `docs/` provide reference plans, template assets, and architecture notes used in reviews.

## Build, Test, and Development Commands
- `cargo run --bin layercake -- -p sample/kvm_control_flow_plan.yaml` runs the CLI against a known plan.
- `npm run backend:build` / `npm run backend:test` wrap `cargo build` and `cargo test -p layercake-core`.
- `npm run frontend:dev` starts the React app with Vite; `npm run frontend:build` performs type-checking and bundling.
- `npm run tauri:dev` launches the desktop shell; `npm run tauri:build` produces release binaries per OS script.
- `npm run frontend:install` bootstraps UI dependencies; rerun after upgrading packages.

## Coding Style & Naming Conventions
- Format Rust with `cargo fmt` and lint using `cargo clippy --all-targets --all-features` before submitting.
- Keep Rust modules snake_case and public types CamelCase; prefer `crate::` imports for internal modules.
- React components and hooks live under `frontend/src`; use PascalCase for components, camelCase for hooks/utilities.
- Align JSON, YAML, and Handlebars templates in `resources/` with two-space indentation; avoid trailing whitespace.

## Testing Guidelines
- Place Rust integration tests in `layercake-core/tests/`; mimic the patterns in `graph_edit_replay_test.rs`.
- Regenerate golden files under `layercake-core/tests/reference-output/` only when behavior changes are intentional.
- For UI changes, run `npm run frontend:build` and attach before/after screenshots or GIFs of affected flows.
- Cover new pipeline or plan stages with scenario tests exercising CSV/TSV fixtures under `sample/`.

## Commit & Pull Request Guidelines
- Follow the existing Conventional Commit style (`feat:`, `fix:`, `refactor:`) with concise, imperative summaries.
- Reference related issues or specs (e.g., `SPECIFICATION.md`) and note schema migrations or template updates.
- PRs should detail the motivation, list manual test commands executed, and include screenshots for UI-facing changes.
- Update relevant docs (README, IMPLEMENTATION.md, samples) whenever behavior or developer workflows shift.
