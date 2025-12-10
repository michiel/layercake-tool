# Refactoring Plan: Harmonise Dataset and Graph Structures

## Executive Summary

**Current State**: Datasets and Graphs are structurally identical (nodes, edges, layers, annotations, metadata) but implemented with different database schemas, service interfaces, and conversion logic.

**Key Architectural Change**: Layers are no longer used for rendering Graphs directly. They are only used during import/export to construct the project-wide layer palette.

**Goal**: Harmonise the data model, eliminate duplication, and create a unified graph data structure that serves both source (Dataset) and computed (Graph) use cases.

---

## 1. Current Architecture Analysis

### 1.1 Structural Similarity

Both Dataset and Graph contain the same logical structure:

```
GraphData {
    nodes: Vec<Node>     // id, label, layer, is_partition, belongs_to, weight, comment, attributes
    edges: Vec<Edge>     // id, source, target, label, layer, weight, comment, attributes
    layers: Vec<Layer>   // id, label, background_color, text_color, border_color
    annotations: Vec<String> or String
    metadata: JSON
}
```

### 1.2 Current Implementation Differences

| Aspect | Dataset (`data_sets`) | Graph (`graphs`) |
|--------|----------------------|------------------|
| **Purpose** | Source data from uploads | Computed pipeline results |
| **Storage** | `blob` + `graph_json` string | Normalised tables (`graph_nodes`, `graph_edges`, `graph_layers`) |
| **Children Tables** | `dataset_graph_nodes`, `dataset_graph_edges`, `dataset_graph_layers` | `graph_nodes`, `graph_edges`, `graph_layers` |
| **Weight Type** | `i32` in dataset tables | `f64` in graph tables |
| **Required Fields** | `label`, `layer` required in CSV | `label`, `layer` optional in DB |
| **Attributes Field** | `attributes` (JSON) | `attrs` (JSON) - inconsistent naming |
| **Layer Rendering** | Contributes to project palette | ~~Used for rendering~~ (deprecated) |
| **Edit Support** | No edit tracking | Edit replay with `GraphEdit` |
| **Change Detection** | File-based (`processed_at`) | Hash-based (`source_hash`) |
| **Traceability** | None | `dataset_id` on all child records |

### 1.3 Conversion Points

**Dataset → Graph** (via `GraphBuilder::build_graph()`):
- Parse `graph_json` string into in-memory `Graph` struct
- Insert into normalised `graph_nodes`, `graph_edges`, `graph_layers` tables
- Type conversion: `i32` weight → `f64` weight
- Add `dataset_id` for traceability

**Graph → Virtual Dataset** (via `graph_to_data_set()` for DAG chaining):
- Read from `graph_nodes`, `graph_edges` tables
- Serialise to `graph_json` string
- Create virtual `data_sets` row (no `blob`, `origin: "manual_edit"`)
- Type conversion: `f64` weight → `i32` weight

---

## 2. Problems with Current Architecture

### 2.1 Duplication and Inconsistency

**Database Schema Duplication**:
- 6 nearly identical tables: `dataset_graph_{nodes,edges,layers}` and `graph_{nodes,edges,layers}`
- Different field types for same logical data (weight: i32 vs f64)
- Inconsistent naming (`attributes` vs `attrs`)
- Different nullability rules (`label`/`layer` required vs optional)

**Service Layer Duplication**:
- `DataSetService` and `GraphService` have overlapping responsibilities
- Both parse/validate graph structure
- Both handle layer extraction and normalisation
- Different APIs for the same logical operations

**Conversion Overhead**:
- Multiple type conversions (i32 ↔ f64 for weight)
- Serialisation/deserialisation between `graph_json` string and tables
- Graph → Virtual Dataset conversion creates temporary in-memory structures

**Layer Confusion**:
- Layers stored in both `dataset_graph_layers` and `graph_layers` tables
- Neither used for rendering (project palette is authoritative)
- Redundant storage of layer definitions
- Unclear ownership and lifecycle

### 2.2 Maintenance Burden

- Schema migrations must update 6 tables
- New attributes require changes in multiple places
- GraphQL resolvers duplicated for Dataset and Graph
- Validation logic duplicated
- Export logic has two code paths

### 2.3 Performance Issues

- Graph → Virtual Dataset conversion reads entire graph from DB just to serialise to JSON
- Dataset parsing reads entire `graph_json` string into memory
- Unnecessary data copying during DAG execution
- No lazy loading of graph data

---

## 3. Proposed Architecture

### 3.1 Unified Graph Data Model

**Core Principle**: A graph is a graph, regardless of whether it's a source (Dataset) or computed result (Graph). The storage and lifecycle determine the differences, not the structure.

#### 3.1.1 Single Graph Storage Table Hierarchy

Replace 6 tables with 3:

```sql
-- New unified tables
CREATE TABLE graph_data (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,

    -- Source and lifecycle
    source_type TEXT NOT NULL,  -- 'dataset', 'computed', 'manual'
    dag_node_id TEXT,           -- Links to plan_dag_nodes.id (NULL for datasets)

    -- Dataset-specific
    file_format TEXT,           -- 'csv', 'tsv', 'json', NULL for computed
    origin TEXT,                -- 'file_upload', 'rag_agent', 'manual_edit', NULL
    filename TEXT,
    blob BLOB,                  -- Raw file bytes; must be NULL for computed/manual
    file_size INTEGER,          -- Size in bytes; NULL for computed/manual
    processed_at TIMESTAMP,     -- When dataset parsing finished

    -- Computed graph-specific
    source_hash TEXT,           -- Hash for change detection (NULL for datasets)
    computed_date TIMESTAMP,

    -- Edit tracking (for computed graphs only)
    last_edit_sequence INTEGER DEFAULT 0,
    has_pending_edits BOOLEAN DEFAULT FALSE,
    last_replay_at TIMESTAMP,

    -- Common metadata
    node_count INTEGER NOT NULL DEFAULT 0,
    edge_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    metadata JSON,              -- Flexible metadata
    annotations JSON,           -- Ordered list of markdown strings
    status TEXT NOT NULL,       -- 'active', 'processing', 'error' canonical status

    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,

    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE TABLE graph_data_nodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- surrogate key, auto-assigned
    graph_data_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,  -- user-provided node ID, unique per graph_data
    label TEXT,                 -- Optional (required during import, but can be NULL in DB)
    layer TEXT,                 -- Optional (same reasoning)
    weight REAL,                -- Use REAL (f64) consistently
    is_partition BOOLEAN NOT NULL DEFAULT FALSE,
    belongs_to TEXT,            -- references another node's external_id in same graph
    comment TEXT,
    source_dataset_id INTEGER,  -- Traceability to original dataset
    attributes JSON,            -- Consistent naming
    created_at TIMESTAMP NOT NULL,

    FOREIGN KEY (graph_data_id) REFERENCES graph_data(id) ON DELETE CASCADE,
    UNIQUE (graph_data_id, external_id)
);

CREATE TABLE graph_data_edges (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- surrogate key, auto-assigned
    graph_data_id INTEGER NOT NULL,
    external_id TEXT NOT NULL,  -- user-provided edge ID, unique per graph_data
    source TEXT NOT NULL,       -- references node external_id in same graph
    target TEXT NOT NULL,       -- references node external_id in same graph
    label TEXT,
    layer TEXT,
    weight REAL,
    comment TEXT,
    source_dataset_id INTEGER,
    attributes JSON,
    created_at TIMESTAMP NOT NULL,

    FOREIGN KEY (graph_data_id) REFERENCES graph_data(id) ON DELETE CASCADE,
    FOREIGN KEY (graph_data_id, source)
        REFERENCES graph_data_nodes(graph_data_id, external_id) ON DELETE CASCADE,
    FOREIGN KEY (graph_data_id, target)
        REFERENCES graph_data_nodes(graph_data_id, external_id) ON DELETE CASCADE,
    UNIQUE (graph_data_id, external_id)
);

-- Layers table REMOVED - layers only exist in project_layers
-- graph_data nodes/edges reference layer IDs that must exist in project_layers

-- Indexes for performance (see Section 3.1.4 for rationale)
CREATE INDEX idx_graph_data_project ON graph_data(project_id);
CREATE INDEX idx_graph_data_dag_node ON graph_data(dag_node_id);
CREATE INDEX idx_graph_data_source_type ON graph_data(project_id, source_type);
CREATE INDEX idx_graph_data_status ON graph_data(status);

CREATE INDEX idx_nodes_graph ON graph_data_nodes(graph_data_id);
CREATE INDEX idx_nodes_external ON graph_data_nodes(graph_data_id, external_id);
CREATE INDEX idx_nodes_layer ON graph_data_nodes(layer);
CREATE INDEX idx_nodes_belongs_to ON graph_data_nodes(graph_data_id, belongs_to);

CREATE INDEX idx_edges_graph ON graph_data_edges(graph_data_id);
CREATE INDEX idx_edges_external ON graph_data_edges(graph_data_id, external_id);
CREATE INDEX idx_edges_source ON graph_data_edges(graph_data_id, source);
CREATE INDEX idx_edges_target ON graph_data_edges(graph_data_id, target);
CREATE INDEX idx_edges_source_target ON graph_data_edges(source, target);
CREATE INDEX idx_edges_layer ON graph_data_edges(layer);
```

