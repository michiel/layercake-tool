# Dataset Source Tracking Implementation Plan

## Overview

Currently, when datasets are used as inputs for graphs, the information about source datasets for individual nodes and edges is lost during transformations and merges. This plan ensures that `source_dataset_id` is properly preserved throughout the pipeline so we can trace which dataset each node/edge originally came from.

**Key Goals:**
- Preserve `source_dataset_id` on nodes/edges when they pass through transformations
- Ensure dataset imports correctly set `source_dataset_id`
- Make dataset source information available in GraphQL and UI
- No transformation tracking needed - focus only on dataset provenance

## Current State

### Database Schema
- `graph_data_nodes` has `source_dataset_id INTEGER NULL` column
- `graph_data_edges` has `source_dataset_id INTEGER NULL` column
- Both refer to `data_sets.id` when the node/edge came from a dataset
- **Already exists** - no schema changes needed

### In-Memory Graph Structure (Rust)
```rust
pub struct Graph {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
    pub annotations: Option<String>,
}

pub struct Node {
    pub id: String,
    // ... other fields
    pub dataset: Option<i32>,  // Maps to source_dataset_id
}

pub struct Edge {
    pub id: String,
    // ... other fields
    pub dataset: Option<i32>,  // Maps to source_dataset_id
}
```

### Current Issues
1. **Dataset imports may not set `source_dataset_id`** - Need to verify and fix
2. **Transformations may lose `source_dataset_id`** - Filter/Transform/Merge need to preserve it
3. **GraphQL may not expose dataset info** - Need to add resolvers
4. **UI doesn't show dataset source** - Need to display which dataset nodes/edges came from

## Target State

### Database Schema
- **No changes needed** - Keep existing `source_dataset_id` columns as-is

### In-Memory Graph Structure
- **No changes needed** - Keep existing `dataset: Option<i32>` field
- Dataset metadata can be looked up from project's datasets when needed

### Behavior Changes

1. **Dataset Import**: Always set `source_dataset_id` on imported nodes/edges
2. **Filter Operations**: Preserve `source_dataset_id` on filtered nodes/edges
3. **Transform Operations**:
   - Preserve `source_dataset_id` on modified nodes/edges
   - New nodes/edges created by transforms have `source_dataset_id = NULL`
4. **Merge Operations**: Preserve `source_dataset_id` from source nodes/edges
5. **GraphQL**: Add resolver to fetch dataset details by ID
6. **UI**: Display dataset name/info for nodes/edges with `source_dataset_id`

## Implementation Phases

### Phase 1: Audit Current State

**Goal**: Understand where `source_dataset_id` is currently being set/preserved

**Tasks**:
1. Audit `GraphDataService::replace_nodes()` - verify it sets `source_dataset_id`
2. Audit `GraphDataService::replace_edges()` - verify it sets `source_dataset_id`
3. Audit `GraphDataService::load_full()` - verify it populates `node.dataset` and `edge.dataset`
4. Check all transformation builders:
   - `FilterBuilder::filter_graph()` - does it preserve `dataset` field?
   - `TransformBuilder` operations - do they preserve `dataset` field?
   - `MergeBuilder::merge_sources()` - does it preserve `dataset` field?
5. Document current behavior in test file

**Success Criteria**:
- Clear documentation of which operations currently preserve `source_dataset_id`
- List of gaps where `source_dataset_id` is being lost

### Phase 2: Fix Dataset Import

**Goal**: Ensure all dataset imports set `source_dataset_id`

**Tasks**:
1. Update `GraphDataService::replace_nodes()` in `layercake-core/src/services/graph_data_service.rs`:
   ```rust
   pub async fn replace_nodes(
       &self,
       graph_data_id: i32,
       nodes: &[graph_data_nodes::Model],
       source_dataset_id: Option<i32>, // NEW parameter
   ) -> Result<()> {
       // ... existing code ...

       for node in nodes.iter() {
           let active = graph_data_nodes::ActiveModel {
               graph_data_id: Set(graph_data_id),
               external_id: Set(node.external_id.clone()),
               source_dataset_id: Set(source_dataset_id), // Set from parameter
               // ... other fields
           };
           // ... insert
       }
   }
   ```

