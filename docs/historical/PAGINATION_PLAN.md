# GraphQL Pagination Implementation Plan

**Date**: 2025-10-26
**Status**: Planned (Not Yet Implemented)
**Priority**: High (Phase 1.3 - To Be Implemented)

## Overview

Add pagination support to `graph_preview` query to prevent out-of-memory issues when loading large graphs.

## Current State

### Problematic Query (lines 724-784 in queries/mod.rs)

```rust
async fn graph_preview(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
    node_id: String,
) -> Result<Option<GraphPreview>> {
    // ...
    // Get ALL nodes for this graph (⚠️ No limit!)
    let nodes = graph_nodes::Entity::find()
        .filter(graph_nodes::Column::GraphId.eq(graph.id))
        .all(&context.db)  // ← Loads entire table
        .await?;

    // Get ALL edges for this graph (⚠️ No limit!)
    let edges = graph_edges::Entity::find()
        .filter(graph_edges::Column::GraphId.eq(graph.id))
        .all(&context.db)  // ← Loads entire table
        .await?;

    // Returns everything at once
    Ok(Some(GraphPreview {
        nodes: node_previews,  // Could be 100K+ nodes
        edges: edge_previews,  // Could be 1M+ edges
        // ...
    }))
}
```

###Risk

- **Memory**: Loading 100K nodes × 1KB each = 100MB per request
- **Network**: Transferring massive JSON responses
- **Frontend**: Browser crashes rendering large datasets
- **Database**: Expensive full table scans

## Target State

### 1. Add Pagination Input Type

```rust
// Add to layercake-core/src/graphql/types/preview.rs or types/mod.rs

#[derive(InputObject, Clone, Debug)]
pub struct PaginationInput {
    #[graphql(default = 100)]
    #[graphql(validator(maximum = "1000"))]  // Prevent abuse
    pub limit: u64,

    #[graphql(default = 0)]
    pub offset: u64,
}

impl Default for PaginationInput {
    fn default() -> Self {
        Self {
            limit: 100,
            offset: 0,
        }
    }
}
```

### 2. Add Paged Result Type

```rust
// Add to layercake-core/src/graphql/types/preview.rs

#[derive(SimpleObject, Clone, Debug)]
pub struct PagedResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub has_more: bool,
    pub offset: u64,
    pub limit: u64,
}

impl<T> PagedResult<T> {
    pub fn new(items: Vec<T>, total_count: i64, offset: u64, limit: u64) -> Self {
        let has_more = (offset + limit) < (total_count as u64);
        Self {
            items,
            total_count,
            has_more,
            offset,
            limit,
        }
    }
}
```

**Note**: async-graphql doesn't support generic SimpleObject out of the box. We'll need concrete types:

```rust
#[derive(SimpleObject, Clone, Debug)]
pub struct PagedGraphNodes {
    pub items: Vec<GraphNodePreview>,
    pub total_count: i64,
    pub has_more: bool,
    pub offset: u64,
    pub limit: u64,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct PagedGraphEdges {
    pub items: Vec<GraphEdgePreview>,
    pub total_count: i64,
    pub has_more: bool,
    pub offset: u64,
    pub limit: u64,
}
```

### 3. Update GraphPreview Type

```rust
// layercake-core/src/graphql/types/preview.rs

#[derive(SimpleObject, Clone, Debug)]
pub struct GraphPreview {
    pub node_id: String,
    pub graph_id: i32,
    pub name: String,
    pub nodes: PagedGraphNodes,      // ← Changed from Vec
    pub edges: PagedGraphEdges,      // ← Changed from Vec
    pub layers: Vec<Layer>,          // Layers usually small, no pagination needed
    pub node_count: i32,             // Total count (for UI)
    pub edge_count: i32,             // Total count (for UI)
    pub execution_state: String,
    pub computed_date: Option<String>,
    pub error_message: Option<String>,
}
```

### 4. Update Query Signature

```rust
// layercake-core/src/graphql/queries/mod.rs

async fn graph_preview(
    &self,
    ctx: &Context<'_>,
    project_id: i32,
    node_id: String,
    #[graphql(default)] nodes_pagination: PaginationInput,
    #[graphql(default)] edges_pagination: PaginationInput,
) -> Result<Option<GraphPreview>> {
    let context = ctx.data::<GraphQLContext>()?;

    // Find graph
    let graph = graphs::Entity::find()
        .filter(graphs::Column::ProjectId.eq(project_id))
        .filter(graphs::Column::NodeId.eq(&node_id))
        .one(&context.db)
        .await?;

    let graph = match graph {
        Some(g) => g,
        None => return Ok(None),
    };

    // Get TOTAL counts (for pagination metadata)
    let total_node_count = graph_nodes::Entity::find()
        .filter(graph_nodes::Column::GraphId.eq(graph.id))
        .count(&context.db)
        .await? as i64;

    let total_edge_count = graph_edges::Entity::find()
        .filter(graph_edges::Column::GraphId.eq(graph.id))
        .count(&context.db)
        .await? as i64;

    // Get PAGINATED nodes
    let nodes = graph_nodes::Entity::find()
        .filter(graph_nodes::Column::GraphId.eq(graph.id))
        .limit(nodes_pagination.limit)
        .offset(nodes_pagination.offset)
        .all(&context.db)
        .await?;

    // Get PAGINATED edges
    let edges = graph_edges::Entity::find()
        .filter(graph_edges::Column::GraphId.eq(graph.id))
        .limit(edges_pagination.limit)
        .offset(edges_pagination.offset)
        .all(&context.db)
        .await?;

    // Get layers (no pagination - usually small)
    let db_layers = graph_layers::Entity::find()
        .filter(graph_layers::Column::GraphId.eq(graph.id))
        .all(&context.db)
        .await?;

    // Convert to preview format
    let node_previews: Vec<GraphNodePreview> =
        nodes.into_iter().map(GraphNodePreview::from).collect();

    let edge_previews: Vec<GraphEdgePreview> =
        edges.into_iter().map(GraphEdgePreview::from).collect();

    let layer_previews: Vec<Layer> =
        db_layers.into_iter().map(Layer::from).collect();

    // Build paged results
    let paged_nodes = PagedGraphNodes {
        items: node_previews,
        total_count: total_node_count,
        has_more: (nodes_pagination.offset + nodes_pagination.limit) < (total_node_count as u64),
        offset: nodes_pagination.offset,
        limit: nodes_pagination.limit,
    };

    let paged_edges = PagedGraphEdges {
        items: edge_previews,
        total_count: total_edge_count,
        has_more: (edges_pagination.offset + edges_pagination.limit) < (total_edge_count as u64),
        offset: edges_pagination.offset,
        limit: edges_pagination.limit,
    };

    Ok(Some(GraphPreview {
        node_id,
        graph_id: graph.id,
        name: graph.name,
        nodes: paged_nodes,
        edges: paged_edges,
        layers: layer_previews,
        node_count: graph.node_count,
        edge_count: graph.edge_count,
        execution_state: graph.execution_state,
        computed_date: graph.computed_date.map(|d| d.to_rfc3339()),
        error_message: graph.error_message,
    }))
}
```

