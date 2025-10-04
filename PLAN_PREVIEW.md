# Plan: DataSource and Graph Node Preview via Backend Pipeline

## Overview

Add preview functionality for both DataSource and Graph nodes by extending the existing backend CLI pipeline to populate database entities for each DAG node. The database tables are created during initialization, and DAG node operations create/update/delete entities (rows) in these tables with appropriate project scoping.

## Architecture Context

### Current Backend State
- **Existing CLI**: Backend has a CLI interface with pipeline concept for data processing
- **Full Dataset Processing**: All transformations operate on complete datasets, not subsets
- **File-based Input**: CSV files (nodes.csv, links.csv) are input to the pipeline
- **Database Schema**: Pre-existing tables for DataSources, Graphs, etc.

### Target Architecture
- **Fixed Database Schema**: Tables created during database initialization (migration)
- **Entity Management**: Each DAG node → entity (row) in corresponding table
- **Project Scoping**: All entities reference project_id for multi-tenancy
- **One-to-One Mapping**: Each DataSourceNode → one DataSource entity, each GraphNode → one Graph entity
- **Event-Driven Updates**: Changes to DAG nodes trigger pipeline execution to populate entities
- **Query for Preview**: Frontend queries these pre-computed entities for display

## System Flow

```
User Action (DAG Change)
    ↓
Frontend Mutation (createNode, updateNode, addEdge)
    ↓
Backend Event Handler
    ↓
Pipeline Executor
    ↓
┌─────────────────────────┐
│ DataSourceNode Created  │ → Create DataSource entity → Import CSV data
│ GraphNode Created       │ → Create Graph entity (empty initially)
│ Edge Created/Deleted    │ → Recalculate affected downstream graphs
│ Config Updated          │ → Reprocess affected node and downstream
│ Node Deleted            │ → Delete corresponding entity
└─────────────────────────┘
    ↓
Database Entities Updated
    ↓
Frontend Query for Preview → Display in UI
```

## Database Schema

### Pre-existing Tables (Created at Initialization)

```sql
-- DataSources table
CREATE TABLE datasources (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  node_id TEXT NOT NULL,  -- Links to PlanDag node ID
  name TEXT NOT NULL,
  file_path TEXT NOT NULL,
  file_type TEXT NOT NULL, -- 'NODES' or 'EDGES'
  import_date TIMESTAMP,
  row_count INTEGER,
  column_info JSONB,  -- Schema: [{name, type, nullable}, ...]
  execution_state TEXT NOT NULL DEFAULT 'not_started',
  error_message TEXT,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, node_id)
);

-- DataSource data table (normalized storage)
CREATE TABLE datasource_rows (
  id SERIAL PRIMARY KEY,
  datasource_id INTEGER NOT NULL REFERENCES datasources(id) ON DELETE CASCADE,
  row_number INTEGER NOT NULL,
  data JSONB NOT NULL,  -- Row data as JSON
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(datasource_id, row_number)
);

-- Graphs table
CREATE TABLE graphs (
  id SERIAL PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  node_id TEXT NOT NULL,  -- Links to PlanDag GraphNode ID
  name TEXT NOT NULL,
  execution_state TEXT NOT NULL DEFAULT 'not_started',
  computed_date TIMESTAMP,
  source_hash TEXT,  -- Hash of upstream data for change detection
  node_count INTEGER DEFAULT 0,
  edge_count INTEGER DEFAULT 0,
  error_message TEXT,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(project_id, node_id)
);

-- Graph nodes table
CREATE TABLE graph_nodes (
  id TEXT NOT NULL,  -- Node ID from source data
  graph_id INTEGER NOT NULL REFERENCES graphs(id) ON DELETE CASCADE,
  label TEXT,
  layer TEXT,
  weight REAL,
  is_partition BOOLEAN DEFAULT FALSE,
  attrs JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (graph_id, id)
);

-- Graph edges table
CREATE TABLE graph_edges (
  id TEXT NOT NULL,  -- Edge ID from source data
  graph_id INTEGER NOT NULL REFERENCES graphs(id) ON DELETE CASCADE,
  source TEXT NOT NULL,
  target TEXT NOT NULL,
  label TEXT,
  layer TEXT,
  weight REAL,
  attrs JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  PRIMARY KEY (graph_id, id)
);

-- Indexes for performance
CREATE INDEX idx_datasources_project ON datasources(project_id);
CREATE INDEX idx_datasources_node ON datasources(project_id, node_id);
CREATE INDEX idx_datasource_rows_datasource ON datasource_rows(datasource_id);
CREATE INDEX idx_graphs_project ON graphs(project_id);
CREATE INDEX idx_graphs_node ON graphs(project_id, node_id);
CREATE INDEX idx_graph_nodes_graph ON graph_nodes(graph_id);
CREATE INDEX idx_graph_edges_graph ON graph_edges(graph_id);
```

