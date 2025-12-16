# Origin Tracking Implementation Plan

## Overview

Currently, when datasets are used as inputs for graphs, the information about source datasets for individual nodes and edges is lost during transformations and merges. This plan implements comprehensive origin tracking to:

- Track which dataset each node/edge originally came from
- Track when transformations create new nodes/edges
- Maintain a registry of all origins (datasets + transformations) in graph metadata
- Provide lineage/provenance information for graph elements throughout the pipeline

**Key Concepts:**
- **Origin**: A source that creates nodes/edges (either a dataset or a transformation)
- **Origin ID**: A unique identifier for an origin (dataset ID or transformation-specific ID)
- **Origins Metadata**: A map stored in `graph_data.metadata` that describes all origins contributing to a graph

## Current State

### Database Schema
- `graph_data` table has `metadata` JSON column (currently unused for origins)
- `graph_data_nodes` has `source_dataset_id INTEGER` column
- `graph_data_edges` has `source_dataset_id INTEGER` column
- Both refer to `data_sets.id` when the node/edge came from a dataset

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
    pub dataset: Option<i32>,  // Currently maps to source_dataset_id
}

pub struct Edge {
    pub id: String,
    // ... other fields
    pub dataset: Option<i32>,  // Currently maps to source_dataset_id
}
```

## Target State

### Database Schema Changes

1. **Rename columns** (breaking change requires migration):
   - `graph_data_nodes.source_dataset_id` → `origin_id` (TEXT, nullable)
   - `graph_data_edges.source_dataset_id` → `origin_id` (TEXT, nullable)
   - Store as TEXT to support both `dataset:<id>` and `transform:<dag_node_id>` formats

2. **Use `graph_data.metadata` JSON column** to store origins map:
   ```json
   {
     "origins": {
       "dataset:42": {
         "type": "dataset",
         "datasetId": 42,
         "name": "Customer Data",
         "fileName": "customers.csv"
       },
       "dataset:43": {
         "type": "dataset",
         "datasetId": 43,
         "name": "Order Data",
         "fileName": "orders.json"
       },
       "transform:filter-node-123": {
         "type": "transformation",
         "nodeId": "filter-node-123",
         "nodeType": "FilterNode",
         "operation": "layer_filter",
         "config": { "layer": "important" }
       },
       "transform:merge-abc": {
         "type": "transformation",
         "nodeId": "merge-abc",
         "nodeType": "MergeNode",
         "operation": "merge"
       }
     }
   }
   ```

### In-Memory Graph Structure Changes

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Graph {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
    pub annotations: Option<String>,
    /// Map of origin_id -> origin metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origins: Option<HashMap<String, Origin>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Origin {
    Dataset {
        #[serde(rename = "datasetId")]
        dataset_id: i32,
        name: String,
        #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
        file_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        origin: Option<String>,
    },
    Transformation {
        #[serde(rename = "nodeId")]
        node_id: String,
        #[serde(rename = "nodeType")]
        node_type: String,
        operation: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        config: Option<serde_json::Value>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    // ... existing fields ...

    /// Origin ID referencing the origins map (replaces dataset field)
    #[serde(rename = "originId", skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /// DEPRECATED: Use origin_id instead. Kept for backwards compatibility during migration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use origin_id instead")]
    pub dataset: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    // ... existing fields ...

    /// Origin ID referencing the origins map (replaces dataset field)
    #[serde(rename = "originId", skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<String>,

    /// DEPRECATED: Use origin_id instead. Kept for backwards compatibility during migration.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use origin_id instead")]
    pub dataset: Option<i32>,
}
```

### GraphQL Schema Changes

```graphql
type Graph {
  id: ID!
  projectId: ID!
  # ... existing fields ...
  origins: [Origin!]
}

union Origin = DatasetOrigin | TransformationOrigin

type DatasetOrigin {
  type: String! # "dataset"
  originId: String! # "dataset:42"
  datasetId: ID!
  name: String!
  fileName: String
  origin: String
}

type TransformationOrigin {
  type: String! # "transformation"
  originId: String! # "transform:filter-node-123"
  nodeId: String!
  nodeType: String!
  operation: String!
  config: JSON
}

type Node {
  id: String!
  # ... existing fields ...
  originId: String
  origin: Origin
}

type Edge {
  id: String!
  # ... existing fields ...
  originId: String
  origin: Origin
}
```

## Implementation Phases

### Phase 1: Database Schema Migration

**Goal**: Migrate database columns without breaking existing functionality