**Consistency rules**:
- `blob` must be populated only for `source_type = 'dataset'`; enforce NULL for computed/manual rows.
- `annotations` is a JSON array of ordered markdown strings.
- `node_count` and `edge_count` are authoritative and must stay in sync with child tables (no `layer_count` column).
- `status` is the canonical lifecycle indicator with values: 'pending' | 'processing' | 'active' | 'error'
- `weight` is always REAL/f64.
- Surrogate INT PKs (`id`) are auto-assigned; externally meaningful IDs stored in `external_id` must be unique per `graph_data`.
- Edge foreign keys enforce referential integrity: source/target must reference valid node external_ids in same graph.

#### 3.1.2 Status Lifecycle Mapping

**Canonical Status Values**: `'pending' | 'processing' | 'active' | 'error'`

**Mapping from ExecutionState** (for migration):
```rust
fn map_execution_state_to_status(exec_state: &str) -> &str {
    match exec_state {
        "Completed" => "active",
        "Processing" => "processing",
        "NotStarted" | "Pending" => "pending",
        "Error" => "error",
        _ => "error"  // Unknown states treated as error
    }
}
```

**Usage**:
- Datasets: 'processing' during import, 'active' when complete, 'error' on failure
- Computed graphs: 'pending' when created, 'processing' during execution, 'active' when complete, 'error' on failure
- Manual graphs: Created as 'active'

#### 3.1.3 Layer Handling

**Decision**: Remove `graph_layers` and `dataset_graph_layers` tables entirely.

**Rationale**:
- Layers are no longer used for rendering individual graphs
- Project-wide palette (`project_layers`) is the single source of truth
- Reduces storage duplication and synchronisation issues

**New Flow**:
1. **Dataset Import**: Extract unique layer IDs from nodes/edges, populate `project_layers` if missing
2. **Graph Build**: Validate that all node/edge layer IDs exist in `project_layers`
3. **Rendering**: Resolve layer styling from `project_layers` only
4. **Export**: Include layer definitions from `project_layers` in exported files
5. **Validation Strictness**: Strict validation; missing layers fail the operation (no auto-create).

#### 3.1.4 Index Strategy

**Critical Indexes for Performance**:

**graph_data table**:
- `idx_graph_data_project`: Queries by project_id (list all graphs in project)
- `idx_graph_data_dag_node`: Lookup graph by DAG node ID (pipeline execution)
- `idx_graph_data_source_type`: Filter by source type within project (list datasets vs computed)
- `idx_graph_data_status`: Filter by status (show active/error graphs)

**graph_data_nodes table**:
- `idx_nodes_graph`: Load all nodes for a graph (most common query)
- `idx_nodes_external`: Unique constraint enforcement + lookup by external_id
- `idx_nodes_layer`: Find all nodes using a layer (palette validation)
- `idx_nodes_belongs_to`: Traverse hierarchy (find children of partition node)

**graph_data_edges table**:
- `idx_edges_graph`: Load all edges for a graph
- `idx_edges_external`: Unique constraint enforcement + lookup by external_id
- `idx_edges_source`: Find outgoing edges from node (graph algorithms)
- `idx_edges_target`: Find incoming edges to node
- `idx_edges_source_target`: Check if edge exists between two nodes
- `idx_edges_layer`: Find all edges using a layer (palette validation)

**Composite Index Rationale**:
- `(graph_data_id, external_id)`: Enables efficient FK constraint checking for edges
- `(graph_data_id, source/target)`: Localizes edge lookups within single graph
- `(project_id, source_type)`: Common filter pattern in UI

**Migration Note**: Create indexes AFTER bulk insert in Phase 2 for performance.

#### 3.1.5 Unified In-Memory Model

```rust
// layercake-core/src/graph_data.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub source_type: GraphDataSource,
    pub dag_node_id: Option<String>,

    // Counts (kept in sync with child tables)
    pub node_count: i32,
    pub edge_count: i32,

    // Content
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub annotations: Vec<String>,
    pub metadata: serde_json::Value,

    // Source-specific
    pub dataset_info: Option<DatasetInfo>,
    pub computed_info: Option<ComputedInfo>,

    // Lifecycle
    pub status: GraphDataStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDataSource {
    Dataset,   // From file upload or RAG agent
    Computed,  // From DAG execution
    Manual,    // Created directly by user
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub file_format: FileFormat,
    pub origin: DatasetOrigin,
    pub filename: String,
    pub blob: Option<Vec<u8>>,  // May be None if not loaded
    pub file_size: Option<i64>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedInfo {
    pub source_hash: Option<String>,
    pub computed_date: Option<DateTime<Utc>>,
    pub last_edit_sequence: i32,
    pub has_pending_edits: bool,
    pub last_replay_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub db_id: Option<i32>,     // surrogate PK in DB (not exposed to clients)
    pub id: String,             // external_id from dataset/graph files
    pub label: String,           // Required in struct, validated during creation
    pub layer: String,           // Required in struct, validated against project_layers
    pub weight: f64,             // Use f64 consistently
    pub is_partition: bool,
    pub belongs_to: Option<String>,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub db_id: Option<i32>,     // surrogate PK in DB (not exposed to clients)
    pub id: String,             // external_id from dataset/graph files
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: f64,
    pub comment: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphDataStatus {
    Active,
    Processing,
    Error,
}
```

### 3.2 Unified Service Layer

#### 3.2.1 GraphDataService