## Requirements

### Functional Requirements

1. **Entity Management**
   - Create entity in `datasources` or `graphs` table when DAG node is created
   - Delete entity when DAG node is deleted
   - Update entity when node configuration changes
   - Maintain project_id references for proper scoping

2. **DataSource Entities**
   - Store CSV data in `datasource_rows` table as JSONB
   - Track column schema in `column_info` field
   - Support both node CSVs and edge CSVs
   - Populate immediately when DataSourceNode is configured

3. **Graph Entities**
   - Store graph nodes in `graph_nodes` table
   - Store graph edges in `graph_edges` table
   - Computed from upstream DataSource/Transform/Merge nodes
   - Track source data hash for change detection

4. **Preview API**
   - Query entities by project_id and node_id
   - Support pagination (offset/limit) for datasource_rows
   - Return schema information and metadata
   - Return execution state and error information

5. **Incremental Updates**
   - Detect when upstream changes require recomputation
   - Only reprocess affected subgraph, not entire DAG
   - Track computation state (not_started, pending, processing, completed, error)

### Non-Functional Requirements

1. **Performance**
   - Entity creation should be async (non-blocking)
   - Large imports should use batch processing
   - Preview queries should be fast (< 100ms for first 100 rows)

2. **Reliability**
   - Failed imports should not leave partial data (use transactions)
   - Retry logic for transient errors
   - Clear error messages for data quality issues

3. **Scalability**
   - Support datasets up to 1M rows per datasource
   - Handle DAGs with up to 100 nodes
   - Concurrent execution of independent subgraphs

## Implementation Plan

### Phase 1: Database Schema & Migrations ✅ COMPLETED

**Files**:
- `layercake-core/src/database/migrations/m010_create_pipeline_tables.rs` (created)
- `layercake-core/src/database/entities/datasources.rs` (created)
- `layercake-core/src/database/entities/datasource_rows.rs` (created)
- `layercake-core/src/database/entities/graphs.rs` (created)
- `layercake-core/src/database/entities/graph_nodes.rs` (created)
- `layercake-core/src/database/entities/graph_edges.rs` (created)

**Tasks**:

1. **Create Database Migration**
   - Add tables: `datasources`, `datasource_rows`, `graphs`, `graph_nodes`, `graph_edges`
   - Add indexes for performance
   - Add foreign key constraints for data integrity

2. **Define Rust Models**
   ```rust
   // DataSource entity
   pub struct DataSource {
       pub id: i32,
       pub project_id: i32,
       pub node_id: String,
       pub name: String,
       pub file_path: String,
       pub file_type: FileType,  // NODES or EDGES
       pub import_date: Option<DateTime<Utc>>,
       pub row_count: Option<i32>,
       pub column_info: Option<serde_json::Value>,
       pub execution_state: ExecutionState,
       pub error_message: Option<String>,
       pub metadata: Option<serde_json::Value>,
   }

   pub struct DataSourceRow {
       pub id: i32,
       pub datasource_id: i32,
       pub row_number: i32,
       pub data: serde_json::Value,
   }

   // Graph entity
   pub struct Graph {
       pub id: i32,
       pub project_id: i32,
       pub node_id: String,
       pub name: String,
       pub execution_state: ExecutionState,
       pub computed_date: Option<DateTime<Utc>>,
       pub source_hash: Option<String>,
       pub node_count: i32,
       pub edge_count: i32,
       pub error_message: Option<String>,
       pub metadata: Option<serde_json::Value>,
   }

   pub struct GraphNode {
       pub id: String,
       pub graph_id: i32,
       pub label: Option<String>,
       pub layer: Option<String>,
       pub weight: Option<f64>,
       pub is_partition: bool,
       pub attrs: Option<serde_json::Value>,
   }

   pub struct GraphEdge {
       pub id: String,
       pub graph_id: i32,
       pub source: String,
       pub target: String,
       pub label: Option<String>,
       pub layer: Option<String>,
       pub weight: Option<f64>,
       pub attrs: Option<serde_json::Value>,
   }

   pub enum ExecutionState {
       NotStarted,
       Pending,
       Processing,
       Completed,
       Error,
   }
   ```

