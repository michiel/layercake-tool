# Layercake RAG / Data Acquisition Implementation Plan

## Objectives
- Provide a per-project Retrieval Augmented Generation (RAG) surface that lets agents consume curated project knowledge without duplicating rig-core functionality.
- Introduce a dedicated data acquisition crate that owns file ingestion, tagging, knowledge base (KB) management, and dataset generation flows.
- Persist raw files, embeddings, and vector indices in the existing SeaORM-managed SQLite database so every runtime modality (CLI, API, Tauri) can access the same context.
- Deliver a tagged knowledge graph that unifies raw files, generated datasets, and existing project graph nodes/edges for downstream DAG execution.

## Scope & Boundaries
- **In scope:** new workspace crate (`layercake-data-acquisition`), SeaORM entities/migrations for files/tags/vector store, ingestion/parsing pipelines, rig-backed embedding + RAG orchestration, backend APIs, frontend (Source management, Knowledge base, Data set creation) views, tests and docs.
- **Out of scope:** vendor-specific hosted vector databases, cross-project knowledge sharing, generalized document redaction, or authz changes beyond tag visibility within a project.
- **Assumptions:** legacy “Data source” entities have already been renamed to “Data sets”; rig-core OpenAI provider remains available; SQLite DB size is acceptable for initial release.

## Architecture Overview
- **Workspace crate:** `layercake-data-acquisition` under `layercake-core/` workspace. Exposes feature-gated modules for ingestion, tagging, vector store management, and dataset generation. Re-exports integration traits consumed by CLI/API/Tauri layers.
- **Database layer (SeaORM):**
  - `files` table: id (uuid), project_id, filename, media_type, size_bytes, blob (BLOB), checksum, created_by, created_at.
  - `file_tags` join table.
  - `tags` table with scope enum (file/dataset/node/edge) + color metadata.
  - `kb_documents` table: file_id nullable (for synthetic entries), chunk_id, chunk_text, metadata (JSON), embedding (vector as BLOB), embedding_model, created_at.
  - `vector_index_state` table to track versioning/rebuild state per project + semantic search configuration.
  - Additional joins for dataset/node/edge tags referencing existing tables without duplicating entities.
- **Services:**
  - `IngestionService`: handles upload, dedupe, chunking, and dispatch to parsers (`text`, `markdown`, `csv`, `pdf`, `odf`, `ods`, `xlsx`, `docx`). Uses pluggable parser trait allowing future types.
  - `EmbeddingService`: wraps rig `EmbeddingsBuilder`, streams chunks, batches embeddings, writes to `kb_documents`.
  - `VectorSearchService`: rig vector store backed by the same SQLite tables, exposed as `VectorStore` impl reused by agents.
  - `TagService`: CRUD for tags + association helpers for files/datasets/nodes/edges.
  - `DatasetGenerator`: uses rig context RAG agent templates/prompts to author layercake datasets, persists them as standard project datasets.
- **APIs & Surfaces:**
  - CLI commands + API endpoints under `/projects/:id/data-acquisition/...`.
  - Tauri commands bridging to the crate’s services.
  - Frontend pages:
    - `Source Management`: upload/list files, tagging UI, select files for embedding.
    - `Knowledge Base`: show vector store stats, rebuild/clear controls, search previews.
    - `Data Set Creation`: prompt library, run RAG-based dataset generation jobs, tag outputs.

## Detailed Plan & Timeline
| Phase | Duration | Deliverables |
| --- | --- | --- |
| **0. Prerequisite validation** | 0.5 week | Confirm “Data sets” rename migrations are merged; audit existing rig-core usage; document DB size and retention expectations. |
| **1. Crate & schema foundation** | 1 week | Scaffold `layercake-data-acquisition` crate (lib + feature flags). Add SeaORM entities and migrations for `files`, `tags`, joins, `kb_documents`, `vector_index_state`. Update workspace manifests and CI to build/test the new crate. |
| **2. File ingestion pipeline** | 1.5 weeks | Implement upload APIs (CLI/API/Tauri) writing BLOBs to SQLite with streaming to avoid memory spikes. Build parser trait with adapters: plain text/markdown via `pulldown_cmark`, CSV via `csv` crate, spreadsheets via `calamine`, PDFs via `pdf_extract` (stubbing for binary formats until libs chosen). Add checksum dedupe & tagging. Unit + integration tests with fixture files under `sample/`. |
| **3. Embedding & vector store integration** | 1 week | Implement rig-backed `EmbeddingService`, chunking heuristics, batching, and persistence to `kb_documents`. Build SQLite-backed `VectorStore` impl (or adapt rig’s existing connectors) with configurable similarity search parameters. Expose background jobs for reindexing and UI-triggered rebuild/clear. |
| **4. Knowledge base + tagging UX** | 1 week | Backend endpoints for listing KB docs, search previews, status metrics. Frontend components for Source Management + Knowledge Base sections, including tag management modals. Wire to Tauri commands and ensure CLI parity. |
| **5. Dataset generation workflows** | 1 week | Create prompt templates (stored under `resources/`), define `DatasetGenerator` service that composes rig context agents with selected files/tags, writes outputs to project datasets. Build UI for prompt selection, run history, and dataset tagging. Add scenario tests ensuring generated datasets feed the plan DAG. |
| **6. Hardening & release** | 0.5 week | Full `cargo fmt/clippy`, `npm run backend:test`, `npm run frontend:build`, golden-file updates, documentation (README, IMPLEMENTATION, samples). Load-test ingestion on representative corpus, confirm SQLite file size, add metrics/logging hooks. |

