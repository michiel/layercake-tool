# GraphQL API Migration Guide: DataSet/Graph → GraphData

**Status**: Draft
**Created**: 2025-12-10
**Target Audience**: Frontend developers

---

## Overview

The GraphQL API is being unified to use a single `GraphData` type instead of separate `DataSet` and `Graph` types. This guide explains how to migrate your frontend code.

**Why?**
- Eliminates duplication between datasets and computed graphs
- Simplifies API with unified queries and mutations
- Better performance with lazy-loading resolvers
- Consistent field names and types

**Timeline**:
- Phase 1 (Now): New GraphData API available, old types still work
- Phase 2 (3-6 months): Gradual migration encouraged
- Phase 3 (TBD): Old types removed (requires frontend migration complete)

---

## Quick Reference

### Type Mapping

| Legacy Type | New Type | Filter |
|------------|----------|--------|
| `DataSet` | `GraphData` | `sourceType: "dataset"` |
| `Graph` | `GraphData` | `sourceType: "computed"` |

### Query Migration

| Legacy Query | New Query | Notes |
|-------------|-----------|-------|
| `dataSet(id: Int!)` | `graphData(id: Int!)` | Returns GraphData |
| `dataSets(projectId: Int!)` | `graphDataList(projectId: Int!, sourceType: "dataset")` | Filter by source type |
| `graph(id: Int!)` | `graphData(id: Int!)` | Returns GraphData |
| `graphs(projectId: Int!)` | `graphDataList(projectId: Int!, sourceType: "computed")` | Filter by source type |
| N/A | `graphDataByDagNode(dagNodeId: String!)` | New: lookup by DAG node |

### Mutation Migration

| Legacy Mutation | New Mutation | Status |
|----------------|--------------|--------|
| `createDataSetFromFile` | N/A | Keep using legacy (AppContext) |
| `updateDataSet` | `updateGraphData` | Prefer new |
| `deleteDataSet` | N/A | Keep using legacy for now |
| `updateGraph` | `updateGraphData` | Prefer new |
| `replayGraphEdits` | `replayGraphDataEdits` | Prefer new |
| `clearGraphEdits` | `clearGraphDataEdits` | Prefer new |

---

## Field Mapping

### Common Fields (Both Types)

All these fields exist in GraphData:

```graphql
type GraphData {
  id: Int!
  projectId: Int!
  name: String!
  status: String!
  errorMessage: String
  nodeCount: Int!
  edgeCount: Int!
  metadata: JSON
  annotations: [GraphDataAnnotation!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}
```

### DataSet-Specific Fields

| DataSet Field | GraphData Field | Notes |
|--------------|----------------|-------|
| `fileFormat` | `fileFormat` | Optional in GraphData |
| `origin` | `origin` | Optional in GraphData |
| `filename` | `filename` | Optional in GraphData |
| `fileSize` | `fileSize` | Optional in GraphData |
| `processedAt` | `processedAt` | Optional in GraphData |
| `graphJson` | ❌ REMOVED | Use `nodes` and `edges` queries instead |
| `layerCount` | ❌ REMOVED | Layers now in project palette |
| `hasLayers` | ❌ REMOVED | Check project palette instead |

### Graph-Specific Fields

| Graph Field | GraphData Field | Notes |
|------------|----------------|-------|
| `nodeId` | `dagNodeId` | Renamed for clarity |
| `executionState` | `status` | Renamed: "running"→"processing", "completed"→"active", "failed"→"error" |
| `computedDate` | `computedDate` | Same |
| `sourceHash` | `sourceHash` | Same |

### New Fields (GraphData Only)

```graphql
type GraphData {
  # Discriminator
  sourceType: String!  # "dataset" or "computed"

  # Helper methods
  isDataset: Boolean!
  isComputed: Boolean!
  isReady: Boolean!
  hasError: Boolean!
  fileSizeFormatted: String  # Human-readable size

  # Edit tracking (for computed graphs)
  lastEditSequence: Int!
  hasPendingEdits: Boolean!
  lastReplayAt: DateTime
}
```

---

## Migration Examples

### Example 1: Fetch Dataset by ID

**Before**:
```graphql
query GetDataSet($id: Int!) {
  dataSet(id: $id) {
    id
    name
    fileFormat
    nodeCount
    edgeCount
    status
    graphJson  # DEPRECATED
  }
}
```

**After**:
```graphql
query GetGraphData($id: Int!) {
  graphData(id: $id) {
    id
    name
    sourceType
    fileFormat
    nodeCount
    edgeCount
    status
    # Use lazy-loading for nodes/edges
    nodes {
      id
      label
      layer
      weight
    }
    edges {
      id
      source
      target
      weight
    }
  }
}
```

### Example 2: List All Datasets

**Before**:
```graphql
query ListDataSets($projectId: Int!) {
  dataSets(projectId: $projectId) {
    id
    name
    filename
    processedAt
  }
}
```

**After**:
```graphql
query ListDatasets($projectId: Int!) {
  graphDataList(projectId: $projectId, sourceType: "dataset") {
    id
    name
    filename
    processedAt
  }
}
```

### Example 3: List All Computed Graphs

**Before**:
```graphql
query ListGraphs($projectId: Int!) {
  graphs(projectId: $projectId) {
    id
    name
    executionState
    nodeCount
    computedDate
  }
}
```

**After**:
```graphql
query ListGraphs($projectId: Int!) {
  graphDataList(projectId: $projectId, sourceType: "computed") {
    id
    name
    status  # Renamed from executionState
    nodeCount
    computedDate
  }
}
```

### Example 4: Update Metadata

