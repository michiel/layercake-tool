# ADR-001: Graph Data Model - Normalized vs JSON Storage

**Date**: 2025-09-20
**Status**: Accepted
**Deciders**: Development Team
**Technical Story**: [Implementation of interactive graph editing with real-time collaboration]

## Context and Problem Statement

During the design of the layercake interactive application, we faced a fundamental architectural decision about how to store graph data (nodes, edges, layers) in the database. The choice directly impacts performance, query capabilities, CRDT integration, and long-term maintainability.

**Key Requirements:**
- Real-time collaborative editing with CRDT synchronization
- Support for complex graph analysis queries (shortest paths, centrality, cycles)
- Fast loading for large graphs (10K+ nodes)
- Integration with existing export system
- Scalable to enterprise use cases

## Decision Drivers

- **Performance**: Fast graph loading and rendering
- **Query Complexity**: Support for graph analysis algorithms
- **Scalability**: Handle graphs from 100 to 100,000+ nodes
- **CRDT Integration**: Real-time collaborative editing
- **Maintainability**: Debugging, schema evolution, data integrity
- **Export Integration**: Compatibility with existing transformation pipeline

## Considered Options

### Option A: Pure JSON Storage
Store entire graph as JSON documents in the `graphs` table:

```sql
CREATE TABLE graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    nodes TEXT NOT NULL,        -- JSON array of nodes
    edges TEXT NOT NULL,        -- JSON array of edges
    layers TEXT NOT NULL,       -- JSON array of layers
    crdt_state TEXT NOT NULL
);
```

### Option B: Normalized Relational Tables
Store graph entities in separate normalized tables:

```sql
CREATE TABLE graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    crdt_state TEXT NOT NULL
);

CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    properties TEXT,
    FOREIGN KEY (graph_id) REFERENCES graphs(id)
);
-- Similar for edges, layers
```

### Option C: Hybrid Approach (Selected)
Normalized tables with JSON cache for performance:

```sql
CREATE TABLE graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    -- JSON cache for fast loading
    nodes_cache TEXT,
    edges_cache TEXT,
    layers_cache TEXT,
    cache_version INTEGER DEFAULT 1,
    crdt_state TEXT NOT NULL
);

CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    crdt_vector TEXT NOT NULL,
    last_modified_by INTEGER,
    FOREIGN KEY (graph_id) REFERENCES graphs(id)
);
-- Similar for edges, layers with CRDT metadata
```

## Decision Outcome

**Chosen option: Option C - Hybrid Approach with Normalized Tables + JSON Cache**

### Rationale

#### Why Not Pure JSON (Option A):

1. **Query Limitations**: Complex graph analysis becomes nearly impossible
   ```sql
   -- Finding nodes with specific labels becomes complex
   SELECT * FROM graphs
   WHERE JSON_SEARCH(nodes, 'one', 'specific_label', NULL, '$[*].label') IS NOT NULL;

   -- vs. simple normalized query:
   SELECT DISTINCT g.* FROM graphs g
   JOIN nodes n ON g.id = n.graph_id
   WHERE n.label = 'specific_label';
   ```

2. **Scalability Issues**:
   - 10K nodes = ~1MB JSON documents
   - 100K nodes = ~10MB JSON documents
   - Entire document must be loaded/saved for single node updates
   - No efficient indexing on node properties

3. **Development Complexity**:
   - No compile-time type safety
   - Difficult debugging of large JSON documents
   - No database-level referential integrity
   - Complex schema migration logic

#### Why Not Pure Normalized (Option B):

1. **Performance Concerns**:
   - Complex JOINs for large graphs
   - Multiple database roundtrips for graph loading
   - Slower initial rendering for large graphs

2. **Cache Misses**: No optimization for read-heavy workloads

#### Why Hybrid Approach (Option C):

**✅ **Performance Benefits**:
- **Fast Path**: JSON cache provides 5-10x faster loading for large graphs
- **Smart Loading**: Automatic cache vs normalized selection based on data freshness
- **Optimized Bulk Operations**: Efficient multi-entity changes