2. Update `GraphDataService::replace_edges()` similarly:
   ```rust
   pub async fn replace_edges(
       &self,
       graph_data_id: i32,
       edges: &[graph_data_edges::Model],
       source_dataset_id: Option<i32>, // NEW parameter
   ) -> Result<()> {
       // Similar changes to replace_nodes
   }
   ```

3. Update dataset import flow in `DataSetService::create_from_file()`:
   - When calling `replace_nodes()` and `replace_edges()`, pass `Some(dataset.id)`
   - Ensure the dataset ID is available at the time of import

4. Add test to verify nodes/edges have `source_dataset_id` after import

**Code Locations**:
- `layercake-core/src/services/graph_data_service.rs`: `replace_nodes()`, `replace_edges()`
- `layercake-core/src/app_context/data_set_operations.rs`: dataset import logic

**Success Criteria**:
- All nodes/edges imported from datasets have `source_dataset_id` set to dataset ID
- Test verifies `source_dataset_id` persistence
- No breaking changes to existing code

### Phase 3: Preserve Dataset Info in Transformations

**Goal**: Ensure transformations preserve `source_dataset_id` on nodes/edges

**Tasks**:

1. **FilterBuilder** (`layercake-core/src/pipeline/filter_builder.rs`):
   - Verify nodes/edges keep their `dataset` field when filtered
   - No changes likely needed - filtering just removes elements, doesn't modify them
   - Add test to confirm

