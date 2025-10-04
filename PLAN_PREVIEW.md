# Plan: DataSource and Graph Node Preview via Backend Pipeline Tables

## Overview

Add preview functionality for both DataSource and Graph nodes by extending the existing backend CLI pipeline to maintain database tables for each DAG node. Updates to the DAG trigger data processing on the backend, populating these tables which can then be queried for preview.

## Architecture Context

### Current Backend State
- **Existing CLI**: Backend has a CLI interface with pipeline concept for data processing
- **Full Dataset Processing**: All transformations operate on complete datasets, not subsets
- **File-based Input**: CSV files (nodes.csv, links.csv) are input to the pipeline

### Target Architecture
- **DataSource Tables**: Each DAG DataSourceNode → corresponding DataSource table in database
- **LayercakeGraph Tables**: Each DAG GraphNode → corresponding LayercakeGraph table in database
- **Event-Driven Updates**: Changes to DAG nodes trigger pipeline execution to update tables
- **Query for Preview**: Frontend queries these pre-computed tables for display

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
│ DataSourceNode Created  │ → Create DataSource Table → Import CSV data
│ GraphNode Created       │ → Create LayercakeGraph Table (empty initially)
│ Edge Created/Deleted    │ → Recalculate affected downstream tables
│ Config Updated          │ → Reprocess affected node and downstream
└─────────────────────────┘
    ↓
Database Tables Updated
    ↓
