# Layercake Interactive Application Implementation Plan - COMPREHENSIVE REVISION

## Executive Summary

**PREVIOUS PLAN ASSESSMENT**: The original implementation plan significantly underestimated complexity and misaligned with specification requirements. This comprehensive revision provides a realistic, technically-sound approach to building the layercake interactive application.

**KEY CHANGES FROM ORIGINAL PLAN**:
- ✅ Acknowledges 30-36 month realistic timeline (vs optimistic 18-24 months)
- ✅ Addresses core CRDT synchronization requirements (vs abandoning them)
- ✅ Properly separates Plan DAG metadata from Graph data (vs conflating them)
- ✅ Includes comprehensive frontend architecture (vs basic placeholders)
- ✅ Realistic team size: 4-6 developers (vs 2-3)
- ✅ Addresses all critical gaps identified in feasibility review

## Critical Assessment of Original Plan

### 🔴 **Major Issues Identified in Previous Implementation**

1. **Data Model Fundamentally Wrong**
   - Previous: Plans stored as "YAML content"
   - Required: Plans as DAG metadata with graph copy relationships
   - Impact: Complete database redesign needed

2. **CRDT Requirements Abandoned Without Justification**
   - Previous: "Optimistic updates with conflict detection"
   - Required: "Changes tracked and shared as CRDT objects"
   - Impact: Core synchronization architecture missing

3. **Frontend Complexity Severely Underestimated**
   - Previous: "Basic components" in Phase 1
   - Required: Complex graph editors with hierarchy visualization
   - Impact: 12-18 months additional development needed

4. **Testing Strategy Insufficient**
   - Previous: Basic unit/integration tests
   - Required: "All functionality must have test coverage"
   - Impact: Comprehensive testing framework needed

### 🟡 **Technical Debt in Current Codebase**

- **Existing API/Database**: Can be discarded as per directive
- **Export System**: Solid foundation, can be preserved and enhanced
- **CLI Functionality**: Maintain compatibility during transition
- **Cargo Workspace**: Good foundation for modular development

## Revised Architecture Overview

### **Target Architecture (Realistic)**

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Layercake Interactive Platform                  │
├─────────────────────────────────────────────────────────────────────┤
│                         Frontend Layer                             │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                    React Application                            │ │
│  │                  (Mantine UI + ReactFlow)                      │ │
│  │                                                                 │ │
│  │ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐ │ │
│  │ │ PlanVisualEditor│ │ GraphSpreadsheet│ │  GraphVisualEditor  │ │ │
│  │ │  (ReactFlow)    │ │    Editor       │ │   (ReactFlow)       │ │ │
│  │ └─────────────────┘ └─────────────────┘ └─────────────────────┘ │ │
│  │                                                                 │ │
│  │ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐ │ │
│  │ │ PlanGraphEditor │ │  Hierarchy      │ │  Real-time CRDT     │ │ │
│  │ │  (ReactFlow)    │ │   Viewer        │ │   Collaboration     │ │ │
│  │ └─────────────────┘ └─────────────────┘ └─────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│                         API Layer                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────┐ │
│  │   GraphQL API   │  │   MCP API       │  │   WebSocket API     │ │
│  │  (Primary)      │  │  (AI Tools)     │  │  (Real-time)        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│                      Business Logic Layer                          │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                   Unified Service Layer                         │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐ │ │
│  │  │   Plan      │ │   Graph     │ │      CRDT Sync              │ │ │
│  │  │  Service    │ │  Service    │ │      Service                │ │ │
│  │  └─────────────┘ └─────────────┘ └─────────────────────────────┘ │ │
│  │                                                                 │ │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐ │ │
│  │  │   Import    │ │   Export    │ │   Transformation           │ │ │
│  │  │   Service   │ │   Service   │ │      Service               │ │ │
│  │  └─────────────┘ └─────────────┘ └─────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────────┤
│                       Data Layer                                   │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                     Database (SeaORM)                           │ │
│  │       SQLite (default) or PostgreSQL (production)              │ │
│  │                                                                 │ │
│  │   User → Group → Project → Plan (DAG) → Graph → Nodes/Edges    │ │
│  │                     ↓                                           │ │
│  │              CRDT Change Log                                   │ │
│  │         (Distributed Synchronization)                          │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

## Comprehensive Implementation Plan

### **Phase 1: Foundation & Data Model (6 months)**

#### **Goal**: Complete data model redesign and core backend services

#### **1.1 CRDT Research & Selection (Weeks 1-2)**

**Research Task**: Evaluate CRDT libraries for Rust ecosystem

**Candidates to Evaluate**:
```toml
# Primary candidates
yrs = "0.18"           # Yjs port to Rust - mature, battle-tested
automerge = "0.5"      # Pure Rust implementation
diamond-types = "0.2"  # High-performance text CRDT
loro = "0.16"          # Modern CRDT with rich data types
```

**Evaluation Criteria**:
- **Performance**: Benchmarks with 10K+ node graphs
- **Data Types**: Support for JSON objects (graph data)
- **Network Efficiency**: Compressed synchronization protocols
- **Maturity**: Production use, active maintenance
- **Integration**: Rust ecosystem compatibility

**Recommendation Process**:
1. Prototype each library with sample graph data
2. Benchmark sync performance with simulated multi-user scenarios
3. Test integration with SeaORM for persistence
4. Document findings with performance metrics

#### **1.2 New Database Schema Design (Weeks 3-4)**

**Complete Schema Redesign** based on clarified specification:

```sql
-- Core entity relationships
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE groups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE user_groups (
    user_id INTEGER NOT NULL,
    group_id INTEGER NOT NULL,
    role TEXT NOT NULL DEFAULT 'member', -- member, admin
    PRIMARY KEY (user_id, group_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES groups(id)
);

-- UPDATED: Projects now contain Plan DAG directly (1:1 relationship)
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    plan_dag TEXT NOT NULL,      -- JSON: Plan DAG structure (nodes & edges)
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- REMOVED: plans table (Plan DAG now embedded in projects)
-- REMOVED: plan_graph_links table (no longer needed)

-- REFINED: Hybrid approach - Normalized tables with JSON cache for performance
CREATE TABLE graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    is_copy BOOLEAN NOT NULL DEFAULT FALSE,
    original_graph_id INTEGER, -- for copies
    copy_metadata TEXT,        -- JSON: copy-specific data

    -- HYBRID: JSON cache for fast loading (denormalized)
    nodes_cache TEXT,           -- JSON cache of all nodes
    edges_cache TEXT,           -- JSON cache of all edges
    layers_cache TEXT,          -- JSON cache of all layers
    cache_version INTEGER DEFAULT 1, -- Cache invalidation tracking

    -- CRDT state management
    crdt_state TEXT NOT NULL,   -- CRDT document state
    last_crdt_sync DATETIME,    -- Last successful CRDT sync

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (original_graph_id) REFERENCES graphs(id)
);

-- ENHANCED: Normalized tables for queries and integrity with CRDT support
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    layer_id TEXT,
    position_x REAL,
    position_y REAL,
    properties TEXT, -- JSON

    -- ENHANCED: CRDT integration for normalized data
    crdt_vector TEXT NOT NULL,      -- CRDT version vector for this node
    last_modified_by INTEGER,       -- User who last modified this node
    last_modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (graph_id) REFERENCES graphs(id),
    FOREIGN KEY (last_modified_by) REFERENCES users(id),
    UNIQUE(graph_id, node_id)
);

CREATE TABLE edges (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    source_node_id TEXT NOT NULL,
    target_node_id TEXT NOT NULL,
    label TEXT,
    properties TEXT, -- JSON

    -- ENHANCED: CRDT integration for normalized data
    crdt_vector TEXT NOT NULL,      -- CRDT version vector for this edge
    last_modified_by INTEGER,       -- User who last modified this edge
    last_modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (graph_id) REFERENCES graphs(id),
    FOREIGN KEY (last_modified_by) REFERENCES users(id)
);

CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    layer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    properties TEXT, -- JSON

    -- ENHANCED: CRDT integration for normalized data
    crdt_vector TEXT NOT NULL,      -- CRDT version vector for this layer
    last_modified_by INTEGER,       -- User who last modified this layer
    last_modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (graph_id) REFERENCES graphs(id),
    FOREIGN KEY (last_modified_by) REFERENCES users(id),
    UNIQUE(graph_id, layer_id)
);

-- REFINED: Enhanced CRDT change log for efficient synchronization
CREATE TABLE graph_changes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    entity_type TEXT NOT NULL,    -- 'node', 'edge', 'layer'
    entity_id TEXT NOT NULL,      -- node_id, edge_id, layer_id
    change_type TEXT NOT NULL,    -- 'create', 'update', 'delete'
    change_data TEXT NOT NULL,    -- JSON change payload
    crdt_vector TEXT NOT NULL,    -- CRDT version vector
    user_id INTEGER NOT NULL,
    session_id TEXT,              -- For grouping related changes
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (graph_id) REFERENCES graphs(id),
    FOREIGN KEY (user_id) REFERENCES users(id),

    -- Indexes for efficient sync queries
    INDEX idx_graph_changes_graph_timestamp (graph_id, timestamp),
    INDEX idx_graph_changes_session (session_id),
    INDEX idx_graph_changes_vector (crdt_vector)
);

-- Performance indexes for hybrid approach
CREATE INDEX idx_graphs_cache_version ON graphs(cache_version);
CREATE INDEX idx_nodes_graph_modified ON nodes(graph_id, last_modified_at);
CREATE INDEX idx_nodes_label ON nodes(label);                    -- Fast label searches
CREATE INDEX idx_edges_source_target ON edges(source_node_id, target_node_id); -- Fast graph traversal
CREATE INDEX idx_layers_graph ON layers(graph_id);

-- REMOVED: plan_graph_links table (Plan DAG embedded in projects)

-- Import snapshots for synchronization
CREATE TABLE imports (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    import_type TEXT NOT NULL, -- csv, rest, sql
    source_metadata TEXT NOT NULL, -- JSON: connection details
    snapshot_data BLOB,        -- Compressed snapshot
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);
```

