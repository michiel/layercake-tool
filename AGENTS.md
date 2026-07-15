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

## Removed Surfaces
- The chat, RAG, and MCP surfaces have been removed from this workspace to keep Layercake focused on plan editing, graph tooling, data acquisition, and exports. Refer to `plans/20260122-de-feature.md` for the decision log and the remaining work history.

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