3. **Implement CRUD Operations**
   - `DataSource::create(project_id, node_id, config)`
   - `DataSource::update(id, config)`
   - `DataSource::delete(id)`
   - `DataSource::find_by_node(project_id, node_id)`
   - Similar for Graph entity

**Estimated effort**: 6-8 hours

### Phase 2: Pipeline Extension - Data Import & Transform ✅ COMPLETED

**Files**:
- `layercake-core/src/pipeline/mod.rs` (created)
- `layercake-core/src/pipeline/datasource_importer.rs` (created)
- `layercake-core/src/pipeline/graph_builder.rs` (created)
- `layercake-core/src/pipeline/dag_executor.rs` (created)

**Tasks**:

1. **DataSource Importer**
   - Read CSV file from configured path
   - Infer schema and validate
   - Create DataSource entity
   - Batch insert rows into `datasource_rows` table
   - Update execution_state and metadata
   - Handle CSV parsing errors gracefully

   ```rust
   pub async fn import_datasource(
       db: &PgPool,
       project_id: i32,
       node_id: &str,
       config: &DataSourceConfig
   ) -> Result<DataSource> {
       // Create DataSource entity
       let datasource = DataSource::create(db, project_id, node_id, config).await?;

       // Read and parse CSV
       let (rows, schema) = parse_csv(&config.file_path)?;

       // Batch insert rows
       insert_rows_batch(db, datasource.id, rows).await?;

       // Update entity with results
       datasource.update_state(db, ExecutionState::Completed, rows.len(), schema).await?;

       Ok(datasource)
   }
   ```

2. **Graph Builder**
   - Read from upstream DataSource/Transform/Merge entities
   - Apply transformation logic (existing pipeline code)
   - Create Graph entity
   - Populate `graph_nodes` and `graph_edges` tables
   - Update execution_state and metadata
   - Validate graph structure

   ```rust
   pub async fn build_graph(
       db: &PgPool,
       project_id: i32,
       node_id: &str,
       upstream_sources: Vec<&DataSource>
   ) -> Result<Graph> {
       // Create Graph entity
       let graph = Graph::create(db, project_id, node_id).await?;

       // Compute source hash
       let source_hash = compute_hash(&upstream_sources);

       // Build graph from upstream data
       let (nodes, edges) = build_graph_data(&upstream_sources)?;

       // Insert nodes and edges
       insert_graph_nodes(db, graph.id, nodes).await?;
       insert_graph_edges(db, graph.id, edges).await?;

       // Update entity
       graph.update_state(db, ExecutionState::Completed, source_hash).await?;

       Ok(graph)
   }
   ```

3. **DAG Executor**
   - Topological sort of DAG to determine execution order
   - Identify affected subgraph when node/edge changes
   - Execute pipeline stages in order:
     1. DataSource import
     2. Merge operations
     3. Transform operations
     4. Graph construction
   - Track execution state per node
   - Parallel execution of independent branches

4. **State Management**
   - Add `execution_state` field to PlanDag nodes (or sync from entities)
   - Broadcast state changes via WebSocket subscriptions
   - Update frontend in real-time

**Estimated effort**: 20-28 hours

**Status**: ✅ COMPLETED

