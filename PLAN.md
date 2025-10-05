# Plan: Direct Graph Execution from Data Sources

## Status: ✅ Implemented (Option A)

## Overview

Implemented direct graph execution that reads from `data_sources` table without background processing or duplicate data storage. Graph nodes build on-demand when user clicks execute button.

## Architecture

### Data Flow

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

2. **DagExecutor** (`layercake-core/src/pipeline/dag_executor.rs`)
   - Updated to accept `plan_id` parameter
   - Passes plan_id to GraphBuilder for config lookup
   - Handles topological execution ordering

3. **executeNode Mutation** (`layercake-core/src/graphql/mutations/mod.rs`)
   - GraphQL mutation: `executeNode(projectId: Int!, nodeId: String!)`
   - Returns `NodeExecutionResult { success, message, nodeId }`
   - Fetches all plan nodes and edges
   - Calls `DagExecutor.execute_node()`
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

### Step-by-Step Usage

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

## What Was NOT Implemented

### Merge Node
- **Status**: Not implemented
- **Reason**: Graph node can directly read from multiple DataSource nodes
- **Impact**: Low priority - same functionality achieved via Graph node

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

### Transform and Merge Nodes
- Implement `MergeBuilder` to combine sources with custom logic
- Implement transformations (filters, mappings, etc.)
- Chain operations: DataSource → Transform → Merge → Graph

## Files Modified

### Backend
- `layercake-core/src/pipeline/graph_builder.rs` - Direct data_sources read
- `layercake-core/src/pipeline/dag_executor.rs` - Added plan_id parameter
- `layercake-core/src/graphql/mutations/mod.rs` - Added executeNode mutation
- `layercake-core/src/graphql/types/plan_dag.rs` - Added dataSourceId to config
- `layercake-core/src/graphql/queries/mod.rs` - Modified datasource_preview

### Frontend
- `frontend/src/graphql/preview.ts` - Added EXECUTE_NODE mutation
- `frontend/src/components/editors/PlanVisualEditor/nodes/GraphNode.tsx` - Added execute button
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx` - Updated preview
- `frontend/src/components/visualization/DataPreview.tsx` - Created (table view)
- `frontend/src/components/visualization/DataPreviewDialog.tsx` - Created

## Commits

1. `feat: implement on-demand DataSource preview from data_sources table`
2. `feat: implement direct graph execution from data_sources`
3. `feat: add execute button to Graph nodes`

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

1. Test complete workflow end-to-end
2. Fix any build errors (cargo + npm)
3. Verify graph execution works with sample data
4. Document any issues found
