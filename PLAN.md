# Plan: Direct Graph Execution from Data Sources

## Status: ✅ Implemented (Option A + Merge Nodes)

## Overview

Implemented direct graph execution that reads from `data_sources` table without background processing or duplicate data storage. Graph nodes build on-demand when user clicks execute button. Merge nodes automatically execute when triggered by downstream Graph nodes, enabling complex data combination workflows.

## Architecture

### Data Flow (Basic)

```
Upload CSV → data_sources (graph_json) → DataSource Node (dataSourceId in config)
                                               ↓
                                          Graph Node (via edges)
                                               ↓
                                     Click Execute Button
                                               ↓
                              GraphBuilder reads data_sources.graph_json
                                               ↓
                           Combines nodes/edges from all upstream sources
                                               ↓
                        Populates graphs, graph_nodes, graph_edges tables
                                               ↓
                                    Execution state: completed
                                               ↓
                                      Click Preview Button
                                               ↓
                                    Graph Visualization
```

### Data Flow (With Merge Node)

```
DataSource(nodes.csv) → data_sources (graph_json) → DataSource Node A
                                                            ↓
DataSource(edges.csv) → data_sources (graph_json) → DataSource Node B
                                                            ↓
                                                       Merge Node
                                                            ↓
                                                       Graph Node
                                                            ↓
                                                  Click Execute Button
                                                            ↓
                                              DagExecutor finds upstream nodes
                                                            ↓
                                            Execute in topological order:
                                            1. DataSource A (skipped - already processed)
                                            2. DataSource B (skipped - already processed)
                                            3. Merge Node (combines A + B data)
                                            4. Graph Node (reads from Merge result)
                                                            ↓
                               MergeBuilder reads graph_json from DataSource A & B
                                                            ↓
                              Combines nodes and edges, stores in graphs table
                                                            ↓
                             GraphBuilder reads from graphs table (Merge result)
                                                            ↓
                                  Final graph in graphs table with all data
                                                            ↓
                                                  Preview available
```

### No Duplicate Data

- **data_sources.graph_json**: Source of truth (uploaded file processed once)
- **graphs/graph_nodes/graph_edges**: Computed view (built from data_sources on demand)
- **datasources/datasource_rows**: NOT USED (old pipeline approach)

## Implemented Features

### Backend

1. **GraphBuilder.build_graph()** (`layercake-core/src/pipeline/graph_builder.rs`)
   - Reads `plan_dag_nodes` config to get `dataSourceId` for each upstream node
   - Queries `data_sources` table directly (no pipeline processing)
   - Parses `graph_json` field to extract nodes/edges
   - Supports three data types:
     - `data_type: "nodes"` → extracts `graph_json.nodes[]`
     - `data_type: "edges"` → extracts `graph_json.edges[]`
     - `data_type: "graph"` → extracts both nodes and edges
   - Combines all nodes and edges from multiple upstream sources
   - Validates edge references
   - Populates `graphs`, `graph_nodes`, `graph_edges` tables
   - Sets `execution_state: "completed"` when done

2. **MergeBuilder.merge_sources()** (`layercake-core/src/pipeline/merge_builder.rs`)
   - Combines nodes and edges from multiple upstream sources
   - Supports reading from:
     - DataSource nodes: reads `data_sources.graph_json`
     - Graph nodes: reads `graph_nodes` and `graph_edges` tables
     - Merge nodes: reads `graph_nodes` and `graph_edges` tables (recursive merging)
   - Deduplicates nodes and edges by ID
   - Stores merged result in `graphs` table (reuses Graph infrastructure)
   - Implements change detection via SHA256 hash of all upstream source IDs
   - Returns `graphs::Model` with `execution_state: "completed"`

3. **DagExecutor** (`layercake-core/src/pipeline/dag_executor.rs`)
   - Updated to accept `plan_id` parameter
   - Passes plan_id to GraphBuilder and MergeBuilder for config lookup
   - Handles topological execution ordering
   - **New**: `execute_with_dependencies()` - executes node and all upstream ancestors
   - **New**: `find_upstream_nodes()` - traverses DAG backwards to find all dependencies
   - Ensures Merge nodes execute before downstream Graph nodes

4. **executeNode Mutation** (`layercake-core/src/graphql/mutations/mod.rs`)
   - GraphQL mutation: `executeNode(projectId: Int!, nodeId: String!)`
   - Returns `NodeExecutionResult { success, message, nodeId }`
   - Fetches all plan nodes and edges
   - Calls `DagExecutor.execute_with_dependencies()` (executes upstream dependencies first)
   - Returns success/error to frontend

