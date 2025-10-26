# Migration Guide: Deprecated Mutations

**Date**: 2025-10-26
**Status**: Active Migration
**Affected Mutation**: `updatePlanDag`

## Overview

The `updatePlanDag` mutation has been deprecated because its bulk replace operation conflicts with delta-based real-time collaboration features. You should migrate to using individual node and edge mutations.

## Deprecated Mutation

```graphql
mutation UpdatePlanDag($projectId: Int!, $planDag: PlanDagInput!) {
  updatePlanDag(projectId: $projectId, planDag: $planDag) {
    version
    nodes { id type position { x y } }
    edges { id source target }
  }
}
```

**Deprecation Notice**:
> "This bulk replace operation conflicts with delta-based real-time updates. Use addPlanDagNode, updatePlanDagNode, deletePlanDagNode, addPlanDagEdge, and deletePlanDagEdge mutations instead for better collaboration support."

## Replacement Mutations

### Adding a Node

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    version: "1.0"
    nodes: [
      { id: "datasource_001", type: DataSourceNode, ...existing nodes... },
      { id: "datasource_002", type: DataSourceNode, ...NEW NODE... }
    ]
    edges: [...all existing edges...]
  })
}
```

**New way (incremental):**
```graphql
mutation AddNode($projectId: Int!, $node: PlanDagNodeInput!) {
  addPlanDagNode(projectId: $projectId, node: $node) {
    id
    type
    position { x y }
    metadata { label description }
    config
  }
}
```

**Example:**
```graphql
mutation {
  addPlanDagNode(
    projectId: 1,
    node: {
      nodeType: DataSourceNode,
      position: { x: 100, y: 200 },
      metadata: { label: "My Data Source", description: "CSV import" },
      config: "{\"dataSourceId\": 42}"
    }
  ) {
    id  # Returns generated ID like "datasource_002"
    type
    position { x y }
  }
}
```

### Updating a Node

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    nodes: [
      { id: "datasource_001", ...UPDATED FIELDS... },
      ...all other nodes unchanged...
    ]
    edges: [...all edges...]
  })
}
```

**New way (incremental):**
```graphql
mutation UpdateNode($projectId: Int!, $nodeId: String!, $updates: PlanDagNodeUpdateInput!) {
  updatePlanDagNode(projectId: $projectId, nodeId: $nodeId, updates: $updates) {
    id
    position { x y }
    metadata { label description }
    config
  }
}
```

**Example:**
```graphql
mutation {
  updatePlanDagNode(
    projectId: 1,
    nodeId: "datasource_001",
    updates: {
      position: { x: 150, y: 250 },
      metadata: { label: "Updated Label", description: "New description" }
    }
  ) {
    id
    position { x y }
  }
}
```

### Deleting a Node

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    nodes: [...all nodes EXCEPT the one to delete...],
    edges: [...update edges that referenced deleted node...]
  })
}
```

**New way (incremental):**
```graphql
mutation DeleteNode($projectId: Int!, $nodeId: String!) {
  deletePlanDagNode(projectId: $projectId, nodeId: $nodeId)
}
```

**Example:**
```graphql
mutation {
  deletePlanDagNode(projectId: 1, nodeId: "datasource_001")  # Returns true
}
```

### Adding an Edge

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    nodes: [...all nodes...],
    edges: [
      ...existing edges...,
      { source: "datasource_001", target: "graph_001", ...NEW EDGE... }
    ]
  })
}
```

**New way (incremental):**
```graphql
mutation AddEdge($projectId: Int!, $edge: PlanDagEdgeInput!) {
  addPlanDagEdge(projectId: $projectId, edge: $edge) {
    id
    source
    target
    metadata { label dataType }
  }
}
```

**Example:**
```graphql
mutation {
  addPlanDagEdge(
    projectId: 1,
    edge: {
      source: "datasource_001",
      target: "graph_001",
      metadata: { label: "nodes", dataType: GraphData }
    }
  ) {
    id  # Returns generated ID like "edge-datasource_001-graph_001-abc123"
    source
    target
  }
}
```