**Tasks**:
1. Create migration `m20251217_000001_add_origin_tracking.rs`
   - Add `origin_id TEXT` column to `graph_data_nodes` (nullable)
   - Add `origin_id TEXT` column to `graph_data_edges` (nullable)
   - Backfill existing `origin_id` from `source_dataset_id`: `UPDATE graph_data_nodes SET origin_id = 'dataset:' || source_dataset_id WHERE source_dataset_id IS NOT NULL`
   - Same for `graph_data_edges`
   - Add indices: `CREATE INDEX idx_nodes_origin ON graph_data_nodes(origin_id)` and `CREATE INDEX idx_edges_origin ON graph_data_edges(origin_id)`

2. Update SeaORM entities
   - `layercake-core/src/database/entities/graph_data_nodes.rs`: Add `origin_id` field
   - `layercake-core/src/database/entities/graph_data_edges.rs`: Add `origin_id` field
   - Keep `source_dataset_id` fields temporarily for backwards compatibility

**Success Criteria**:
- Migration runs successfully on test database
- All tests pass with both `origin_id` and `source_dataset_id` populated
- No data loss

### Phase 2: Core Data Model Updates

**Goal**: Update Rust types to support origin tracking

**Tasks**:
1. Add `Origin` enum to `layercake-core/src/graph.rs`
   ```rust
   #[derive(Serialize, Deserialize, Clone, Debug)]
   #[serde(tag = "type", rename_all = "lowercase")]
   pub enum Origin {
       Dataset {
           #[serde(rename = "datasetId")]
           dataset_id: i32,
           name: String,
           #[serde(rename = "fileName", skip_serializing_if = "Option::is_none")]
           file_name: Option<String>,
           #[serde(skip_serializing_if = "Option::is_none")]
           origin: Option<String>,
       },
       Transformation {
           #[serde(rename = "nodeId")]
           node_id: String,
           #[serde(rename = "nodeType")]
           node_type: String,
           operation: String,
           #[serde(skip_serializing_if = "Option::is_none")]
           config: Option<serde_json::Value>,
       },
   }
   ```

2. Update `Graph` struct in `layercake-core/src/graph.rs`
   - Add `pub origins: Option<HashMap<String, Origin>>`

3. Update `Node` and `Edge` structs
   - Add `pub origin_id: Option<String>`
   - Deprecate but keep `pub dataset: Option<i32>` for backwards compatibility

4. Add helper functions:
   ```rust
   impl Graph {
       pub fn add_dataset_origin(&mut self, dataset_id: i32, name: String, file_name: Option<String>, origin: Option<String>) -> String {
           let origin_id = format!("dataset:{}", dataset_id);
           self.origins.get_or_insert_with(HashMap::new).insert(
               origin_id.clone(),
               Origin::Dataset { dataset_id, name, file_name, origin }
           );
           origin_id
       }

       pub fn add_transformation_origin(&mut self, node_id: String, node_type: String, operation: String, config: Option<serde_json::Value>) -> String {
           let origin_id = format!("transform:{}", node_id);
           self.origins.get_or_insert_with(HashMap::new).insert(
               origin_id.clone(),
               Origin::Transformation { node_id, node_type, operation, config }
           );
           origin_id
       }

       pub fn get_origin(&self, origin_id: &str) -> Option<&Origin> {
           self.origins.as_ref()?.get(origin_id)
       }
   }
   ```

**Success Criteria**:
- All types compile
- Serialization/deserialization tests pass
- Helper methods work correctly

### Phase 3: Dataset Import with Origin Tracking

**Goal**: Set origin_id when importing datasets

**Tasks**:
1. Update `GraphDataService::replace_nodes()` in `layercake-core/src/services/graph_data_service.rs`
   - When creating node records, generate `origin_id` from dataset info
   - Set `origin_id = format!("dataset:{}", dataset_id)` for nodes from datasets

2. Update `GraphDataService::replace_edges()`
   - Same logic for edges

3. Update `GraphDataService::load_full()` to populate `Graph.origins`
   - Query all unique `origin_id` values from nodes and edges
   - For `dataset:*` origins, fetch dataset metadata from `data_sets` table
   - Build `origins` HashMap
   - Populate `node.origin_id` and `edge.origin_id` when building Graph

4. Update dataset import in `DataSetService::create_from_file()`
   - Ensure dataset ID is available when calling `graph_data_service.replace_nodes/edges`
   - Pass dataset metadata so origins can be populated