Frontend Query for Preview → Display in UI
```

## Requirements

### Functional Requirements

1. **Table Management**
   - Create database table when DAG node is created
   - Delete table when DAG node is deleted
   - Update table when node configuration changes
   - Cascade updates through downstream nodes when dependencies change

2. **DataSource Tables**
   - Schema matches CSV structure (dynamic columns)
   - Populated immediately when DataSourceNode is configured
   - Support both node CSVs (id, label, layer, etc.) and edge CSVs (source, target, etc.)
   - Include metadata columns: `_node_id`, `_import_date`, `_row_number`

3. **LayercakeGraph Tables**
   - Schema: standard graph structure (nodes + edges)
   - Computed from upstream DataSource/Transform/Merge nodes
   - Include metadata: `_node_id`, `_computed_date`, `_source_hash`
   - Store both node and edge data in normalized form

4. **Preview API**
   - Query any table by node ID
   - Support pagination (offset/limit)
   - Return schema information (columns, types)
   - Return row count and metadata

5. **Incremental Updates**
   - Detect when upstream changes require recomputation
   - Only reprocess affected subgraph, not entire DAG
   - Track computation state (pending, processing, complete, error)

### Non-Functional Requirements

1. **Performance**
   - Table creation should be async (non-blocking)
   - Large imports should use batch processing
   - Preview queries should be fast (< 100ms for first 100 rows)

2. **Reliability**
   - Failed imports should not leave partial data
   - Retry logic for transient errors
   - Clear error messages for data quality issues

3. **Scalability**
   - Support datasets up to 1M rows per node
   - Handle DAGs with up to 100 nodes
   - Concurrent execution of independent subgraphs

## Implementation Plan

### Phase 1: Pipeline Extension - Table Management

**Files**:
- `backend/src/pipeline/dag_executor.rs` (new)
- `backend/src/pipeline/table_manager.rs` (new)
- `backend/src/db/schema.rs` (extend)

**Tasks**:

1. **Create Table Manager**
   - `create_datasource_table(node_id, csv_schema)` → Creates table with dynamic schema
   - `create_graph_table(node_id)` → Creates standard graph table structure
   - `drop_table(node_id)` → Drops table when node deleted
   - `table_exists(node_id)` → Check if table exists
   - `get_table_info(node_id)` → Return schema and row count

2. **Define Table Schemas**
   - DataSource tables: Dynamic based on CSV columns + metadata columns
   - Graph tables: Fixed schema for nodes and edges

   ```sql
   -- DataSource table (dynamic columns)
   CREATE TABLE datasource_{node_id} (
     _row_id SERIAL PRIMARY KEY,
     _node_id TEXT NOT NULL,
     _import_date TIMESTAMP NOT NULL,
     _row_number INTEGER NOT NULL,
     -- CSV columns added dynamically
     {column_name} {column_type},
     ...
   );

   -- Graph table (fixed schema)
   CREATE TABLE graph_{node_id}_nodes (
     id TEXT PRIMARY KEY,
     label TEXT,
     layer TEXT,
     weight REAL,
     is_partition BOOLEAN,
     attrs JSONB,
     _node_id TEXT NOT NULL,
     _computed_date TIMESTAMP NOT NULL
   );

   CREATE TABLE graph_{node_id}_edges (
     id TEXT PRIMARY KEY,
     source TEXT NOT NULL,
     target TEXT NOT NULL,
     label TEXT,
     layer TEXT,
     weight REAL,
     attrs JSONB,
     _node_id TEXT NOT NULL,
     _computed_date TIMESTAMP NOT NULL
   );
   ```

3. **Implement Table Lifecycle Hooks**
   - Hook into `createNode` mutation → trigger table creation
   - Hook into `deleteNode` mutation → trigger table deletion
   - Hook into `updateNode` mutation → trigger table update
   - Hook into `addEdge`/`deleteEdge` mutations → trigger downstream recalculation

**Estimated effort**: 8-12 hours

### Phase 2: Pipeline Extension - Data Import & Transform

**Files**:
- `backend/src/pipeline/datasource_importer.rs` (new)
- `backend/src/pipeline/graph_builder.rs` (new)
- `backend/src/pipeline/dag_executor.rs` (extend)

**Tasks**:

1. **DataSource Importer**
   - Read CSV file from configured path
   - Infer or validate schema
   - Batch insert into DataSource table
   - Handle CSV parsing errors gracefully
   - Update node metadata with import statistics

2. **Graph Builder**
   - Read from upstream DataSource/Transform/Merge tables
   - Apply transformation logic (existing pipeline code)
   - Write to Graph table (nodes + edges)
   - Validate graph structure (no cycles, valid references)
   - Update node metadata with computation statistics

3. **DAG Executor**
   - Topological sort of DAG to determine execution order
   - Identify affected subgraph when node/edge changes
   - Execute pipeline stages in order:
     1. DataSource import
     2. Merge operations
     3. Transform operations
     4. Graph construction
   - Track execution state per node (pending → processing → complete/error)
   - Parallel execution of independent branches

4. **State Management**
   - Add `execution_state` field to PlanDag nodes
   - States: `not_started`, `pending`, `processing`, `completed`, `error`
   - Add `execution_metadata` field: timestamps, row counts, error messages
   - Broadcast state changes via WebSocket subscriptions

**Estimated effort**: 16-24 hours

### Phase 3: GraphQL API for Preview

**Files**:
- `backend/graphql/schema.graphql` (extend)
- `backend/graphql/resolvers/preview.rs` (new)

**Tasks**:

1. **Add GraphQL Schema**
   ```graphql
   # Generic table preview
   type TableColumn {
     name: String!
     dataType: String!
     nullable: Boolean!
   }

   type TableRow {
     rowNumber: Int!
     data: JSON!
   }

   type TablePreview {
     nodeId: String!
     nodeType: String!
     tableName: String!
     totalRows: Int!
     columns: [TableColumn!]!
     rows: [TableRow!]!
     executionState: String!
     lastUpdated: DateTime
     errorMessage: String
   }

   # DataSource-specific preview
   type DataSourcePreview {
     nodeId: String!
     filename: String!
     fileSize: Int
     totalRows: Int!
     columns: [TableColumn!]!
     rows: [TableRow!]!
     importDate: DateTime
     executionState: String!
     errorMessage: String
   }

   # Graph-specific preview
   type GraphNodePreview {
     id: String!
     label: String!
     layer: String!
     weight: Float
     isPartition: Boolean!
     attrs: JSON
   }

   type GraphEdgePreview {
     id: String!
     source: String!
     target: String!
     label: String!
     layer: String!
     weight: Float
     attrs: JSON
   }

   type GraphPreview {
     nodeId: String!
     nodes: [GraphNodePreview!]!
     edges: [GraphEdgePreview!]!
     executionState: String!
     lastUpdated: DateTime
     errorMessage: String
   }

   extend type Query {
     # Generic table preview (works for any node)
     tablePreview(
       projectId: Int!
       nodeId: String!
       limit: Int = 100
       offset: Int = 0
     ): TablePreview

     # DataSource-specific preview
     dataSourcePreview(
       projectId: Int!
       nodeId: String!
       limit: Int = 100
       offset: Int = 0
     ): DataSourcePreview

     # Graph-specific preview
     graphPreview(
       projectId: Int!
       nodeId: String!
     ): GraphPreview
   }
   ```

2. **Implement Resolvers**
   - `tablePreview`: Query generic table, return rows as JSON
   - `dataSourcePreview`: Query DataSource table with metadata
   - `graphPreview`: Query Graph table (nodes + edges)
   - Handle pagination efficiently with OFFSET/LIMIT
   - Include execution state and error information

3. **Error Handling**
   - Return meaningful errors if table doesn't exist
   - Return state information if processing not complete
   - Handle database query errors gracefully

**Estimated effort**: 6-8 hours

### Phase 4: Frontend GraphQL Integration

**Files**:
- `frontend/src/graphql/preview.ts` (new)
- Update existing `frontend/src/graphql/graphs.ts`

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
         filename
         fileSize
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
         executionState
         lastUpdated
         errorMessage
       }
     }
   `
   ```

2. **Define TypeScript Interfaces**
   ```typescript
   export interface TableColumn {
     name: string
     dataType: string
     nullable: boolean
   }

   export interface TableRow {
     rowNumber: number
     data: Record<string, any>
   }

   export interface DataSourcePreviewResponse {
     nodeId: string
     filename: string
     fileSize?: number
     totalRows: number
     columns: TableColumn[]
     rows: TableRow[]
     importDate?: string
     executionState: string
     errorMessage?: string
   }

   export interface GraphPreviewResponse {
     nodeId: string
     nodes: GraphNodePreview[]
     edges: GraphEdgePreview[]
     executionState: string
     lastUpdated?: string
     errorMessage?: string
   }
   ```

**Estimated effort**: 2-3 hours

### Phase 5: Frontend Components - DataSource Preview

**Files**:
- `frontend/src/components/visualization/DataSourcePreview.tsx` (new)
- `frontend/src/components/visualization/DataSourcePreviewDialog.tsx` (new)
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` (extend)

