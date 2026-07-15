# Web-First: Remove Tauri, Ship a Single Binary with Embedded Web UI

**Date:** 2026-07-15
**Branch:** `feat/web-first-remove-tauri`
**Goal:** Remove the Tauri desktop shell and instead ship the existing web UI embedded inside the `layercake-server` binary. The binary serves the SPA same-origin over HTTP; it works both as a local desktop replacement (bind loopback, optionally auto-open browser) and as a self-hosted server (`--host 0.0.0.0`).

## Context / Findings

The codebase is already **React SPA ↔ Axum GraphQL server**. Tauri is only a desktop launcher on top.

- `layercake-server/src/main.rs` already boots the Axum server standalone (`--port`, `--database`, `--cors-origin`).
- `layercake-server/src/server/mod.rs:45` binds `0.0.0.0:{port}` (hard-coded today).
- `layercake-server/src/server/app.rs` already serves SPAs from disk via `ServeDir`/`ServeFile` (projections viewer) — template for serving the main UI.
- `include_dir` is already a workspace + `layercake-server` dependency, so embedding adds no new crates.
- Frontend `getGraphQLEndpoints()` already has a non-Tauri "web mode" path (`VITE_API_BASE_URL` or `http://localhost:3001`).
- **The server never enforces `x-tauri-secret`** — only reads `x-layercake-session`. So dropping the secret is frontend-only cleanup, no server auth removal needed.
- Removing Tauri deletes **113 of 583 workspace crates (~19%)** — the GTK/WebKit/glib/wry/objc stack + Tauri codegen. Big win on cold/CI builds, negligible on warm incremental.

### Tauri reference sites (frontend)
- `frontend/src/main.tsx` — Tauri init branch (keep web-mode branch only).
- `frontend/src/graphql/client.ts` — `initializeTauriServer`, `getServerInfo`/`isTauriApp`/`waitForServer`, secret auth link.
- `frontend/src/utils/tauri.ts` — delete entirely.
- `frontend/src/components/settings/DatabaseSettings.tsx` — entirely Tauri IPC (`get_database_info`, `reinitialize_database`, `show_database_location`). No web equivalent — needs decision.
- Projection-window openers (Tauri `WebviewWindow` branch + `window.open` fallback):
  - `frontend/src/pages/workbench/ProjectionsPage.tsx`
  - `frontend/src/pages/ProjectArtefactsPage.tsx`
  - `frontend/src/components/editors/PlanVisualEditor/nodes/ProjectionNode.tsx`
- `__TAURI__` UI toggles (hide/show buttons): `ControlPanel.tsx`, `NodeToolbar.tsx`, `PlanVisualEditor.tsx`, `AdvancedToolbar.tsx`.

### CI / build
- `.github/workflows/tauri-build.yml` — delete entirely.
- `.github/workflows/release.yml` — remove tauri-cli install + `cargo tauri build` + tauri artifact-collection block; keep the plain cross binary build; add frontend build **before** cargo build.
- Root `package.json` — remove `tauri:*` scripts, `tauri`/`desktop` keywords.
- `scripts/build-{linux,macos,windows}.sh` — Tauri bundlers, delete.

## Decisions

- **Deployment:** both local + self-hosted. Default bind `127.0.0.1`; `--host` to override.
- **Assets:** embedded via `include_dir!` (single self-contained binary).
- **Local auth secret:** dropped (loopback default). Note where auth would slot in if exposed.

---

## Stage 1: Embed & serve the main web UI (additive)
**Goal:** `layercake-server` serves the built `frontend/dist` SPA embedded in the binary, same-origin with the API.
**Success Criteria:** `npm run frontend:build` then `cargo run -p layercake-server -- --port 3001`, open `http://localhost:3001`, app loads and talks to GraphQL. API routes (`/graphql`, `/health`, `/api/*`, `/projections/*`) still resolve; unknown paths return `index.html` (client routing works).
**Tests:** manual smoke via `/run` or curl for `/health` + `/` returning HTML; existing server tests still pass.
**Status:** Complete

**Implementation notes:**
- Added `layercake-server/src/server/static_assets.rs` embedding `../frontend/dist` via `include_dir!` (0.6.2 — literal relative path, no `$CARGO_MANIFEST_DIR` expansion), with `spa_fallback` handler that serves the matched asset or falls back to `index.html`.
- Added `mime_guess = "2.0"` to `layercake-server/Cargo.toml` for content-type detection.
- Registered `pub mod static_assets` in `server/mod.rs`; wired `.fallback(spa_fallback)` onto the final router in `app.rs` (after all API/asset routes, so they take priority).
- Verified: `/health`→JSON, `/`→`text/html` index, `/assets/*.js`→`text/javascript`, unknown SPA route→index.html, `/graphql` GET+POST still route correctly (not swallowed by fallback).
- Note: `include_dir!` requires `frontend/dist` to exist at compile time — CI must run `npm run frontend:build` before `cargo build` (handled in Stage 4).