## Implementation Steps

### Step 1: Backend Types (Week 1, Day 1-2)
1. Create `PaginationInput` in `types/mod.rs` or new `types/pagination.rs`
2. Create `PagedGraphNodes` and `PagedGraphEdges` in `types/preview.rs`
3. Update `GraphPreview` struct
4. Run `cargo check` to verify types compile

### Step 2: Update Query (Week 1, Day 3)
1. Update `graph_preview` signature in `queries/mod.rs`
2. Add pagination logic (limit/offset)
3. Add count queries for totals
4. Build paged result objects
5. Test with `cargo test`

### Step 3: Frontend Updates (Week 1, Day 4-5)
1. Regenerate GraphQL types from schema
2. Update `GraphPreview` component to handle paged data
3. Add "Load More" button or infinite scroll
4. Update state management for pagination
5. Add loading states

### Step 4: Testing (Week 2)
1. Unit tests for pagination logic
2. Integration tests with various page sizes
3. Performance tests with large graphs
4. Frontend E2E tests
5. Load testing

## Files to Modify

### Backend
- `layercake-core/src/graphql/types/preview.rs` (or create `types/pagination.rs`)
- `layercake-core/src/graphql/queries/mod.rs`
- `layercake-core/src/graphql/types/mod.rs` (exports)

### Frontend
- `frontend/src/__generated__/graphql.ts` (regenerated)
- `frontend/src/components/PlanVisualEditor/PreviewPanel.tsx` (or similar)
- `frontend/src/hooks/useGraphPreview.ts` (or similar)

## Breaking Changes

**GraphQL Schema Change:**
```graphql
# Before
type GraphPreview {
  nodes: [GraphNodePreview!]!
  edges: [GraphEdgePreview!]!
}

# After
type GraphPreview {
  nodes: PagedGraphNodes!
  edges: PagedGraphEdges!
}

type PagedGraphNodes {
  items: [GraphNodePreview!]!
  totalCount: Int!
  hasMore: Boolean!
  offset: Int!
  limit: Int!
}
```

**Migration Strategy:**
1. Add new paginated query as `graphPreviewPaginated` first
2. Migrate frontend incrementally
3. Deprecate old `graphPreview` query
4. Remove after migration period (e.g., 2 sprints)

## Performance Impact

### Before
- Query time: O(n) where n = total nodes
- Memory: O(n) - loads all data
- Network: O(n) - sends all data

### After
- Query time: O(limit) - only fetches one page
- Memory: O(limit) - bounded by page size
- Network: O(limit) - sends only one page

### Example
- Graph with 100,000 nodes
- Page size: 100
- **Before**: 100,000 rows loaded, ~100MB memory, ~50MB network
- **After**: 100 rows loaded, ~100KB memory, ~50KB network
- **Improvement**: 1000x reduction in memory and network usage

## Success Criteria

- [ ] `graph_preview` query accepts pagination parameters
- [ ] Query returns paged results with metadata
- [ ] Frontend displays paginated data correctly
- [ ] "Load More" or pagination UI works
- [ ] Performance tests show < 200ms p95 for paginated queries
- [ ] Memory usage stays < 10MB per request
- [ ] No regressions in existing functionality

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Breaking frontend | HIGH | Use separate `graphPreviewPaginated` query initially |
| Off-by-one errors | MEDIUM | Comprehensive tests for edge cases |
| Inconsistent state | MEDIUM | Proper cache invalidation in frontend |
| Poor UX | LOW | Good loading states, clear pagination controls |

## Future Enhancements

1. **Cursor-based pagination**: More efficient than offset-based
2. **Filtering**: Combine with search/filter capabilities
3. **Sorting**: Allow sorting by different fields
4. **Field selection**: Only fetch requested fields (GraphQL feature)
5. **Caching**: Cache pages on backend/frontend

## References

- **SeaORM pagination**: https://www.sea-ql.org/SeaORM/docs/basic-crud/select/#pagination
- **async-graphql examples**: https://github.com/async-graphql/examples
- **Related issue**: See GRAPHQL_SCHEMA_REVIEW.md Section 9

---

**Status**: Implementation plan complete. Ready for development in Phase 1.3.