**Code Example**:
```rust
// In GraphDataService::replace_nodes
let origin_id = Some(format!("dataset:{}", dataset_id));
for node in nodes {
    let active = graph_data_nodes::ActiveModel {
        graph_data_id: Set(graph_data_id),
        external_id: Set(node.external_id.clone()),
        origin_id: Set(origin_id.clone()), // NEW
        // ... other fields
    };
    // ... insert
}

// In GraphDataService::load_full
let (gd, nodes, edges) = /* load from DB */;

// Collect all unique origin_ids
let mut origin_ids: HashSet<String> = nodes.iter()
    .filter_map(|n| n.origin_id.as_ref())
    .chain(edges.iter().filter_map(|e| e.origin_id.as_ref()))
    .map(|s| s.to_string())
    .collect();

// Build origins map
let mut origins = HashMap::new();
for origin_id in origin_ids {
    if let Some(dataset_id_str) = origin_id.strip_prefix("dataset:") {
        if let Ok(dataset_id) = dataset_id_str.parse::<i32>() {
            if let Some(ds) = data_sets::Entity::find_by_id(dataset_id).one(&self.db).await? {
                origins.insert(origin_id.clone(), Origin::Dataset {
                    dataset_id,
                    name: ds.name,
                    file_name: Some(ds.filename),
                    origin: ds.origin,
                });
            }
        }
    }
    // Transformation origins handled in Phase 4
}

let graph = Graph {
    // ... other fields
    origins: if origins.is_empty() { None } else { Some(origins) },
};
```

**Success Criteria**:
- New dataset imports have `origin_id` set on all nodes/edges
- Loading a graph populates the `origins` map with dataset metadata
- Tests verify origin tracking for dataset imports

### Phase 4: Transformation Origin Tracking

**Goal**: Track when transformations create or modify nodes/edges

**Tasks**:
1. Update `FilterBuilder` in `layercake-core/src/pipeline/filter_builder.rs`
   - When a filter removes nodes/edges, keep origin_id unchanged on remaining elements
   - Add transformation origin to graph metadata if any filtering occurred
   ```rust
   let transform_origin_id = graph.add_transformation_origin(
       node_id.to_string(),
       "FilterNode".to_string(),
       "layer_filter".to_string(),
       Some(json!({"layer": layer_name}))
   );
   ```

2. Update `TransformBuilder` in `layercake-core/src/pipeline/transform_builder.rs`
   - For transformations that modify nodes/edges (weights, labels, attributes):
     - Preserve origin_id from source nodes/edges
     - Add transformation origin to metadata
   - For transformations that CREATE new nodes/edges:
     - Set `origin_id = format!("transform:{}", node_id)` on new elements
     - Add transformation origin to metadata

3. Update `MergeBuilder` in `layercake-core/src/pipeline/merge_builder.rs`
   - Merge `origins` maps from all upstream graphs
   - Preserve `origin_id` on nodes/edges from upstream
   - Add merge transformation to origins if creating combined elements:
     ```rust
     let merge_origin_id = merged_graph.add_transformation_origin(
         node_id.to_string(),
         "MergeNode".to_string(),
         "merge".to_string(),
         None
     );
     ```

4. Update `GraphDataBuilder` in `layercake-core/src/pipeline/graph_data_builder.rs`
   - When persisting computed graphs, serialize `origins` map to `graph_data.metadata`
   - When loading, deserialize `origins` from metadata

**Code Example**:
```rust
// In FilterBuilder::filter_graph
pub async fn filter_graph(&self, /* ... */) -> Result<Graph> {
    let mut graph = /* load upstream graph */;

    // Add transformation origin for this filter operation
    let transform_origin_id = graph.add_transformation_origin(
        node_id.to_string(),
        "FilterNode".to_string(),
        "layer_filter".to_string(),
        Some(json!({"layer": layer_name}))
    );

    // Filter nodes/edges but preserve their origin_id
    graph.nodes.retain(|n| n.layer == layer_name);
    graph.edges.retain(|e| e.layer == layer_name);

    Ok(graph)
}

// In TransformBuilder for creating new edges
pub async fn create_edges(&self, /* ... */) -> Result<Graph> {
    let mut graph = /* load upstream */;

    let transform_origin_id = graph.add_transformation_origin(
        node_id.to_string(),
        "TransformNode".to_string(),
        "create_edges".to_string(),
        Some(json!({"algorithm": "shortest_path"}))
    );

    for new_edge in computed_edges {
        graph.edges.push(Edge {
            id: new_edge.id,
            source: new_edge.source,
            target: new_edge.target,
            origin_id: Some(transform_origin_id.clone()), // NEW
            // ... other fields
        });
    }

    Ok(graph)
}
```