Core pipeline implementation is done:
- DatasourceImporter handles CSV (nodes/edges) and JSON (graph) imports
- GraphBuilder constructs graphs from multiple upstream datasources
- DagExecutor performs topological sort and manages execution order
- Lifecycle hook structure added to GraphQL mutations (TODO markers in place)

Note: Actual lifecycle hook execution will be activated in Phase 3.

### Phase 3: Node Lifecycle Hooks

**Files**:
- `backend/src/commands/plan_dag.rs` (extend)
- `backend/src/pipeline/mod.rs` (new)

**Tasks**:

1. **Hook into Mutations**
   - `createNode` → Create DataSource/Graph entity, trigger import/build if configured
   - `updateNode` → Update entity config, trigger re-import/re-build
   - `deleteNode` → Delete entity (cascade deletes rows via foreign key)
   - `addEdge` → Trigger downstream graph recomputation
   - `deleteEdge` → Mark downstream graphs as stale

2. **Trigger Execution**
   ```rust
   // In createNode mutation handler
   if node_type == PlanDagNodeType::DataSource {
       if let Some(file_path) = config.file_path {
           // Trigger async import
           tokio::spawn(async move {
               match import_datasource(db, project_id, node_id, config).await {
                   Ok(_) => broadcast_state_change(project_id, node_id, "completed"),
                   Err(e) => broadcast_state_change_error(project_id, node_id, e),
               }
           });
       }
   }
   ```

3. **Background Job Queue** (Optional)
   - Use job queue for long-running imports/builds
   - Track job progress
   - Retry failed jobs

**Estimated effort**: 8-12 hours

### Phase 4: GraphQL API for Preview

**Files**:
- `backend/graphql/schema.graphql` (extend)
- `backend/graphql/resolvers/preview.rs` (new)

**Tasks**:

1. **Add GraphQL Schema**
   ```graphql
   # DataSource-specific preview
   type DataSourcePreview {
       nodeId: String!
       datasourceId: Int!
       filename: String!
       fileSize: Int
       fileType: String!
       totalRows: Int!
       columns: [TableColumn!]!
       rows: [TableRow!]!
       importDate: DateTime
       executionState: String!
       errorMessage: String
   }

   type TableColumn {
       name: String!
       dataType: String!
       nullable: Boolean!
   }

   type TableRow {
       rowNumber: Int!
       data: JSON!
   }

   # Graph-specific preview
   type GraphPreview {
       nodeId: String!
       graphId: Int!
       nodes: [GraphNodePreview!]!
       edges: [GraphEdgePreview!]!
       nodeCount: Int!
       edgeCount: Int!
       executionState: String!
       computedDate: DateTime
       errorMessage: String
   }

   type GraphNodePreview {
       id: String!
       label: String
       layer: String
       weight: Float
       isPartition: Boolean!
       attrs: JSON
   }

   type GraphEdgePreview {
       id: String!
       source: String!
       target: String!
       label: String
       layer: String
       weight: Float
       attrs: JSON
   }

   extend type Query {
       # DataSource preview with pagination
       dataSourcePreview(
           projectId: Int!
           nodeId: String!
           limit: Int = 100
           offset: Int = 0
       ): DataSourcePreview

       # Graph preview (returns all nodes/edges)
       graphPreview(
           projectId: Int!
           nodeId: String!
       ): GraphPreview
   }
   ```

2. **Implement Resolvers**
   ```rust
   pub async fn datasource_preview(
       ctx: &Context,
       project_id: i32,
       node_id: String,
       limit: Option<i32>,
       offset: Option<i32>,
   ) -> Result<DataSourcePreview> {
       let db = ctx.db_pool();

       // Find DataSource entity
       let datasource = DataSource::find_by_node(db, project_id, &node_id).await?;

       // Query rows with pagination
       let rows = datasource.get_rows(db, limit.unwrap_or(100), offset.unwrap_or(0)).await?;

       Ok(DataSourcePreview {
           node_id,
           datasource_id: datasource.id,
           filename: datasource.name,
           file_type: datasource.file_type,
           total_rows: datasource.row_count.unwrap_or(0),
           columns: datasource.column_info.unwrap_or_default(),
           rows,
           execution_state: datasource.execution_state,
           error_message: datasource.error_message,
       })
   }

   pub async fn graph_preview(
       ctx: &Context,
       project_id: i32,
       node_id: String,
   ) -> Result<GraphPreview> {
       let db = ctx.db_pool();

       // Find Graph entity
       let graph = Graph::find_by_node(db, project_id, &node_id).await?;

       // Query nodes and edges
       let nodes = graph.get_nodes(db).await?;
       let edges = graph.get_edges(db).await?;

       Ok(GraphPreview {
           node_id,
           graph_id: graph.id,
           nodes,
           edges,
           node_count: graph.node_count,
           edge_count: graph.edge_count,
           execution_state: graph.execution_state,
           computed_date: graph.computed_date,
           error_message: graph.error_message,
       })
   }
   ```

