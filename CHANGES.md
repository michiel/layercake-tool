# Changelog

## v0.4.0 - 2026-07-16

### CLI & self-update
- `layercake update` installs the latest release for your platform: it resolves the correct archive by name (skipping `.sha256`/signature sidecars so it never mistakes a checksum file for the binary), verifies the checksum, and replaces the binary in place. Supports `--check`, `--force`, `--pre-release`, `--backup`/`--rollback`, and `--dry-run`.
- Added a `linux-aarch64` release target (`layercake-<ver>-linux-aarch64.tar.gz`) alongside `linux-x86_64`, `macos-aarch64`, and `windows-x86_64`; the installer script and updater already resolve it automatically.
- Added the JSON-first `layercake query` command (and shared GraphQL helpers) so datasets, plans, DAG nodes/edges, and export previews can be managed by CLI or automation using canonical `dataset:/plan:/plannode:` identifiers.
- Added the `layercake repl` shell that keeps project/plan context, parses simple commands (`set`, `list nodes`, `create node`, `download export`, etc.), and prints structured JSON responses so interactive agents can operate without the browser.
- `layercake doctor` resolves the database from a running server's `/health`, opens it read-only, refuses a missing/no-tables database with a clear message, and supports `--strict` for CI.
- `layercake api info` verifies the server via `/health`, reports the server's absolute database path, and detects live ports; `layercake schema type <Name>` gained `--only-inputs`/`--only-mutations`.

### Server & data model
- Backend graph queries/validation run solely on `graph_data`; legacy graph tables were dropped (`m20251215_000001_drop_legacy_graph_tables.rs`). GraphQL Graph/GraphData expose `graphDataId`/`legacyGraphId` plus `sourceType`; prefer `graphDataId` for all mutations/queries.
- The server warns loudly (with the resolved absolute path) before creating a new database file, so running from the wrong directory no longer silently creates a stray empty database.
- Node execution timings carry an `ExecutionPhase` (Execute/Render), exposed as `NodeExecutionTiming.phase`.
- Added `diffDatasets` (structural graph diff), `renamePlanDagNode`, `exportProject`, `previewStoryContext`, `pruneOrphanedGraphs`, `applyPalettePreset`, and palette-preset/WCAG-contrast queries.

### Stories, sequences & rendering
- Stories can source sequences from computed graphs via `enabledGraphIds`, with a computed-graph source picker in the UI.
- Switching a sequence artefact's render target between Mermaid and PlantUML now keeps the output-path extension in sync, and the server validates `renderTarget`↔extension so mismatched outputs are rejected. Neutralised a `;` that broke the Mermaid sequence grammar.
- Loading a deleted projection in the viewer now shows a clean "not found" state instead of a top-level GraphQL error (`projectionGraph`/`projectionState` return null for a missing projection).

### Frontend
- Upgraded Mermaid to v11 and migrated to Apollo Client 4.2 (typed `TypedDocumentNode` documents throughout). Removed unused dependencies (assistant-ui, top-level react-dnd, stale `@types`). Fixed a latent bug where graph node/edge `attributes` were silently dropped on add/update.

### Removed features & auth
- Removed the chat/RAG/MCP surface (see `plans/20260122-de-feature.md`); added stub migrations so migration history still satisfies the applied-versions list.
- `LAYERCAKE_LOCAL_AUTH_BYPASS` defaults to `true` for local runs (unless explicitly disabled), avoiding "Actor is not authorized for write:project" errors when editing plans locally; `dev.sh` propagates it to the backend.

## v0.3.6 - 2025-11-26
- Added full Story and Sequence authoring workflows (GraphQL, UI editor, artefact previews, export fixes) so workbench projects can narrate execution paths alongside graph artefacts.
- Expanded dataset management with validation workflows, streamlined editors (copy/paste helpers, prompt library previews), merge/split helpers, and better metadata so pipelines can curate graph bundles without leaving the app.
- Overhauled project palette/layer tooling with clipboard import/export, alias management, enforced palette coverage, dataset-provided palettes, and a new “Generate palette” action that seeds auto-colored sources.
- Refined plan/workbench UX with reorganized overview cards, multi-plan navigation + DAG operations, partition-aware drop/connect behavior, and improved drag-to-connect menus.
- Improved export/knowledge-base capabilities by surfacing knowledge assets, fixing PlantUML + Mermaid rendering gaps, adding desktop plan lists, and keeping story/sequence nodes in sync across downloads.
- Delivered dozens of quality fixes (node label lookups, dataset comments, partition flags, plan notes, dataset toggles) and test coverage to keep archives, graph exports, and template rendering reliable.

## v0.2.0 - 2025-02-15
- Add Tauri desktop packaging scripts and cross-platform build instructions.
- Document repository guidelines for new contributors.
- Wire CI to produce platform bundles with required Linux dependencies.

## v0.1.x - 2024
- Initial releases of the Layercake CLI and collaboration backend.