### Frontend

1. **Execute Button** (`frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx`)
   - Green filled play button (IconPlayerPlayFilled)
   - Only shows when Graph node has upstream connections (isConfigured)
   - Calls `EXECUTE_NODE` mutation
   - Shows loading spinner during execution
   - Displays success/error notifications
   - Refetches execution state after completion

2. **Preview Button** (`frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx`)
   - Blue outline play button (IconPlayerPlay)
   - Only shows when `execution_state: "completed"`
   - Opens GraphPreviewDialog with force-graph visualization

3. **Execution State Badge**
   - Shows current state: "Not Started", "Processing", "Ready", "Error"
   - Color-coded: gray, blue, green, red
   - Updates after execute completes

4. **GraphQL Integration** (`frontend/src/graphql/preview.ts`)
   - `EXECUTE_NODE` mutation
   - `NodeExecutionResult` interface
   - Integrated into existing preview hooks

## User Workflow

### Basic Workflow (Direct Graph)

1. **Upload Data Sources**
   - Upload `nodes.csv` → creates data_source (id: 1, data_type: "nodes")
   - Upload `edges.csv` → creates data_source (id: 2, data_type: "edges")
   - Both files processed to `graph_json` format (status: "active")

2. **Create Plan DAG**
   - Add DataSource node "Nodes" → assign data_source id: 1
   - Add DataSource node "Edges" → assign data_source id: 2
   - Add Graph node "My Graph"
   - Connect "Nodes" → "My Graph"
   - Connect "Edges" → "My Graph"

3. **Execute Graph**
   - Graph node shows green "Execute" button (configured with 2 upstream edges)
   - Click Execute → mutation fires
   - Badge shows "Processing" with spinner
   - GraphBuilder reads graph_json from both data sources
   - Combines nodes and edges
   - Populates database tables
   - Badge updates to "Ready"

4. **Preview Graph**
   - Blue "Preview" button appears (execution complete)
   - Click Preview → opens dialog
   - Force-graph visualization displays nodes and edges

### Merge Workflow (With Merge Node)

1. **Upload Data Sources**
   - Upload `nodes.csv` → creates data_source (id: 1, data_type: "nodes")
   - Upload `edges.csv` → creates data_source (id: 2, data_type: "edges")
   - Both files processed to `graph_json` format (status: "active")

2. **Create Plan DAG with Merge**
   - Add DataSource node "Nodes" → assign data_source id: 1
   - Add DataSource node "Edges" → assign data_source id: 2
   - Add Merge node "Combined Data"
   - Add Graph node "My Graph"
   - Connect "Nodes" → "Combined Data"
   - Connect "Edges" → "Combined Data"
   - Connect "Combined Data" → "My Graph"

3. **Execute Graph (Automatic Upstream Execution)**
   - Graph node shows green "Execute" button
   - Click Execute → mutation fires
   - DagExecutor finds all upstream nodes: Nodes, Edges, Combined Data
   - Executes in topological order:
     1. DataSource "Nodes" (skipped - already processed on upload)
     2. DataSource "Edges" (skipped - already processed on upload)
     3. Merge "Combined Data" (executes - combines nodes and edges)
     4. Graph "My Graph" (executes - reads from Merge result)
   - Badge updates to "Ready"

4. **Preview Graph**
   - Blue "Preview" button appears
   - Click Preview → visualizes the final merged graph

## Database Tables

### data_sources (Source of Truth)
```sql
id, project_id, name, filename, source_type, data_type,
graph_json, status, processed_at
```
- Created on file upload
- `graph_json`: Contains parsed nodes/edges
- `status: "active"` when ready
- Never modified by graph execution

### graphs (Computed View)
```sql
id, project_id, node_id, name, execution_state,
computed_date, source_hash, node_count, edge_count
```
- Created/updated by `executeNode`
- `node_id`: Links to plan_dag_nodes
- `execution_state`: "not_started", "processing", "completed", "error"
- `source_hash`: SHA256 of upstream data_sources (for change detection)

### graph_nodes & graph_edges (Computed Data)
```sql
-- graph_nodes
id, graph_id, label, layer, weight, is_partition, attrs

-- graph_edges
id, graph_id, source, target, label, layer, weight, attrs
```
- Populated by GraphBuilder
- Deleted and recreated on each execution (clear_graph_data)

## Merge Node Implementation