#### **1.3 SeaORM Entity Generation (Week 5)**

**Generate Rust Entities** from new schema:

```rust
// src/database/entities/mod.rs - Updated entity structure
pub mod users;
pub mod groups;
pub mod user_groups;
pub mod projects;     // Now includes plan_dag field
pub mod graphs;
pub mod nodes;
pub mod edges;
pub mod layers;
pub mod crdt_changes;
pub mod imports;
// REMOVED: plans, plan_graph_links

// UPDATED: Projects entity with embedded Plan DAG
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub group_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub plan_dag: String,    // JSON: Plan DAG structure
    pub created_by: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

// Enhanced graph entity with CRDT support
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "graphs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_copy: bool,
    pub original_graph_id: Option<i32>,
    pub copy_metadata: Option<String>,
    pub crdt_state: String, // Serialized CRDT document
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

#### **1.4 Core Services Architecture (Weeks 6-8)**

**Service Layer Design** with CRDT integration:

```rust
// REFINED: Enhanced CRDT service for hybrid approach
use selected_crdt_library::*; // Based on research findings

pub struct CrdtService {
    db: DatabaseConnection,
    crdt_documents: Arc<RwLock<HashMap<i32, CrdtDocument>>>, // In-memory CRDT docs
}

impl CrdtService {
    // REFINED: Hybrid CRDT management
    pub async fn create_graph_document(&self, graph_id: i32) -> Result<CrdtDocument>;

    // ENHANCED: Apply changes to both CRDT and normalized tables
    pub async fn apply_node_change(&self, graph_id: i32, user_id: i32, change: NodeChange) -> Result<()> {
        let session_id = Uuid::new_v4().to_string();

        // 1. Apply to CRDT document
        let crdt_vector = self.apply_to_crdt_document(graph_id, &change).await?;

        // 2. Apply to normalized table
        self.apply_to_normalized_table(graph_id, user_id, &change, &crdt_vector).await?;

        // 3. Log change for synchronization
        self.log_change(graph_id, user_id, session_id, "node", &change, &crdt_vector).await?;

        // 4. Invalidate JSON cache
        self.invalidate_graph_cache(graph_id).await?;

        Ok(())
    }

    pub async fn apply_edge_change(&self, graph_id: i32, user_id: i32, change: EdgeChange) -> Result<()> {
        let session_id = Uuid::new_v4().to_string();

        // Similar pattern for edges
        let crdt_vector = self.apply_to_crdt_document(graph_id, &change).await?;
        self.apply_to_normalized_table(graph_id, user_id, &change, &crdt_vector).await?;
        self.log_change(graph_id, user_id, session_id, "edge", &change, &crdt_vector).await?;
        self.invalidate_graph_cache(graph_id).await?;

        Ok(())
    }

    // REFINED: Efficient synchronization between instances
    pub async fn sync_with_remote(&self, graph_id: i32, remote_changes: Vec<GraphChange>) -> Result<SyncResult> {
        let mut conflicts = Vec::new();
        let mut applied_changes = Vec::new();

        for remote_change in remote_changes {
            match self.apply_remote_change(graph_id, remote_change).await {
                Ok(()) => applied_changes.push(remote_change),
                Err(ConflictError(conflict)) => conflicts.push(conflict),
                Err(e) => return Err(e),
            }
        }

        // Rebuild cache if any changes applied
        if !applied_changes.is_empty() {
            self.rebuild_graph_cache(graph_id).await?;
        }

        Ok(SyncResult {
            applied_changes,
            conflicts,
            graph_state: self.get_graph_crdt_state(graph_id).await?,
        })
    }