Replace `DataSetService` and parts of `GraphService` with unified `GraphDataService`:

```rust
// layercake-core/src/services/graph_data_service.rs

pub struct GraphDataService {
    db: DatabaseConnection,
}

impl GraphDataService {
    // Creation
    pub async fn create_from_file(
        &self,
        project_id: i32,
        name: String,
        file: UploadedFile,
    ) -> Result<GraphData> { ... }

    pub async fn create_from_json(
        &self,
        project_id: i32,
        name: String,
        graph_json: serde_json::Value,
    ) -> Result<GraphData> { ... }

    pub async fn create_computed(
        &self,
        project_id: i32,
        dag_node_id: String,
        name: String,
    ) -> Result<GraphData> { ... }

    // Retrieval
    pub async fn get_by_id(&self, id: i32) -> Result<GraphData> { ... }

    pub async fn get_by_dag_node(&self, dag_node_id: &str) -> Result<Option<GraphData>> { ... }

    pub async fn list_datasets(&self, project_id: i32) -> Result<Vec<GraphData>> { ... }

    pub async fn list_computed(&self, project_id: i32) -> Result<Vec<GraphData>> { ... }

    // Data access (lazy loading)
    pub async fn load_nodes(&self, graph_data_id: i32) -> Result<Vec<GraphNode>> { ... }

    pub async fn load_edges(&self, graph_data_id: i32) -> Result<Vec<GraphEdge>> { ... }

    pub async fn load_full(&self, graph_data_id: i32) -> Result<GraphData> { ... }

    // Updates
    pub async fn update_nodes(
        &self,
        graph_data_id: i32,
        nodes: Vec<GraphNode>,
    ) -> Result<()> { ... }

    pub async fn update_edges(
        &self,
        graph_data_id: i32,
        edges: Vec<GraphEdge>,
    ) -> Result<()> { ... }

    // Lifecycle
    pub async fn mark_processing(&self, id: i32) -> Result<()> { ... }

    pub async fn mark_complete(&self, id: i32, source_hash: String) -> Result<()> { ... }

    pub async fn mark_error(&self, id: i32, error: String) -> Result<()> { ... }

    // Validation
    pub async fn validate_layers(&self, graph_data: &GraphData) -> Result<Vec<String>> {
        // Returns missing layer IDs
    }

    pub async fn validate_structure(&self, graph_data: &GraphData) -> Result<()> { ... }
}
```

#### 3.2.2 LayerPaletteService (new)

Extract layer management into dedicated service:

```rust
// layercake-core/src/services/layer_palette_service.rs

pub struct LayerPaletteService {
    db: DatabaseConnection,
}

impl LayerPaletteService {
    // Project palette management
    pub async fn get_project_palette(&self, project_id: i32) -> Result<Vec<Layer>> { ... }

    pub async fn extract_layers_from_graph_data(
        &self,
        graph_data: &GraphData,
    ) -> Result<Vec<Layer>> {
        // Extract unique layer IDs from nodes and edges
        // Look up definitions from project_layers
        // Return missing layers for user review
    }

    pub async fn import_layers_from_dataset(
        &self,
        project_id: i32,
        dataset_id: i32,
        overwrite: bool,
    ) -> Result<Vec<Layer>> {
        // Add dataset's layers to project palette
        // Handle conflicts based on overwrite flag
    }

    pub async fn add_layer(
        &self,
        project_id: i32,
        layer: Layer,
    ) -> Result<Layer> { ... }

    pub async fn update_layer(
        &self,
        project_id: i32,
        layer_id: &str,
        layer: Layer,
    ) -> Result<Layer> { ... }

    pub async fn delete_layer(
        &self,
        project_id: i32,
        layer_id: &str,
    ) -> Result<()> { ... }

    // Validation
    pub async fn validate_layer_references(
        &self,
        project_id: i32,
        graph_data: &GraphData,
    ) -> Result<ValidationResult> {
        // Check that all layer IDs in nodes/edges exist in project_layers
        // Return missing layer IDs and orphaned layer definitions
    }
}

pub struct ValidationResult {
    pub missing_layers: Vec<String>,    // Layer IDs referenced but not in palette
    pub orphaned_layers: Vec<String>,   // Layer IDs in palette but never used
    pub is_valid: bool,
}
```

### 3.3 GraphQL API Changes

#### 3.3.1 Unified GraphData Type

```rust
// layercake-core/src/graphql/types/graph_data.rs

#[derive(async_graphql::SimpleObject)]
pub struct GraphDataGql {
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub source_type: String,  // 'dataset', 'computed', 'manual'
    pub dag_node_id: Option<String>,

    // Counts
    pub node_count: i32,
    pub edge_count: i32,

    // Status
    pub status: String,
    pub error_message: Option<String>,

    // Dataset-specific (NULL for computed)
    pub file_format: Option<String>,
    pub origin: Option<String>,
    pub filename: Option<String>,
    pub file_size: Option<i64>,
    pub processed_at: Option<DateTime<Utc>>,

    // Computed-specific (NULL for datasets)
    pub source_hash: Option<String>,
    pub computed_date: Option<DateTime<Utc>>,
    pub last_edit_sequence: Option<i32>,
    pub has_pending_edits: Option<bool>,

    // Common
    pub annotations: Vec<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_graphql::Object]
impl GraphDataGql {
    // Lazy load children
    async fn nodes(&self, ctx: &Context<'_>) -> Result<Vec<GraphNodeGql>> {
        let service = ctx.data::<Arc<GraphDataService>>()?;
        service.load_nodes(self.id).await
    }

    async fn edges(&self, ctx: &Context<'_>) -> Result<Vec<GraphEdgeGql>> {
        let service = ctx.data::<Arc<GraphDataService>>()?;
        service.load_edges(self.id).await
    }

    async fn layers(&self, ctx: &Context<'_>) -> Result<Vec<LayerGql>> {
        let palette_service = ctx.data::<Arc<LayerPaletteService>>()?;

        // Load nodes/edges to get unique layer IDs
        // TODO: avoid double-fetch when nodes/edges are already requested (use DataLoader or cached context)
        let nodes = self.nodes(ctx).await?;
        let edges = self.edges(ctx).await?;

        let layer_ids: HashSet<String> = nodes.iter()
            .map(|n| n.layer.clone())
            .chain(edges.iter().map(|e| e.layer.clone()))
            .collect();

        // Look up from project palette
        palette_service.get_layers_by_ids(self.project_id, layer_ids).await
    }

    // Type-specific helpers
    async fn is_dataset(&self) -> bool {
        self.source_type == "dataset"
    }

    async fn is_computed(&self) -> bool {
        self.source_type == "computed"
    }

    async fn can_edit(&self) -> bool {
        self.source_type == "computed"
    }
}

// Lifecycle mapping (canonical)
// - status: 'active' | 'processing' | 'error' for all graph_data rows
```

#### 3.3.2 Backwards Compatibility

For gradual migration, maintain facade types:

```rust
// layercake-core/src/graphql/types/data_set.rs (facade)

#[derive(async_graphql::SimpleObject)]
pub struct DataSet {
    // Delegates to GraphDataGql internally
    // Filters to source_type = 'dataset'
}

// layercake-core/src/graphql/types/graph.rs (facade)

#[derive(async_graphql::SimpleObject)]
pub struct Graph {
    // Delegates to GraphDataGql internally
    // Filters to source_type = 'computed'
}
```

