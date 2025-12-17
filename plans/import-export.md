# Import/Export Enhancements Plan

## Objectives & Scope
- Expand project import/export to support both ZIP-based and filesystem-based flows with explicit user choices.
- Persist an optional import/export "connection" path on a project to enable one-click re-import/re-export.
- Ensure filesystem exports cleanly mirror current project assets (additions, updates, deletions) while preserving non-Layercake files and dotfiles.

## Assumptions / Constraints
- Existing ZIP export/import remains the default path; new filesystem options should reuse the same serialization format (project bundle layout).
- Desktop (Tauri) and web builds share the same GraphQL surface; filesystem paths are only allowed on trusted/desktop contexts.
- Dotfiles (e.g., `.git`) and files not previously written by Layercake must never be modified or removed during exports.
- Export bundle content is deterministic; we can compute the expected file set given a project ID without side effects.

## Data Model & API Surface
1) **Project connection path**
   - Add nullable `import_export_path` (text) to `projects` table with migration + SeaORM model update.
   - Expose on GraphQL `Project` type and allow updates via a dedicated mutation (e.g., `setProjectImportExportPath(projectId, path, keepConnection: Boolean)`).
2) **Import/export operations**
   - Extend existing export resolver/service to accept `mode: ZIP | FILESYSTEM` and `targetPath` (for filesystem).
   - Add `reimportProject(projectId)` and `reexportProject(projectId)` mutations that use the stored `import_export_path`; validate presence and accessibility.
   - Ensure existing import/export mutations remain backward compatible (default ZIP behavior when mode omitted).

## Backend Implementation Plan (layercake-core)
1) **Migration & models**
   - Create migration adding `import_export_path` column with index (nullable).
   - Update `projects` SeaORM entity/model and GraphQL types to surface the field.
2) **Service layer**
   - Refactor project export service to support a filesystem target:
     - Accept `ExportTarget::Zip(File)` or `ExportTarget::Directory(PathBuf)`.
     - When directory mode: create directory if missing; build list of files to write (relative paths); before writing, remove previously exported files that are absent from the new set while skipping dotfiles and non-Layercake files.
     - Implement "previous exports" detection by scanning the target dir for known bundle paths (plans/, datasets/, graphs/, metadata files) and deleting only those not in the new bundle; leave unknown/dotfiles untouched.
   - Add project import service support for directory source:
     - Read bundle structure from a directory (same layout as unzipped export).
     - Honor `keep_connection` flag: when true, persist source directory path on the project record.
     - Validate directory readability and bundle completeness; error with structured codes for UI handling.
   - Implement reimport/reexport helpers that validate stored path, existence, and permissions; reuse import/export logic.
3) **GraphQL resolvers**
   - Extend import/export mutations to accept `mode`, `path`, and `keepConnection`.
   - Add mutations for `setProjectImportExportPath`, `reimportProject`, and `reexportProject`.
   - Add `importExportPath` to `Project` query type.
4) **Validation & security**
   - Restrict filesystem paths to absolute paths and optional allowlist/`BASE_DIR` if configured.
   - Prevent traversal outside allowed roots; sanitize inputs.
   - Add logging and error surfaces for permission errors, missing path, or partial bundle detection.

## Frontend Plan (Vite/React)
1) **UI: project export**
   - Replace current "Export" dropdown with:
     - `Export ZIP`
     - `Export to filesystem` (opens path picker / text input).
     - `Export as template` (existing).
   - For filesystem export, collect target directory, call GraphQL mutation with `mode=FILESYSTEM`.
2) **UI: project import**
   - Change Import Project CTA to a split menu:
     - `Import (ZIP)` (existing file picker)
     - `Import (Filesystem)` with directory selector and checkbox `Keep connection`.
   - On success with `keep_connection`, update project view state to show the linked path.
3) **UI: connected actions**
   - On Project page, when `importExportPath` is set, show buttons:
     - `Re-import from source`
     - `Re-export to source`
   - Display the path (truncate with tooltip) and a "Change/Remove connection" action that drives `setProjectImportExportPath`.
4) **GraphQL client**
   - Add new mutations/fields to Apollo ops/types (codegen update if applicable).
   - Handle error cases (path missing, permission denied) with toasts or inline alerts.

## Tauri/Desktop Considerations
1) Implement directory picker using Tauri dialog; for web, fallback to text input or disable filesystem mode if unsupported.
2) Ensure backend is invoked locally (no remote path exposure) when running in desktop shell.

## CLI (optional)
- Add `layercake project export --mode filesystem --path <dir>` and `import --mode filesystem --keep-connection`.
- Add `project reimport` / `project reexport` commands that use the stored path.

## Testing & Verification
- Unit tests for export service: ZIP vs directory modes, deletion of stale bundle files, preservation of dotfiles/foreign files.
- Integration tests for GraphQL mutations (import/export/reimport/reexport) using temp dirs.
- Frontend e2e/smoke: menu renders new options, keeps connection flag, reimport/reexport buttons show when connected.
- Manual test on sample DB with directory containing extra files to confirm no dotfiles or unknown files are touched.

## Rollout Notes
- Migration introduces nullable field; no downtime expected.
- Document new flags in README/BUILD and add UI help text explaining connection behavior and safe directory expectations.