**Success Criteria**:
- Filter operations preserve origin_id and add transformation metadata
- Transform operations that create new elements assign transformation origin_id
- Merge operations combine origins from all inputs
- Transformations are visible in graph metadata

### Phase 5: GraphQL API Updates

**Goal**: Expose origin tracking through GraphQL

**Tasks**:
1. Add `Origin` types to `layercake-core/src/graphql/types/graph.rs`:
   ```rust
   #[derive(Union)]
   pub enum OriginType {
       Dataset(DatasetOrigin),
       Transformation(TransformationOrigin),
   }

   #[derive(SimpleObject)]
   pub struct DatasetOrigin {
       #[graphql(name = "type")]
       pub origin_type: String,
       #[graphql(name = "originId")]
       pub origin_id: String,
       #[graphql(name = "datasetId")]
       pub dataset_id: i32,
       pub name: String,
       #[graphql(name = "fileName")]
       pub file_name: Option<String>,
       pub origin: Option<String>,
   }

   #[derive(SimpleObject)]
   pub struct TransformationOrigin {
       #[graphql(name = "type")]
       pub origin_type: String,
       #[graphql(name = "originId")]
       pub origin_id: String,
       #[graphql(name = "nodeId")]
       pub node_id: String,
       #[graphql(name = "nodeType")]
       pub node_type: String,
       pub operation: String,
       pub config: Option<serde_json::Value>,
   }
   ```

2. Update `Graph` GraphQL type to include `origins` field
3. Update `Node` and `Edge` types to include `originId` and resolved `origin` field
4. Add resolver to lookup origin from origins map:
   ```rust
   impl Node {
       async fn origin(&self, ctx: &Context<'_>) -> Option<OriginType> {
           let origin_id = self.origin_id.as_ref()?;
           let graph = ctx.data::<Graph>().ok()?;
           let origin = graph.get_origin(origin_id)?;
           // Convert Origin enum to OriginType union
       }
   }
   ```

**Success Criteria**:
- GraphQL queries can fetch origins array
- Node and edge queries can resolve their origin
- Schema validates and generates correct TypeScript types

### Phase 6: Frontend Integration

**Goal**: Display origin information in UI

**Tasks**:
1. Update TypeScript types in `frontend/src/types/graph.ts`:
   ```typescript
   export type Origin = DatasetOrigin | TransformationOrigin

   export interface DatasetOrigin {
     type: 'dataset'
     originId: string
     datasetId: number
     name: string
     fileName?: string
     origin?: string
   }

   export interface TransformationOrigin {
     type: 'transformation'
     originId: string
     nodeId: string
     nodeType: string
     operation: string
     config?: any
   }

   export interface Node {
     id: string
     // ... existing fields
     originId?: string
     origin?: Origin
   }

   export interface Edge {
     id: string
     // ... existing fields
     originId?: string
     origin?: Origin
   }

   export interface Graph {
     name: string
     nodes: Node[]
     edges: Edge[]
     layers: Layer[]
     annotations?: string
     origins?: Record<string, Origin>
   }
   ```

2. Update GraphQL fragments to fetch origin data
   ```graphql
   fragment NodeFields on Node {
     id
     label
     layer
     # ... existing fields
     originId
     origin {
       ... on DatasetOrigin {
         type
         originId
         datasetId
         name
         fileName
       }
       ... on TransformationOrigin {
         type
         originId
         nodeId
         nodeType
         operation
       }
     }
   }
   ```

3. Add origin badge/chip to node/edge displays in `GraphSpreadsheetEditor`:
   ```tsx
   {node.origin && (
     <Badge variant="outline" className="text-xs">
       {node.origin.type === 'dataset'
         ? `Dataset: ${node.origin.name}`
         : `Transform: ${node.origin.operation}`
       }
     </Badge>
   )}
   ```

4. Add Origins panel to graph detail view showing all contributing sources
   ```tsx
   <OriginsList origins={graph.origins} />
   ```

**Success Criteria**:
- UI shows origin badges on nodes/edges
- Origins panel displays all datasets and transformations
- Clicking an origin filters/highlights related elements

### Phase 7: Migration & Cleanup

**Goal**: Complete transition to origin_id, deprecate old fields

**Tasks**:
1. Create migration `m20251217_000002_drop_source_dataset_id.rs`
   - Remove `source_dataset_id` column from `graph_data_nodes`
   - Remove `source_dataset_id` column from `graph_data_edges`
   - This is a breaking migration - only run after verifying Phase 1-6 work

2. Remove deprecated `dataset` field from `Node` and `Edge` structs