### 3.4 Pipeline Changes

#### 3.4.1 GraphBuilder Simplification

```rust
// layercake-core/src/pipeline/graph_builder.rs

pub struct GraphBuilder {
    graph_data_service: Arc<GraphDataService>,
    palette_service: Arc<LayerPaletteService>,
}

impl GraphBuilder {
    pub async fn build_graph(
        &self,
        plan_id: i32,
        dag_node: &plan_dag_nodes::Model,
    ) -> Result<GraphData> {
        // Parse upstream sources from config
        let upstream_ids = self.parse_upstream_sources(&dag_node.config_json)?;

        // Load all upstream graph_data (no conversion needed!)
        let upstream_graphs: Vec<GraphData> = upstream_ids
            .iter()
            .map(|id| self.graph_data_service.load_full(*id))
            .collect::<Result<Vec<_>>>().await?;

        // Merge nodes/edges (no type conversion needed!)
        let merged = self.merge_graphs(upstream_graphs)?;

        // Validate layers exist in project palette; strict failure on missing
        let validation = self.palette_service
            .validate_layer_references(dag_node.project_id, &merged)
            .await?;

        if !validation.missing_layers.is_empty() {
            return Err(anyhow!(
                "Missing layers in project palette: {:?}",
                validation.missing_layers
            ));
        }

        // Compute source hash
        let source_hash = self.compute_source_hash(&upstream_graphs)?;

        // Create computed graph_data
        let mut graph_data = self.graph_data_service.create_computed(
            dag_node.project_id,
            dag_node.id.clone(),
            dag_node.label.clone(),
        ).await?;

        // Store nodes/edges
        self.graph_data_service.update_nodes(graph_data.id, merged.nodes).await?;
        self.graph_data_service.update_edges(graph_data.id, merged.edges).await?;

        // Mark complete with hash
        self.graph_data_service.mark_complete(graph_data.id, source_hash).await?;

        // Replay edits if any
        if let Some(computed_info) = &graph_data.computed_info {
            if computed_info.has_pending_edits {
                self.replay_edits(graph_data.id).await?;
            }
        }

        Ok(graph_data)
    }

    // NO MORE graph_to_data_set conversion!
    // Upstream sources can be either datasets or computed graphs directly
}
```

---

## 4. Migration Strategy

### 4.1 Phase 1: Create Unified Tables (Non-Breaking)

**Goal**: Introduce new `graph_data` tables alongside existing tables.

**Tasks**:
1. Create migration for `graph_data`, `graph_data_nodes`, `graph_data_edges` tables
2. Create `GraphData` struct and enums
3. Implement `GraphDataService` (basic CRUD)
4. Implement `LayerPaletteService`
5. Add unit tests for new services

**Success Criteria**:
- New tables created
- Services compile and pass tests
- No changes to existing code

**Estimated Effort**: 3-4 days

**Status**: Implemented migrations, entities, and service scaffolding (`graph_data`, `graph_data_nodes`, `graph_data_edges`, `GraphDataService`, `LayerPaletteService`).

---

### 4.2 Phase 2: Data Migration (Batch)

**Goal**: Copy data from old tables to new unified tables.

**Tasks**:
1. **Pre-migration validation**:
   - Verify no dataset ID >= 1,000,000 (for collision prevention)
   - Identify annotation format types (NULL/empty/JSON/text) for proper normalization
   - Verify all edge source/target references exist in nodes
2. **Migrate graph_data table** (datasets and graphs):
   - Datasets: keep original IDs (< 1M)
   - Graphs: offset IDs by +1,000,000
   - Normalize annotations to JSON arrays
   - Map execution_state to canonical status
3. **Migrate graph_data_nodes** (from dataset_graph_nodes and graph_nodes):
   - Let database auto-assign surrogate `id`
   - Store original ID in `external_id`
   - Convert i32 weight → f64 for datasets
   - Rename `attrs` → `attributes` for graphs
4. **Migrate graph_data_edges** (similar to nodes)
5. **Migrate graph_edits table**:
   - Update graph_id references (apply +1M offset for computed graphs)
   - Verify all target_id references match external_id in new tables
   - Test edit replay on sample migrated graph
6. **Update foreign key references**:
   - plan_dag_nodes.config_json (graphId field +1M for computed)
   - Any other tables referencing graph IDs
7. **Create indexes** (after bulk insert for performance)
8. **Recompute counts**: Update node_count/edge_count in graph_data
9. **Validation queries**: Verify integrity (counts, FK consistency, annotation format)

**Status**: Added migration `m20251210_000004_migrate_existing_graph_data` (non-destructive) that copies datasets/graphs and children into unified tables, applies +1,000,000 offset to computed graph IDs and children (and graph_edits graph_id), normalizes annotations, backfills counts, updates plan_dag_nodes.config_json graphId fields, captures validation counts (datasets/graphs/nodes/edges + orphaned edge refs + plan configs below offset + FK checks), and reseeds sequences (SQLite/Postgres).

**Migration Script Outline**:
```sql
-- Copy datasets
INSERT INTO graph_data (
    id, project_id, name, source_type, dag_node_id,
    file_format, origin, filename, blob, file_size, processed_at,
    status, node_count, edge_count, error_message, metadata, annotations,
    created_at, updated_at
)
SELECT
    id, project_id, name, 'dataset', NULL,
    file_format, origin, filename, blob, file_size, processed_at,
    status, 0, 0, error_message, metadata,
    json(annotations),  -- normalize to JSON array
    created_at, updated_at
FROM data_sets;

-- Copy dataset nodes
INSERT INTO graph_data_nodes (
    id, graph_data_id, label, layer, weight, is_partition,
    belongs_to, comment, source_dataset_id, attributes, created_at
)
SELECT
    id, dataset_id, label, layer, CAST(weight AS REAL), is_partition,
    belongs_to, comment, dataset_id, attributes, CURRENT_TIMESTAMP
FROM dataset_graph_nodes;

-- Repeat for edges, graphs, graph_nodes, graph_edges
```

**Success Criteria**:
- All data copied to new tables
- Counts match between old and new tables
- All annotations properly normalized to JSON arrays
- graph_edits references updated and validated
- All edge source/target FKs valid
- Sample edit replay succeeds
- Rollback plan tested

**Validation Queries** (see Appendix C for full set):
- Row count equality (old vs new)
- Node/edge count consistency
- Foreign key integrity
- Annotation format verification
- Edit replay test on sample graphs

**Estimated Effort**: 3-4 days (increased from 2-3 due to graph_edits + annotation handling)

**Status**: Validation queries implemented in migration and tested (counts, FK integrity, annotations).

---

### 4.3 Phase 3: Update Pipeline to Use graph_data

**Goal**: Migrate `GraphBuilder` and plan execution to use new unified tables.

**Tasks**:
1. Update `GraphBuilder::build_graph()` to use `GraphDataService`
2. Remove `graph_to_data_set()` conversion logic
3. Update DAG executor to work with `graph_data` directly
4. Update edit replay logic
5. Update change detection and hash computation
6. Add integration tests for full DAG execution

**Success Criteria**:
- Plan DAG execution works with new tables
- Graph chaining works without conversion
- Edit replay functions correctly
- Change detection triggers correctly
- All pipeline tests pass

**Estimated Effort**: 4-5 days