Total estimated duration: ~6.5 weeks, assuming two engineers (backend + frontend) working in parallel from Phase 2 onward.

## Technical Notes & Suitability
- **Single-DB approach:** Keeping files, embeddings, and vector metadata in the project SQLite DB guarantees portability for CLI-driven workflows and keeps Tauri packaging simple. Rig’s vector store traits operate over any backend; implementing a SQLite adapter avoids new infra.
- **Tagging model:** Normalized `tags` table with scoped joins allows consistent filtering in graph editors and dataset selectors. Reuse existing `Taggable` patterns if present; otherwise introduce trait-based helpers in the new crate.
- **Parser strategy:** Start with open-source crates already permitted in the project; encapsulate each parser behind a `DocumentAdapter` so future formats (HTML, PPTX) can be added without touching ingestion flow.
- **Background work:** Embedding jobs can run via existing async runtime (Tokio) inside the crate, exposed as tasks triggered by CLI/API/Tauri. Use job status rows in `vector_index_state` to resume/retry.
- **Frontend:** Extend Vite React app with new routes under `frontend/src/pages/data-acquisition`. Reuse component primitives for tables/forms/tags; coordinate state via existing query hooks (e.g., React Query) or add ones if missing.

## Risks & Mitigations
- **Large file storage growth:** PDFs/Office files stored as BLOBs can bloat SQLite. Enforce per-file + per-project quotas, surface usage in UI, and consider compression before persistence.
- **Parser reliability/licensing:** Some formats (odf/office) require heavy dependencies; vet crates for license compatibility and sandboxing. Provide graceful degradation (skip + alert) when parsers fail.
- **Embedding cost & latency:** OpenAI embedding throughput may bottleneck. Batch chunk uploads, cache embeddings via checksum, and allow offline providers (e.g., local models) via rig provider abstraction.
- **Concurrency & locking:** Simultaneous rebuilds could lock SQLite. Use advisory locking rows in `vector_index_state` and run long-running jobs outside Tauri’s main thread.
- **Security/privacy:** Uploaded files may hold secrets; ensure rest encryption (if configured) and tag-based access control align with existing project permissions. Audit logs for upload/download actions.

## Alternatives & Trade-offs
- **External vector DB (Qdrant, Pinecone):** Offers scalability and ANN features but adds infra + network latency, conflicting with offline requirements. Consider as future opt-in backend once the SQLite path stabilizes.
- **Filesystem storage for files:** Would reduce DB size but complicate portability/backups. Current choice favors simplicity; revisit if SQLite proves insufficient.
- **Dedicated tagging service:** Could keep tags decoupled from data acquisition, but embedding tagging alongside ingestion simplifies UX and ensures synchronous updates with files/datasets.

## Validation Strategy
- **Automated:** Unit tests for parsers, SeaORM entity round-trips, embedding batching logic. Integration tests under `layercake-core/tests/` that ingest fixture files, run embeddings against a mocked provider, and query vector search results.
- **Manual:** Run `cargo run --bin layercake -- -p sample/kvm_control_flow_plan.yaml` with new flags to ingest sample docs, verify dataset generation flows. UI smoke tests via `npm run frontend:dev` + screenshot capture. Tauri packaging dry-run for macOS/Linux to ensure SQLite migrations run correctly.
- **Metrics/Observability:** Instrument ingestion duration, chunk counts, embedding latency. Emit events to existing logging infrastructure for auditability.

## Documentation & Rollout
- Update `docs/` with user guides for Source Management, Knowledge Base, and Data Set Creation workflows.
- Extend `sample/` plans and `resources/` templates to demonstrate RAG-powered dataset generation.
- Provide migration notes (DB changes, environment variables for embedding providers) and feature flags for staged rollout.