**Before**:
```graphql
mutation UpdateDataSet($id: Int!, $input: UpdateDataSetInput!) {
  updateDataSet(id: $id, input: $input) {
    id
    name
  }
}
```

**After**:
```graphql
mutation UpdateGraphData($id: Int!, $input: UpdateGraphDataInput!) {
  updateGraphData(id: $id, input: $input) {
    id
    name
    metadata
  }
}
```

### Example 5: Replay Edits

**Before**:
```graphql
mutation ReplayEdits($graphId: Int!) {
  replayGraphEdits(graphId: $graphId) {
    id
    nodeCount
    edgeCount
  }
}
```

**After**:
```graphql
mutation ReplayEdits($graphDataId: Int!) {
  replayGraphDataEdits(graphDataId: $graphDataId) {
    id
    nodeCount
    edgeCount
    hasPendingEdits
    lastReplayAt
  }
}
```

---

## Migration Strategy

### Recommended Approach

1. **Start with reads** (queries):
   - Replace `dataSet(id)` with `graphData(id)` in new components
   - Keep legacy queries in existing components during transition
   - Both APIs work simultaneously

2. **Update list queries**:
   - Add `sourceType` filter to `graphDataList` queries
   - Test with both legacy and new queries in parallel

3. **Migrate mutations**:
   - Use new `updateGraphData` for metadata changes
   - Use new `replayGraphDataEdits` for edit replay
   - Keep create/delete using legacy mutations for now

4. **Remove graph_json usage**:
   - Replace `graphJson` parsing with `nodes`/`edges` queries
   - Use GraphQL fragments for common node/edge fields

5. **Cleanup**:
   - Once all components migrated, remove legacy query code
   - Update TypeScript types to use GraphData

### Gradual Migration Pattern

```typescript
// 1. Create GraphData types
interface GraphData {
  id: number;
  projectId: number;
  name: string;
  sourceType: 'dataset' | 'computed';
  // ... other fields
}

// 2. Add helper to check if using new API
const USE_NEW_API = process.env.REACT_APP_USE_GRAPH_DATA === 'true';

// 3. Dual query approach
const GET_DATASET_LEGACY = gql`...`;
const GET_GRAPH_DATA = gql`...`;

const query = USE_NEW_API ? GET_GRAPH_DATA : GET_DATASET_LEGACY;

// 4. Normalize response
const normalizeToDataSet = (data: GraphData): DataSet => ({
  id: data.id,
  name: data.name,
  // ... map fields
});
```

---

## Breaking Changes

### Removed Fields

1. **`graphJson` (DataSet)**
   - **Why**: Serialised graph data no longer stored
   - **Migration**: Use lazy-loading `nodes` and `edges` queries
   - **Example**:
     ```graphql
     # Before
     query { dataSet(id: 1) { graphJson } }

     # After
     query {
       graphData(id: 1) {
         nodes { id, label, layer, weight }
         edges { id, source, target, weight }
       }
     }
     ```

2. **`layerCount` / `hasLayers` (DataSet)**
   - **Why**: Layers now managed in project palette
   - **Migration**: Query `project.layers` instead
   - **Example**:
     ```graphql
     # Before
     query { dataSet(id: 1) { layerCount, hasLayers } }

     # After
     query {
       graphData(id: 1) {
         project {
           layers { id, label, backgroundColor }
         }
       }
     }
     ```

### Renamed Fields

1. **`executionState` → `status`**
   - **Mapping**:
     - `"running"` → `"processing"`
     - `"completed"` → `"active"`
     - `"failed"` → `"error"`
     - `"pending"` → `"pending"`

2. **`nodeId` → `dagNodeId`**
   - Same value, clearer name

### Optional vs Required

Some fields that were required in DataSet are now optional in GraphData:
- `fileFormat`, `origin`, `filename`, `fileSize`
- These are `null` for computed graphs, populated for datasets

---

## Testing

### Parallel Testing Approach

```typescript
describe('GraphData migration', () => {
  it('should return same data from both APIs', async () => {
    // Query legacy API
    const legacyResult = await client.query({
      query: GET_DATASET_LEGACY,
      variables: { id: 1 }
    });

    // Query new API
    const newResult = await client.query({
      query: GET_GRAPH_DATA,
      variables: { id: 1 }
    });

    // Normalize and compare
    const normalized = normalizeGraphData(newResult.data.graphData);
    expect(normalized).toMatchObject(legacyResult.data.dataSet);
  });
});
```

### Validation Checklist

- [ ] All dataset queries return correct data
- [ ] All computed graph queries return correct data
- [ ] Nodes/edges lazy-loading works
- [ ] Mutations update correctly
- [ ] Edit replay works
- [ ] Status mapping correct (executionState → status)
- [ ] TypeScript types updated
- [ ] Error handling tested
- [ ] Loading states tested

---

## FAQ

**Q: Do I need to migrate immediately?**
A: No, both APIs work simultaneously. Migrate gradually.

**Q: What about `graphJson` data?**
A: Use `nodes` and `edges` queries instead. More efficient.

**Q: Can I query both datasets and graphs together?**
A: Yes! Use `graphDataList` without `sourceType` filter.

**Q: What about file uploads?**
A: Keep using legacy `createDataSetFromFile` mutation for now.

**Q: When will old types be removed?**
A: Not for at least 3-6 months, with advance notice.

---

## Support

- Backend docs: `plans/refactor-datasets-and-graphs.md`
- API audit: `docs/graph_id_audit.md`
- Questions: Ask in #backend or file issue

---

**Last Updated**: 2025-12-10