**Tasks**:

1. **Create DataSourcePreview Component**
   - Mantine Table with virtualized scrolling
   - Sticky column headers
   - Display execution state badge (pending, processing, completed, error)
   - Show loading skeleton while data fetches
   - Show error message if import failed
   - Show "Processing..." message if state is pending/processing

2. **Create DataSourcePreviewDialog Component**
   - Modal with table preview
   - Header: filename, row count, column count, import date
   - Status indicator: execution state
   - Footer: pagination controls (if needed)
   - Error alert if errorMessage present

3. **Update DataSourceNode**
   - Add centered play button (IconPlayerPlay or IconTable)
   - Add state management for preview dialog
   - Query on preview open
   - Handle all execution states:
     - `not_started`: Show message "Click to import data"
     - `pending`/`processing`: Show spinner, disable preview temporarily
     - `completed`: Show table preview
     - `error`: Show error message

**Estimated effort**: 5-7 hours

### Phase 6: Frontend Components - Graph Preview Enhancement

**Files**:
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` (update)
- `frontend/src/components/visualization/GraphPreview.tsx` (update)

**Tasks**:

1. **Update GraphNode Query**
   - Change from GET_GRAPH_DATA to GET_GRAPH_PREVIEW
   - Use node ID instead of project ID for lookup
   - Handle execution states like DataSourceNode

2. **Update GraphPreview Component**
   - Use data from graph table instead of generic graphData query
   - Handle empty graphs (no nodes/edges)
   - Show execution state
   - Show processing indicator if graph is being computed

**Estimated effort**: 2-3 hours

### Phase 7: Execution State UI Indicators

**Files**:
- `frontend/src/components/editors/PlanVisualEditor/nodes/BaseNode.tsx` (extend)
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` (extend)
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` (extend)

**Tasks**:

1. **Add Execution State Badges**
   - Add `executionState` to node data
   - Display badge in node header:
     - `not_started`: No badge or "Not Started" badge (gray)
     - `pending`: "Pending" badge (yellow)
     - `processing`: "Processing" badge with spinner (blue)
     - `completed`: "Ready" badge (green)
     - `error`: "Error" badge (red)

2. **Add Visual Indicators**
   - Processing spinner overlay on node
   - Error icon on node
   - Pulsing animation for processing state
   - Tooltip with last update time

3. **Subscribe to State Updates**
   - Listen to WebSocket for execution state changes
   - Update badges in real-time
   - Show toast notification when processing completes

**Estimated effort**: 4-6 hours

### Phase 8: Testing & Documentation

**Tasks**:

1. **Backend Tests**
   - Unit tests for table manager
   - Unit tests for DAG executor
   - Integration tests for pipeline execution
   - Test error scenarios (invalid CSV, cycles, missing files)

2. **Frontend Tests**
   - Unit tests for preview components
   - Integration tests for preview flow
   - Test all execution states
   - Test error handling

3. **End-to-End Tests**
   - Create DataSource node → verify table created
   - Import CSV → verify table populated
   - Create Graph node connected to DataSource → verify graph computed
   - Preview DataSource → verify data displayed
   - Preview Graph → verify graph displayed
   - Update DataSource config → verify downstream recomputation
   - Delete node → verify table dropped

4. **Documentation**
   - Update architecture documentation
   - Document table schema conventions
   - Document execution state machine
   - Add troubleshooting guide

**Estimated effort**: 8-12 hours

## File Changes Summary

### New Backend Files
- `backend/src/pipeline/dag_executor.rs` - DAG execution orchestration
- `backend/src/pipeline/table_manager.rs` - Database table lifecycle management
- `backend/src/pipeline/datasource_importer.rs` - CSV import to tables
- `backend/src/pipeline/graph_builder.rs` - Graph construction from sources
- `backend/graphql/resolvers/preview.rs` - Preview query resolvers

### Modified Backend Files
- `backend/src/db/schema.rs` - Add dynamic table support
- `backend/graphql/schema.graphql` - Add preview queries
- `backend/src/commands/mod.rs` - Hook pipeline execution to mutations

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

### Table Naming Convention
- DataSource tables: `datasource_{node_id}`
- Graph node tables: `graph_{node_id}_nodes`
- Graph edge tables: `graph_{node_id}_edges`
- Use node ID (not label) for uniqueness and stability

### When to Trigger Execution

**Immediate Execution**:
- DataSource node created with file path configured → Import CSV
- Edge added connecting source to sink → Recompute sink
- Node config updated → Recompute node and downstream

**Deferred Execution**:
- Node created without config → Wait for config
- Edge deleted → Mark downstream as stale, recompute on next access

### State Transitions

```
not_started → pending → processing → completed
                                   → error