## Stage 2: Origin-relative frontend + drop Tauri branches
**Goal:** Frontend defaults to same-origin relative endpoints; all Tauri code paths removed.
**Success Criteria:** `tsc --noEmit` passes with zero `@tauri-apps/*` imports remaining; `main.tsx` renders web-mode only; projection openers use `window.open`; `client.ts` has no secret link; `VITE_API_BASE_URL` still overrides for dev.
**Tests:** `tsc --noEmit`; `grep -r "@tauri-apps\|__TAURI__\|isTauriApp" frontend/src` returns nothing.
**Status:** Complete

**Implementation notes:**
- `graphql/client.ts`: replaced Tauri server discovery with `getApiBaseUrl()` → `window.location.origin` (same-origin), `VITE_API_BASE_URL` override for dev. Removed `initializeTauriServer`, secret auth header, ws secret param. Endpoints now relative.
- `main.tsx`: removed the Tauri init branch + loading/error states; renders web-mode only.
- Deleted `utils/tauri.ts` and `utils/tauri-api-mock.js`.
- Projection viewer openers (`ProjectionsPage`, `ProjectArtefactsPage`, `ProjectionNode`): collapsed to the `window.open` web path using `window.location.origin`.
- `__TAURI__` UI toggles removed (`ControlPanel`, `NodeToolbar`, `AdvancedToolbar`, `PlanVisualEditor`): `draggable` always on; drag/drop guards removed. Removed the Tauri-only pointer-drag machinery (`handleNodePointerDragStart` now a no-op kept for the `AdvancedToolbar` prop; deleted `handlePointerDrop`, `draggingNode` state + preview overlay + unused `Card` import).
- **DatabaseSettings removed** (per decision): deleted component, its `/settings/database` route, and the nav link in `App.tsx`.
- `vite.config.ts`: dropped Tauri detection + mock alias; dev proxy now covers `/api`, `/graphql` (ws), `/projections` (ws) so dev uses same-origin relative endpoints.
- `frontend/package.json`: removed `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-shell`.
- Verified: `tsc --noEmit` clean; `npm run build` succeeds; grep for tauri in `frontend/src` empty; rebuilt server serves fresh embedded UI; same-origin `POST /graphql` returns `{"projects":[]}`; removed route falls through to SPA; no server panic. (Real-browser drive skipped — no Chrome in env; covered via curl.)
- **Follow-up:** file bd issue for possible GraphQL reimplementation of DB info/reinitialise (Stage 4 / session close).

## Stage 3: Local-first launch ergonomics
**Goal:** Server binds loopback by default, external opt-in, optional browser auto-open.
**Success Criteria:** `--host` flag (default `127.0.0.1`), `--port` retained; bind uses `{host}:{port}`; optional `--open` launches browser.
**Tests:** run locally → binds `127.0.0.1`; `--host 0.0.0.0` binds externally; `cargo build -p layercake-server` clean.
**Status:** Complete

**Implementation notes:**
- `main.rs` + CLI `Serve`: added `--host` (default `127.0.0.1`) and `--open`; `start_server` signature now `(host, port, database, cors_origin, open_browser)`.
- `server/mod.rs`: bind `{host}:{port}` (was hard-coded `0.0.0.0`); log a clickable loopback URL even when bound to `0.0.0.0`/`::`; added best-effort `open_in_browser` (xdg-open/open/cmd start, non-fatal on failure).
- Updated both callers (`layercake-server/main.rs`, `layercake-cli` serve).
- Verified: `--help` shows flags; default bind reachable only on loopback (external IP → unreachable); `--host 0.0.0.0` reachable externally + loopback; URL log shows `127.0.0.1`. `--open` not runtime-tested (no browser in env; path is non-fatal).

## Stage 4: Remove Tauri crate + CI/build cleanup
**Goal:** `src-tauri` gone; workspace + CI build without any Tauri toolchain.
**Success Criteria:** `src-tauri/` deleted, removed from workspace `members`; root `package.json` + `scripts/` Tauri bits removed; `tauri-build.yml` deleted; `release.yml` builds plain binary with frontend built first; `cargo build --workspace` succeeds with no Tauri crates in `cargo tree`.
**Tests:** `cargo build --workspace`; `cargo tree --workspace | grep -i tauri` empty; single binary runs and serves UI.
**Status:** Not Started

## Open Questions / Notes
- **DatabaseSettings.tsx**: **DECIDED — remove** the component + its route/menu entry (option a). Relies on Tauri-only IPC (`get_database_info`, `reinitialize_database`, `show_database_location`) with no web equivalent. Self-hosted admins manage the SQLite file directly. File a bd issue to track possible GraphQL reimplementation later.
- Auth: no gate in embedded/local mode. If `--host 0.0.0.0` is used in the wild, a token/reverse-proxy is the user's responsibility for now; note in docs.