3. **Error Handling**
   - Return meaningful errors if entity doesn't exist
   - Return state information if processing not complete
   - Handle database query errors gracefully

**Estimated effort**: 6-8 hours

**Status**: ✅ COMPLETED

Implemented GraphQL queries and types:
- DataSourcePreview, GraphPreview, TableColumn, TableRow types
- datasourcePreview() resolver with pagination (limit/offset)
- graphPreview() resolver with complete node/edge data
- Entity lookup via (project_id, node_id) unique constraint

**Files Created**:
- `layercake-core/src/graphql/types/preview.rs`

**Files Modified**:
- `layercake-core/src/graphql/queries/mod.rs` (+132 lines)
- `layercake-core/src/graphql/types/mod.rs` (added preview module)

### Phase 5: Frontend GraphQL Integration ✅ COMPLETED

**Files**:
- `frontend/src/graphql/preview.ts` (created)
- `frontend/src/hooks/usePreview.ts` (created)

**Status**: ✅ COMPLETED

Implemented frontend GraphQL integration:
- TypeScript queries: `GET_DATASOURCE_PREVIEW`, `GET_GRAPH_PREVIEW`
- Complete type definitions for all response data
- React hooks: `useDataSourcePreview`, `useGraphPreview`
- Execution state helpers and type guards
- Cache-and-network fetch policy for real-time updates

**Tasks**:

1. **Create GraphQL Queries**
   ```typescript
   export const GET_DATASOURCE_PREVIEW = gql`
     query GetDataSourcePreview(
       $projectId: Int!
       $nodeId: String!
       $limit: Int
       $offset: Int
     ) {
       dataSourcePreview(
         projectId: $projectId
         nodeId: $nodeId
         limit: $limit
         offset: $offset
       ) {
         nodeId
         datasourceId
         filename
         fileSize
         fileType
         totalRows
         columns {
           name
           dataType
           nullable
         }
         rows {
           rowNumber
           data
         }
         importDate
         executionState
         errorMessage
       }
     }
   `

   export const GET_GRAPH_PREVIEW = gql`
     query GetGraphPreview($projectId: Int!, $nodeId: String!) {
       graphPreview(projectId: $projectId, nodeId: $nodeId) {
         nodeId
         graphId
         nodes {
           id
           label
           layer
           weight
           isPartition
           attrs
         }
         edges {
           id
           source
           target
           label
           layer
           weight
           attrs
         }
         nodeCount
         edgeCount
         executionState
         computedDate
         errorMessage
       }
     }
   `
   ```

2. **Define TypeScript Interfaces**
   ```typescript
   export interface DataSourcePreviewResponse {
       nodeId: string
       datasourceId: number
       filename: string
       fileSize?: number
       fileType: string
       totalRows: number
       columns: TableColumn[]
       rows: TableRow[]
       importDate?: string
       executionState: string
       errorMessage?: string
   }

   export interface GraphPreviewResponse {
       nodeId: string
       graphId: number
       nodes: GraphNodePreview[]
       edges: GraphEdgePreview[]
       nodeCount: number
       edgeCount: number
       executionState: string
       computedDate?: string
       errorMessage?: string
   }
   ```

**Estimated effort**: 2-3 hours

### Phase 6: Frontend Components - DataSource Preview