**Status** (Updated 2025-12-10 Evening):
- ✅ `GraphDataBuilder` merges upstream graph_data, validates layers, computes source hashes, reuses existing graph_data when hashes match, and marks complete
- ✅ Wired into `DagExecutor` when `graphDataIds` are provided; legacy `GraphBuilder` remains for configs without unified IDs
- ✅ GraphDataService API completed with all convenience methods (list_datasets, list_computed, mark_processing, mark_error, create_computed, create_from_json)
- ✅ Comprehensive integration tests added (`tests/graph_data_builder_test.rs`) covering:
  - Upstream graph merging
  - Layer validation (missing layers fail)
  - Change detection and hash-based reuse
  - Convenience method functionality
  - Lazy loading (load_nodes, load_edges, load_full)
- ✅ Created `docs/graph_id_audit.md` to track legacy references (47 GraphService refs, 47 DataSetService refs, 2 graph_to_data_set refs)
- ✅ **NEW**: Edit replay fully implemented for graph_data:
  - GraphDataEditApplicator applies edits to graph_data_nodes/edges
  - GraphDataService edit methods (replay_edits, clear_edits, update_edit_metadata, etc.)
  - Integration tests for node/edge create/update/delete operations
  - Edit sequencing and metadata tracking validated
  - Skips layer edits (Phase 5 will remove layer storage)
- ✅ **NEW**: Removed graph_to_data_set conversion (90 lines deleted):
  - Legacy GraphBuilder now fails with clear migration error for chaining
  - Directs users to migrate to graphDataIds config
  - GraphDataBuilder handles chaining natively without conversion
- ✅ **NEW**: Comprehensive DAG execution integration tests (`tests/dag_executor_graph_data_test.rs`):
  - Simple graph build from graph_data dataset
  - Graph chaining (computed graph → computed graph)
  - Change detection prevents unnecessary rebuilds
  - Affected nodes execution (incremental updates)
  - All tests use unified graphDataIds path (no legacy DataSetNode references)

**Completion**: ~98% (was 95%)

**Remaining Work**:
- ❌ Phase out legacy `GraphBuilder` paths and update GraphQL/MCP/console callers to use `graphDataId` (optional - dual path works)

**Next Actions** (Priority Order):
1. Use `docs/graph_id_audit.md` to systematically remove legacy references (optional)
2. Begin Phase 4 GraphQL API migration (recommended next step)

---

### 4.4 Phase 4: Update GraphQL API (with Facades)

**Goal**: Expose new unified `GraphDataGql` type while maintaining backwards compatibility.

**Tasks**:
1. Create `GraphDataGql` type
2. Implement lazy-loading resolvers for nodes/edges/layers
3. Create facade types `DataSet` and `Graph` that delegate to `GraphDataGql`
4. Update mutations to use `GraphDataService`
5. Add deprecation warnings to old types
6. Update frontend to use new unified type (optional, can defer)

**Success Criteria**:
- New `graphData` query works
- Old `dataSet` and `graph` queries still work (facades)
- Mutations work with new service
- GraphQL schema validates
- Frontend continues to function

**Estimated Effort**: 3-4 days

**Status** (Updated 2025-12-10 Evening):
- ✅ **GraphData type created** (`layercake-core/src/graphql/types/graph_data.rs`):
  - Unified type with source_type discriminator ("dataset" or "computed")
  - All fields from legacy Graph and DataSet types included
  - Lazy-loading resolvers for nodes/edges via graph_data_nodes/edges tables
  - Helper resolvers: isDataset, isComputed, isReady, hasError, fileSizeFormatted
  - From<graph_data::Model> conversion implemented
- ✅ **GraphQL queries added**:
  - `graphData(id: i32)`: Get by ID
  - `graphDataList(projectId: i32, sourceType: Option<String>)`: List with optional filtering
  - `graphDataByDagNode(dagNodeId: String)`: Get by DAG node ID
- ✅ **GraphQL mutations added** (`layercake-core/src/graphql/mutations/graph_data.rs`):
  - `updateGraphData(id: i32, input: UpdateGraphDataInput)`: Update name/metadata
  - `replayGraphDataEdits(graphDataId: i32)`: Apply pending edits
  - `clearGraphDataEdits(graphDataId: i32)`: Clear all edits
  - Note: Create/delete mutations deferred (dataset creation still uses legacy AppContext)

**Completion**: ~70%

**Completed Work** (2025-12-10 Evening):
- ✅ **Facade pattern implemented**:
  - `From<graph_data::Model> for DataSet`: Maps graph_data with source_type="dataset" to DataSet
  - `From<graph_data::Model> for Graph`: Maps graph_data with source_type="computed" to Graph
  - Field mapping with sensible defaults for deprecated fields:
    - graph_json → empty (deprecated, use nodes/edges queries)
    - description → extracted from metadata JSON
    - layer_count → 0 (layers now in project palette)
    - execution_state → mapped from status ("processing"→"running", "active"→"completed", etc.)
    - annotations → bidirectional JSON conversion
  - Both facades coexist with legacy From implementations for smooth migration

**Remaining Work**:
- ❌ Add deprecation warnings to old types
- ❌ Update existing queries to optionally use graph_data table with facades
- ❌ Frontend migration to use unified GraphData type (optional, can defer)

**Next Actions**:
1. Add deprecation warnings to DataSet/Graph GraphQL types
2. Consider updating existing dataSet/graph queries to delegate to graph_data (optional)
3. Document migration path for frontend

---

### 4.5 Phase 5: Remove Layer Storage from graph_data

**Goal**: Stop storing layers in graph-specific tables; use project palette exclusively.

**Tasks**:
1. Update `GraphDataService::create_from_file()` to extract layers and add to project palette
2. Update rendering logic to resolve layers from `project_layers` only
3. Update export logic to include layers from project palette
4. Remove layer-related code from `GraphBuilder`
5. Update validation to check layer references against project palette

**Success Criteria**:
- Layers imported from datasets appear in project palette
- Rendering uses project palette only
- Exports include project palette layers
- No orphaned layer data
- Layer validation detects missing layers

**Estimated Effort**: 2-3 days

---

### 4.6 Phase 6: Cleanup (Breaking)

**Goal**: Remove old tables and deprecated code.

**Tasks**:
1. Drop old tables: `data_sets`, `dataset_nodes`, `dataset_graph_*`, `graphs`, `graph_*`
2. Remove `DataSetService` and merge remaining logic into `GraphDataService`
3. Remove facade GraphQL types (`DataSet`, `Graph`)
4. Update all remaining references to use `GraphData`
5. Remove `graph_to_data_set()` and related conversion code
6. Update documentation

**Success Criteria**:
- Old tables dropped
- Old services removed
- All tests pass
- Documentation updated
- No references to old code

**Estimated Effort**: 2-3 days

---

## 5. Implementation Timeline

**Total Estimated Effort**: 16-22 developer days (3-4 weeks)

| Phase | Duration | Dependencies | Risk Level |
|-------|----------|--------------|------------|
| **Phase 1**: Create unified tables | 3-4 days | None | Low |
| **Phase 2**: Data migration | 2-3 days | Phase 1 | Medium |
| **Phase 3**: Update pipeline | 4-5 days | Phase 2 | High |
| **Phase 4**: Update GraphQL | 3-4 days | Phase 3 | Medium |
| **Phase 5**: Remove layer storage | 2-3 days | Phase 4 | Medium |
| **Phase 6**: Cleanup | 2-3 days | Phase 5 | Low |