### MergeBuilder
- **Status**: ✅ Implemented
- **Purpose**: Combine nodes and edges from multiple upstream sources (DataSource, Graph, or Merge nodes)
- **Features**:
  - Reads from `data_sources.graph_json` for DataSource nodes
  - Reads from `graph_nodes`/`graph_edges` tables for Graph/Merge nodes
  - Deduplicates nodes and edges by ID
  - Stores results in `graphs` table (reuses Graph infrastructure)
  - Implements change detection via SHA256 `source_hash`
  - Automatic execution when downstream Graph nodes trigger

### Upstream Dependency Execution
- **Status**: ✅ Implemented
- **Method**: `execute_with_dependencies()` in DagExecutor
- **Behavior**: When executing a node, automatically finds and executes all upstream ancestors in topological order
- **Example**: Click execute on Graph → executes DataSources, then Merge, then Graph

## What Was NOT Implemented

### Transform Node
- **Status**: Not implemented
- **Purpose**: Filter or transform data (e.g., filter nodes by layer, map attributes)
- **Impact**: Medium priority - useful for data manipulation workflows

### Automatic Lifecycle Hooks
- **Status**: Not implemented (TODO markers exist in mutations)
- **Reason**: Manual execute button provides better user control
- **Current**: User must click "Execute" to trigger graph building
- **Future**: Could auto-execute when edges are connected

### DataSource Pipeline Import
- **Status**: Not needed for new system
- **Old Approach**: `DatasourceImporter` would read CSV and populate `datasource_rows`
- **New Approach**: `data_sources.graph_json` already contains parsed data
- **Impact**: Simpler, no duplicate storage

## Future Enhancements

### Change Detection
- Currently: Graph rebuilds from scratch on each execute
- Enhancement: Compare `source_hash` and skip if unchanged
- Implementation: Already coded but always rebuilds for now

### Incremental Updates
- Track which upstream sources changed
- Only recompute affected downstream nodes
- Use topological sort in DagExecutor

### Background Execution
- Run graph building asynchronously
- Poll execution state until complete
- Show progress updates

### Transform Nodes
- Implement transformations (filters, mappings, etc.)
- Chain operations: DataSource → Transform → Merge → Graph
- Examples: filter by layer, map attributes, compute derived values

## Files Modified

### Backend
- `layercake-core/src/pipeline/graph_builder.rs` - Direct data_sources read
- `layercake-core/src/pipeline/dag_executor.rs` - Added plan_id parameter, execute_with_dependencies, find_upstream_nodes
- `layercake-core/src/pipeline/merge_builder.rs` - **NEW** - MergeBuilder implementation
- `layercake-core/src/pipeline/mod.rs` - Added merge_builder module export
- `layercake-core/src/graphql/mutations/mod.rs` - Added executeNode mutation with dependency execution
- `layercake-core/src/graphql/types/plan_dag.rs` - Added dataSourceId to config
- `layercake-core/src/graphql/queries/mod.rs` - Modified datasource_preview

### Frontend
- `frontend/src/graphql/preview.ts` - Added EXECUTE_NODE mutation
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` - Added execute button
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` - Updated preview
- `frontend/src/components/visualization/DataPreview.tsx` - Created (table view)
- `frontend/src/components/visualization/DataPreviewDialog.tsx` - Created

## Commits

### Initial Implementation (Option A)
1. `feat: implement on-demand DataSource preview from data_sources table`
2. `feat: implement direct graph execution from data_sources`
3. `feat: add execute button to Graph nodes`
4. `fix: resolve cargo and npm build errors`
5. `fix: correct node type matching in DagExecutor`

### Merge Node Implementation (Option B)
6. `feat: implement MergeBuilder for combining data from multiple upstream sources`
7. `feat: add automatic upstream dependency execution for DAG nodes`

## Testing Checklist

- [x] Upload CSV file → data_sources created
- [x] Assign to DataSource node → dataSourceId saved
- [x] DataSource preview works (reads graph_json)
- [x] Connect DataSources to Graph node → isConfigured true
- [x] Execute button appears on Graph node
- [ ] Click Execute → graph builds successfully
- [ ] Execution state badge updates to "Ready"
- [ ] Preview button appears
- [ ] Click Preview → graph visualization displays

## Next Steps

### Immediate Testing
1. Test basic workflow: DataSource → Graph → Preview
2. Test merge workflow: DataSource + DataSource → Merge → Graph → Preview
3. Verify upstream dependency execution works correctly
4. Test with sample CSV files (nodes.csv + edges.csv)

### Future Work
1. Implement Transform nodes for data filtering/mapping
2. Add automatic lifecycle hooks (auto-execute on edge connect)
3. Implement incremental updates (skip unchanged nodes)
4. Add background execution with progress tracking