**Files**:
- `frontend/src/components/visualization/DataSourcePreview.tsx` (new)
- `frontend/src/components/visualization/DataSourcePreviewDialog.tsx` (new)
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` (extend)

**Tasks**:

1. **Create DataSourcePreview Component**
   - Mantine Table with virtualized scrolling
   - Sticky column headers
   - Display execution state badge
   - Show loading skeleton while fetching
   - Show error message if import failed
   - Show "Processing..." if state is pending/processing

2. **Create DataSourcePreviewDialog Component**
   - Modal with table preview
   - Header: filename, row count, column count, import date
   - Status indicator: execution state badge
   - Footer: pagination controls
   - Error alert if errorMessage present

3. **Update DataSourceNode**
   - Add centered play button (IconTable or IconPlayerPlay)
   - Add state management for preview dialog
   - Query on preview open
   - Handle all execution states

**Estimated effort**: 5-7 hours

### Phase 7: Frontend Components - Graph Preview Enhancement

**Files**:
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` (update)
- `frontend/src/components/visualization/GraphPreview.tsx` (update)

**Tasks**:

1. **Update GraphNode Query**
   - Change to use GET_GRAPH_PREVIEW
   - Use node ID for lookup
   - Handle execution states

2. **Update GraphPreview Component**
   - Use data from graph entity
   - Handle empty graphs
   - Show execution state
   - Show processing indicator if computing

**Estimated effort**: 2-3 hours

### Phase 8: Execution State UI Indicators

**Files**:
- `frontend/src/components/editors/PlanVisualEditor/nodes/BaseNode.tsx` (extend)
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` (extend)
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` (extend)

**Tasks**:

1. **Add Execution State Badges**
   - Query execution state from entity
   - Display badge on node:
     - `not_started`: "Not Started" (gray)
     - `pending`: "Pending" (yellow)
     - `processing`: "Processing" with spinner (blue)
     - `completed`: "Ready" (green)
     - `error`: "Error" (red)

2. **Add Visual Indicators**
   - Processing spinner overlay
   - Error icon
   - Pulsing animation for processing
   - Tooltip with last update time

3. **Subscribe to State Updates**
   - Listen to WebSocket for execution state changes
   - Update badges in real-time
   - Show toast notification when processing completes

**Estimated effort**: 4-6 hours

### Phase 9: Testing & Documentation

**Tasks**:

1. **Backend Tests**
   - Unit tests for entity CRUD operations
   - Unit tests for DAG executor
   - Integration tests for pipeline execution
   - Test error scenarios

2. **Frontend Tests**
   - Unit tests for preview components
   - Integration tests for preview flow
   - Test all execution states
   - Test error handling

3. **End-to-End Tests**
   - Create DataSource node → verify entity created
   - Import CSV → verify rows populated
   - Create Graph node → verify graph computed
   - Preview DataSource → verify table displayed
   - Preview Graph → verify visualization displayed
   - Update config → verify recomputation
   - Delete node → verify entity deleted

4. **Documentation**
   - Update architecture documentation
   - Document entity schema
   - Document execution state machine
   - Add troubleshooting guide

**Estimated effort**: 8-12 hours

## File Changes Summary

### New Backend Files
- `backend/migrations/XXXX_add_pipeline_tables.sql` - Database schema
- `backend/src/db/models/datasource.rs` - DataSource entity model
- `backend/src/db/models/graph.rs` - Graph entity model
- `backend/src/pipeline/datasource_importer.rs` - CSV import logic
- `backend/src/pipeline/graph_builder.rs` - Graph construction logic
- `backend/src/pipeline/dag_executor.rs` - DAG execution orchestration
- `backend/graphql/resolvers/preview.rs` - Preview query resolvers

### Modified Backend Files
- `backend/graphql/schema.graphql` - Add preview queries
- `backend/src/commands/plan_dag.rs` - Hook pipeline execution to mutations

### New Frontend Files
- `frontend/src/graphql/preview.ts` - Preview queries
- `frontend/src/components/visualization/DataSourcePreview.tsx` - Table component
- `frontend/src/components/visualization/DataSourcePreviewDialog.tsx` - Table dialog