2. **TransformBuilder** (`layercake-core/src/pipeline/transform_builder.rs`):
   - For operations that modify existing nodes/edges (weight changes, label updates):
     - **Preserve** the `dataset` field from source
   - For operations that create new nodes/edges:
     - **Set** `dataset = None` (new elements don't come from a dataset)
   - Review each transform operation type and update accordingly

3. **MergeBuilder** (`layercake-core/src/pipeline/merge_builder.rs`):
   - Already has logic to preserve `dataset` field - verify it works:
     ```rust
     // In merge_sources() around line 64
     entry.dataset = entry.dataset.or(node.dataset);
     ```
   - Add test to confirm dataset IDs are preserved from upstream graphs

**Code Example** (TransformBuilder):
```rust
// When modifying existing nodes
pub async fn update_node_weights(&self, /* ... */) -> Result<Graph> {
    let mut graph = /* load upstream graph */;

    for node in &mut graph.nodes {
        node.weight = compute_new_weight(node);
        // node.dataset is preserved automatically - no change needed
    }

    Ok(graph)
}

// When creating new edges
pub async fn create_derived_edges(&self, /* ... */) -> Result<Graph> {
    let mut graph = /* load upstream graph */;

    for new_edge in compute_edges() {
        graph.edges.push(Edge {
            id: new_edge.id,
            source: new_edge.source,
            target: new_edge.target,
            dataset: None, // NEW element, no dataset source
            // ... other fields
        });
    }

    Ok(graph)
}
```

**Success Criteria**:
- Filter preserves `dataset` field
- Transform preserves `dataset` on modified elements, sets `None` on new elements
- Merge preserves `dataset` from upstream sources
- Tests verify preservation through full pipeline

### Phase 4: GraphQL API Updates

**Goal**: Expose dataset source information through GraphQL

**Tasks**:

1. Verify `Node` and `Edge` GraphQL types expose `dataset` field:
   ```rust
   // In layercake-core/src/graphql/types/graph.rs

   #[derive(SimpleObject)]
   pub struct Node {
       pub id: String,
       pub label: String,
       // ... other fields

       /// Source dataset ID if this node came from a dataset
       #[graphql(name = "datasetId")]
       pub dataset_id: Option<i32>,
   }

   #[derive(SimpleObject)]
   pub struct Edge {
       pub id: String,
       pub source: String,
       pub target: String,
       // ... other fields

       /// Source dataset ID if this edge came from a dataset
       #[graphql(name = "datasetId")]
       pub dataset_id: Option<i32>,
   }
   ```

2. Add resolver to fetch dataset details (optional - for convenience):
   ```rust
   impl Node {
       async fn dataset(&self, ctx: &Context<'_>) -> Result<Option<DataSet>> {
           let dataset_id = match self.dataset_id {
               Some(id) => id,
               None => return Ok(None),
           };

           let context = ctx.data::<GraphQLContext>()?;
           let dataset = data_sets::Entity::find_by_id(dataset_id)
               .one(&context.db)
               .await?;

           Ok(dataset.map(DataSet::from))
       }
   }
   ```

3. Update GraphQL queries to include dataset info:
   ```graphql
   query GetGraph($id: ID!) {
     graph(id: $id) {
       id
       name
       nodes {
         id
         label
         datasetId
         dataset {
           id
           name
           fileName
         }
       }
       edges {
         id
         source
         target
         datasetId
         dataset {
           id
           name
         }
       }
     }
   }
   ```

**Success Criteria**:
- GraphQL exposes `datasetId` on nodes and edges
- Optional resolver allows fetching full dataset details
- Frontend can query dataset source information

### Phase 5: Frontend Integration

**Goal**: Display dataset source information in UI

**Tasks**:

1. Update TypeScript types (if not auto-generated):
   ```typescript
   // frontend/src/types/graph.ts
   export interface Node {
     id: string
     label: string
     // ... existing fields
     datasetId?: number
     dataset?: {
       id: number
       name: string
       fileName?: string
     }
   }

   export interface Edge {
     id: string
     source: string
     target: string
     // ... existing fields
     datasetId?: number
     dataset?: {
       id: number
       name: string
     }
   }
   ```

2. Add dataset badge to `GraphSpreadsheetEditor`:
   ```tsx
   // frontend/src/components/editors/GraphSpreadsheetEditor/GraphSpreadsheetEditor.tsx

   {node.dataset && (
     <Badge variant="outline" className="text-xs text-blue-600 border-blue-600">
       {node.dataset.name}
     </Badge>
   )}
   ```

3. Add dataset column to spreadsheet view (optional):
   ```tsx
   const columns = [
     'id',
     'label',
     'layer',
     'dataset', // NEW
     // ... other columns
   ]

   // Render dataset name in cell
   {col === 'dataset' && (
     <span className="text-xs text-muted-foreground">
       {node.dataset?.name || '-'}
     </span>
   )}
   ```

4. Add dataset filter to graph view (optional):
   ```tsx
   <Select
     value={selectedDatasetId}
     onChange={(id) => setSelectedDatasetId(id)}
   >
     <option value="">All datasets</option>
     {project.datasets.map(ds => (
       <option key={ds.id} value={ds.id}>{ds.name}</option>
     ))}
   </Select>
   ```

**Success Criteria**:
- Nodes/edges display their source dataset name
- Users can see which dataset each element came from
- Optional: Can filter graph view by dataset source

## Testing Strategy

### Unit Tests

1. **Dataset Import Test** (`layercake-core/src/services/graph_data_service.rs`):
   ```rust
   #[tokio::test]
   async fn test_dataset_id_set_on_import() {
       let service = GraphDataService::new(test_db());
       let graph_data = /* create graph_data */;
       let dataset_id = 42;

       let nodes = vec![/* test nodes */];
       service.replace_nodes(graph_data.id, &nodes, Some(dataset_id)).await?;

       let loaded_nodes = /* load nodes from DB */;
       assert!(loaded_nodes.iter().all(|n| n.source_dataset_id == Some(dataset_id)));
   }
   ```

2. **Transformation Preservation Test**:
   ```rust
   #[tokio::test]
   async fn test_filter_preserves_dataset_id() {
       let filter_builder = FilterBuilder::new(test_db());

       // Create graph with nodes having dataset IDs
       let input_graph = Graph {
           nodes: vec![
               Node { id: "n1".into(), dataset: Some(1), /* ... */ },
               Node { id: "n2".into(), dataset: Some(2), /* ... */ },
           ],
           // ...
       };

       let filtered = filter_builder.filter_graph(/* params */).await?;

       // Verify dataset IDs preserved
       for node in &filtered.nodes {
           assert!(node.dataset.is_some());
       }
   }
   ```

3. **Merge Preservation Test**:
   ```rust
   #[tokio::test]
   async fn test_merge_preserves_dataset_ids() {
       // Merge two graphs with different dataset IDs
       // Verify each node keeps its original dataset ID
   }
   ```

### Integration Tests

**Full Pipeline Test** (`tests/dataset_source_tracking_e2e.rs`):
```rust
#[tokio::test]
async fn test_dataset_source_through_pipeline() {
    let db = test_database().await;
    let project_id = 1;

    // Import two datasets
    let ds1 = import_dataset(&db, project_id, "Dataset A", /* data */).await;
    let ds2 = import_dataset(&db, project_id, "Dataset B", /* data */).await;

    // Create pipeline: DS1 -> Filter -> Merge
    //                   DS2 ----------->
    let dag = create_test_dag(vec![
        ("ds1-node", "DataSourceNode", ds1.id),
        ("ds2-node", "DataSourceNode", ds2.id),
        ("filter-node", "FilterNode", /* config */),
        ("merge-node", "MergeNode", /* config */),
    ]);

    // Execute pipeline
    let executor = DagExecutor::new(db.clone());
    executor.execute_dag(project_id, 1, &dag.nodes, &dag.edges).await?;

    // Load final merged graph
    let merged_graph = load_graph_for_node("merge-node").await?;

    // Verify nodes from DS1 have dataset = Some(ds1.id)
    let ds1_nodes: Vec<_> = merged_graph.nodes.iter()
        .filter(|n| n.id.starts_with("ds1"))
        .collect();
    assert!(ds1_nodes.iter().all(|n| n.dataset == Some(ds1.id)));

    // Verify nodes from DS2 have dataset = Some(ds2.id)
    let ds2_nodes: Vec<_> = merged_graph.nodes.iter()
        .filter(|n| n.id.starts_with("ds2"))
        .collect();
    assert!(ds2_nodes.iter().all(|n| n.dataset == Some(ds2.id)));
}
```

### Frontend Tests
- Snapshot test: Node/edge with dataset badge
- Integration test: Load graph with dataset info, verify display

## Rollout Plan

This is a non-breaking change that can be rolled out incrementally:

1. **Week 1**: Phase 1-2 (Audit + Dataset Import)
   - Audit current behavior
   - Fix dataset import to set `source_dataset_id`
   - Deploy to dev, verify new imports have dataset IDs

2. **Week 2**: Phase 3 (Transformations)
   - Update filter/transform/merge to preserve dataset IDs
   - Deploy to dev, test full pipeline
   - Verify dataset IDs flow through transformations

3. **Week 3**: Phase 4-5 (API + Frontend)
   - Add GraphQL resolvers
   - Update frontend to display dataset source
   - User acceptance testing
   - Gather feedback

4. **Week 4**: Production Deployment
   - Deploy to production
   - Monitor for issues
   - Document feature for users

## Validation Checklist

After implementation, verify:

- [ ] New dataset imports have `source_dataset_id` set on all nodes/edges
- [ ] Filter operations preserve `source_dataset_id`
- [ ] Transform operations preserve `source_dataset_id` on modified elements
- [ ] Transform operations set `source_dataset_id = NULL` on new elements
- [ ] Merge operations preserve `source_dataset_id` from each source
- [ ] GraphQL exposes `datasetId` field on nodes and edges
- [ ] GraphQL can resolve full dataset details
- [ ] UI displays dataset badges on nodes/edges
- [ ] Full pipeline test passes (dataset import -> filter -> merge)
- [ ] No performance degradation
- [ ] All existing tests still pass

## Implementation Complexity

**Estimated Effort**: 2-3 weeks (simplified from original 5-week plan)

**Reduced Scope**:
- ❌ No database migrations needed
- ❌ No Origin enum or complex type hierarchy
- ❌ No origins metadata map in graphs
- ❌ No transformation tracking
- ❌ No origin_id field or renaming
- ✅ Just preserve existing `source_dataset_id` field
- ✅ Ensure it flows through pipeline
- ✅ Display in UI

**Risk Assessment**: Low
- No schema changes = no migration risk
- No new complex types = less code to maintain
- Preserving existing fields = minimal code changes
- Can be tested incrementally

## Future Enhancements

These are **NOT** in scope for this implementation but could be added later:

1. **Transformation tracking**: Track when transforms create new elements
2. **Manual edit tracking**: Track user edits in graph editor
3. **Dataset versioning**: Link to specific dataset versions
4. **Lineage visualization**: Visual graph showing data flow
5. **Dataset-based filtering**: Filter graph view by source dataset
6. **Dataset-based coloring**: Color nodes by dataset in projections

For now, focus on the simple goal: **preserve dataset provenance through the pipeline**.