completed → pending (when upstream changes)
error → pending (on retry/reconfigure)
```

### Incremental Computation

- Hash upstream data to detect changes
- Only recompute if hash changed
- Cache computation results
- Track dependency graph for efficient invalidation

## Success Criteria

- [ ] DataSource node creation triggers table creation
- [ ] CSV import populates DataSource table
- [ ] Graph node connected to source triggers graph computation
- [ ] Preview button on DataSource shows table data
- [ ] Preview button on Graph shows force-graph visualization
- [ ] Execution states displayed accurately on all nodes
- [ ] Real-time updates via WebSocket when processing completes
- [ ] Error states displayed with clear messages
- [ ] Node deletion removes corresponding tables
- [ ] Upstream changes trigger downstream recomputation
- [ ] Performance acceptable for 100K row datasets
- [ ] Multiple concurrent DAG executions don't interfere

## Total Estimated Effort

**51-75 hours** (approximately 6-9 days of development)

Breakdown:
- Phase 1: Table Management - 8-12 hours
- Phase 2: Data Import & Transform - 16-24 hours
- Phase 3: GraphQL API - 6-8 hours
- Phase 4: Frontend GraphQL - 2-3 hours
- Phase 5: DataSource Preview UI - 5-7 hours
- Phase 6: Graph Preview Enhancement - 2-3 hours
- Phase 7: Execution State UI - 4-6 hours
- Phase 8: Testing & Documentation - 8-12 hours

## Risks & Mitigations

### Risk: Large Dataset Performance
**Mitigation**:
- Use streaming/batching for imports
- Add pagination to preview
- Implement background processing with job queue
- Add table indexes for common queries

### Risk: Table Schema Conflicts
**Mitigation**:
- Validate schema on CSV import
- Handle schema changes gracefully
- Version table schemas if needed

### Risk: Concurrent Execution Conflicts
**Mitigation**:
- Use database transactions
- Lock nodes during processing
- Queue execution requests

### Risk: Orphaned Tables
**Mitigation**:
- Implement cleanup routine for orphaned tables
- Track table ownership in metadata
- Add admin command to audit and clean tables

## Future Enhancements

1. **Caching & Memoization**: Cache computation results to avoid reprocessing
2. **Streaming Preview**: Show preview as data loads (progressive rendering)
3. **Export Functionality**: Download table data as CSV
4. **Data Quality Metrics**: Show statistics (null count, unique values, distributions)
5. **Query Builder**: Filter/sort table data before preview
6. **Version History**: Track table versions and allow time-travel
7. **Scheduled Execution**: Periodic re-import of DataSources
8. **Parallel Execution**: Execute independent branches concurrently
9. **Distributed Processing**: Scale to larger datasets with distributed compute

## Migration Plan

### For Existing Projects

1. Run migration script to create tables for existing nodes
2. Re-import all DataSource nodes
3. Recompute all Graph nodes
4. Verify all tables created successfully
5. Update frontend to use new preview API

### For New Projects

- Tables created automatically as part of node creation flow
- No migration needed