**Risks**:
- **Phase 3** is highest risk: pipeline execution is complex and critical
- **Data migration** must be thoroughly tested with production-like data
- **GraphQL changes** may require frontend updates

**Mitigation**:
- Feature flags for gradual rollout
- Parallel testing of old and new pipelines
- Comprehensive integration tests
- Staged rollout (dev → staging → production)

---

## 6. Benefits

### 6.1 Immediate Benefits

**Reduced Duplication**:
- 6 tables → 3 tables (50% reduction)
- 2 services → 1 service (+ dedicated layer service)
- Unified GraphQL type
- Single validation logic
- Single import/export code path

**Performance Improvements**:
- No graph-to-dataset conversion overhead
- Lazy loading of nodes/edges
- Direct DAG chaining without serialisation
- Consistent data types (no i32 ↔ f64 conversion)

**Simplified Layer Management**:
- Single source of truth for layers (project palette)
- No orphaned layer definitions
- Clear layer lifecycle
- Easier palette management UI

### 6.2 Long-Term Benefits

**Maintainability**:
- Fewer files to maintain
- Single API to learn
- Consistent naming conventions
- Easier to add new features

**Extensibility**:
- Easy to add new graph data sources
- Unified transformation pipeline
- Better support for graph versioning
- Simpler backup/restore

**Correctness**:
- Type safety (f64 weights throughout)
- Validation at single point
- Layer reference integrity
- Clearer ownership model

---

## 7. Testing Strategy

### 7.1 Migration Testing

**Data Integrity Tests**:
```rust
#[tokio::test]
async fn test_migration_preserves_dataset_count() {
    let old_count: i64 = data_sets::Entity::find().count(&db).await?;
    run_migration(&db).await?;
    let new_count: i64 = graph_data::Entity::find()
        .filter(graph_data::Column::SourceType.eq("dataset"))
        .count(&db).await?;
    assert_eq!(old_count, new_count);
}

#[tokio::test]
async fn test_migration_preserves_node_data() {
    let old_node = dataset_graph_nodes::Entity::find_by_id("node_1").one(&db).await?;
    run_migration(&db).await?;
    let new_node = graph_data_nodes::Entity::find_by_id(("node_1", 1)).one(&db).await?;

    assert_eq!(old_node.label, new_node.label);
    assert_eq!(old_node.layer, new_node.layer);
    assert_eq!(old_node.weight as f64, new_node.weight);
}
```

### 7.2 Pipeline Testing

**DAG Execution Tests**:
```rust
#[tokio::test]
async fn test_graph_chaining_with_unified_model() {
    // Create dataset
    let dataset = graph_data_service.create_from_file(project_id, file).await?;

    // Create graph node that uses dataset
    let graph1 = graph_builder.build_graph(plan_id, &dag_node1).await?;

    // Create graph node that uses graph1 (chaining)
    let graph2 = graph_builder.build_graph(plan_id, &dag_node2).await?;

    // Verify graph2 has data from graph1
    let nodes = graph_data_service.load_nodes(graph2.id).await?;
    assert!(!nodes.is_empty());
}
```

### 7.3 GraphQL Testing

**API Compatibility Tests**:
```rust
#[tokio::test]
async fn test_graphql_backwards_compatibility() {
    // Old query should still work (facade)
    let query = "query { dataSet(id: 1) { id name nodeCount } }";
    let result = execute_query(schema, query).await?;
    assert!(result.is_ok());

    // New unified query should work
    let query = "query { graphData(id: 1) { id name sourceType nodeCount } }";
    let result = execute_query(schema, query).await?;
    assert!(result.is_ok());
}
```

---

## 8. Rollback Plan

### 8.1 Phase Rollback

Each phase should be reversible:

**Phase 1-2 (New tables created)**:
- Simply don't use new tables
- Old code continues to work
- Drop new tables if needed

**Phase 3-4 (Pipeline/API updated)**:
- Feature flag to switch between old and new pipeline
- Keep old services until Phase 6
- Rollback by disabling feature flag

**Phase 5-6 (Cleanup)**:
- Cannot rollback easily (breaking changes)
- Ensure thorough testing before this phase
- Consider making Phase 6 a separate major version

### 8.2 Data Rollback

**Before Phase 2**:
- Keep old tables intact
- New tables are additive only
- No data loss risk

**During Phase 3-5**:
- Keep old tables as read-only backup
- Write to new tables only
- Can reconstruct old tables from new if needed

**After Phase 6**:
- Old tables dropped
- Backup database before migration
- Document restore procedure

---

## 9. Documentation Updates

### 9.1 User-Facing Documentation

**Update**:
- API documentation (GraphQL schema)
- User guide for layer management
- Migration guide for existing projects

**New**:
- GraphData concept explanation
- Project palette documentation
- Best practices for graph data management

### 9.2 Developer Documentation

**Update**:
- Architecture diagrams
- Service layer documentation
- Database schema documentation

**New**:
- Migration guide for developers
- New service APIs
- Testing guidelines for unified model

---

## 10. Alternatives Considered

### 10.1 Keep Separate Tables, Share Logic

**Approach**: Keep `data_sets` and `graphs` tables separate, but unify service layer logic.

**Pros**:
- Less database migration risk
- Clearer separation of concerns
- Easier rollback

**Cons**:
- Still duplicates database schema
- Conversion overhead remains
- Doesn't solve layer confusion
- Technical debt persists

**Decision**: Rejected. Unifying database schema provides the most benefit.

---

### 10.2 Fully Denormalise (graph_json only)

**Approach**: Store all graph data as JSON blobs (`graph_json`), no normalised tables.

**Pros**:
- Simplest possible schema
- No conversion logic
- Easy to version

**Cons**:
- Can't query nodes/edges efficiently
- Can't validate data structure in database
- Harder to implement edit replay
- Performance issues for large graphs
- No referential integrity

**Decision**: Rejected. Need normalised storage for performance and integrity.

---

### 10.3 Hybrid: JSON + Materialised Views

**Approach**: Store graph_json as authoritative, materialise nodes/edges as views for querying.

**Pros**:
- Single source of truth (JSON)
- Queryable via views
- Flexible schema evolution

**Cons**:
- View performance issues
- Complex edit handling
- Database-specific features
- Not all databases support materialised views well

**Decision**: Rejected. Normalised tables are simpler and more portable.

---

## 11. Open Questions and Decisions

### 11.1 Resolved Items

✅ **Layer validation**: Strict failure on missing layers (no auto-create).
✅ **Node/edge IDs**: Surrogate INT PKs with `external_id` TEXT for user-facing IDs.
✅ **Lifecycle mapping**: `status` is canonical with 4 values: pending/processing/active/error.
✅ **Primary keys**: AUTOINCREMENT surrogate keys, not composite.
✅ **Edge integrity**: Foreign keys enforce source/target reference valid nodes.
✅ **ID collision**: Offset graph IDs by +1M, validate datasets < 1M.
✅ **Annotations**: JSON array normalization with proper NULL/empty/JSON/text handling.
✅ **Indexes**: 15 indexes defined across 3 tables.
✅ **GraphEdit migration**: Explicit migration task in Phase 2.

### 11.2 High Priority Decisions (Required Before Phase 2)

#### Decision 1: Migration Downtime Strategy