    // HYBRID: Get changes since timestamp for efficient sync
    pub async fn get_changes_since(&self, graph_id: i32, since: DateTime<Utc>) -> Result<Vec<GraphChange>> {
        use graph_changes::Entity as GraphChanges;

        GraphChanges::find()
            .filter(graph_changes::Column::GraphId.eq(graph_id))
            .filter(graph_changes::Column::Timestamp.gt(since))
            .order_by_asc(graph_changes::Column::Timestamp)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // CACHE MANAGEMENT: Hybrid approach cache operations
    async fn invalidate_graph_cache(&self, graph_id: i32) -> Result<()> {
        use graphs::Entity as Graphs;

        Graphs::update_many()
            .filter(graphs::Column::Id.eq(graph_id))
            .set(graphs::ActiveModel {
                cache_version: Set(chrono::Utc::now().timestamp() as i32),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(())
    }

    async fn rebuild_graph_cache(&self, graph_id: i32) -> Result<()> {
        // Load from normalized tables
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        let edges = edges::Entity::find()
            .filter(edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        let layers = layers::Entity::find()
            .filter(layers::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        // Serialize to JSON cache
        let nodes_cache = serde_json::to_string(&nodes)?;
        let edges_cache = serde_json::to_string(&edges)?;
        let layers_cache = serde_json::to_string(&layers)?;

        // Update graph with new cache
        use graphs::Entity as Graphs;
        Graphs::update_many()
            .filter(graphs::Column::Id.eq(graph_id))
            .set(graphs::ActiveModel {
                nodes_cache: Set(Some(nodes_cache)),
                edges_cache: Set(Some(edges_cache)),
                layers_cache: Set(Some(layers_cache)),
                cache_version: Set(chrono::Utc::now().timestamp() as i32),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(())
    }
}

// REFINED: Enhanced GraphService with hybrid data access
pub struct GraphService {
    db: DatabaseConnection,
    crdt_service: Arc<CrdtService>,
}

impl GraphService {
    // HYBRID: Fast graph loading with cache
    pub async fn load_graph(&self, graph_id: i32) -> Result<GraphData> {
        let graph = graphs::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or(GraphError::NotFound)?;

        // Fast path: Use JSON cache if valid and recent
        if let (Some(nodes_cache), Some(edges_cache), Some(layers_cache)) =
            (&graph.nodes_cache, &graph.edges_cache, &graph.layers_cache) {

            // Cache is considered valid if updated within last 5 minutes or no recent changes
            let cache_age = chrono::Utc::now().signed_duration_since(graph.updated_at);
            let has_recent_changes = self.has_recent_changes(graph_id, graph.updated_at).await?;

            if cache_age.num_minutes() < 5 || !has_recent_changes {
                return Ok(GraphData::from_json_cache(
                    graph,
                    nodes_cache,
                    edges_cache,
                    layers_cache,
                ));
            }
        }

        // Slow path: Load from normalized tables and rebuild cache
        self.load_and_rebuild_cache(graph_id).await
    }

    // HYBRID: Efficient loading for specific queries
    pub async fn load_graph_for_analysis(&self, graph_id: i32) -> Result<GraphData> {
        // Always use normalized tables for analysis to ensure data consistency
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        let edges = edges::Entity::find()
            .filter(edges::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        let layers = layers::Entity::find()
            .filter(layers::Column::GraphId.eq(graph_id))
            .all(&self.db)
            .await?;

        Ok(GraphData::from_normalized_data(nodes, edges, layers))
    }

    // REFINED: Node operations with CRDT integration
    pub async fn apply_node_change(&self, graph_id: i32, user_id: i32, change: NodeChange) -> Result<Node> {
        // Use CRDT service for consistent change application
        self.crdt_service.apply_node_change(graph_id, user_id, change).await?;

        // Return updated node from normalized table
        nodes::Entity::find()
            .filter(nodes::Column::GraphId.eq(graph_id))
            .filter(nodes::Column::NodeId.eq(&change.node_id))
            .one(&self.db)
            .await?
            .ok_or(GraphError::NodeNotFound)
    }

    pub async fn apply_edge_change(&self, graph_id: i32, user_id: i32, change: EdgeChange) -> Result<Edge> {
        self.crdt_service.apply_edge_change(graph_id, user_id, change).await?;

        edges::Entity::find()
            .filter(edges::Column::GraphId.eq(graph_id))
            .filter(edges::Column::SourceNodeId.eq(&change.source))
            .filter(edges::Column::TargetNodeId.eq(&change.target))
            .one(&self.db)
            .await?
            .ok_or(GraphError::EdgeNotFound)
    }

    // ENHANCED: Graph copy operations with cache management
    pub async fn create_graph_copy(&self, original_id: i32, name: String, user_id: i32) -> Result<Graph> {
        // Load original graph (use cache for fast copying)
        let original_data = self.load_graph(original_id).await?;

        // Create new graph entry
        let new_graph = graphs::ActiveModel {
            project_id: Set(original_data.project_id),
            name: Set(name),
            is_copy: Set(true),
            original_graph_id: Set(Some(original_id)),
            copy_metadata: Set(Some(serde_json::to_string(&CopyMetadata {
                copied_at: chrono::Utc::now(),
                copied_by: user_id,
                original_node_count: original_data.nodes.len(),
                original_edge_count: original_data.edges.len(),
            })?)),
            crdt_state: Set(String::new()), // Initialize empty CRDT state
            ..Default::default()
        };

        let inserted_graph = new_graph.insert(&self.db).await?;

        // Copy all normalized data
        self.copy_normalized_data(original_id, inserted_graph.id, user_id).await?;

        // Initialize CRDT document for new graph
        self.crdt_service.create_graph_document(inserted_graph.id).await?;

        // Build initial cache
        self.crdt_service.rebuild_graph_cache(inserted_graph.id).await?;

        Ok(inserted_graph)
    }

    // REFINED: Change propagation with selective updates
    pub async fn propagate_changes_to_copies(&self, original_id: i32, changes: Vec<GraphChange>) -> Result<Vec<ConflictReport>> {
        let copies = graphs::Entity::find()
            .filter(graphs::Column::OriginalGraphId.eq(original_id))
            .all(&self.db)
            .await?;

        let mut all_conflicts = Vec::new();

        for copy in copies {
            // Check if copy has local modifications that conflict
            let local_changes = self.crdt_service
                .get_changes_since(copy.id, copy.last_crdt_sync.unwrap_or(copy.created_at))
                .await?;

            if local_changes.is_empty() {
                // No local changes, safe to apply all changes
                let sync_result = self.crdt_service.sync_with_remote(copy.id, changes.clone()).await?;
                all_conflicts.extend(sync_result.conflicts);
            } else {
                // Local changes exist, need conflict resolution
                let conflicts = self.detect_conflicts(&changes, &local_changes).await?;
                all_conflicts.extend(conflicts);
            }
        }

        Ok(all_conflicts)
    }

    // HYBRID: Efficient sync operations
    pub async fn sync_graph_with_remote(&self, graph_id: i32, remote_endpoint: &str) -> Result<SyncResult> {
        // Get last sync timestamp
        let graph = graphs::Entity::find_by_id(graph_id).one(&self.db).await?;
        let last_sync = graph.and_then(|g| g.last_crdt_sync).unwrap_or_default();

        // Fetch remote changes since last sync
        let remote_changes = self.fetch_remote_changes(remote_endpoint, graph_id, last_sync).await?;

        // Apply changes through CRDT service
        let sync_result = self.crdt_service.sync_with_remote(graph_id, remote_changes).await?;

        // Update last sync timestamp
        graphs::Entity::update_many()
            .filter(graphs::Column::Id.eq(graph_id))
            .set(graphs::ActiveModel {
                last_crdt_sync: Set(Some(chrono::Utc::now())),
                ..Default::default()
            })
            .exec(&self.db)
            .await?;

        Ok(sync_result)
    }

    // PERFORMANCE: Helper methods for cache management
    async fn has_recent_changes(&self, graph_id: i32, since: DateTime<Utc>) -> Result<bool> {
        let count = graph_changes::Entity::find()
            .filter(graph_changes::Column::GraphId.eq(graph_id))
            .filter(graph_changes::Column::Timestamp.gt(since))
            .count(&self.db)
            .await?;

        Ok(count > 0)
    }

    async fn load_and_rebuild_cache(&self, graph_id: i32) -> Result<GraphData> {
        // Load from normalized tables
        let graph_data = self.load_graph_for_analysis(graph_id).await?;

        // Rebuild cache asynchronously (don't block response)
        let crdt_service = self.crdt_service.clone();
        tokio::spawn(async move {
            if let Err(e) = crdt_service.rebuild_graph_cache(graph_id).await {
                tracing::warn!("Failed to rebuild cache for graph {}: {}", graph_id, e);
            }
        });

        Ok(graph_data)
    }
}

// src/services/plan_service.rs - UPDATED: Simplified DAG-based
pub struct PlanService {
    db: DatabaseConnection,
    graph_service: Arc<GraphService>,
}

impl PlanService {
    // UPDATED: Plan DAG operations work directly on projects
    pub async fn update_project_plan_dag(&self, project_id: i32, dag: PlanDAG) -> Result<Project>;
    pub async fn execute_plan_step(&self, project_id: i32, step_id: &str) -> Result<ExecutionResult>;
    pub async fn validate_plan_dag(&self, dag: &PlanDAG) -> Result<ValidationResult>;
    pub async fn get_plan_execution_status(&self, project_id: i32) -> Result<ExecutionStatus>;

    // Plan node type specific operations
    pub async fn execute_input_node(&self, project_id: i32, node_config: InputNodeConfig) -> Result<Graph>;
    pub async fn execute_merge_node(&self, project_id: i32, node_config: MergeNodeConfig) -> Result<Graph>;
    pub async fn execute_copy_node(&self, project_id: i32, node_config: CopyNodeConfig) -> Result<Graph>;
    pub async fn execute_output_node(&self, project_id: i32, node_config: OutputNodeConfig) -> Result<String>;
}
```

#### **1.5 Migration System (Weeks 9-12)**

**Data Migration Strategy**:

```rust
// src/database/migrations/migration_helpers.rs
pub struct LegacyDataMigrator {
    // Handle migration from existing CLI-based data
}

impl LegacyDataMigrator {
    // UPDATED: Migrate existing YAML plans to embedded Plan DAG structure
    pub async fn migrate_yaml_to_plan_dag(&self) -> Result<MigrationReport>;

    // Convert existing project structure with embedded Plan DAG
    pub async fn migrate_projects_with_plan_dag(&self) -> Result<()>;

    // Initialize CRDT documents for existing graphs
    pub async fn initialize_crdt_documents(&self) -> Result<()>;
}
```

**Phase 1 Deliverables**:
- ✅ CRDT library selected and integrated
- ✅ Complete database redesign implemented
- ✅ Core service architecture with CRDT support
- ✅ Migration tools for existing data
- ✅ Comprehensive test suite (>80% coverage)

### **Phase 2: Backend API & Core Logic (6 months)**

#### **Goal**: Complete backend implementation with GraphQL API and MCP integration

#### **2.1 GraphQL API Implementation (Weeks 1-4)**

**Comprehensive GraphQL Schema**:

```graphql
# Core types based on new data model
type User {
  id: Int!
  username: String!
  email: String!
  groups: [Group!]!
}

type Group {
  id: Int!
  name: String!
  description: String
  users: [User!]!
  projects: [Project!]!
}

type Project {
  id: Int!
  name: String!
  description: String
  group: Group!
  planDAG: PlanDAG!         # UPDATED: Direct embedded Plan DAG (1:1)
  graphs: [Graph!]!
  createdBy: User!
}

# UPDATED: Plan DAG embedded directly in Project (1:1 relationship)
type PlanDAG {
  nodes: [PlanNode!]!
  edges: [PlanEdge!]!
}

type PlanNode {
  id: String!
  type: PlanNodeType!
  config: PlanNodeConfig!  # Strong typing instead of JSON
  position: Position
  metadata: PlanNodeMetadata!
}

# UPDATED: Specific Plan node types per specification
enum PlanNodeType {
  INPUT_NODE     # Import from file/REST/SQL → creates graph
  MERGE_NODE     # Combine inputs/graphs → output graph
  COPY_NODE      # Copy graph → new graph
  OUTPUT_NODE    # Export graph → render target
}

# UPDATED: Node-specific configuration types
union PlanNodeConfig = InputNodeConfig | MergeNodeConfig | CopyNodeConfig | OutputNodeConfig

type InputNodeConfig {
  type: ImportType!          # file, rest, sql
  source: String!            # file path, URL, connection string
  dataType: DataType!        # nodes, edges, layers
  outputGraphId: String!     # target graph
}

type MergeNodeConfig {
  inputGraphIds: [String!]!  # source graphs
  mergeStrategy: String!     # union, intersection, etc.
  outputGraphId: String!     # result graph
}

type CopyNodeConfig {
  sourceGraphId: String!
  copyName: String!
  outputGraphId: String!
}

type OutputNodeConfig {
  sourceGraphId: String!
  renderTarget: String!      # DOT, GML, PlantUML, etc.
  exportPath: String!
}

enum ImportType {
  FILE
  REST
  SQL
}

enum DataType {
  NODES
  EDGES
  LAYERS
}

type PlanNodeMetadata {
  label: String!
  description: String
}

# REFINED: Graphs with hybrid CRDT support
type Graph {
  id: Int!
  name: String!
  description: String
  project: Project!
  isCopy: Boolean!
  originalGraph: Graph
  copies: [Graph!]!

  # HYBRID: Cache and CRDT state management
  cacheVersion: Int!             # Cache invalidation tracking
  lastCrdtSync: DateTime         # Last successful CRDT sync
  crdtState: String!             # CRDT document state

  # Data access options for different use cases
  nodes: [Node!]!                # Always from normalized tables
  edges: [Edge!]!                # Always from normalized tables
  layers: [Layer!]!              # Always from normalized tables

  # Fast loading option for large graphs
  cachedData: GraphCacheData     # Optional cached JSON data

  # Change tracking
  changeHistory: [GraphChange!]!
  recentChanges: [GraphChange!]! # Changes since last cache update
}

type Node {
  id: String!
  label: String!
  graph: Graph!
  layer: Layer
  position: Position
  properties: JSON

  # REFINED: Enhanced CRDT metadata for hybrid approach
  crdtVector: String!           # CRDT version vector
  lastModifiedBy: User!         # User who last modified
  lastModifiedAt: DateTime!     # When last modified
}

type Position {
  x: Float!
  y: Float!
}

# REFINED: Enhanced change tracking for hybrid approach
type GraphChange {
  id: Int!
  graphId: Int!
  entityType: String!           # 'node', 'edge', 'layer'
  entityId: String!             # node_id, edge_id, layer_id
  changeType: String!           # 'create', 'update', 'delete'
  changeData: JSON!             # Change payload
  crdtVector: String!           # CRDT version vector
  user: User!
  sessionId: String!            # For grouping related changes
  timestamp: DateTime!
}

# HYBRID: Optional cached data for fast loading
type GraphCacheData {
  nodesJson: String!            # JSON cache of nodes
  edgesJson: String!            # JSON cache of edges
  layersJson: String!           # JSON cache of layers
  cacheTimestamp: DateTime!     # When cache was generated
  isValid: Boolean!             # Cache validity status
}

# REFINED: Mutations for hybrid CRDT operations
type Mutation {
  # Plan mutations work directly on projects
  updateProjectPlanDAG(projectId: Int!, dag: PlanDAGInput!): Project!
  executePlanStep(projectId: Int!, stepId: String!): ExecutionResult!

  # Graph mutations with hybrid CRDT support
  createGraph(input: CreateGraphInput!): Graph!
  createGraphCopy(originalId: Int!, name: String!): Graph!

  # REFINED: Enhanced CRDT operations for normalized data
  applyNodeChange(graphId: Int!, userId: Int!, change: NodeChangeInput!): NodeChangeResult!
  applyEdgeChange(graphId: Int!, userId: Int!, change: EdgeChangeInput!): EdgeChangeResult!
  applyLayerChange(graphId: Int!, userId: Int!, change: LayerChangeInput!): LayerChangeResult!

  # HYBRID: Bulk operations with cache management
  applyBulkChanges(graphId: Int!, userId: Int!, changes: [BulkChangeInput!]!): BulkChangeResult!

  # REFINED: Synchronization operations
  syncGraphWithRemote(graphId: Int!, remoteChanges: [GraphChangeInput!]!): SyncResult!
  propagateChangesToCopies(originalGraphId: Int!, changes: [GraphChangeInput!]!): PropagationResult!

  # Cache management
  invalidateGraphCache(graphId: Int!): Boolean!
  rebuildGraphCache(graphId: Int!): GraphCacheData!

  # Bulk operations
  importCsvData(projectId: Int!, files: [CsvFileInput!]!): ImportResult!
}

# REFINED: Enhanced result types for hybrid operations
type NodeChangeResult {
  node: Node!
  crdtVector: String!
  cacheInvalidated: Boolean!
  conflicts: [ChangeConflict!]!
}

type EdgeChangeResult {
  edge: Edge!
  crdtVector: String!
  cacheInvalidated: Boolean!
  conflicts: [ChangeConflict!]!
}

type LayerChangeResult {
  layer: Layer!
  crdtVector: String!
  cacheInvalidated: Boolean!
  conflicts: [ChangeConflict!]!
}

type BulkChangeResult {
  appliedChanges: Int!
  failedChanges: Int!
  conflicts: [ChangeConflict!]!
  cacheRebuilt: Boolean!
  newCacheVersion: Int!
}

type SyncResult {
  appliedChanges: [GraphChange!]!
  conflicts: [ChangeConflict!]!
  graphState: String!         # Updated CRDT state
  cacheRebuilt: Boolean!
}

type PropagationResult {
  copiesUpdated: Int!
  conflicts: [ConflictReport!]!
  partialFailures: [PropagationError!]!
}

type ChangeConflict {
  entityType: String!
  entityId: String!
  localChange: GraphChange!
  remoteChange: GraphChange!
  resolutionStrategy: String   # Suggested resolution
}

# REFINED: Subscriptions for real-time updates with hybrid support
type Subscription {
  # REFINED: Enhanced real-time change notifications
  graphChanges(graphId: Int!): GraphChange!
  graphChangesBatch(graphId: Int!, batchSize: Int = 10): [GraphChange!]!

  # Plan execution updates
  planExecution(projectId: Int!): ExecutionUpdate!

  # Project-level updates
  projectUpdates(projectId: Int!): ProjectUpdate!

  # HYBRID: Cache invalidation notifications
  cacheUpdates(graphId: Int!): CacheUpdateNotification!

  # REFINED: Conflict notifications for real-time collaboration
  conflictNotifications(graphId: Int!): ChangeConflict!
}

type CacheUpdateNotification {
  graphId: Int!
  cacheVersion: Int!
  invalidated: Boolean!
  rebuilding: Boolean!
}
```

#### **2.2 MCP API Implementation (Weeks 5-8)**

**Enhanced MCP Tools** for AI integration:

```rust
// src/mcp/tools/graph_analysis.rs
pub struct GraphAnalysisTools {
    graph_service: Arc<GraphService>,
    plan_service: Arc<PlanService>,
}

impl GraphAnalysisTools {
    // Advanced graph analysis tools
    pub async fn analyze_graph_structure(&self, graph_id: i32) -> McpResult<GraphAnalysis>;
    pub async fn find_critical_paths(&self, graph_id: i32) -> McpResult<Vec<Path>>;
    pub async fn detect_cycles(&self, graph_id: i32) -> McpResult<Vec<Cycle>>;
    pub async fn calculate_centrality_measures(&self, graph_id: i32) -> McpResult<CentralityReport>;

    // CRDT-aware tools
    pub async fn analyze_change_patterns(&self, graph_id: i32, time_range: TimeRange) -> McpResult<ChangePatternAnalysis>;
    pub async fn predict_merge_conflicts(&self, graph_id: i32) -> McpResult<ConflictPrediction>;

    // UPDATED: Plan generation tools work with project Plan DAGs
    pub async fn suggest_transformation_plan(&self, source_graph_id: i32, target_description: String) -> McpResult<PlanSuggestion>;
    pub async fn optimize_project_plan_dag(&self, project_id: i32) -> McpResult<OptimizedPlanDAG>;
    pub async fn validate_plan_dag_structure(&self, project_id: i32) -> McpResult<ValidationReport>;
}

// src/mcp/resources/graph_insights.rs
pub struct GraphInsightResources {
    // Provide rich graph data for AI analysis
    pub async fn get_graph_summary(&self, graph_id: i32) -> McpResult<GraphSummary>;
    pub async fn get_change_timeline(&self, graph_id: i32) -> McpResult<ChangeTimeline>;
    pub async fn get_collaboration_metrics(&self, project_id: i32) -> McpResult<CollaborationMetrics>;
}
```

#### **2.3 WebSocket Real-time API (Weeks 9-12)**

**Real-time CRDT Synchronization**:

```rust
// src/websocket/crdt_sync.rs
pub struct CrdtSyncHandler {
    crdt_service: Arc<CrdtService>,
    connection_manager: Arc<ConnectionManager>,
}

impl CrdtSyncHandler {
    pub async fn handle_change_broadcast(&self, graph_id: i32, change: CrdtChange);
    pub async fn sync_peer_to_peer(&self, local_graph_id: i32, remote_endpoint: &str);
    pub async fn handle_conflict_resolution(&self, graph_id: i32, resolution: ConflictResolution);
}

// WebSocket message types
#[derive(Serialize, Deserialize)]
pub enum WsMessage {
    Subscribe { graph_id: i32 },
    Unsubscribe { graph_id: i32 },
    CrdtChange { graph_id: i32, change: CrdtChange },
    SyncRequest { graph_id: i32, since: Option<Timestamp> },
    SyncResponse { graph_id: i32, changes: Vec<CrdtChange> },
    ConflictNotification { graph_id: i32, conflict: MergeConflict },
}
```

**Phase 2 Deliverables**:
- ✅ Complete GraphQL API with CRDT operations
- ✅ Rich MCP API for AI tool integration
- ✅ Real-time WebSocket synchronization
- ✅ Comprehensive API documentation
- ✅ Performance benchmarks (1000+ concurrent users)

### **Phase 3: Frontend Foundation (8 months)**

#### **Goal**: Complete React application with core graph editing functionality

#### **3.1 React Application Setup (Weeks 1-2)**

**Modern React Stack**:

```json
// frontend/package.json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@mantine/core": "^7.0.0",
    "@mantine/hooks": "^7.0.0",
    "@mantine/notifications": "^7.0.0",
    "@mantine/modals": "^7.0.0",
    "@mantine/spotlight": "^7.0.0",

    // Graph editing
    "@xyflow/react": "^12.0.0",
    "d3": "^7.8.0",
    "d3-force": "^3.0.0",
    "react-force-graph-2d": "^1.25.0",
    "react-force-graph-3d": "^1.24.0",

    // State management with CRDT
    "zustand": "^4.4.0",
    "immer": "^10.0.0",
    "y-websocket": "^1.5.0", // WebSocket provider for Yjs
    "yjs": "^13.6.0", // Client-side CRDT

    // GraphQL & Real-time
    "@apollo/client": "^3.8.0",
    "graphql": "^16.8.0",
    "graphql-ws": "^5.14.0",

    // Form handling
    "react-hook-form": "^7.46.0",
    "@hookform/resolvers": "^3.3.0",
    "zod": "^3.22.0",

    // File handling
    "react-dropzone": "^14.2.0",
    "papaparse": "^5.4.0"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.0.0",
    "vite": "^5.0.0",
    "typescript": "^5.2.0",
    "@types/d3": "^7.4.0",
    "vitest": "^1.0.0",
    "@testing-library/react": "^14.0.0",
    "playwright": "^1.40.0"
  }
}
```

#### **3.2 CRDT Integration & State Management (Weeks 3-4)**

**Client-side CRDT Integration**:

```typescript
// src/stores/crdtStore.ts
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';

export class CrdtStore {
  private docs: Map<number, Y.Doc> = new Map();
  private providers: Map<number, WebsocketProvider> = new Map();

  // Initialize CRDT document for a graph
  initializeGraph(graphId: number): Y.Doc {
    const doc = new Y.Doc();

    // Define shared types for graph data
    const nodes = doc.getMap('nodes');
    const edges = doc.getMap('edges');
    const layers = doc.getMap('layers');

    // Set up WebSocket provider for real-time sync
    const provider = new WebsocketProvider(
      `ws://localhost:3000/crdt-sync`,
      `graph-${graphId}`,
      doc
    );

    this.docs.set(graphId, doc);
    this.providers.set(graphId, provider);

    return doc;
  }

  // Apply local changes with conflict resolution
  applyNodeChange(graphId: number, nodeId: string, change: Partial<Node>) {
    const doc = this.docs.get(graphId);
    if (!doc) throw new Error(`Graph ${graphId} not initialized`);

    const nodes = doc.getMap('nodes');
    const nodeMap = nodes.get(nodeId) as Y.Map<any> || new Y.Map();

    // Apply changes atomically
    doc.transact(() => {
      Object.entries(change).forEach(([key, value]) => {
        nodeMap.set(key, value);
      });
      nodes.set(nodeId, nodeMap);
    });
  }
}

// REFINED: Enhanced CRDT sync with hybrid data loading
export function useCrdtSync(graphId: number, options: CrdtSyncOptions = {}) {
  const [crdtStore] = useState(() => new CrdtStore());
  const [doc, setDoc] = useState<Y.Doc | null>(null);
  const [cacheStatus, setCacheStatus] = useState<CacheStatus>('unknown');
  const [loadingStrategy, setLoadingStrategy] = useState<'cache' | 'normalized'>('cache');

  // HYBRID: Intelligent data loading strategy
  const { data: graphData, loading, error } = useQuery(GET_GRAPH_DATA, {
    variables: {
      graphId,
      useCache: options.preferCache !== false,
      includeCache: true
    },
    onCompleted: (data) => {
      // Determine best loading strategy based on data freshness
      if (data.graph.cachedData?.isValid && data.graph.cachedData.cacheTimestamp) {
        const cacheAge = Date.now() - new Date(data.graph.cachedData.cacheTimestamp).getTime();
        setLoadingStrategy(cacheAge < 300000 ? 'cache' : 'normalized'); // 5 minutes
        setCacheStatus(cacheAge < 300000 ? 'valid' : 'stale');
      } else {
        setLoadingStrategy('normalized');
        setCacheStatus('invalid');
      }
    }
  });

  // REFINED: Subscribe to real-time changes with batching
  const { data: changes } = useSubscription(GRAPH_CHANGES_BATCH, {
    variables: { graphId, batchSize: 10 },
    skip: !doc
  });

  // REFINED: Subscribe to cache invalidation notifications
  const { data: cacheUpdates } = useSubscription(CACHE_UPDATES, {
    variables: { graphId },
    onSubscriptionData: ({ subscriptionData }) => {
      if (subscriptionData.data?.cacheUpdates.invalidated) {
        setCacheStatus('invalid');
        // Optionally trigger reload if cache was being used
        if (loadingStrategy === 'cache' && options.autoReloadOnCacheInvalidation) {
          refetch();
        }
      }
    }
  });

  useEffect(() => {
    if (!graphData) return;

    const document = crdtStore.initializeGraph(graphId);
    setDoc(document);

    // HYBRID: Load initial data using optimal strategy
    if (loadingStrategy === 'cache' && graphData.graph.cachedData?.isValid) {
      // Fast path: Load from JSON cache
      crdtStore.loadFromCache(graphId, graphData.graph.cachedData);
    } else {
      // Reliable path: Load from normalized data
      crdtStore.loadFromNormalizedData(graphId, {
        nodes: graphData.graph.nodes,
        edges: graphData.graph.edges,
        layers: graphData.graph.layers
      });
    }

    // Set up change listeners with optimized updates
    const handleUpdate = throttle(() => {
      // Trigger React re-render when CRDT state changes
      forceUpdate();
    }, 16); // 60fps throttling

    document.on('update', handleUpdate);

    return () => {
      document.off('update', handleUpdate);
      crdtStore.cleanup(graphId);
    };
  }, [graphId, graphData, loadingStrategy]);

  // REFINED: Apply changes with conflict handling
  const applyChange = useCallback(async (entityType: 'node' | 'edge' | 'layer', entityId: string, change: any) => {
    try {
      const result = await apolloClient.mutate({
        mutation: APPLY_CHANGE_MUTATION[entityType],
        variables: {
          graphId,
          userId: getCurrentUserId(),
          change: { entityId, ...change }
        }
      });

      // Handle conflicts if any
      if (result.data?.conflicts?.length > 0) {
        onConflictDetected?.(result.data.conflicts);
      }

      // Update local CRDT document
      crdtStore.applyChange(graphId, entityType, entityId, change);

      return result.data;
    } catch (error) {
      console.error('Failed to apply change:', error);
      throw error;
    }
  }, [graphId]);

  // HYBRID: Optimized bulk operations
  const applyBulkChanges = useCallback(async (changes: BulkChange[]) => {
    try {
      const result = await apolloClient.mutate({
        mutation: APPLY_BULK_CHANGES,
        variables: {
          graphId,
          userId: getCurrentUserId(),
          changes
        }
      });

      // Update cache status if cache was rebuilt
      if (result.data?.bulkChangeResult.cacheRebuilt) {
        setCacheStatus('valid');
      }

      return result.data;
    } catch (error) {
      console.error('Failed to apply bulk changes:', error);
      throw error;
    }
  }, [graphId]);

  return {
    doc,
    loading,
    error,
    cacheStatus,
    loadingStrategy,
    applyChange,
    applyBulkChanges,
    isConnected: doc?.isLoaded || false,

    // HYBRID: Utility functions
    invalidateCache: () => apolloClient.mutate({
      mutation: INVALIDATE_GRAPH_CACHE,
      variables: { graphId }
    }),
    rebuildCache: () => apolloClient.mutate({
      mutation: REBUILD_GRAPH_CACHE,
      variables: { graphId }
    })
  };
}

// REFINED: Enhanced CrdtStore with hybrid data loading
export class CrdtStore {
  private docs: Map<number, Y.Doc> = new Map();
  private providers: Map<number, WebsocketProvider> = new Map();

  // HYBRID: Load from JSON cache for fast initialization
  loadFromCache(graphId: number, cachedData: GraphCacheData): void {
    const doc = this.docs.get(graphId);
    if (!doc) return;

    try {
      // Parse cached JSON data
      const nodes = JSON.parse(cachedData.nodesJson);
      const edges = JSON.parse(cachedData.edgesJson);
      const layers = JSON.parse(cachedData.layersJson);

      // Apply to CRDT document
      doc.transact(() => {
        const nodesMap = doc.getMap('nodes');
        const edgesMap = doc.getMap('edges');
        const layersMap = doc.getMap('layers');

        nodes.forEach(node => {
          const nodeMap = new Y.Map();
          Object.entries(node).forEach(([key, value]) => {
            nodeMap.set(key, value);
          });
          nodesMap.set(node.id, nodeMap);
        });

        edges.forEach(edge => {
          const edgeMap = new Y.Map();
          Object.entries(edge).forEach(([key, value]) => {
            edgeMap.set(key, value);
          });
          edgesMap.set(`${edge.source}-${edge.target}`, edgeMap);
        });

        layers.forEach(layer => {
          const layerMap = new Y.Map();
          Object.entries(layer).forEach(([key, value]) => {
            layerMap.set(key, value);
          });
          layersMap.set(layer.id, layerMap);
        });
      });
    } catch (error) {
      console.error('Failed to load from cache:', error);
      // Fallback to normalized loading would be triggered by the caller
    }
  }

  // HYBRID: Load from normalized data for consistency
  loadFromNormalizedData(graphId: number, data: { nodes: any[], edges: any[], layers: any[] }): void {
    const doc = this.docs.get(graphId);
    if (!doc) return;

    doc.transact(() => {
      const nodesMap = doc.getMap('nodes');
      const edgesMap = doc.getMap('edges');
      const layersMap = doc.getMap('layers');

      // Clear existing data
      nodesMap.clear();
      edgesMap.clear();
      layersMap.clear();

      // Load normalized data
      data.nodes.forEach(node => {
        const nodeMap = new Y.Map();
        Object.entries(node).forEach(([key, value]) => {
          nodeMap.set(key, value);
        });
        nodesMap.set(node.nodeId, nodeMap);
      });

      data.edges.forEach(edge => {
        const edgeMap = new Y.Map();
        Object.entries(edge).forEach(([key, value]) => {
          edgeMap.set(key, value);
        });
        edgesMap.set(`${edge.sourceNodeId}-${edge.targetNodeId}`, edgeMap);
      });

      data.layers.forEach(layer => {
        const layerMap = new Y.Map();
        Object.entries(layer).forEach(([key, value]) => {
          layerMap.set(key, value);
        });
        layersMap.set(layer.layerId, layerMap);
      });
    });
  }
}

// REFINED: Type definitions for hybrid approach
interface CrdtSyncOptions {
  preferCache?: boolean;
  autoReloadOnCacheInvalidation?: boolean;
  onConflictDetected?: (conflicts: ChangeConflict[]) => void;
}

type CacheStatus = 'unknown' | 'valid' | 'stale' | 'invalid';

interface BulkChange {
  entityType: 'node' | 'edge' | 'layer';
  entityId: string;
  changeType: 'create' | 'update' | 'delete';
  data: any;
}
```

#### **3.3 Core Components (Weeks 5-12)**

**PlanVisualEditor Component**:

```typescript
// src/components/plan/PlanVisualEditor.tsx
import ReactFlow, {
  Node,
  Edge,
  Controls,
  Background,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  NodeTypes
} from '@xyflow/react';

// UPDATED: Node types for specific Plan DAG node types
const nodeTypes: NodeTypes = {
  inputnode: InputNodeComponent,      // INPUT_NODE
  mergenode: MergeNodeComponent,      // MERGE_NODE
  copynode: CopyNodeComponent,        // COPY_NODE
  outputnode: OutputNodeComponent     // OUTPUT_NODE
};

export function PlanVisualEditor({ projectId }: { projectId: number }) {
  const { data: project, refetch } = useQuery(GET_PROJECT, { variables: { id: projectId } });
  const [updateProjectPlanDAG] = useMutation(UPDATE_PROJECT_PLAN_DAG);

  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);
  const [selectedNode, setSelectedNode] = useState<PlanNode | null>(null);
  const [nodeConfigModalOpen, setNodeConfigModalOpen] = useState(false);

  // UPDATED: Convert Project Plan DAG to ReactFlow format
  useEffect(() => {
    if (project?.planDAG) {
      const flowNodes = project.planDAG.nodes.map(node => ({
        id: node.id,
        type: node.type.toLowerCase().replace('_', ''), // input_node -> inputnode
        position: node.position || { x: 0, y: 0 },
        data: {
          config: node.config,
          metadata: node.metadata,
          projectId,
          onConfigUpdate: handleNodeConfigUpdate,
          onEditConfig: () => openNodeConfigModal(node)
        }
      }));

      const flowEdges = project.planDAG.edges.map(edge => ({
        id: `${edge.source}-${edge.target}`,
        source: edge.source,
        target: edge.target,
        type: 'smoothstep'
      }));

      setNodes(flowNodes);
      setEdges(flowEdges);
    }
  }, [project]);

  const onConnect = useCallback((connection: Connection) => {
    setEdges(eds => addEdge(connection, eds));
    // Auto-save plan changes
    savePlanChanges();
  }, []);

  const handleNodeConfigUpdate = useCallback((nodeId: string, config: any) => {
    setNodes(nodes =>
      nodes.map(node =>
        node.id === nodeId
          ? { ...node, data: { ...node.data, config } }
          : node
      )
    );
    savePlanChanges();
  }, []);

  // UPDATED: Enhanced node configuration handling
  const openNodeConfigModal = useCallback((node: PlanNode) => {
    setSelectedNode(node);
    setNodeConfigModalOpen(true);
  }, []);

  const savePlanChanges = useCallback(() => {
    const dagStructure = {
      nodes: nodes.map(node => ({
        id: node.id,
        type: node.type.toUpperCase().replace('NODE', '_NODE'), // inputnode -> INPUT_NODE
        position: node.position,
        config: node.data.config,
        metadata: node.data.metadata
      })),
      edges: edges.map(edge => ({
        source: edge.source,
        target: edge.target
      }))
    };

    return updateProjectPlanDAG({
      variables: { projectId, dag: dagStructure }
    });
  }, [nodes, edges, projectId]);

  const handleNodeConfigSave = useCallback((nodeId: string, newConfig: PlanNodeConfig, newMetadata: PlanNodeMetadata) => {
    setNodes(nodes =>
      nodes.map(node =>
        node.id === nodeId
          ? {
              ...node,
              data: {
                ...node.data,
                config: newConfig,
                metadata: newMetadata
              }
            }
          : node
      )
    );
    setNodeConfigModalOpen(false);
    savePlanChanges();
  }, [savePlanChanges]);

  return (
    <div className="h-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        fitView
      >
        <Controls />
        <Background />
      </ReactFlow>

      {/* UPDATED: Node configuration modal with type-specific editors */}
      <PlanNodeConfigModal
        opened={nodeConfigModalOpen}
        onClose={() => setNodeConfigModalOpen(false)}
        node={selectedNode}
        onSave={handleNodeConfigSave}
        projectGraphs={project?.graphs || []}
      />
    </div>
  );
}

// UPDATED: Type-specific node configuration modal
function PlanNodeConfigModal({
  opened,
  onClose,
  node,
  onSave,
  projectGraphs
}: {
  opened: boolean;
  onClose: () => void;
  node: PlanNode | null;
  onSave: (nodeId: string, config: PlanNodeConfig, metadata: PlanNodeMetadata) => void;
  projectGraphs: Graph[];
}) {
  if (!node) return null;

  const renderConfigEditor = () => {
    switch (node.type) {
      case 'INPUT_NODE':
        return (
          <InputNodeConfigEditor
            config={node.config as InputNodeConfig}
            metadata={node.metadata}
            onSave={(config, metadata) => onSave(node.id, config, metadata)}
          />
        );
      case 'MERGE_NODE':
        return (
          <MergeNodeConfigEditor
            config={node.config as MergeNodeConfig}
            metadata={node.metadata}
            availableGraphs={projectGraphs}
            onSave={(config, metadata) => onSave(node.id, config, metadata)}
          />
        );
      case 'COPY_NODE':
        return (
          <CopyNodeConfigEditor
            config={node.config as CopyNodeConfig}
            metadata={node.metadata}
            availableGraphs={projectGraphs}
            onSave={(config, metadata) => onSave(node.id, config, metadata)}
          />
        );
      case 'OUTPUT_NODE':
        return (
          <OutputNodeConfigEditor
            config={node.config as OutputNodeConfig}
            metadata={node.metadata}
            availableGraphs={projectGraphs}
            onSave={(config, metadata) => onSave(node.id, config, metadata)}
          />
        );
      default:
        return <div>Unknown node type: {node.type}</div>;
    }
  };

  return (
    <Modal opened={opened} onClose={onClose} size="lg" title={`Configure ${node.metadata.label}`}>
      {renderConfigEditor()}
    </Modal>
  );
}

// UPDATED: Example InputNode configuration editor with file selection
function InputNodeConfigEditor({
  config,
  metadata,
  onSave
}: {
  config: InputNodeConfig;
  metadata: PlanNodeMetadata;
  onSave: (config: InputNodeConfig, metadata: PlanNodeMetadata) => void;
}) {
  const [importType, setImportType] = useState(config.type || 'FILE');
  const [source, setSource] = useState(config.source || '');
  const [dataType, setDataType] = useState(config.dataType || 'NODES');
  const [label, setLabel] = useState(metadata.label || 'Input Node');

  const handleFileSelect = async () => {
    // Desktop integration for file selection
    if (window.__TAURI__) {
      const selected = await window.__TAURI__.dialog.open({
        filters: [{ name: 'CSV', extensions: ['csv'] }]
      });
      if (selected) setSource(selected);
    } else {
      // Web file input fallback
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = '.csv';
      input.onchange = (e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (file) setSource(file.name);
      };
      input.click();
    }
  };

  const handleSave = () => {
    const newConfig: InputNodeConfig = {
      type: importType,
      source,
      dataType,
      outputGraphId: config.outputGraphId || `graph_${Date.now()}`
    };

    const newMetadata: PlanNodeMetadata = {
      label,
      description: metadata.description
    };

    onSave(newConfig, newMetadata);
  };

  return (
    <Stack>
      <TextInput
        label="Node Label"
        value={label}
        onChange={(e) => setLabel(e.target.value)}
      />

      <Select
        label="Import Type"
        value={importType}
        onChange={setImportType}
        data={[
          { value: 'FILE', label: 'File' },
          { value: 'REST', label: 'REST API' },
          { value: 'SQL', label: 'SQL Database' }
        ]}
      />

      {importType === 'FILE' && (
        <Group>
          <TextInput
            style={{ flex: 1 }}
            label="File Path"
            value={source}
            onChange={(e) => setSource(e.target.value)}
          />
          <Button onClick={handleFileSelect}>Browse</Button>
        </Group>
      )}

      {importType === 'REST' && (
        <TextInput
          label="REST Endpoint URL"
          value={source}
          onChange={(e) => setSource(e.target.value)}
          placeholder="https://api.example.com/data"
        />
      )}

      <Select
        label="Data Type"
        value={dataType}
        onChange={setDataType}
        data={[
          { value: 'NODES', label: 'Nodes' },
          { value: 'EDGES', label: 'Edges' },
          { value: 'LAYERS', label: 'Layers' }
        ]}
      />

      <Group justify="flex-end">
        <Button onClick={handleSave}>Save Configuration</Button>
      </Group>
    </Stack>
  );
}
```

**GraphVisualEditor Component**:

```typescript
// src/components/graph/GraphVisualEditor.tsx
export function GraphVisualEditor({ graphId }: { graphId: number }) {
  const { doc, applyChange, isConnected } = useCrdtSync(graphId);
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  // Sync CRDT changes to ReactFlow state
  useEffect(() => {
    if (!doc) return;

    const nodesMap = doc.getMap('nodes');
    const edgesMap = doc.getMap('edges');

    const updateFromCrdt = () => {
      // Convert CRDT state to ReactFlow format
      const flowNodes = Array.from(nodesMap.entries()).map(([id, nodeData]) => ({
        id,
        type: 'custom',
        position: nodeData.get('position') || { x: 0, y: 0 },
        data: {
          label: nodeData.get('label'),
          layer: nodeData.get('layer'),
          properties: nodeData.get('properties')
        }
      }));

      const flowEdges = Array.from(edgesMap.entries()).map(([id, edgeData]) => ({
        id,
        source: edgeData.get('source'),
        target: edgeData.get('target'),
        type: 'smoothstep'
      }));

      setNodes(flowNodes);
      setEdges(flowEdges);
    };

    nodesMap.observe(updateFromCrdt);
    edgesMap.observe(updateFromCrdt);
    updateFromCrdt(); // Initial load

    return () => {
      nodesMap.unobserve(updateFromCrdt);
      edgesMap.unobserve(updateFromCrdt);
    };
  }, [doc]);

  const onNodeDragStop = useCallback((event: any, node: Node) => {
    // Apply position change via CRDT
    applyChange(node.id, { position: node.position });
  }, [applyChange]);

  const onConnect = useCallback((connection: Connection) => {
    if (connection.source && connection.target) {
      const edgeId = `${connection.source}-${connection.target}`;
      applyChange(edgeId, {
        source: connection.source,
        target: connection.target
      });
    }
  }, [applyChange]);

  return (
    <div className="h-full">
      <div className="flex items-center gap-2 p-2 bg-gray-50">
        <Badge color={isConnected ? 'green' : 'red'}>
          {isConnected ? 'Connected' : 'Disconnected'}
        </Badge>
        <Text size="sm">Real-time Collaboration Active</Text>
      </div>

      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeDragStop={onNodeDragStop}
        nodeTypes={customNodeTypes}
        edgeTypes={customEdgeTypes}
      >
        <Controls />
        <Background />
      </ReactFlow>
    </div>
  );
}
```

**GraphSpreadsheetEditor Component**:

```typescript
// src/components/graph/GraphSpreadsheetEditor.tsx
export function GraphSpreadsheetEditor({ graphId }: { graphId: number }) {
  const { doc, applyChange } = useCrdtSync(graphId);
  const [activeTab, setActiveTab] = useState('nodes');
  const [editingCell, setEditingCell] = useState<{row: number, col: string} | null>(null);

  const handleCellEdit = useCallback((nodeId: string, field: string, value: any) => {
    applyChange(nodeId, { [field]: value });
  }, [applyChange]);

  return (
    <Stack>
      <Group justify="space-between">
        <Title order={3}>Graph Data Editor</Title>
        <Group>
          <Button onClick={() => exportToCsv()}>Export CSV</Button>
          <Button onClick={() => importFromCsv()}>Import CSV</Button>
        </Group>
      </Group>

      <Tabs value={activeTab} onChange={setActiveTab}>
        <Tabs.List>
          <Tabs.Tab value="nodes">Nodes</Tabs.Tab>
          <Tabs.Tab value="edges">Edges</Tabs.Tab>
          <Tabs.Tab value="layers">Layers</Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="nodes">
          <NodesTable
            graphId={graphId}
            onCellEdit={handleCellEdit}
            editingCell={editingCell}
            setEditingCell={setEditingCell}
          />
        </Tabs.Panel>

        <Tabs.Panel value="edges">
          <EdgesTable
            graphId={graphId}
            onCellEdit={handleCellEdit}
          />
        </Tabs.Panel>

        <Tabs.Panel value="layers">
          <LayersTable
            graphId={graphId}
            onCellEdit={handleCellEdit}
          />
        </Tabs.Panel>
      </Tabs>
    </Stack>
  );
}
```

#### **3.4 Advanced Features (Weeks 13-16)**

**Hierarchy Visualization**:

```typescript
// src/components/hierarchy/HierarchyViewer.tsx
export function HierarchyViewer({ projectId }: { projectId: number }) {
  const { data: graphs } = useQuery(GET_PROJECT_GRAPHS, { variables: { projectId } });

  // Build hierarchy tree from graph copies
  const hierarchyTree = useMemo(() => {
    if (!graphs) return null;

    return buildHierarchyTree(graphs.project.graphs);
  }, [graphs]);

  return (
    <div className="h-full">
      <Tree
        data={hierarchyTree}
        onNodeSelect={handleNodeSelect}
        onNodeExpand={handleNodeExpand}
        renderNode={({ node, expanded, selected }) => (
          <HierarchyNode
            graph={node.data}
            expanded={expanded}
            selected={selected}
            onCreateCopy={() => createGraphCopy(node.data.id)}
            onViewChanges={() => showChangeHistory(node.data.id)}
          />
        )}
      />
    </div>
  );
}
```

**Phase 3 Deliverables**:
- ✅ Complete React application with Mantine UI
- ✅ All 4 required visual editors (Plan, Graph, Spreadsheet, Hierarchy)
- ✅ Real-time CRDT collaboration
- ✅ Comprehensive component library
- ✅ Mobile-responsive design
- ✅ End-to-end test coverage

### **Phase 4: Desktop Application (4 months)**

#### **Goal**: Tauri-based desktop application with native integration

#### **4.1 Tauri Integration (Weeks 1-2)**

**Desktop Application Setup**:

```toml
# src-tauri/Cargo.toml
[package]
name = "layercake-desktop"
version = "1.0.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = [
  "app-impl",
  "resources",
  "shell-impl",
  "window-impl",
  "fs-impl",
  "dialog-impl"
] }
layercake = { path = "../../", features = ["server", "graphql", "mcp"] }
tokio = { version = "1.0", features = ["full"] }
```

```json
// src-tauri/tauri.conf.json
{
  "productName": "Layercake",
  "version": "1.0.0",
  "identifier": "com.layercake.app",
  "build": {
    "frontendDist": "../frontend/dist",
    "devUrl": "http://localhost:5173"
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "msi", "dmg", "appimage"],
    "windows": {
      "wix": {
        "language": "en-US"
      }
    },
    "macOS": {
      "entitlements": null,
      "exceptionDomain": "",
      "frameworks": [],
      "providerShortName": null,
      "signingIdentity": null
    }
  },
  "app": {
    "windows": [
      {
        "title": "Layercake",
        "width": 1400,
        "height": 900,
        "resizable": true,
        "fullscreen": false,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

#### **4.2 Native File Integration (Weeks 3-4)**

**Desktop-specific Features**:

```rust
// src-tauri/src/commands.rs
use tauri::command;

#[command]
pub async fn import_project_file(path: String) -> Result<String, String> {
    // Import .lcproj files natively
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    // Process project import
    Ok(content)
}

#[command]
pub async fn export_project_file(project_data: String, path: String) -> Result<(), String> {
    std::fs::write(path, project_data)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[command]
pub async fn get_system_info() -> SystemInfo {
    SystemInfo {
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

// Enhanced file dialogs
#[command]
pub async fn show_save_dialog(default_name: String) -> Result<Option<String>, String> {
    use tauri::api::dialog::FileDialogBuilder;

    let path = FileDialogBuilder::new()
        .set_default_name(&default_name)
        .add_filter("Layercake Project", &["lcproj"])
        .save_file();

    Ok(path)
}
```

**Phase 4 Deliverables**:
- ✅ Cross-platform desktop application (Windows, macOS, Linux)
- ✅ Native file associations (.lcproj files)
- ✅ OS integration (system tray, notifications)
- ✅ Offline mode with sync when online
- ✅ Auto-update functionality

### **Phase 5: Advanced Features & Polish (6 months)**

#### **Goal**: Production-ready application with advanced collaboration features

#### **5.1 Advanced CRDT Features (Weeks 1-4)**

**Conflict Resolution UI**:

```typescript
// src/components/collaboration/ConflictResolver.tsx
export function ConflictResolver({
  conflict,
  onResolve
}: {
  conflict: MergeConflict;
  onResolve: (resolution: Resolution) => void;
}) {
  const [selectedResolution, setSelectedResolution] = useState<ResolutionStrategy>('manual');

  return (
    <Modal opened={true} size="xl" title="Merge Conflict Detected">
      <Stack>
        <Alert color="orange" icon={<AlertTriangle />}>
          Multiple users edited the same graph element simultaneously
        </Alert>

        <Group>
          <Badge color="blue">{conflict.localUser.username}</Badge>
          <Text>vs</Text>
          <Badge color="green">{conflict.remoteUser.username}</Badge>
        </Group>

        <Tabs value={selectedResolution} onChange={setSelectedResolution}>
          <Tabs.List>
            <Tabs.Tab value="automatic">Auto-merge</Tabs.Tab>
            <Tabs.Tab value="manual">Manual</Tabs.Tab>
            <Tabs.Tab value="local">Keep Mine</Tabs.Tab>
            <Tabs.Tab value="remote">Keep Theirs</Tabs.Tab>
          </Tabs.List>

          <Tabs.Panel value="manual">
            <ConflictDiffViewer
              localChange={conflict.localChange}
              remoteChange={conflict.remoteChange}
              onManualMerge={(resolved) => onResolve({ type: 'manual', data: resolved })}
            />
          </Tabs.Panel>
        </Tabs>

        <Group justify="flex-end">
          <Button variant="outline" onClick={() => onResolve({ type: 'cancel' })}>
            Cancel
          </Button>
          <Button onClick={() => onResolve({ type: selectedResolution })}>
            Apply Resolution
          </Button>
        </Group>
      </Stack>
    </Modal>
  );
}
```

#### **5.2 Performance Optimization (Weeks 5-8)**

**Large Graph Handling**:

```typescript
// src/hooks/useVirtualizedGraph.ts
export function useVirtualizedGraph(graphId: number, viewport: ViewportBounds) {
  const { doc } = useCrdtSync(graphId);
  const [visibleNodes, setVisibleNodes] = useState<Node[]>([]);
  const [visibleEdges, setVisibleEdges] = useState<Edge[]>([]);

  // Spatial indexing for large graphs
  const spatialIndex = useMemo(() => {
    if (!doc) return null;
    return new RTree(); // R-tree for efficient spatial queries
  }, [doc]);

  useEffect(() => {
    if (!spatialIndex || !doc) return;

    // Query only visible nodes/edges within viewport
    const visible = spatialIndex.search(viewport);
    setVisibleNodes(visible.nodes);
    setVisibleEdges(visible.edges);
  }, [viewport, spatialIndex, doc]);

  return {
    visibleNodes,
    visibleEdges,
    totalNodeCount: doc?.getMap('nodes').size || 0
  };
}
```

#### **5.3 Advanced Export Features (Weeks 9-12)**

**Enhanced Export Pipeline**:

```typescript
// src/services/exportService.ts
export class ExportService {
  async exportWithCustomTemplate(
    graphId: number,
    templateId: string,
    options: ExportOptions
  ): Promise<ExportResult> {
    // Integration with existing Rust export system
    const response = await apolloClient.mutate({
      mutation: EXPORT_GRAPH_CUSTOM,
      variables: { graphId, templateId, options }
    });

    return response.data.exportGraph;
  }

  // Real-time export preview
  async getExportPreview(
    graphId: number,
    format: ExportFormat
  ): Promise<string> {
    // Live preview of export output
    const { data } = await apolloClient.query({
      query: GET_EXPORT_PREVIEW,
      variables: { graphId, format }
    });

    return data.exportPreview.content;
  }
}
```

#### **5.4 Testing & Quality Assurance (Weeks 13-16)**

**Comprehensive Test Suite**:

```typescript
// tests/e2e/collaboration.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Real-time Collaboration', () => {
  test('multiple users can edit same graph simultaneously', async ({ browser }) => {
    // Create two browser contexts (different users)
    const context1 = await browser.newContext();
    const context2 = await browser.newContext();

    const page1 = await context1.newPage();
    const page2 = await context2.newPage();

    // User 1 creates a node
    await page1.goto('/projects/1/graphs/1');
    await page1.click('[data-testid="add-node"]');
    await page1.fill('[data-testid="node-label"]', 'Node from User 1');

    // User 2 should see the node appear in real-time
    await page2.goto('/projects/1/graphs/1');
    await expect(page2.locator('[data-testid="node"]')).toContainText('Node from User 1');

    // Simulate conflict
    await page1.click('[data-testid="node-1"]');
    await page2.click('[data-testid="node-1"]');

    await page1.fill('[data-testid="node-label"]', 'Modified by User 1');
    await page2.fill('[data-testid="node-label"]', 'Modified by User 2');

    // Conflict resolution should appear
    await expect(page1.locator('[data-testid="conflict-dialog"]')).toBeVisible();
    await expect(page2.locator('[data-testid="conflict-dialog"]')).toBeVisible();
  });
});

// tests/performance/large-graphs.spec.ts
test.describe('Performance with Large Graphs', () => {
  test('handles 10,000 node graph smoothly', async ({ page }) => {
    await page.goto('/projects/1/graphs/large-test');

    // Graph should load within 5 seconds
    await expect(page.locator('[data-testid="graph-canvas"]')).toBeVisible({ timeout: 5000 });

    // Zoom operations should be smooth
    const canvas = page.locator('[data-testid="graph-canvas"]');
    await canvas.hover();
    await page.mouse.wheel(0, -500); // Zoom in

    // Should still be responsive
    await expect(canvas).toBeVisible();
  });
});
```

**Phase 5 Deliverables**:
- ✅ Advanced conflict resolution UI
- ✅ Performance optimization for large graphs (10K+ nodes)
- ✅ Enhanced export system with custom templates
- ✅ Comprehensive test coverage (>90%)
- ✅ Production deployment documentation
- ✅ User documentation and tutorials

## Risk Assessment & Mitigation

### 🔴 **High-Risk Areas**

#### **1. CRDT Library Performance**
**Risk**: Selected CRDT library may not scale to large graphs (>10K nodes)
**Mitigation**:
- Benchmark all candidates with realistic data sets
- Implement fallback to operational transforms if needed
- Consider hybrid approach (CRDT for metadata, OT for bulk data)

#### **2. Real-time Synchronization Complexity**
**Risk**: Network partitions and conflicts may cause data corruption
**Mitigation**:
- Implement comprehensive conflict resolution strategies
- Add offline-first capabilities with eventual consistency
- Regular automated backups with point-in-time recovery

#### **3. Frontend Complexity with ReactFlow**
**Risk**: Performance degradation with complex graph hierarchies
**Mitigation**:
- Implement virtualization for large graphs
- Progressive loading of graph sections
- Fallback to simpler visualization for very large graphs

### 🟡 **Medium-Risk Areas**

#### **4. Database Migration Complexity**
**Risk**: Migration from existing CLI data may fail or corrupt data
**Mitigation**:
- Comprehensive backup strategy before migration
- Phased migration with rollback points
- Parallel running of old and new systems during transition

#### **5. Cross-platform Desktop Compatibility**
**Risk**: Tauri application may have platform-specific issues
**Mitigation**:
- Early testing on all target platforms
- Platform-specific CI/CD pipelines
- Gradual rollout starting with single platform

### 🟢 **Low-Risk Areas**

#### **6. GraphQL API Implementation**
**Risk**: Low - well-understood technology with existing examples

#### **7. Export System Integration**
**Risk**: Low - existing Rust export system is robust and proven

## Technical Recommendations

### **1. CRDT Library Selection**

**Recommended**: [Loro](https://loro.dev) or [Yrs](https://github.com/y-crdt/y-crdt)

**Reasoning**:
- **Loro**: Modern, high-performance, designed for collaborative apps
- **Yrs**: Mature Rust port of Yjs, proven in production
- Both support rich JSON data types needed for graph structures
- Both have active development and good documentation

### **2. Database Strategy**

**Recommended**: Hybrid approach
- **SQLite**: Development and single-user deployments
- **PostgreSQL**: Multi-user production deployments
- **CRDT Changes**: Store in separate high-performance store (Redis/RocksDB)

### **3. Frontend Architecture**

**Recommended**:
- **State Management**: Zustand + CRDT integration
- **Graph Rendering**: ReactFlow with D3.js for complex layouts
- **Performance**: React.memo + virtualization for large datasets
- **Testing**: Playwright for E2E, Vitest for unit tests

### **4. Development Workflow**

**Recommended**:
- **Monorepo**: Cargo workspace + frontend in same repository
- **CI/CD**: GitHub Actions with platform-specific builds
- **Testing**: Automated testing at multiple levels (unit, integration, E2E)
- **Documentation**: Living documentation with code examples

## Success Metrics & Timeline

### **Realistic Timeline: 30-36 months**

| Phase | Duration | Team Size | Key Deliverable |
|-------|----------|-----------|-----------------|
| Phase 1 | 6 months | 2-3 backend devs | Complete data model + CRDT integration |
| Phase 2 | 6 months | 2-3 backend devs | Full backend API (GraphQL/MCP/WebSocket) |
| Phase 3 | 8 months | 3-4 frontend devs | Complete React application |
| Phase 4 | 4 months | 1-2 desktop devs | Tauri desktop application |
| Phase 5 | 6 months | 4-6 full team | Production polish + advanced features |

### **Success Criteria by Phase**

**Phase 1 Success Metrics**:
- ✅ CRDT library handles 10K+ node graphs with <100ms sync latency
- ✅ Database migration completes without data loss for 100% of test cases
- ✅ Core services achieve >80% test coverage
- ✅ Real-time sync works between 3+ concurrent instances

**Phase 2 Success Metrics**:
- ✅ GraphQL API supports all required operations with <200ms response times
- ✅ MCP API provides 20+ AI-ready tools and resources
- ✅ WebSocket handles 100+ concurrent connections
- ✅ API documentation is complete and validated

**Phase 3 Success Metrics**:
- ✅ All 4 visual editor components are fully functional
- ✅ Real-time collaboration works smoothly with 10+ concurrent users
- ✅ Graph rendering performs well with 5K+ nodes
- ✅ Mobile-responsive design works on tablets

**Phase 4 Success Metrics**:
- ✅ Desktop app launches on Windows, macOS, and Linux
- ✅ Native file associations work correctly
- ✅ Offline mode maintains full functionality
- ✅ Auto-update system works reliably

**Phase 5 Success Metrics**:
- ✅ Conflict resolution UI handles 95% of conflicts automatically
- ✅ Performance benchmarks meet or exceed targets
- ✅ Test coverage >90% across all components
- ✅ Production deployment documentation is complete

## Conclusion

This comprehensive revision provides a realistic, technically sound implementation plan for the layercake interactive application. The plan acknowledges the significant complexity of the requirements while providing specific technical approaches to address each challenge.

**Key Improvements over Original Plan**:
- ✅ **30-36 month realistic timeline** (vs optimistic 18-24 months)
- ✅ **Proper CRDT integration** addressing core specification requirement
- ✅ **Comprehensive frontend architecture** with specific component designs
- ✅ **Detailed risk assessment** with concrete mitigation strategies
- ✅ **Specific technology recommendations** based on proven solutions
- ✅ **Measurable success criteria** for each phase

This plan provides a solid foundation for building a production-ready layercake interactive application that meets all specification requirements while maintaining realistic expectations for timeline and resources.