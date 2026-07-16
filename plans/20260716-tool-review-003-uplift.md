# Uplift plan — tool-review-003

**Date:** 2026-07-16
**Source:** `reviews/tool-review-003.md` (third pass, project 36). Mostly green — verifies the two prior uplifts landed. Small actionable set; N3/N9 is the one real regression.
**Approach:** structural over patches. Validate every finding against master first (done below).

## Validation (against master)

| # | Finding | Verdict | Notes |
|---|---------|---------|-------|
| **N3/N9** | `/health` + `api info` report a **relative** DB path | **CONFIRMED, the one real bug** | `health.rs` echoes `state.database_path` (raw `--database` literal). `doctor --port N` from another cwd then resolves it relative → creates a wrong empty DB → "no such table: plans". |
| **N3b / Rec2** | doctor uses `mode=rwc` (creates empty DB) and doesn't check for `plans` | **CONFIRMED** | `doctor.rs` connects via `get_database_url` (rwc) with no table check. |
| **N14** | `renamePlanDagNode` SDL comment doesn't state collision/idempotency/WS semantics | **CONFIRMED** | Behaviour is correct (rejects collision, no-op on equal — tested in FR2), just undocumented in SDL. |
| **N12** | `NodeExecutionTiming` has no `phase`; artefact nodes read 0ms (lazy render) | **CONFIRMED** | `helpers.rs` — only nodeId/nodeType/durationMs. Add `phase` to disambiguate execute vs render. |
| **N13** | No `guide graph-lifecycle` | CONFIRMED (docs) | dataset→node→graph_data→artefact lifecycle only in commit msgs. |
| **N10** | checkContrast + quoting note | CONFIRMED (docs) | one line in FR4/palette doc. |
| **Rec3** | `applyPalettePreset` mutation | CONFIRMED (gap) | `palettePresets` exists; no one-shot apply. Low priority but self-contained + high-value. |
| **Rec4** | agent-runbook "common pitfalls" section | CONFIRMED (docs) | turn every review bug into a runbook line. |
| **Rec5** | structured warnings (`type Warning{code,message,nodeId,severity}`) | Deferred | "not urgent — free strings usable". Larger; evaluate, likely defer. |
| **N11** | swatch contrast only for its own bg/text pair | Won't-fix (by design) | reviewer agrees it's fine. |
| **FR3b** | CLI story shorthands (`story list`) | Deferred | tracked; UI exists. |
| **doctor --fix** | remediation | Deferred | issue #89 already filed; could call pruneOrphanedGraphs. Evaluate. |

## Stages

### Stage 1 — N3/N9: report the absolute DB path [CORRECTNESS, the real fix]
Canonicalise the database path server-side so `/health` and `api info` report an absolute path.
- `health.rs`: report `std::fs::canonicalize(state.database_path)` (fall back to the raw value if the file doesn't exist yet), so `doctor --port N` resolves the right file from any cwd.
- `api info`: same — print the absolute DB path.
**Status:** Not Started

### Stage 2 — N3b/Rec2: doctor refuses a no-tables / non-layercake DB [CORRECTNESS]
- When doctor auto-resolves a DB (no explicit `--database`), open **read-only** so it never creates an empty file.
- Before running checks, verify the DB has a `plans` table (or similar sentinel); if not, error "this doesn't look like a layercake database (no 'plans' table): <path>" rather than propagating the raw SQLite error.
**Status:** Not Started

### Stage 3 — N12: phase on NodeExecutionTiming [CLARITY]
Add `phase: ExecutionPhase` (`Execute` | `Render`) to `NodeExecutionTiming` (artefact nodes render lazily on export, so they read 0ms during executePlan — mark them `Render`/skipped so the 0 isn't misleading). Or, minimally, a doc comment + a boolean. Prefer the enum.
**Status:** Not Started

### Stage 4 — N14: renamePlanDagNode SDL comment [DOCS/SCHEMA]
Expand the mutation's doc comment to state: rejects a colliding `newId`; no-op when `newId == oldId`; rewrites incident edges + computed-graph origins. (WS-event parity: verify whether it bumps plan version like updatePlanDagNode — it does via `bump_plan_version`; note it.)
**Status:** Not Started

### Stage 5 — Rec3: applyPalettePreset mutation [FEATURE, self-contained]
`applyPalettePreset(projectId, presetName): [ProjectLayer!]!` — upsert the preset's swatches as project layers in one call. Reuses `palette::presets()` + the existing layer upsert. Closes the "one-click apply" gap.
**Status:** Not Started

### Stage 6 — Docs: graph-lifecycle guide + runbook pitfalls + N10 [DOCS]
- `doc guide graph-lifecycle` (N13): dataset → DataSetNode → GraphNode/Merge/Transform → `graph_data` row → GraphArtefactNode → exported bytes; created/invalidated/pruned per stage; ties in `pruneOrphanedGraphs` + doctor.
- agent-runbook "Common pitfalls" (Rec4): empty diagram → doctor; `.mmd` contains `@startuml` → renderTarget drift (now validated); mmdc rejects output → semicolons (now neutralised); wrong-cwd doctor → absolute path now reported.
- N10: one line in the palette/FR4 doc about quoting hex via `--variables`.
**Status:** Not Started

## Deferred / file as beads
- Rec5 structured warnings — evaluate; likely a bead (behaviour-shaping, larger).
- doctor --fix (#89), FR3b CLI shorthands, dataset↔graph diff (bead layercake-tool-0li) — carry.
- N11 won't-fix.

## Guiding principle
Structural: fix the seam (absolute path at the source, not per-caller patching); make failures visible (doctor refuses a bad DB instead of a cryptic SQLite error). N3/N9 is the priority — it's the one thing keeping the reviewer from "all green".
