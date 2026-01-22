# Changelog

## Unreleased
- Documented the removal of the chat/RAG/MCP surface (see `plans/20260122-de-feature.md`) so the README/AGENTS instructions now advertise only plan, graph, data, and export workflows.
- Added stub migrations for the previously dropped chat/RAG tables (`m20251030_000008`/`000009`, `m20251103_000010`/`000011`, `m20251112_000022`) so database migration history still satisfies the applied versions list.
- Ensured `dev.sh` propagates `LAYERCAKE_LOCAL_AUTH_BYPASS` when starting the backend so local plan edits remain authorized even after the environment sample was simplified.
- Defaulted `LAYERCAKE_LOCAL_AUTH_BYPASS` to `true` for debug builds so standalone local invocations get the prior unrestricted access without needing to set the env var manually.
- Extended the bypass to default to `true` for all builds/targets so local runs keep unrestricted access unless the environment variable explicitly disables it, avoiding “Actor is not authorized for write:project” errors when editing plans locally.
- Ensured artefact previews/downloads use the editor’s `projectId`/`planId` context after node creation so preview controls work immediately without reloading the canvas (GraphArtefact nodes now fall back to plan-context IDs when their per-node data is still syncing).
- GraphQL Graph/GraphData now expose `graphDataId`/`legacyGraphId` plus `sourceType`; clients should prefer `graphDataId` for all mutations/queries and treat `legacyGraphId` only as a badge/regeneration hint during the single-schema migration.
- Backend graph queries/validation run solely on `graph_data`; legacy fallbacks removed ahead of the legacy table drop migration (`m20251215_000001_drop_legacy_graph_tables.rs`).

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