**✅ **Query Capabilities**:
- **Full SQL Support**: Complex graph analysis using normalized tables
- **Efficient Indexing**: B-tree indexes on frequently queried fields
- **Graph Algorithms**: Shortest paths, centrality calculations, cycle detection

**✅ **CRDT Integration**:
- **Dual Updates**: Changes applied to both CRDT and normalized tables
- **Conflict Resolution**: Enhanced detection using normalized data + CRDT vectors
- **Selective Sync**: Efficient change propagation with change log

**✅ **Scalability**:
- **Small Graphs (< 1K nodes)**: Normalized performance sufficient
- **Medium Graphs (1K-10K nodes)**: 3-5x improvement with cache
- **Large Graphs (10K+ nodes)**: 5-10x improvement with cache
- **Enterprise Scale**: Supports very large graphs with graceful degradation

**✅ **Maintainability**:
- **Type Safety**: SeaORM entities provide compile-time guarantees
- **Debugging**: Structured data easy to inspect and debug
- **Schema Evolution**: Standard database migration tools
- **Data Integrity**: Foreign key constraints and validation

**✅ **Export Integration**:
- **Compatibility**: Existing export system works with structured data
- **Performance**: Can use either cache or normalized data as appropriate
- **Consistency**: Single source of truth in normalized tables

### Implementation Strategy

#### Cache Management:
```rust
impl GraphService {
    pub async fn load_graph(&self, graph_id: i32) -> Result<GraphData> {
        // Fast path: Use JSON cache if valid and recent
        if self.is_cache_valid(graph_id).await? {
            return self.load_from_cache(graph_id).await;
        }

        // Reliable path: Load from normalized tables
        self.load_and_rebuild_cache(graph_id).await
    }
}
```

#### CRDT Integration:
```rust
impl CrdtService {
    pub async fn apply_node_change(&self, graph_id: i32, user_id: i32, change: NodeChange) -> Result<()> {
        // 1. Apply to CRDT document
        let crdt_vector = self.apply_to_crdt_document(graph_id, &change).await?;

        // 2. Apply to normalized table
        self.apply_to_normalized_table(graph_id, user_id, &change, &crdt_vector).await?;

        // 3. Invalidate cache
        self.invalidate_graph_cache(graph_id).await?;
    }
}
```

## Consequences

### Positive:
- **Best of Both Worlds**: Fast loading + complex queries
- **Future-Proof**: Scales from small to enterprise-level graphs
- **Robust CRDT**: Enhanced real-time collaboration
- **Developer Experience**: Type-safe, debuggable, maintainable
- **Performance Monitoring**: Built-in cache hit/miss metrics

### Negative:
- **Complexity**: More sophisticated cache management logic
- **Storage Overhead**: ~2x storage usage (normalized + cache)
- **Cache Coherency**: Must maintain consistency between cache and tables
- **Development Time**: Additional implementation complexity

### Risks and Mitigations:

| Risk | Mitigation |
|------|------------|
| Cache inconsistency | Robust invalidation + rebuilding mechanisms |
| Increased complexity | Comprehensive test suite + monitoring |
| Storage overhead | Cache compression + configurable cache TTL |
| Performance regression | Fallback to normalized data + performance monitoring |

## Monitoring and Success Metrics

- **Cache Hit Rate**: Target >80% for frequently accessed graphs
- **Load Performance**: <2s for 10K node graphs, <5s for 50K node graphs
- **Query Performance**: Complex analysis queries <10s for large graphs
- **Sync Latency**: Real-time updates <100ms for CRDT synchronization
- **Storage Efficiency**: <3x overhead compared to pure normalized approach

## Related Decisions

- [Future ADR]: CRDT Library Selection (Loro vs Yrs vs Automerge)
- [Future ADR]: Real-time Synchronization Protocol Design
- [Future ADR]: Frontend State Management for Large Graphs

## Implementation Notes

This decision directly impacts:
- Database schema design in Phase 1 (months 1-6)
- Service layer architecture throughout development
- Frontend performance optimization strategies
- Real-time collaboration implementation
- Export system integration approach

The hybrid approach provides a solid foundation for building a production-ready layercake interactive application that can scale from individual use to enterprise deployments while maintaining excellent performance and developer experience.