### Modified Frontend Files
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` - Add preview
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` - Update preview
- `frontend/src/components/visualization/GraphPreview.tsx` - Use new API
- `frontend/src/types/plan-dag.ts` - Add execution state types

## Architecture Decisions

### Entity Scoping
- All entities scoped by `project_id`
- One project can have multiple DataSource entities (one per DataSourceNode)
- One project can have multiple Graph entities (one per GraphNode)
- Use `(project_id, node_id)` as unique constraint

### When to Trigger Execution

**Immediate Execution**:
- DataSource node created with file path configured → Import CSV
- Edge added connecting source to sink → Recompute sink graph
- Node config updated → Re-import/re-build

**Deferred Execution**:
- Node created without config → Wait for configuration
- Edge deleted → Mark downstream as stale, recompute on next access

### State Transitions

```
not_started → pending → processing → completed
                                   → error

completed → pending (when upstream changes)
error → pending (on retry/reconfigure)
```

### Incremental Computation

- Hash upstream data to detect changes (store in `source_hash`)
- Only recompute if hash changed
- Track dependency graph for efficient invalidation

## Success Criteria

- [ ] Database migration creates all required tables
- [ ] DataSource node creation creates entity in `datasources` table
- [ ] CSV import populates `datasource_rows` table
- [ ] Graph node creation creates entity in `graphs` table
- [ ] Graph computation populates `graph_nodes` and `graph_edges` tables
- [ ] Preview button on DataSource shows table data
- [ ] Preview button on Graph shows force-graph visualization
- [ ] Execution states displayed accurately on all nodes
- [ ] Real-time updates via WebSocket when processing completes
- [ ] Error states displayed with clear messages
- [ ] Node deletion removes corresponding entity (cascade)
- [ ] Upstream changes trigger downstream recomputation
- [ ] Performance acceptable for 100K row datasets
- [ ] Multiple projects isolated by project_id scoping

## Total Estimated Effort

**61-87 hours** (approximately 8-11 days of development)

Breakdown:
- Phase 1: Database Schema - 6-8 hours
- Phase 2: Data Import & Transform - 20-28 hours
- Phase 3: Node Lifecycle Hooks - 8-12 hours
- Phase 4: GraphQL API - 6-8 hours
- Phase 5: Frontend GraphQL - 2-3 hours
- Phase 6: DataSource Preview UI - 5-7 hours
- Phase 7: Graph Preview Enhancement - 2-3 hours
- Phase 8: Execution State UI - 4-6 hours
- Phase 9: Testing & Documentation - 8-12 hours

## Risks & Mitigations

### Risk: Large Dataset Performance
**Mitigation**:
- Use batch inserts for datasource_rows
- Add pagination to preview queries
- Implement background processing for imports
- Add database indexes on frequently queried columns
- Consider partitioning datasource_rows by datasource_id

### Risk: JSONB Query Performance
**Mitigation**:
- Add GIN indexes on JSONB columns if needed
- Consider denormalizing frequently accessed fields
- Cache preview results

### Risk: Concurrent Execution Conflicts
**Mitigation**:
- Use database transactions for entity updates
- Lock entities during processing
- Queue execution requests to prevent conflicts

### Risk: Orphaned Entities
**Mitigation**:
- Use ON DELETE CASCADE for foreign keys
- Add cleanup job to verify entity integrity
- Log entity lifecycle events

## Future Enhancements

1. **Caching & Memoization**: Cache computation results to avoid reprocessing
2. **Streaming Preview**: Show preview as data loads (progressive rendering)
3. **Export Functionality**: Download entity data as CSV
4. **Data Quality Metrics**: Show statistics in preview
5. **Query Builder**: Filter/sort data before preview
6. **Version History**: Track entity versions for time-travel
7. **Scheduled Execution**: Periodic re-import of DataSources
8. **Parallel Execution**: Execute independent branches concurrently

## Migration Plan

### For Existing Projects

1. Run database migration to create new tables
2. Create entities for existing DAG nodes (migration script)
3. Trigger imports/builds for all existing nodes
4. Verify all entities created successfully
5. Update frontend to use new preview API

### For New Projects

- Tables already exist from initialization
- Entities created automatically as part of node creation flow
- No migration needed