### Deleting an Edge

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    nodes: [...all nodes...],
    edges: [...all edges EXCEPT the one to delete...]
  })
}
```

**New way (incremental):**
```graphql
mutation DeleteEdge($projectId: Int!, $edgeId: String!) {
  deletePlanDagEdge(projectId: $projectId, edgeId: $edgeId)
}
```

**Example:**
```graphql
mutation {
  deletePlanDagEdge(projectId: 1, edgeId: "edge-datasource_001-graph_001-abc123")  # Returns true
}
```

### Moving Multiple Nodes (Batch Operation)

**Old way (bulk replace):**
```graphql
mutation {
  updatePlanDag(projectId: 1, planDag: {
    nodes: [
      { id: "datasource_001", position: { x: 100, y: 200 }, ...rest... },
      { id: "graph_001", position: { x: 300, y: 200 }, ...rest... },
      ...all other nodes...
    ]
    edges: [...all edges...]
  })
}
```

**New way (batch):**
```graphql
mutation BatchMove($projectId: Int!, $positions: [NodePositionInput!]!) {
  batchMovePlanDagNodes(projectId: $projectId, positions: $positions)
}
```

**Example:**
```graphql
mutation {
  batchMovePlanDagNodes(
    projectId: 1,
    positions: [
      { nodeId: "datasource_001", position: { x: 100, y: 200 } },
      { nodeId: "graph_001", position: { x: 300, y: 200 } }
    ]
  )  # Returns true
}
```

## Migration Strategy

### Option 1: Immediate Full Migration (Recommended for New Code)

Replace all `updatePlanDag` calls with individual mutations immediately.

**Pros:**
- Clean break from deprecated pattern
- Better real-time collaboration support
- More granular change tracking

**Cons:**
- Requires more code changes upfront

### Option 2: Gradual Migration (For Legacy Code)

Keep using `updatePlanDag` for complex operations while migrating simple ones first.

**Timeline:**
- **Week 1-2**: Migrate simple operations (add/delete single node/edge)
- **Week 3-4**: Migrate batch operations (moving nodes)
- **Week 5-6**: Migrate complex operations (restructuring DAG)
- **Week 7+**: Remove all `updatePlanDag` usage

### Option 3: Hybrid Approach

Use individual mutations for user-initiated changes, but keep `updatePlanDag` for bulk imports or migrations.

**Note**: This is NOT recommended as it creates inconsistency.

## Code Examples

### React/TypeScript Migration

**Before:**
```typescript
// Old way - bulk replace
const updateDag = async () => {
  const result = await client.mutate({
    mutation: UPDATE_PLAN_DAG,
    variables: {
      projectId: 1,
      planDag: {
        version: "1.0",
        nodes: [...allNodes, newNode],  // Must send ALL nodes
        edges: allEdges,                  // Must send ALL edges
      }
    }
  });
};
```

**After:**
```typescript
// New way - incremental
const addNode = async () => {
  const result = await client.mutate({
    mutation: ADD_PLAN_DAG_NODE,
    variables: {
      projectId: 1,
      node: {
        nodeType: 'DataSourceNode',
        position: { x: 100, y: 200 },
        metadata: { label: 'My Node' },
        config: JSON.stringify({ dataSourceId: 42 })
      }
    }
  });

  const newNodeId = result.data.addPlanDagNode.id;
  console.log('Created node:', newNodeId);
};
```

### Frontend Cache Updates

With individual mutations, you can update Apollo cache incrementally:

```typescript
const [addNode] = useMutation(ADD_PLAN_DAG_NODE, {
  update(cache, { data: { addPlanDagNode } }) {
    // Update cache incrementally instead of refetching entire DAG
    cache.modify({
      fields: {
        getPlanDag(existingDag = {}) {
          return {
            ...existingDag,
            nodes: [...(existingDag.nodes || []), addPlanDagNode]
          };
        }
      }
    });
  }
});
```

## Benefits of Migration

### 1. Real-time Collaboration
Individual mutations work with GraphQL subscriptions for live updates:

```graphql
subscription {
  planDagDeltaChanged(projectId: 1) {
    projectId
    version
    patches {
      op      # "add" | "remove" | "replace"
      path    # "/nodes/3"
      value   # {...node data...}
    }
  }
}
```

### 2. Better Conflict Resolution
With individual mutations, conflicts are easier to resolve:
- User A adds node → independent operation
- User B moves node → independent operation
- Both can succeed without overwriting each other

With bulk replace:
- User A updates DAG (all nodes)
- User B updates DAG (all nodes)
- Last write wins, changes lost!

### 3. Performance
- **Network**: Send only changed data (not entire DAG)
- **Database**: Update only affected rows
- **Frontend**: Incremental cache updates (no full refetch)

### 4. Auditability
Individual mutations create clear audit trail:
- "User added datasource_042 at 10:30:15"
- "User moved graph_003 to (150, 200) at 10:30:22"

Bulk replace:
- "User updated entire DAG at 10:30:15" (what changed?)

## Timeline

| Phase | Duration | Action |
|-------|----------|--------|
| **Now** | Immediate | Deprecation notice added to GraphQL schema |
| **Month 1** | 4 weeks | Migrate critical paths to new mutations |
| **Month 2** | 4 weeks | Migrate remaining code, add warnings |
| **Month 3** | 4 weeks | Remove `updatePlanDag` mutation entirely |

## Support

If you encounter issues during migration:

1. **Documentation**: See GraphQL schema documentation for mutation signatures
2. **Examples**: Check `frontend/src/graphql/mutations/` for usage examples
3. **Issues**: Report migration blockers in GitHub issues
4. **Questions**: Ask in #graphql-api Slack channel

## Testing Your Migration

```graphql
# Test 1: Add a node
mutation TestAdd {
  addPlanDagNode(projectId: 1, node: { nodeType: DataSourceNode, position: { x: 0, y: 0 }, metadata: { label: "Test" }, config: "{}" }) {
    id
  }
}

# Test 2: Update the node
mutation TestUpdate($nodeId: String!) {
  updatePlanDagNode(projectId: 1, nodeId: $nodeId, updates: { position: { x: 100, y: 100 } }) {
    id
    position { x y }
  }
}

# Test 3: Delete the node
mutation TestDelete($nodeId: String!) {
  deletePlanDagNode(projectId: 1, nodeId: $nodeId)
}
```

---

**Last Updated**: 2025-10-26
**Status**: Active - Begin Migration Now
**Deprecation Removal**: Planned for Month 3 (tentative)
