# Data Acquisition / RAG Implementation Notes

## Crate Layout
- `layercake-data-acquisition` is a dedicated workspace crate that houses:
  - `services::DataAcquisitionService` – orchestrates ingestion, tagging, embedding, vector search, and dataset generation.
  - `ingestion` – pluggable parsers (`text/plain` enabled by default) with `DocumentChunk` abstractions.
  - `embeddings` – thin wrapper over rig/openai embedding models.
  - `vector_store::SqliteVectorStore` – persists embeddings inside the main SeaORM SQLite database (table `kb_documents`).
  - `entities` – SeaORM models for `files`, `tags`, join tables, `kb_documents`, and `vector_index_state`.
  - `dataset_generation` – prompt driven dataset generator that currently targets OpenAI via rig; providers are optional.

Creating the service:

```rust
use layercake_data_acquisition::services::DataAcquisitionService;

let provider_hint = std::env::var("LAYERCAKE_EMBEDDING_PROVIDER")
    .ok()
    .or_else(|| std::env::var("LAYERCAKE_CHAT_PROVIDER").ok());
let provider_config = layercake_data_acquisition::config::EmbeddingProviderConfig::from_env();
let acquisition = DataAcquisitionService::new(db.clone(), provider_hint, provider_config);
```

`DataAcquisitionService` automatically inspects `OPENAI_API_KEY` and wires embedding/dataset helpers only when credentials exist. In the app server we read `LAYERCAKE_EMBEDDING_PROVIDER` (falling back to `LAYERCAKE_CHAT_PROVIDER` and finally the matching env vars for CLI/console tooling) and pass it as the `provider_hint` so embeddings follow whatever provider/model the operator selected.

## Database Schema
Migration `m20251110_000016_create_data_acquisition_tables` adds:
- `tags` with `(name, scope)` uniqueness and optional color metadata.
- `files` storing uploaded blobs (SeaORM-managed SQLite BLOB) and uploader metadata.
- Join tables (`file_tags`, `dataset_tags`, `graph_node_tags`, `graph_edge_tags`) for polymorphic tagging.
- `kb_documents` storing chunk text, embeddings (binary), and metadata.
- `vector_index_state` tracking per-project KB status, timestamps, and last errors.
- `m20251111_000017_add_embedding_provider_to_vector_state` extends `vector_index_state` with `embedding_provider` and `embedding_model` so we can enforce consistent providers before mixing embeddings. Run the migrations after pulling via `cargo run --bin layercake -- db migrate up --database layercake.db`.

## Backend / GraphQL Surface
- `AppContext` now exposes `data_acquisition_service()`.
- Queries:
  - `knowledgeBaseStatus(projectId: Int!)`
  - `dataAcquisitionFiles(projectId: Int!)`
  - `dataAcquisitionTags(scope: String)`
- Mutations:
  - `ingestFile(input: IngestFileInput!)` – accepts base64 payloads and optional tags.
  - `runKnowledgeBaseCommand(input: KnowledgeBaseCommandInput!)` – rebuild or clear project KB.
  - `generateDatasetFromPrompt(input: DatasetGenerationInput!)` – runs RAG prompts and returns YAML.

GraphQL types live under `graphql/types/data_acquisition.rs`.

## Frontend Entry Point
- Added `/projects/:projectId/data-acquisition` route + sidebar link.
- `DataAcquisitionPage` (Vite/React) provides three cards:
  1. **Source Management** – upload files, view recent uploads, control tag defaults.
  2. **Knowledge Base** – observe KB metrics plus rebuild/clear actions.
  3. **Data Set Creation** – craft prompts, select tag filters, and inspect generated datasets.

The page relies on the new GraphQL operations and is meant as a functional placeholder until richer UX (drag-drop, progress indicators, previews) ships.

## Follow-up Tasks
- Add binary format parsers (PDF, DOCX, XLSX) under `ingestion::parsers`.
- Stream embeddings with chunk-level progress + job queue.
- Persist dataset outputs back into the plan DAG via existing dataset services.
- Extend Tauri/CLI shells to call the new service without GraphQL.