**Options**:
- A) Full downtime (2-6 hours) - Simplest, safest
- B) Dual-write (no downtime) - Complex, double storage
- C) Read-only during migration - Users can view but not edit

**Recommendation**: Option A (Full downtime)

**Status**: ⏳ Pending decision

---

#### Decision 2: Test Data Volume Requirements

**Proposed Tiers**:
- **Minimal** (Phase 2): 5 projects, 25 graphs, 100-1k nodes/edges, 50 edits
- **Realistic** (Phase 3): 50 projects, 250 graphs, 1k-10k nodes/edges, 500 edits
- **Stress** (Optional): 500 projects, 2.5k graphs, up to 1M nodes in largest

**Recommendation**: Require Minimal for Phase 2, Realistic for Phase 3

**Status**: ⏳ Pending decision

---

#### Decision 3: Rollback Safety (Dual-Write in Phase 3)

**Question**: Implement dual-write to both old and new tables during Phase 3-5?

**Recommendation**: Yes, maintain for 2 weeks post-Phase 5

**Rollback Windows**:
- Phase 1-2: Full rollback (drop new tables)
- Phase 3-5: Rollback via dual-write (if enabled)
- Phase 6: Point of no return (old tables dropped)

**Status**: ⏳ Pending decision

---

### 11.3 Medium Priority Decisions (Required Before Phase 3)

#### Decision 4: Maximum Graph Size

**Question**: What's the supported maximum graph size?

**Options**:
- Document recommended limit (e.g., 100k nodes)
- Enforce hard limit with error
- Add pagination for large graphs

**Recommendation**: Document + enforce 100k node limit, add pagination later if needed

**Status**: ⏳ Pending decision

---

#### Decision 5: GraphQL Facade Deprecation Timeline

**Question**: When to remove old DataSet/Graph types?

**Options**:
- Remove after frontend migration (2-3 months)
- Keep until v2.0 (major version)
- Feature flag with gradual deprecation

**Recommendation**: Remove after frontend fully migrated (2-3 months post-Phase 4)

**Status**: ⏳ Pending decision

---

### 11.4 Low Priority Decisions (Can Defer)

#### Decision 6: Foreign Key Cascade for source_dataset_id

**Question**: What happens when source dataset is deleted?

**Options**:
- CASCADE: Delete dependent graphs
- SET NULL: Preserve graphs, lose traceability
- RESTRICT: Prevent deletion if in use
- No FK: Current state

**Recommendation**: SET NULL (preserve computed graphs)

**Status**: ⏳ Pending decision

---

### 11.5 Implementation Decisions

For the initial implementation, we will proceed with **recommended defaults** for all pending decisions. These can be changed before production deployment based on stakeholder feedback.

**Proceeding with**:
1. Full downtime migration (simplest for v1)
2. Minimal + Realistic test data tiers
3. Dual-write enabled in Phase 3
4. 100k node recommended limit (soft)
5. 3-month facade deprecation period
6. SET NULL for source_dataset_id FK

---

## 12. Success Metrics

### 12.1 Code Metrics

**Before Refactoring**:
- Database tables: 9 (data_sets, dataset_nodes, dataset_graph_*, graphs, graph_*)
- Service files: 2 (DataSetService, GraphService)
- Conversion functions: ~500 LOC
- GraphQL types: 2 (DataSet, Graph)

**After Refactoring**:
- Database tables: 3 (graph_data, graph_data_nodes, graph_data_edges)
- Service files: 2 (GraphDataService, LayerPaletteService)
- Conversion functions: 0 LOC
- GraphQL types: 1 (GraphData)

**Target Reduction**: ~40% fewer files, ~500 LOC removed

---

### 12.2 Performance Metrics

**Measure**:
- Graph-to-virtual-dataset conversion time (should be eliminated)
- DAG execution time for graph chaining (should improve 20-30%)
- Memory usage during pipeline execution (should reduce 15-20%)
- Database query count per DAG execution (should reduce ~25%)

---

### 12.3 Quality Metrics

**Before**:
- Type conversion bugs (weight i32 ↔ f64)
- Layer synchronisation issues
- Orphaned layer definitions

**After**:
- Zero type conversion bugs (consistent f64)
- Zero layer sync issues (single source of truth)
- Zero orphaned layers (project palette only)

---

## 13. Conclusion

Harmonising Dataset and Graph structures into a unified `GraphData` model will:

1. **Eliminate duplication**: 6 tables → 3, shared logic, single API
2. **Improve performance**: No conversion overhead, direct chaining, lazy loading
3. **Simplify layer management**: Project palette as single source of truth
4. **Enhance maintainability**: Fewer files, consistent types, clearer ownership
5. **Enable future features**: Versioning, branching, better collaboration

The migration is complex but achievable through careful phased implementation with rollback options at each stage. The long-term benefits justify the upfront effort.

**Recommended Next Steps**:
1. Review and approve this plan
2. Create feature branch for Phase 1
3. Implement unified table schema
4. Begin data migration testing with production-like data
5. Schedule weekly progress reviews

---

## Appendix A: Database Schema Comparison

### Current Schema (Fragmented)

```
data_sets (id, project_id, name, file_format, blob, graph_json, ...)
  ├─ dataset_graph_nodes (id, dataset_id, label, layer, weight:i32, ...)
  ├─ dataset_graph_edges (id, dataset_id, source, target, label, layer, weight:i32, ...)
  └─ dataset_graph_layers (id, dataset_id, label, bg_color, text_color, ...)

graphs (id, project_id, node_id, execution_state, source_hash, ...)
  ├─ graph_nodes (id, graph_id, label, layer, weight:f64, ...)
  ├─ graph_edges (id, graph_id, source, target, label, layer, weight:f64, ...)
  └─ graph_layers (id, graph_id, layer_id, name, bg_color, text_color, ...)

project_layers (id, project_id, layer_id, name, colors, source_dataset_id, enabled)
```

### Proposed Schema (Unified)

```
graph_data (id, project_id, name, source_type, dag_node_id, file_format, blob, source_hash, ...)
  ├─ graph_data_nodes (id, graph_data_id, label, layer, weight:f64, source_dataset_id, ...)
  └─ graph_data_edges (id, graph_data_id, source, target, label, layer, weight:f64, source_dataset_id, ...)

project_layers (id, project_id, layer_id, name, colors, source_graph_data_id, enabled)
```

**Reduction**: 9 tables → 3 tables (66% reduction)

---

## Appendix B: Conversion Function Removal

### Functions to Remove

```rust
// layercake-core/src/pipeline/graph_builder.rs
async fn graph_to_data_set(&self, graph: &graphs::Model) -> Result<data_sets::Model>
    // ~150 LOC, no longer needed

// layercake-core/src/services/data_set_service.rs
fn parse_graph_json(&self, json: &str) -> Result<(Vec<Node>, Vec<Edge>, Vec<Layer>)>
    // Replaced by GraphDataService::load_full()

// layercake-core/src/graph.rs
impl From<dataset_graph_nodes::Model> for Node
impl From<graph_nodes::Model> for Node
    // Single conversion path in unified model
```

**Total LOC Removed**: ~500 lines

---

## Appendix C: Migration SQL Reference