3. Update all code using `node.dataset` to use `node.origin_id`

4. Update documentation and examples

**Success Criteria**:
- No references to `source_dataset_id` in codebase
- All tests pass
- Documentation updated

## Testing Strategy

### Unit Tests
- `graph.rs`: Test Origin enum serialization/deserialization
- Helper methods: `add_dataset_origin()`, `add_transformation_origin()`
- Migration: backfill logic correctness

### Integration Tests
1. **Dataset Import Test** (`tests/origin_tracking_dataset.rs`):
   ```rust
   #[tokio::test]
   async fn test_dataset_origin_tracking() {
       // Import dataset
       // Verify origin_id = "dataset:<id>" on all nodes/edges
       // Verify origins map contains dataset metadata
   }
   ```

2. **Transform Origin Test** (`tests/origin_tracking_transform.rs`):
   ```rust
   #[tokio::test]
   async fn test_transformation_origin_tracking() {
       // Import dataset -> filter -> transform
       // Verify filtered nodes keep dataset origin_id
       // Verify new nodes have transform origin_id
       // Verify origins map contains both dataset and transform
   }
   ```

3. **Merge Origin Test** (`tests/origin_tracking_merge.rs`):
   ```rust
   #[tokio::test]
   async fn test_merge_origin_tracking() {
       // Merge two datasets
       // Verify origins map has both datasets
       // Verify nodes preserve their source origin_id
   }
   ```

4. **Full Pipeline Test** (`tests/origin_tracking_e2e.rs`):
   ```rust
   #[tokio::test]
   async fn test_full_pipeline_origin_tracking() {
       // Dataset A -> Transform -> Merge
       // Dataset B -> Filter     /
       //                        |
       //                        v
       //                  Final Graph
       //
       // Verify final graph has:
       // - origins map with Dataset A, Dataset B, Transform, Filter, Merge
       // - Correct origin_id on all nodes/edges
   }
   ```

### Frontend Tests
- Snapshot tests for origin badges
- Integration test: load graph with origins, verify display

## Rollout Plan

1. **Week 1**: Phase 1 (Database Migration)
   - Deploy migration to dev environment
   - Verify backfill correctness
   - Monitor for issues

2. **Week 2**: Phase 2-3 (Core Models + Dataset Import)
   - Deploy to dev
   - Test dataset imports
   - Verify origin tracking on new datasets

3. **Week 3**: Phase 4 (Transformations)
   - Deploy to dev
   - Test transform/filter/merge operations
   - Verify origin propagation

4. **Week 4**: Phase 5-6 (API + Frontend)
   - Deploy to dev
   - User acceptance testing
   - Gather feedback

5. **Week 5**: Phase 7 (Cleanup)
   - Deploy final breaking migration
   - Remove deprecated fields
   - Production deployment

## Open Questions & Decisions

1. **Q**: Should we track intermediate transformation steps?
   **A**: Yes, but only transformations that create/modify nodes/edges. Pure filtering preserves origins.

2. **Q**: How to handle manual edits in graph editor?
   **A**: Manual edits get a special origin type `manual:<edit_id>` with user/timestamp info.

3. **Q**: What if a node is created by merging two nodes with different origins?
   **A**: Assign merge transformation as origin; original origins still in metadata for reference.

4. **Q**: Should origin_id be indexed for performance?
   **A**: Yes, add index in Phase 1 for efficient filtering by origin.

## Performance Considerations

- **Origins map size**: For large graphs with many datasets, origins map could grow. Mitigated by:
  - Only including origins actually used by nodes/edges in the graph
  - Lazy-loading origin details on frontend

- **Database queries**: Loading origins requires joining with `data_sets` table
  - Cache dataset metadata during graph load
  - Consider denormalizing dataset name into origin_id if needed

- **Serialization overhead**: Origins map adds to JSON size
  - Skip serialization when `origins` is empty
  - Consider compression for large graphs

## Future Enhancements

1. **Origin-based filtering**: Filter graph to show only nodes from specific origin
2. **Lineage visualization**: Visual graph of data flow from datasets through transformations
3. **Origin-based colors**: Color nodes/edges by their origin in projections
4. **Provenance export**: Export full lineage report for compliance/auditing
5. **Manual edit tracking**: Track user edits as transformation origins with user/timestamp
6. **Version tracking**: Link origins to specific dataset versions

## Success Metrics

- 100% of nodes/edges in new graphs have origin_id set
- Origins map correctly reflects all contributing datasets and transformations
- Zero data loss during migration
- UI displays origin information without performance degradation
- Test coverage >90% for origin tracking code