```sql
-- Phase 2: Data migration from old to new tables

-- 1. Migrate datasets
INSERT INTO graph_data (
    id, project_id, name, source_type, dag_node_id,
    file_format, origin, filename, blob, file_size, processed_at,
    status, node_count, edge_count, error_message, metadata, annotations,
    created_at, updated_at
)
SELECT
    id, project_id, name, 'dataset', NULL,
    file_format, origin, filename, blob, file_size, processed_at,
    status, 0, 0, error_message, metadata,
    json(annotations),  -- normalize to JSON array
    created_at, updated_at
FROM data_sets;

-- 2. Migrate dataset nodes
INSERT INTO graph_data_nodes (
    id, graph_data_id, external_id, label, layer, weight, is_partition,
    belongs_to, comment, source_dataset_id, attributes, created_at
)
SELECT
    NULL, dataset_id, id, label, layer,
    CAST(weight AS REAL),  -- i32 → f64 conversion
    is_partition, belongs_to, comment, dataset_id, attributes,
    CURRENT_TIMESTAMP
FROM dataset_graph_nodes;

-- 3. Migrate dataset edges
INSERT INTO graph_data_edges (
    id, graph_data_id, external_id, source, target, label, layer, weight,
    comment, source_dataset_id, attributes, created_at
)
SELECT
    NULL, dataset_id, id, source, target, label, layer,
    CAST(weight AS REAL),  -- i32 → f64 conversion
    comment, dataset_id, attributes,
    CURRENT_TIMESTAMP
FROM dataset_graph_edges;

-- 4. Migrate computed graphs
INSERT INTO graph_data (
    id, project_id, name, source_type, dag_node_id,
    file_format, origin, filename, blob, file_size, processed_at,
    source_hash, computed_date,
    last_edit_sequence, has_pending_edits, last_replay_at,
    status, node_count, edge_count, error_message, annotations, metadata,
    created_at, updated_at
)
SELECT
    id, project_id, name, 'computed', node_id,
    NULL, NULL, NULL, NULL, NULL, NULL,
    source_hash, computed_date,
    last_edit_sequence, has_pending_edits, last_replay_at,
    status,  -- status is canonical; map prior pipeline states before migration
    0, 0, error_message, json(annotations), metadata,
    created_at, updated_at
FROM graphs;

-- 5. Migrate graph nodes (already f64 weight, no conversion needed)
INSERT INTO graph_data_nodes (
    id, graph_data_id, external_id, label, layer, weight, is_partition,
    belongs_to, comment, source_dataset_id, attributes, created_at
)
SELECT
    NULL, graph_id, id, label, layer, weight, is_partition,
    belongs_to, comment, dataset_id, attrs,  -- Rename attrs → attributes
    created_at
FROM graph_nodes;

-- 6. Migrate graph edges
INSERT INTO graph_data_edges (
    id, graph_data_id, external_id, source, target, label, layer, weight,
    comment, source_dataset_id, attributes, created_at
)
SELECT
    NULL, graph_id, id, source, target, label, layer, weight,
    comment, dataset_id, attrs,  -- Rename attrs → attributes
    created_at
FROM graph_edges;

-- 7. Update node/edge counts in graph_data
UPDATE graph_data
SET node_count = (
    SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = graph_data.id
),
edge_count = (
    SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = graph_data.id
);

-- 8. Validation queries
SELECT 'Dataset migration' AS check_name,
       (SELECT COUNT(*) FROM data_sets) AS old_count,
       (SELECT COUNT(*) FROM graph_data WHERE source_type = 'dataset') AS new_count;

SELECT 'Graph migration' AS check_name,
       (SELECT COUNT(*) FROM graphs) AS old_count,
       (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed') AS new_count;

SELECT 'Node migration' AS check_name,
       (SELECT COUNT(*) FROM dataset_graph_nodes) + (SELECT COUNT(*) FROM graph_nodes) AS old_count,
       (SELECT COUNT(*) FROM graph_data_nodes) AS new_count;

-- 9. Create indexes (after bulk insert for performance)
CREATE INDEX idx_graph_data_project ON graph_data(project_id);
CREATE INDEX idx_graph_data_dag_node ON graph_data(dag_node_id);
CREATE INDEX idx_graph_data_source_type ON graph_data(project_id, source_type);
CREATE INDEX idx_graph_data_status ON graph_data(status);

CREATE INDEX idx_nodes_graph ON graph_data_nodes(graph_data_id);
CREATE INDEX idx_nodes_external ON graph_data_nodes(graph_data_id, external_id);
CREATE INDEX idx_nodes_layer ON graph_data_nodes(layer);
CREATE INDEX idx_nodes_belongs_to ON graph_data_nodes(graph_data_id, belongs_to);

CREATE INDEX idx_edges_graph ON graph_data_edges(graph_data_id);
CREATE INDEX idx_edges_external ON graph_data_edges(graph_data_id, external_id);
CREATE INDEX idx_edges_source ON graph_data_edges(graph_data_id, source);
CREATE INDEX idx_edges_target ON graph_data_edges(graph_data_id, target);
CREATE INDEX idx_edges_source_target ON graph_data_edges(source, target);
CREATE INDEX idx_edges_layer ON graph_data_edges(layer);

-- 10. Update node_count/edge_count in graph_data
UPDATE graph_data
SET node_count = (
    SELECT COUNT(*) FROM graph_data_nodes WHERE graph_data_id = graph_data.id
),
edge_count = (
    SELECT COUNT(*) FROM graph_data_edges WHERE graph_data_id = graph_data.id
);

-- 11. Validation queries (see full set in validation section below)
SELECT 'Dataset migration' AS check_name,
       (SELECT COUNT(*) FROM data_sets) AS old_count,
       (SELECT COUNT(*) FROM graph_data WHERE source_type = 'dataset') AS new_count;

SELECT 'Graph migration' AS check_name,
       (SELECT COUNT(*) FROM graphs) AS old_count,
       (SELECT COUNT(*) FROM graph_data WHERE source_type = 'computed') AS new_count;

SELECT 'Node migration' AS check_name,
       (SELECT COUNT(*) FROM dataset_graph_nodes) + (SELECT COUNT(*) FROM graph_nodes) AS old_count,
       (SELECT COUNT(*) FROM graph_data_nodes) AS new_count;

SELECT 'Edge migration' AS check_name,
       (SELECT COUNT(*) FROM dataset_graph_edges) + (SELECT COUNT(*) FROM graph_edges) AS old_count,
       (SELECT COUNT(*) FROM graph_data_edges) AS new_count;

-- Verify no orphaned edges
SELECT COUNT(*) AS orphaned_edges
FROM graph_data_edges e
WHERE NOT EXISTS (
    SELECT 1 FROM graph_data_nodes n
    WHERE n.graph_data_id = e.graph_data_id
      AND n.external_id = e.source
)
OR NOT EXISTS (
    SELECT 1 FROM graph_data_nodes n
    WHERE n.graph_data_id = e.graph_data_id
      AND n.external_id = e.target
);
-- Should return 0

-- Verify all edits reference valid graphs
SELECT COUNT(*) AS invalid_edit_references
FROM graph_edits ge
WHERE NOT EXISTS (
    SELECT 1 FROM graph_data gd
    WHERE gd.id = ge.graph_id
      AND gd.source_type = 'computed'
);
-- Should return 0

-- Verify annotation formats
SELECT annotation_type, COUNT(*) AS count
FROM annotation_analysis
GROUP BY annotation_type;
-- All should be 'null' or 'json' after migration
```
