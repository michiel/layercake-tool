# Layercake Implementation Plan V2 - Conservative Approach

## Executive Summary

This implementation plan provides a **realistic and technically feasible** roadmap for transforming layercake into an interactive graph editing platform with **Plan DAG architecture**. Based on comprehensive analysis of the existing codebase, this approach builds incrementally on the robust foundation already in place (~70% complete) while implementing the critical Plan DAG with LayercakeGraph objects.

**Key Strategic Decision**: Evolve from YAML plans to Plan DAG with LayercakeGraph objects, enabling graph projections via copy→transform→graph pipelines while maintaining backward compatibility.

### Current State Assessment

**✅ IMPLEMENTED + ENHANCED (Strong Foundation)**:
- **Unified GraphQL API**: Complete CRUD + Real-time subscriptions (REST API removed for simplicity)
- **MCP Integration**: Enhanced with 25+ tools, real-time events, workflow automation
- **Database Layer**: Enhanced SeaORM with hierarchies, plan DAGs, subscription publishing
- **Export System**: All formats (JSON, CSV, DOT, GML, PlantUML, Mermaid, Custom)
- **Plan Execution**: Trait-based execution system with enhanced error handling
- **CLI Interface**: Complete command-line functionality (unchanged)
- **Real-time Architecture**: GraphQL subscriptions with conflict detection and offline support

**📊 Enhanced Foundation Statistics**:
- **Database**: 7+ entities (projects, plan DAGs, layercake graphs, nodes, edges, layers, collaborations)
- **GraphQL API**: Unified API with queries, mutations, AND subscriptions
- **MCP Tools**: 25+ enhanced tools with real-time integration
- **Export Formats**: 8+ different output formats supported
- **CLI Commands**: 6 main commands (unchanged for backward compatibility)
- **Real-time Features**: GraphQL subscriptions, conflict resolution, offline support
- **✅ MAJOR ENHANCEMENT COMPLETED**: Plan DAG architecture with LayercakeGraph hierarchy

### Recommended Implementation Strategy

**Timeline**: 12-18 months (conservative estimate building on existing foundation)
**Risk Level**: LOW-MEDIUM (leveraging proven architecture)
**Team Size**: 2-3 developers

## Technical Architecture Overview

### System Design Principles

1. **Plan DAG Evolution**: Transform YAML plans to Plan DAG with LayercakeGraph objects as first-class entities
2. **Graph Projection Architecture**: Enable copy→transform→graph pipelines for graph variants
3. **Separation of Concerns**: Plan DAG contains metadata; Graph table contains actual nodes/edges
4. **Incremental Enhancement**: Build on existing 70% complete foundation
5. **Backward Compatibility**: Maintain YAML import/export during transition
6. **Performance First**: Optimize for large graphs (10K+ nodes) and complex DAGs

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend Applications                        │
├─────────────────┬─────────────────┬─────────────────┬───────────────┤
│  Tauri Desktop  │  Web Interface  │   Mobile PWA    │  CLI Interface│
│   (React)       │    (React)      │    (React)      │  (Existing)   │
└─────────────────┴─────────────────┴─────────────────┴───────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                      Visual Editor Layer                           │
├─────────────────┬─────────────────┬─────────────────────────────────┤
│  Plan Visual    │  Graph Visual   │    Graph Spreadsheet            │
│    Editor       │    Editor       │       Editor                    │
│ (ReactFlow)     │  (ReactFlow)    │   (Mantine Table)              │
│                 │                 │                                │
│ • YAML Plans    │ • Node/Edge     │ • Bulk Operations              │
│ • Drag & Drop   │   Editing       │ • Data Validation              │
│ • Real-time     │ • Layer Support │ • Import/Export                │
│   Preview       │ • Real-time     │ • Search/Filter                │
│                 │   Collaboration │                                │
└─────────────────┴─────────────────┴─────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                        API Gateway                                 │
├─────────────────┬─────────────────┬─────────────────┬───────────────┤
│   GraphQL API   │    REST API     │    MCP API      │   WebSocket   │
│ (Query/Mutation)│   (CRUD Ops)    │  (AI Tools)     │ (Real-time)   │
│                 │                 │                 │               │
│ ✅ COMPLETE     │ ✅ COMPLETE     │ ✅ ~95% DONE    │ 🚧 NEW        │
│ • Projects      │ • Full CRUD     │ • 14 Tools      │ • Live Updates│
│ • Plans         │ • OpenAPI       │ • Resources     │ • Multi-user  │
│ • Graph Data    │ • Health Check  │ • Prompts       │ • Presence    │
└─────────────────┴─────────────────┴─────────────────┴───────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                     Real-time Collaboration                        │
├─────────────────────────────────────────────────────────────────────┤
│                     Simple Operational Transform                   │
│  ┌─────────────────────────┐  ┌─────────────────────────────────┐    │
│  │   Operation Queue       │  │    Conflict Resolution         │    │
│  │  • Edit Operations      │  │  • Last-Writer-Wins            │    │
│  │  • User Presence        │  │  • Automatic Merge             │    │
│  │  • Change Broadcast     │  │  • Manual Conflict UI          │    │
│  └─────────────────────────┘  └─────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                      Business Logic Layer                          │
├─────────────────┬─────────────────┬─────────────────────────────────┤
│ Plan Execution  │  Graph Service  │    Import/Export Service       │
│                 │                 │                                │
│ ✅ COMPLETE     │ ✅ COMPLETE     │ ✅ COMPLETE                     │
│ • YAML Plans    │ • Graph CRUD    │ • CSV Import                   │
│ • Transformers  │ • Validation    │ • Multi-format Export          │
│ • Dependencies  │ • Constraints   │ • Template System              │
│ • Error Handle  │ • Lineage       │ • Custom Renderers             │
└─────────────────┴─────────────────┴─────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────────┐
│                         Data Layer                                 │
├─────────────────┬─────────────────┬─────────────────────────────────┤
│   SeaORM DB     │  Subscription   │      GraphQL Cache              │
│                 │    Publisher    │                                │
│ ✅ ENHANCED     │ ✅ NEW          │ ✅ NEW                          │
│ • Hierarchies   │ • Event Stream  │ • Apollo Cache                 │
│ • Plan DAGs     │ • Change Log    │ • Optimistic UI                │
│ • Validations   │ • Conflict Det  │ • Offline Support              │
│ • Migrations    │ • Pub/Sub       │ • Normalization                │
│ • Constraints   │ • Filtering     │ • Cache Policies               │
└─────────────────┴─────────────────┴─────────────────────────────────┘
```

## Database Schema Extensions

### Current Schema (Requires Evolution)

```sql
-- ✅ EXISTING TABLES
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

-- 🚧 EXISTING - NEEDS EVOLUTION TO PLAN DAG
CREATE TABLE plans (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    name TEXT NOT NULL,
    yaml_content TEXT NOT NULL,  -- ⚠️ EVOLVE TO plan_dag_json
    dependencies TEXT, -- JSON array of plan IDs
    status TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    layer_id TEXT,
    properties TEXT -- JSON
);

CREATE TABLE edges (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    edge_id TEXT NOT NULL,
    source_node_id TEXT NOT NULL,
    target_node_id TEXT NOT NULL,
    label TEXT,
    properties TEXT -- JSON
);

CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    layer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    properties TEXT -- JSON
);
```

## 🚨 CRITICAL: Plan DAG Architecture

### Plan DAG vs Current YAML System

**Current System (YAML Plans)**:
- Plans stored as YAML text in `plans.yaml_content`
- Direct execution of YAML transformation steps
- No graph object separation
- Limited reusability and composition

**Target System (Plan DAG)**:
- Plans stored as JSON DAG in `projects.plan_dag_json`
- LayercakeGraph objects as first-class DAG nodes
- Copy→Transform→Graph pipelines for projections
- Full separation: Plan DAG (metadata) + Graph table (data)

### Plan DAG Node Types

```rust
// Improved Plan DAG Node Architecture - Consistent and Clear Design
// All nodes share common fields: id, label, position, metadata
// Each node type has specific configuration in the config field

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDagNodeBase {
    pub id: String,
    pub label: String,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub ui_state: UIState, // Collapsed, expanded, selected, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanDagNode {
    /// InputNode: Loads data from external sources into the pipeline
    /// Produces: GraphData for downstream consumption
    Input {
        #[serde(flatten)]
        base: PlanDagNodeBase,
        config: InputNodeConfig,
    },

    /// StorageNode: Persists graph data as a LayercakeGraph entity
    /// Consumes: GraphData from upstream nodes
    /// Produces: Reference to persisted LayercakeGraph
    Storage {
        #[serde(flatten)]
        base: PlanDagNodeBase,
        config: StorageNodeConfig,
        layercake_graph_id: Option<i32>, // None until graph is persisted
    },

    /// ProcessNode: Applies operations to flowing data without persistence
    /// Consumes: GraphData from upstream nodes
    /// Produces: Modified GraphData for downstream consumption
    Process {
        #[serde(flatten)]
        base: PlanDagNodeBase,
        config: ProcessNodeConfig,
    },

    /// OutputNode: Exports data to external formats/destinations
    /// Consumes: GraphData or references to LayercakeGraph
    /// Produces: External files/data
    Output {
        #[serde(flatten)]
        base: PlanDagNodeBase,
        config: OutputNodeConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputNodeConfig {
    pub source_type: InputSourceType,
    pub validation_rules: Vec<ValidationRule>,
    pub import_options: ImportOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeConfig {
    pub graph_name: String,
    pub graph_type: LayercakeGraphType,
    pub parent_graph_id: Option<i32>,
    pub auto_persist: bool, // Whether to persist immediately or wait for explicit trigger
    pub retention_policy: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessNodeConfig {
    pub operation_type: ProcessOperationType,
    pub parameters: serde_json::Value, // Operation-specific parameters
    pub error_handling: ErrorHandlingStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputNodeConfig {
    pub export_format: ExportFormat,
    pub destination: ExportDestination,
    pub template_options: Option<TemplateOptions>,
    pub post_processing: Vec<PostProcessStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessOperationType {
    Copy {
        deep_copy: bool,
        filter_criteria: Option<FilterCriteria>,
    },
    Transform {
        transformations: Vec<TransformationRule>,
        preserve_original: bool,
    },
    Merge {
        merge_strategy: MergeStrategy,
        conflict_resolution: ConflictResolution,
    },
    Filter {
        criteria: FilterCriteria,
        include_edges: bool,
    },
    Aggregate {
        grouping_fields: Vec<String>,
        aggregation_functions: Vec<AggregationFunction>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSourceType {
    /// File-based data sources
    File {
        path: String,
        format: FileFormat,
        encoding: Option<String>,
    },
    /// HTTP/REST API endpoints
    RestApi {
        url: String,
        method: HttpMethod,
        headers: HashMap<String, String>,
        auth: Option<AuthConfig>,
        pagination: Option<PaginationConfig>,
    },
    /// Database connections
    Database {
        connection_string: String,
        query: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    /// Stream/message queue sources
    Stream {
        endpoint: String,
        protocol: StreamProtocol,
        subscription_config: StreamConfig,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileFormat {
    CsvNodes,
    CsvEdges,
    CsvLayers,
    Json,
    Xml,
    Yaml,
    Parquet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDag {
    pub nodes: Vec<PlanDagNode>,
    pub edges: Vec<PlanDagEdge>,
    pub metadata: PlanDagMetadata,
    pub execution_config: ExecutionConfig,
    pub validation_state: ValidationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDagMetadata {
    pub version: String,
    pub schema_version: u32, // For backward compatibility
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub execution_statistics: Option<ExecutionStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub max_parallel_nodes: usize,
    pub timeout_per_node: Duration,
    pub retry_policy: RetryPolicy,
    pub error_handling_strategy: GlobalErrorHandlingStrategy,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationState {
    pub is_valid: bool,
    pub last_validated: DateTime<Utc>,
    pub validation_errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub dependency_cycles: Vec<DependencyCycle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,  // Source node ID
    pub target: String,  // Target node ID
    pub connection_type: ConnectionType,
    pub metadata: EdgeMetadata,
    pub validation_rules: Vec<ConnectionValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub label: Option<String>,
    pub description: Option<String>,
    pub data_schema: Option<DataSchema>, // Expected data structure flowing through
    pub performance_hints: PerformanceHints,
}

/// ConnectionType defines the nature of data flow between nodes
/// Each type has specific validation rules and execution semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// StreamingData: Real-time data flow from producer to consumer
    /// Used for: Input → Process, Process → Process, Process → Storage
    /// Data: GraphData flows through the connection
    StreamingData {
        data_type: DataType,
        batch_size: Option<usize>,
        buffer_config: BufferConfig,
    },

    /// StorageReference: Reference to persisted LayercakeGraph
    /// Used for: Storage → Process, Storage → Output
    /// Data: Graph ID reference, actual data loaded on demand
    StorageReference {
        lazy_loading: bool,
        cache_policy: CachePolicy,
    },

    /// ControlFlow: Execution dependency without data transfer
    /// Used for: Ensuring execution order without data flow
    /// Data: No data transfer, pure dependency
    ControlFlow {
        wait_for_completion: bool,
        timeout: Option<Duration>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    GraphData,      // Complete graph with nodes, edges, layers
    NodeSet,        // Subset of nodes
    EdgeSet,        // Subset of edges
    LayerData,      // Layer definitions
    Metadata,       // Graph metadata only
    ValidationResult, // Validation/analysis results
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionValidationRule {
    pub rule_type: ValidationRuleType,
    pub error_action: ValidationErrorAction,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRuleType {
    RequiredDataType(DataType),
    MaxDataSize(usize),
    SchemaCompatibility(String),
    NodeTypeCompatibility { source_type: String, target_type: String },
}
```

### LayercakeGraph Objects in Plan DAG

```rust
// LayercakeGraph represents graph instances within Plan DAG
// Enhanced with clear hierarchy constraints and validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayercakeGraph {
    pub id: i32,
    pub project_id: i32,
    pub graph_name: String,

    // HIERARCHY CONSTRAINTS:
    // - parent_graph_id: Must reference an existing LayercakeGraph in same project
    // - generation: Auto-calculated as parent.generation + 1
    // - Max hierarchy depth: 10 levels (configurable)
    // - Circular references: Prevented by validation
    // - Deletion cascade: Children deleted when parent is deleted
    pub parent_graph_id: Option<i32>,
    pub generation: u8,  // Changed from i32 to u8, max 255 levels

    // GRAPH TYPE CONSTRAINTS:
    // - Root: parent_graph_id must be None, generation must be 0
    // - Copy: Must have parent, preserves all data from parent at creation time
    // - Transform: Must have parent, applies transformations to parent data
    // - Merge: Must reference multiple sources in metadata.source_graph_ids
    // - Scenario: User-created projection, can have any parent
    pub graph_type: LayercakeGraphType,

    pub metadata: LayercakeGraphMetadata,
    pub validation_state: GraphValidationState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // COMPUTED FIELDS (not stored, calculated on load):
    // - child_count: Number of direct children
    // - descendant_count: Total descendants in hierarchy
    // - data_size: Approximate size of graph data
    // - last_modified_descendant: Most recent update in hierarchy
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayercakeGraphMetadata {
    pub description: Option<String>,
    pub tags: Vec<String>,

    // LINEAGE TRACKING:
    // - For Copy: records exact parent state at copy time
    // - For Transform: records transformation rules applied
    // - For Merge: records all source graphs and merge strategy
    pub lineage: GraphLineage,

    // HIERARCHY NAVIGATION:
    pub hierarchy_path: Vec<String>, // Path from root: ["root", "level1", "level2"]
    pub sibling_names: Vec<String>,  // Names of graphs at same hierarchy level

    // STATISTICS (cached for performance):
    pub node_count: usize,
    pub edge_count: usize,
    pub layer_count: usize,
    pub last_statistics_update: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphLineage {
    Root {
        import_source: ImportSource,
        import_timestamp: DateTime<Utc>,
    },
    Copy {
        source_graph_id: i32,
        source_snapshot_hash: String, // Hash of source data at copy time
        copy_timestamp: DateTime<Utc>,
    },
    Transform {
        source_graph_id: i32,
        transformation_rules: Vec<TransformationRule>,
        transform_timestamp: DateTime<Utc>,
    },
    Merge {
        source_graph_ids: Vec<i32>,
        merge_strategy: MergeStrategy,
        merge_timestamp: DateTime<Utc>,
    },
    Scenario {
        base_graph_id: i32,
        scenario_changes: Vec<ScenarioChange>,
        created_by: String,
        scenario_timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphValidationState {
    pub is_valid: bool,
    pub last_validated: DateTime<Utc>,
    pub hierarchy_errors: Vec<HierarchyValidationError>,
    pub data_integrity_errors: Vec<DataIntegrityError>,
    pub constraint_violations: Vec<ConstraintViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyValidationError {
    CircularReference { cycle_path: Vec<i32> },
    InvalidParentReference { parent_id: i32, reason: String },
    ExceededMaxDepth { current_depth: u8, max_depth: u8 },
    OrphanedGraph { missing_parent_id: i32 },
    GenerationMismatch { expected: u8, actual: u8 },
}

// HIERARCHY MANAGEMENT HELPER METHODS:
impl LayercakeGraph {
    /// Validates hierarchy constraints before creation/update
    pub fn validate_hierarchy_constraints(
        &self,
        existing_graphs: &[LayercakeGraph]
    ) -> Result<(), Vec<HierarchyValidationError>> {
        let mut errors = Vec::new();

        // Check circular references
        if let Some(cycle) = self.detect_circular_reference(existing_graphs) {
            errors.push(HierarchyValidationError::CircularReference { cycle_path: cycle });
        }

        // Validate parent reference
        if let Some(parent_id) = self.parent_graph_id {
            if !existing_graphs.iter().any(|g| g.id == parent_id) {
                errors.push(HierarchyValidationError::InvalidParentReference {
                    parent_id,
                    reason: "Parent graph does not exist".to_string(),
                });
            }
        }

        // Check max depth
        if self.generation > MAX_HIERARCHY_DEPTH {
            errors.push(HierarchyValidationError::ExceededMaxDepth {
                current_depth: self.generation,
                max_depth: MAX_HIERARCHY_DEPTH,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Gets all ancestors up to root
    pub fn get_ancestry_chain(&self, all_graphs: &[LayercakeGraph]) -> Vec<LayercakeGraph> {
        let mut chain = Vec::new();
        let mut current_id = self.parent_graph_id;

        while let Some(parent_id) = current_id {
            if let Some(parent) = all_graphs.iter().find(|g| g.id == parent_id) {
                chain.push(parent.clone());
                current_id = parent.parent_graph_id;
            } else {
                break; // Invalid hierarchy, stop traversal
            }
        }

        chain.reverse(); // Root first
        chain
    }

    /// Gets all direct children
    pub fn get_children(&self, all_graphs: &[LayercakeGraph]) -> Vec<LayercakeGraph> {
        all_graphs
            .iter()
            .filter(|g| g.parent_graph_id == Some(self.id))
            .cloned()
            .collect()
    }
}

const MAX_HIERARCHY_DEPTH: u8 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayercakeGraphType {
    /// Root: Original imported graph, no parent allowed
    /// Constraints: parent_graph_id = None, generation = 0
    Root,

    /// Copy: Exact duplicate of parent graph at specific point in time
    /// Constraints: Must have parent, preserves parent structure exactly
    Copy,

    /// Transform: Result of applying transformations to parent graph
    /// Constraints: Must have parent, transformation rules stored in lineage
    Transform,

    /// Merge: Result of combining multiple source graphs
    /// Constraints: Multiple sources referenced in lineage.source_graph_ids
    Merge,

    /// Scenario: User-created projection or "what-if" analysis
    /// Constraints: Can have any parent, changes tracked in lineage
    Scenario,
}

// Example Plan DAG workflows with clear hierarchy:

// 1. Simple Copy with Transform:
// Input(CSV) → Storage("Base", Root, gen=0) → Process(Copy) → Process(Transform) → Storage("Transformed", Transform, gen=1, parent=Base)

// 2. Multiple Independent Scenarios:
//                                         ┌→ Process(Copy) → Process(Transform) → Storage("Scenario A", Scenario, gen=1, parent=Base)
// Input(CSV) → Storage("Base", Root, gen=0) ─┤
//                                         └→ Process(Copy) → Storage("Exact Copy", Copy, gen=1, parent=Base)

// 3. Complex Multi-Generation Pipeline:
// Input(CSV) → Storage("Raw", Root, gen=0) → Process(Copy) → Process(Transform) → Storage("Processed", Transform, gen=1, parent=Raw)
//                     ↓                                                                ↓
//              Process(Filter) → Storage("Filtered", Transform, gen=1, parent=Raw)   Process(Aggregate) → Storage("Summary", Transform, gen=2, parent=Processed)

// 4. Merge Workflow:
// Storage("Dataset A", Root, gen=0) ─┐
//                                    ├→ Process(Merge) → Storage("Combined", Merge, gen=1, parents=[A,B])
// Storage("Dataset B", Root, gen=0) ─┘
```

### Database Schema Evolution with Migration Strategy

```sql
-- 🚧 PHASE 1: BACKWARD-COMPATIBLE ADDITIONS (No breaking changes)

-- Add new columns with default values to existing tables
ALTER TABLE projects ADD COLUMN plan_dag_json TEXT DEFAULT NULL;
ALTER TABLE projects ADD COLUMN schema_version INTEGER DEFAULT 1;
ALTER TABLE projects ADD COLUMN migration_state TEXT DEFAULT 'legacy'; -- 'legacy', 'migrating', 'migrated'

-- Create new LayercakeGraph table with hierarchy constraints
CREATE TABLE layercake_graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    graph_name TEXT NOT NULL,

    -- HIERARCHY CONSTRAINTS with validation
    parent_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE,
    generation INTEGER NOT NULL DEFAULT 0 CHECK (generation >= 0 AND generation <= 10),

    -- TYPE CONSTRAINTS
    graph_type TEXT NOT NULL CHECK (graph_type IN ('root', 'copy', 'transform', 'merge', 'scenario')),

    -- METADATA
    metadata TEXT NOT NULL DEFAULT '{}', -- JSON LayercakeGraphMetadata
    validation_state TEXT NOT NULL DEFAULT '{}', -- JSON GraphValidationState

    -- TIMESTAMPS
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- CONSTRAINTS
    UNIQUE(project_id, graph_name), -- Unique names within project

    -- ROOT TYPE CONSTRAINTS
    CHECK (
        (graph_type = 'root' AND parent_graph_id IS NULL AND generation = 0) OR
        (graph_type != 'root' AND parent_graph_id IS NOT NULL AND generation > 0)
    )
);

-- Create indexes for hierarchy queries
CREATE INDEX idx_layercake_graphs_parent ON layercake_graphs(parent_graph_id);
CREATE INDEX idx_layercake_graphs_project_generation ON layercake_graphs(project_id, generation);
CREATE INDEX idx_layercake_graphs_type ON layercake_graphs(graph_type);

-- 🚧 PHASE 2: ADD FOREIGN KEYS TO EXISTING TABLES (Gradual migration)

-- Add new columns to existing tables (nullable during migration)
ALTER TABLE nodes ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE;
ALTER TABLE edges ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE;
ALTER TABLE layers ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE;

-- Create indexes for performance
CREATE INDEX idx_nodes_layercake_graph ON nodes(layercake_graph_id);
CREATE INDEX idx_edges_layercake_graph ON edges(layercake_graph_id);
CREATE INDEX idx_layers_layercake_graph ON layers(layercake_graph_id);

-- 🚧 PHASE 3: OPERATION TRACKING

CREATE TABLE graph_edit_operations (
    id INTEGER PRIMARY KEY,
    layercake_graph_id INTEGER NOT NULL REFERENCES layercake_graphs(id) ON DELETE CASCADE,
    operation_type TEXT NOT NULL CHECK (operation_type IN (
        'create_node', 'update_node', 'delete_node',
        'create_edge', 'update_edge', 'delete_edge',
        'create_layer', 'update_layer', 'delete_layer',
        'bulk_operation', 'transform_operation'
    )),
    operation_data TEXT NOT NULL, -- JSON operation details
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_reproducible BOOLEAN NOT NULL DEFAULT TRUE,
    user_session_id TEXT,
    batch_id TEXT, -- For grouping related operations

    -- Performance optimization
    FOREIGN KEY (layercake_graph_id) REFERENCES layercake_graphs(id)
);

CREATE INDEX idx_graph_operations_graph ON graph_edit_operations(layercake_graph_id, applied_at);
CREATE INDEX idx_graph_operations_batch ON graph_edit_operations(batch_id);

-- 🚧 MIGRATION PROCEDURES

-- Trigger to automatically update updated_at timestamp
CREATE TRIGGER update_layercake_graphs_timestamp
    BEFORE UPDATE ON layercake_graphs
    FOR EACH ROW
    EXECUTE FUNCTION update_timestamp();

-- Function to validate hierarchy constraints
CREATE OR REPLACE FUNCTION validate_graph_hierarchy()
RETURNS TRIGGER AS $$
BEGIN
    -- Prevent circular references
    IF NEW.parent_graph_id IS NOT NULL THEN
        WITH RECURSIVE hierarchy AS (
            SELECT id, parent_graph_id, 1 as depth
            FROM layercake_graphs
            WHERE id = NEW.parent_graph_id

            UNION ALL

            SELECT lg.id, lg.parent_graph_id, h.depth + 1
            FROM layercake_graphs lg
            JOIN hierarchy h ON lg.id = h.parent_graph_id
            WHERE h.depth < 20 -- Prevent infinite loops
        )
        SELECT 1 FROM hierarchy WHERE id = NEW.id;

        IF FOUND THEN
            RAISE EXCEPTION 'Circular reference detected in graph hierarchy';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_graph_hierarchy
    BEFORE INSERT OR UPDATE ON layercake_graphs
    FOR EACH ROW
    EXECUTE FUNCTION validate_graph_hierarchy();

-- 🚧 SCHEMA VERSIONING STRATEGY

CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    rollback_sql TEXT, -- SQL to rollback this migration
    is_breaking BOOLEAN NOT NULL DEFAULT FALSE
);

-- Insert initial migration records
INSERT INTO schema_migrations (version, description, is_breaking) VALUES
(1, 'Initial layercake schema', FALSE),
(2, 'Add Plan DAG support to projects', FALSE),
(3, 'Add LayercakeGraph hierarchy system', FALSE),
(4, 'Add operation tracking', FALSE);

-- Automated backup and restore procedures

-- Function to create schema backup
CREATE OR REPLACE FUNCTION create_schema_backup(backup_name TEXT)
RETURNS TEXT AS $$
DECLARE
    backup_path TEXT;
    tables_to_backup TEXT[];
BEGIN
    backup_path := '/tmp/layercake_backup_' || backup_name || '_' || to_char(now(), 'YYYY_MM_DD_HH24_MI_SS') || '.sql';

    -- List all layercake tables
    tables_to_backup := ARRAY[
        'projects', 'plans', 'nodes', 'edges', 'layers',
        'layercake_graphs', 'graph_edit_operations',
        'user_sessions', 'collaboration_operations',
        'schema_migrations', 'migration_execution_log'
    ];

    -- Create backup using pg_dump (this would be called from application)
    -- pg_dump --data-only --table=projects --table=plans ... > backup_path

    -- Log backup creation
    INSERT INTO migration_execution_log (
        migration_version, step_number, step_description,
        sql_executed, execution_time_ms, success
    ) VALUES (
        0, 0, 'Schema backup created: ' || backup_name,
        'pg_dump to ' || backup_path, 0, TRUE
    );

    RETURN backup_path;
END;
$$ LANGUAGE plpgsql;

-- View for monitoring migration status
CREATE VIEW migration_status AS
SELECT
    m.version,
    m.description,
    m.status,
    m.is_breaking,
    m.applied_at,
    m.min_app_version,
    CASE
        WHEN m.status = 'completed' THEN 'Ready'
        WHEN m.status = 'pending' AND check_migration_prerequisites(m.version) THEN 'Can Apply'
        WHEN m.status = 'pending' THEN 'Waiting for Dependencies'
        ELSE m.status
    END as readiness_status,
    (
        SELECT COUNT(*)
        FROM migration_execution_log mel
        WHERE mel.migration_version = m.version AND mel.success = FALSE
    ) as error_count
FROM schema_migrations m
ORDER BY m.version;

-- Function to get current effective schema version
CREATE OR REPLACE FUNCTION get_effective_schema_version()
RETURNS INTEGER AS $$
BEGIN
    RETURN COALESCE(
        (SELECT MAX(version) FROM schema_migrations WHERE status = 'completed'),
        0
    );
END;
$$ LANGUAGE plpgsql;
```

### Phase 1 Extensions (Plan DAG Migration) - UPDATED

```sql
-- 🚧 PROJECT HIERARCHY (Separate from LayercakeGraph hierarchy)
-- Projects can contain multiple LayercakeGraphs, creating a two-level hierarchy:
-- Project Hierarchy: For organizational/access control
-- Graph Hierarchy: For data lineage and transformations

ALTER TABLE projects ADD COLUMN parent_project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE;
ALTER TABLE projects ADD COLUMN hierarchy_level INTEGER DEFAULT 0 CHECK (hierarchy_level >= 0 AND hierarchy_level <= 5);
ALTER TABLE projects ADD COLUMN is_scenario BOOLEAN DEFAULT FALSE;
ALTER TABLE projects ADD COLUMN access_level TEXT DEFAULT 'private' CHECK (access_level IN ('private', 'shared', 'public'));

-- Prevent project hierarchy cycles
CREATE OR REPLACE FUNCTION validate_project_hierarchy()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.parent_project_id IS NOT NULL THEN
        WITH RECURSIVE project_hierarchy AS (
            SELECT id, parent_project_id, 1 as depth
            FROM projects
            WHERE id = NEW.parent_project_id

            UNION ALL

            SELECT p.id, p.parent_project_id, ph.depth + 1
            FROM projects p
            JOIN project_hierarchy ph ON p.id = ph.parent_project_id
            WHERE ph.depth < 10
        )
        SELECT 1 FROM project_hierarchy WHERE id = NEW.id;

        IF FOUND THEN
            RAISE EXCEPTION 'Circular reference detected in project hierarchy';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_project_hierarchy
    BEFORE INSERT OR UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION validate_project_hierarchy();

-- 🚧 ENHANCED COLLABORATION TABLES
CREATE TABLE user_sessions (
    id INTEGER PRIMARY KEY,
    session_id TEXT UNIQUE NOT NULL,
    user_name TEXT NOT NULL,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    layercake_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE, -- Current graph being edited

    -- Session state
    last_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cursor_position TEXT DEFAULT '{}', -- JSON cursor/selection state
    viewport_state TEXT DEFAULT '{}',  -- JSON zoom/pan state

    -- Session metadata
    user_agent TEXT,
    ip_address INET,
    session_duration INTERVAL,

    UNIQUE(session_id, project_id)
);

CREATE TABLE collaboration_operations (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES user_sessions(session_id) ON DELETE CASCADE,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    layercake_graph_id INTEGER REFERENCES layercake_graphs(id) ON DELETE CASCADE,

    -- Operation details
    operation_type TEXT NOT NULL CHECK (operation_type IN (
        'create', 'update', 'delete', 'move', 'copy', 'transform', 'bulk'
    )),
    entity_type TEXT NOT NULL CHECK (entity_type IN (
        'node', 'edge', 'layer', 'plan_dag', 'graph', 'project'
    )),
    entity_id TEXT NOT NULL,

    -- Operation data and conflict resolution
    operation_data TEXT NOT NULL, -- JSON operation details
    conflict_resolution_data TEXT, -- JSON conflict resolution info

    -- Timestamps and status
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    applied BOOLEAN NOT NULL DEFAULT FALSE,
    applied_at TIMESTAMP,

    -- Operational transform data
    operation_vector_clock TEXT, -- JSON vector clock for ordering
    causality_dependencies TEXT, -- JSON array of operation IDs this depends on

    -- Performance indexes
    INDEX idx_collab_ops_session (session_id, timestamp),
    INDEX idx_collab_ops_entity (entity_type, entity_id),
    INDEX idx_collab_ops_graph (layercake_graph_id, timestamp)
);

-- Conflict detection and resolution table
CREATE TABLE operation_conflicts (
    id INTEGER PRIMARY KEY,
    operation_a_id INTEGER NOT NULL REFERENCES collaboration_operations(id) ON DELETE CASCADE,
    operation_b_id INTEGER NOT NULL REFERENCES collaboration_operations(id) ON DELETE CASCADE,
    conflict_type TEXT NOT NULL CHECK (conflict_type IN (
        'concurrent_edit', 'delete_vs_update', 'type_mismatch', 'constraint_violation'
    )),
    resolution_strategy TEXT CHECK (resolution_strategy IN (
        'last_writer_wins', 'merge_changes', 'manual_resolution', 'reject_operation'
    )),
    resolution_data TEXT, -- JSON resolution details
    resolved_at TIMESTAMP,
    resolved_by TEXT, -- User who resolved the conflict

    UNIQUE(operation_a_id, operation_b_id)
);
```

## Revised Implementation Phases (Updated for Maintainability)

### Design Changes for Better Maintainability:

1. **Separation of Concerns**: Plan DAG execution separated from data models
2. **Trait-Based Architecture**: Execution strategies implemented as pluggable traits
3. **Versioned Schema**: Explicit migration strategy with rollback support
4. **Clear Error Boundaries**: Each component has defined error handling
5. **Simplified State Management**: Reduced complex state passing between operations

## Implementation Phases

### Phase 1: Frontend Foundation (Months 1-4)

#### Month 1-2: Tauri Desktop Application

**React Application Setup**
```typescript
// Unified Frontend Technology Stack - GraphQL + MCP Only
React 18+ with TypeScript
Mantine UI v7 (component library)
Apollo Client v3 (GraphQL with Subscriptions)
ReactFlow v11 (visual editing)
// Zustand REMOVED - Apollo Cache is single source of truth
Vite (build tool)
Tauri v2 (desktop wrapper)

// Key Architecture Decisions:
// 1. Apollo Client manages ALL state (local + remote)
// 2. GraphQL Subscriptions for real-time updates
// 3. No REST API - GraphQL handles all CRUD operations
// 4. MCP integration via GraphQL mutations
// 5. Offline support via Apollo Cache persistence
```

**Unified Project Structure - Apollo-Centric**
```
frontend/
├── src/
│   ├── components/
│   │   ├── editors/
│   │   │   ├── PlanVisualEditor/     # ReactFlow plan editor
│   │   │   ├── GraphVisualEditor/    # ReactFlow graph editor
│   │   │   └── GraphSpreadsheetEditor/ # Mantine table editor
│   │   ├── project/
│   │   │   ├── ProjectList/          # Project management
│   │   │   ├── ProjectCreate/        # Project creation
│   │   │   └── ProjectSettings/      # Project configuration
│   │   ├── common/
│   │   │   ├── Layout/               # App layout
│   │   │   ├── Navigation/           # Navigation components
│   │   │   ├── Loading/              # Loading states
│   │   │   ├── ErrorBoundary/        # GraphQL error handling
│   │   │   └── ConnectionStatus/     # Connection state UI
│   │   └── collaboration/
│   │       ├── UserPresence/         # Online users display
│   │       ├── ConflictResolver/     # Conflict resolution UI
│   │       └── OperationQueue/       # Offline operation queue
│   ├── graphql/                      # UNIFIED GraphQL layer
│   │   ├── client.ts                 # Apollo Client configuration
│   │   ├── queries/                  # Query definitions
│   │   ├── mutations/                # Mutation definitions
│   │   ├── subscriptions/            # Real-time subscriptions
│   │   ├── fragments/                # Reusable fragments
│   │   ├── cache/                    # Cache configuration
│   │   ├── policies/                 # Type policies
│   │   └── links/                    # Apollo Link chain
│   ├── hooks/                        # Unified React hooks
│   │   ├── useConnection/            # Connection management
│   │   ├── useOptimisticUpdates/     # Optimistic UI
│   │   ├── useCollaboration/         # Real-time collaboration
│   │   ├── useErrorHandling/         # Error boundary hooks
│   │   └── useOfflineQueue/          # Offline operation queue
│   ├── types/
│   │   ├── graphql.ts                # Generated GraphQL types
│   │   ├── apollo.ts                 # Apollo-specific types
│   │   └── collaboration.ts          # Collaboration types
│   └── utils/
│       ├── validation/               # Data validation
│       ├── transforms/               # Data transformations
│       ├── cache-utils/              # Apollo cache utilities
│       └── mcp-integration/          # MCP tool integration

# REMOVED:
# - stores/ directory (Zustand replaced by Apollo Cache)
# - useWebSocket/ hooks (replaced by GraphQL subscriptions)
# - export/ utilities (handled by GraphQL mutations)
```

**Key Deliverables**
- ✅ Tauri desktop application with React frontend
- ✅ Apollo GraphQL client connected to existing backend
- ✅ Project management interface (list, create, delete)
- ✅ Basic navigation and layout structure
- ✅ Connection to existing REST and GraphQL APIs

**Success Criteria**
- Desktop app builds and runs on all platforms
- Can list, create, and delete projects via existing APIs
- GraphQL queries/mutations work correctly
- No breaking changes to existing backend

#### Month 3-4: Plan DAG Visual Editor

**ReactFlow Plan DAG Editor Implementation**
```typescript
// Plan DAG Visual Editor Component Architecture
interface PlanDagNode {
  id: string;
  type: 'input' | 'graph' | 'copy' | 'transform' | 'merge' | 'output';
  position: { x: number; y: number };
  data: {
    label: string;
    config: PlanDagNodeConfig;
    status: 'pending' | 'running' | 'completed' | 'error';
    // For GraphNodes: reference to LayercakeGraph
    layercakeGraphId?: number;
    graphMetadata?: GraphMetadata;
  };
}

interface PlanDagEdge {
  id: string;
  source: string;
  target: string;
  type: 'dataflow' | 'dependency' | 'transformation';
}

// Custom Node Types for Plan DAG
const planDagNodeTypes = {
  inputNode: InputNodeComponent,        // CSV files, REST endpoints, SQL
  graphNode: GraphNodeComponent,        // LayercakeGraph references
  copyNode: CopyNodeComponent,          // Graph copying operations
  transformNode: TransformNodeComponent, // Graph transformations
  mergeNode: MergeNodeComponent,        // Graph merging operations
  outputNode: OutputNodeComponent,      // Export formats
};

// Improved GraphNode component - shows persistence state
const GraphNodeComponent: React.FC<NodeProps> = ({ data, id }) => {
  const { layercakeGraphId, graphMetadata, label } = data;
  const [graphStats, setGraphStats] = useState<GraphStats | null>(null);
  const isPersisted = layercakeGraphId !== null;

  // Load graph statistics if persisted
  useEffect(() => {
    if (layercakeGraphId) {
      loadGraphStats(layercakeGraphId).then(setGraphStats);
    }
  }, [layercakeGraphId]);

  const handleEditGraph = () => {
    if (layercakeGraphId) {
      openGraphEditor(layercakeGraphId);
    }
  };

  return (
    <div className={`plan-dag-graph-node ${isPersisted ? 'persisted' : 'pending'}`}>
      <Handle type="target" position={Position.Top} />

      <div className="node-header">
        <DatabaseIcon size={16} />
        <span className="node-type">LayercakeGraph</span>
        {isPersisted && <CheckIcon size={12} className="persisted-icon" />}
      </div>

      <div className="node-content">
        <h4>{label}</h4>
        {isPersisted ? (
          <div className="graph-stats">
            <span>ID: {layercakeGraphId}</span>
            {graphStats && (
              <>
                <span>{graphStats.nodeCount} nodes</span>
                <span>{graphStats.edgeCount} edges</span>
                <span>{graphStats.layerCount} layers</span>
              </>
            )}
            <Button size="xs" onClick={handleEditGraph}>
              Edit Graph
            </Button>
          </div>
        ) : (
          <div className="pending-state">
            <span>Will be created when pipeline executes</span>
          </div>
        )}
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

// CopyNode component - shows operation
const CopyNodeComponent: React.FC<NodeProps> = ({ data }) => {
  return (
    <div className="copy-node operation-node">
      <Handle type="target" position={Position.Top} />

      <div className="node-header">
        <CopyIcon size={16} />
        <span>Copy Operation</span>
      </div>

      <div className="node-content">
        <h4>{data.label}</h4>
        <p>Creates copy of input graph data</p>
        <small>Data flows through to next node</small>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

// TransformNode component - shows operation
// NOTE (2025-02): current implementation uses `data.config.transforms` to
// render summaries; this legacy snippet references the older `transform_config`.
const TransformNodeComponent: React.FC<NodeProps> = ({ data }) => {
  const { transform_config } = data;

  return (
    <div className="transform-node operation-node">
      <Handle type="target" position={Position.Top} />

      <div className="node-header">
        <TransformIcon size={16} />
        <span>Transform Operation</span>
      </div>

      <div className="node-content">
        <h4>{data.label}</h4>
        <p>Applies transformations to flowing data</p>
        {transform_config && (
          <div className="transform-summary">
            <small>{Object.keys(transform_config).join(', ')}</small>
          </div>
        )}
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

// Improved Copy→Transform→Graph Pipeline Creation
const createCopyTransformPipeline = (
  sourceGraphNode: PlanDagNode,
  transformConfig: TransformConfig,
  targetGraphName: string
) => {
  // 1. Create CopyNode (operation, no persistence)
  const copyNode: PlanDagNode = {
    id: generateId(),
    type: 'copy',
    position: { x: sourceGraphNode.position.x + 200, y: sourceGraphNode.position.y },
    data: {
      label: 'Copy Operation',
      copy_config: {},
      status: 'pending',
    },
  };

// 2. Create TransformNode (operation, no persistence)
//    (modern flow writes `transforms: GraphTransform[]` instead of a single config blob)
const transformNode: PlanDagNode = {
    id: generateId(),
    type: 'transform',
    position: { x: copyNode.position.x + 200, y: copyNode.position.y },
    data: {
      label: 'Transform Operation',
      transform_config: transformConfig,
      status: 'pending',
    },
  };

  // 3. Create target GraphNode (will persist result)
  const targetGraphNode: PlanDagNode = {
    id: generateId(),
    type: 'graph',
    position: { x: transformNode.position.x + 200, y: transformNode.position.y },
    data: {
      label: targetGraphName,
      layercakeGraphId: null, // Will be created during execution
      graphMetadata: {
        graphType: 'transform',
        sourceGraphId: sourceGraphNode.data.layercakeGraphId,
      },
      status: 'pending',
    },
  };

  // 4. Create connections
  const edges: PlanDagEdge[] = [
    {
      id: generateId(),
      source: sourceGraphNode.id,
      target: copyNode.id,
      type: 'dataflow',
    },
    {
      id: generateId(),
      source: copyNode.id,
      target: transformNode.id,
      type: 'dataflow',
    },
    {
      id: generateId(),
      source: transformNode.id,
      target: targetGraphNode.id,
      type: 'dataflow',
    },
  ];

  return { nodes: [copyNode, transformNode, targetGraphNode], edges };
};

// Multiple Independent Copies Handler
const handleMultipleCopies = () => {
  // Allow multiple outbound connections from GraphNode
  const onConnect = (connection: Connection) => {
    const sourceNode = getNode(connection.source);
    const targetNode = getNode(connection.target);

    if (sourceNode?.type === 'graphNode' && targetNode?.type === 'copyNode') {
      // Allow multiple copy operations from same graph
      addEdge({
        id: generateEdgeId(),
        source: connection.source,
        target: connection.target,
        type: 'dataflow',
        label: 'Graph Data',
      });
    }
  };

  // Visual feedback for multiple connections
  const onConnectStart = (event: any, { nodeId, handleType }: any) => {
    if (handleType === 'source' && getNodeType(nodeId) === 'graphNode') {
      showConnectionHint('Can create multiple copies from this graph');
    }
  };
};
```

**Plan DAG JSON Integration**
```typescript
// Real-time Plan DAG JSON synchronization
const usePlanDagSync = (projectId: string) => {
  const [planDagJson, setPlanDagJson] = useState<string>('');
  const [flowElements, setFlowElements] = useState<(PlanDagNode | PlanDagEdge)[]>([]);
  const [layercakeGraphs, setLayercakeGraphs] = useState<LayercakeGraph[]>([]);

  // Convert ReactFlow elements to Plan DAG JSON
  const elementsToPlanDag = useCallback((elements: (PlanDagNode | PlanDagEdge)[]) => {
    const nodes = elements.filter(el => 'type' in el) as PlanDagNode[];
    const edges = elements.filter(el => 'source' in el) as PlanDagEdge[];

    const planDag: PlanDag = {
      nodes: nodes.map(convertReactFlowNodeToPlanDagNode),
      edges: edges.map(convertReactFlowEdgeToPlanDagEdge),
      metadata: {
        version: '1.0',
        created_at: new Date().toISOString(),
        layercake_graphs: layercakeGraphs,
      },
    };

    return JSON.stringify(planDag, null, 2);
  }, [layercakeGraphs]);

  // Convert Plan DAG JSON to ReactFlow elements
  const planDagToElements = useCallback((planDagJson: string) => {
    const planDag: PlanDag = JSON.parse(planDagJson);

    const nodes = planDag.nodes.map(convertPlanDagNodeToReactFlow);
    const edges = planDag.edges.map(convertPlanDagEdgeToReactFlow);

    setLayercakeGraphs(planDag.metadata.layercake_graphs || []);

    return [...nodes, ...edges];
  }, []);

  // YAML backward compatibility export
  const exportToYAML = useCallback((planDag: PlanDag) => {
    // Convert Plan DAG to legacy YAML format for CLI compatibility
    return convertPlanDagToYAML(planDag);
  }, []);

  // YAML import with Plan DAG conversion
  const importFromYAML = useCallback((yamlContent: string) => {
    // Convert legacy YAML to Plan DAG structure
    return convertYAMLToPlanDag(yamlContent);
  }, []);
};

// LayercakeGraph management hooks
const useLayercakeGraphs = (projectId: string) => {
  const [graphs, setGraphs] = useState<LayercakeGraph[]>([]);

  const createGraph = useCallback(async (config: CreateGraphConfig) => {
    const newGraph = await graphService.createLayercakeGraph({
      projectId,
      ...config,
    });
    setGraphs(prev => [...prev, newGraph]);
    return newGraph;
  }, [projectId]);

  const copyGraph = useCallback(async (sourceGraphId: number, targetName: string) => {
    // 1. Create new LayercakeGraph instance
    const targetGraph = await createGraph({
      graphName: targetName,
      parentGraphId: sourceGraphId,
      graphType: 'copy',
    });

    // 2. Copy all nodes, edges, layers to new graph
    await graphService.copyGraphData(sourceGraphId, targetGraph.id);

    return targetGraph;
  }, [createGraph]);

  const transformGraph = useCallback(async (
    sourceGraphId: number,
    transformConfig: TransformConfig,
    targetName: string
  ) => {
    // 1. Create new LayercakeGraph for transform result
    const targetGraph = await createGraph({
      graphName: targetName,
      parentGraphId: sourceGraphId,
      graphType: 'transform',
    });

    // 2. Apply transformation using existing transform engine
    await graphService.applyTransformation(sourceGraphId, targetGraph.id, transformConfig);

    return targetGraph;
  }, [createGraph]);

  return { graphs, createGraph, copyGraph, transformGraph };
};
```

**Key Deliverables**
- ✅ Complete Plan DAG Visual Editor with ReactFlow
- ✅ All 6 Plan DAG node types implemented (Input, Graph, Copy, Transform, Merge, Output)
- ✅ LayercakeGraph objects as first-class DAG nodes
- ✅ Copy→Transform→Graph pipeline creation
- ✅ Plan DAG JSON storage with YAML backward compatibility
- ✅ Node configuration popup editors for each node type
- ✅ Graph hierarchy visualization within Plan DAG

**Success Criteria**
- Plan DAG editor creates valid JSON DAG structures
- LayercakeGraph objects properly reference graph data
- Copy→Transform pipelines execute correctly
- Plan DAG supports complex graphs (50+ nodes, 10+ LayercakeGraphs)
- YAML export maintains CLI compatibility
- Graph projections and scenarios work as designed

### Phase 2: LayercakeGraph Implementation & Collaboration (Months 5-8)

#### Month 5-6: LayercakeGraph Service Implementation

**LayercakeGraph Backend Implementation**
```rust
// LayercakeGraph service for managing graph instances
pub struct LayercakeGraphService {
    db: DatabaseConnection,
}

impl LayercakeGraphService {
    pub async fn create_layercake_graph(&self, config: CreateGraphConfig) -> Result<LayercakeGraph> {
        let graph = layercake_graphs::ActiveModel {
            project_id: Set(config.project_id),
            graph_name: Set(config.graph_name),
            parent_graph_id: Set(config.parent_graph_id),
            generation: Set(config.generation.unwrap_or(0)),
            graph_type: Set(config.graph_type.to_string()),
            metadata: Set(serde_json::to_string(&config.metadata)?),
            ..Default::default()
        };

        let result = graph.insert(&self.db).await?;
        Ok(result)
    }

    pub async fn copy_graph_data(&self, source_graph_id: i32, target_graph_id: i32) -> Result<()> {
        // Copy nodes
        let source_nodes = Node::find()
            .filter(node::Column::LayercakeGraphId.eq(source_graph_id))
            .all(&self.db)
            .await?;

        for node in source_nodes {
            let new_node = node::ActiveModel {
                id: NotSet,
                layercake_graph_id: Set(target_graph_id),
                project_id: Set(node.project_id),
                node_id: Set(node.node_id),
                label: Set(node.label),
                layer_id: Set(node.layer_id),
                properties: Set(node.properties),
            };
            new_node.insert(&self.db).await?;
        }

        // Copy edges
        let source_edges = Edge::find()
            .filter(edge::Column::LayercakeGraphId.eq(source_graph_id))
            .all(&self.db)
            .await?;

        for edge in source_edges {
            let new_edge = edge::ActiveModel {
                id: NotSet,
                layercake_graph_id: Set(target_graph_id),
                project_id: Set(edge.project_id),
                edge_id: Set(edge.edge_id),
                source_node_id: Set(edge.source_node_id),
                target_node_id: Set(edge.target_node_id),
                label: Set(edge.label),
                properties: Set(edge.properties),
            };
            new_edge.insert(&self.db).await?;
        }

        // Copy layers
        let source_layers = Layer::find()
            .filter(layer::Column::LayercakeGraphId.eq(source_graph_id))
            .all(&self.db)
            .await?;

        for layer in source_layers {
            let new_layer = layer::ActiveModel {
                id: NotSet,
                layercake_graph_id: Set(target_graph_id),
                project_id: Set(layer.project_id),
                layer_id: Set(layer.layer_id),
                name: Set(layer.name),
                color: Set(layer.color),
                properties: Set(layer.properties),
            };
            new_layer.insert(&self.db).await?;
        }

        Ok(())
    }

    pub async fn apply_transformation(&self,
        source_graph_id: i32,
        target_graph_id: i32,
        transform_config: TransformConfig
    ) -> Result<()> {
        // 1. First copy the source graph data
        self.copy_graph_data(source_graph_id, target_graph_id).await?;

        // 2. Apply transformation using existing transform engine
        let transform_engine = TransformEngine::new(&self.db);
        transform_engine.apply_to_graph(target_graph_id, transform_config).await?;

        // 3. Record transformation operation for reproducibility
        self.record_graph_operation(target_graph_id, GraphOperation::Transform {
            source_graph_id,
            transform_config: transform_config.clone(),
        }).await?;

        Ok(())
    }

    pub async fn get_graph_hierarchy(&self, root_graph_id: i32) -> Result<GraphHierarchy> {
        // Build complete graph hierarchy tree
        let mut hierarchy = GraphHierarchy::new();

        fn collect_children(graph_id: i32, db: &DatabaseConnection) -> Vec<LayercakeGraph> {
            LayercakeGraph::find()
                .filter(layercake_graphs::Column::ParentGraphId.eq(graph_id))
                .all(db)
                .await
                .unwrap_or_default()
        }

        hierarchy.build_tree(root_graph_id, &self.db).await?;
        Ok(hierarchy)
    }
}
```

**Plan DAG Execution Engine**
```rust
// Plan DAG execution engine
pub struct PlanDagExecutor {
    db: DatabaseConnection,
    graph_service: LayercakeGraphService,
}

impl PlanDagExecutor {
    pub async fn execute_plan_dag(&self, project_id: i32) -> Result<ExecutionResult> {
        // 1. Load Plan DAG from project
        let project = Project::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        let plan_dag: PlanDag = serde_json::from_str(
            &project.plan_dag_json.unwrap_or_default()
        )?;

        // 2. Topological sort of DAG nodes
        let execution_order = self.topological_sort(&plan_dag)?;

        // 3. Execute nodes in order
        for node_id in execution_order {
            let node = plan_dag.nodes.iter()
                .find(|n| n.id() == node_id)
                .ok_or_else(|| anyhow!("Node not found: {}", node_id))?;

            self.execute_node(node, &plan_dag).await?;
        }

        Ok(ExecutionResult::Success)
    }

    // Improved execution with data flow model
    async fn execute_plan_dag(&self, project_id: i32) -> Result<ExecutionResult> {
        let project = Project::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        let plan_dag: PlanDag = serde_json::from_str(
            &project.plan_dag_json.unwrap_or_default()
        )?;

        // Find all execution paths (from InputNode to GraphNode)
        let execution_paths = self.find_execution_paths(&plan_dag)?;

        // Execute each path independently (supports multiple copies)
        for path in execution_paths {
            self.execute_path_async(path).await?;
        }

        Ok(ExecutionResult::Success)
    }

    async fn execute_path_async(&self, path: Vec<&PlanDagNode>) -> Result<()> {
        let mut graph_data: Option<GraphData> = None;

        for node in path {
            match node {
                PlanDagNode::InputNode { input_type, config, .. } => {
                    // Load initial data
                    graph_data = Some(self.load_input_data(input_type, config).await?);
                },

                PlanDagNode::GraphNode { layercake_graph_id, .. } => {
                    if let Some(graph_id) = layercake_graph_id {
                        // Load existing persisted graph
                        graph_data = Some(self.load_graph_data(*graph_id).await?);
                    } else {
                        // Create new graph from flowing data
                        if let Some(data) = &graph_data {
                            let new_graph = self.graph_service
                                .create_layercake_graph_from_data(data.clone())
                                .await?;

                            // Update node with persisted graph ID
                            self.update_graph_node_id(node.id(), new_graph.id).await?;
                        }
                    }
                },

                PlanDagNode::CopyNode { copy_config, .. } => {
                    // Copy graph data (in memory, not persisted)
                    if let Some(data) = &graph_data {
                        graph_data = Some(self.deep_copy_graph_data(data, copy_config)?);
                    }
                },

                PlanDagNode::TransformNode { transform_config, .. } => {
                    // Apply transformation to flowing data
                    if let Some(data) = &graph_data {
                        graph_data = Some(self.apply_transformation(data, transform_config).await?);
                    }
                },

                PlanDagNode::MergeNode { merge_config, .. } => {
                    // Merge multiple data streams (implementation depends on merge strategy)
                    graph_data = Some(self.merge_graph_data(graph_data, merge_config).await?);
                },

                PlanDagNode::OutputNode { export_format, export_config, filename, .. } => {
                    // Export from flowing data or connected GraphNode
                    if let Some(data) = &graph_data {
                        self.export_graph_data(data, export_format, export_config, filename).await?;
                    }
                },
            }
        }

        Ok(())
    }

    fn find_execution_paths(&self, plan_dag: &PlanDag) -> Result<Vec<Vec<&PlanDagNode>>> {
        let mut paths = Vec::new();

        // Find all InputNodes as starting points
        let input_nodes: Vec<&PlanDagNode> = plan_dag.nodes.iter()
            .filter(|node| matches!(node, PlanDagNode::InputNode { .. }))
            .collect();

        // Trace paths from each InputNode
        for input_node in input_nodes {
            let mut current_path = vec![input_node];
            self.trace_path_recursive(input_node, &mut current_path, plan_dag, &mut paths);
        }

        Ok(paths)
    }

    fn resolve_graph_node_id(&self, node_id: &str, plan_dag: &PlanDag) -> Result<i32> {
        // Find GraphNode in Plan DAG and return its layercake_graph_id
/// Plan DAG validator for execution readiness
pub struct PlanDagValidator;

impl PlanDagValidator {
    pub fn validate_for_execution(plan_dag: &PlanDag) -> Result<(), ValidationError> {
        // Check for cycles (already done in topological sort, but explicit check)
        Self::check_cycles(plan_dag)?;

        // Validate node connections
        Self::validate_connections(plan_dag)?;

        // Check resource requirements
        Self::validate_resources(plan_dag)?;

        Ok(())
    }

    fn check_cycles(plan_dag: &PlanDag) -> Result<(), ValidationError> {
        // Cycle detection using DFS
        use std::collections::{HashMap, HashSet};

        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for node in &plan_dag.nodes {
            adjacency.insert(ExecutionPlanner::extract_node_id(node), Vec::new());
        }

        for edge in &plan_dag.edges {
            adjacency.get_mut(&edge.source).unwrap().push(edge.target.clone());
        }

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node_id in adjacency.keys() {
            if !visited.contains(node_id) {
                if Self::has_cycle_dfs(node_id, &adjacency, &mut visited, &mut rec_stack) {
                    return Err(ValidationError::CyclicDependency);
                }
            }
        }

        Ok(())
    }

    fn has_cycle_dfs(
        node: &str,
        adjacency: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::has_cycle_dfs(neighbor, adjacency, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn validate_connections(plan_dag: &PlanDag) -> Result<(), ValidationError> {
        for edge in &plan_dag.edges {
            // Validate edge connection rules based on ConnectionType
            match &edge.connection_type {
                ConnectionType::StreamingData { data_type, .. } => {
                    Self::validate_data_flow_connection(&edge.source, &edge.target, data_type, plan_dag)?;
                },
                ConnectionType::StorageReference { .. } => {
                    Self::validate_storage_reference_connection(&edge.source, &edge.target, plan_dag)?;
                },
                ConnectionType::ControlFlow { .. } => {
                    // Control flow connections are always valid
                },
            }
        }
        Ok(())
    }

    fn validate_data_flow_connection(
        source_id: &str,
        target_id: &str,
        _data_type: &DataType,
        plan_dag: &PlanDag,
    ) -> Result<(), ValidationError> {
        let source_node = plan_dag.nodes.iter()
            .find(|n| ExecutionPlanner::extract_node_id(n) == source_id)
            .ok_or_else(|| ValidationError::NodeNotFound(source_id.to_string()))?;

        let target_node = plan_dag.nodes.iter()
            .find(|n| ExecutionPlanner::extract_node_id(n) == target_id)
            .ok_or_else(|| ValidationError::NodeNotFound(target_id.to_string()))?;

        // Validate that source can produce data and target can consume it
        match (source_node, target_node) {
            (PlanDagNode::Input { .. }, PlanDagNode::Storage { .. }) => Ok(()),
            (PlanDagNode::Input { .. }, PlanDagNode::Process { .. }) => Ok(()),
            (PlanDagNode::Process { .. }, PlanDagNode::Storage { .. }) => Ok(()),
            (PlanDagNode::Process { .. }, PlanDagNode::Process { .. }) => Ok(()),
            (PlanDagNode::Process { .. }, PlanDagNode::Output { .. }) => Ok(()),
            _ => Err(ValidationError::InvalidConnection {
                source: source_id.to_string(),
                target: target_id.to_string(),
                reason: "Invalid data flow connection type".to_string(),
            }),
        }
    }

    fn validate_storage_reference_connection(
        source_id: &str,
        target_id: &str,
        plan_dag: &PlanDag,
    ) -> Result<(), ValidationError> {
        let source_node = plan_dag.nodes.iter()
            .find(|n| ExecutionPlanner::extract_node_id(n) == source_id)
            .ok_or_else(|| ValidationError::NodeNotFound(source_id.to_string()))?;

        // Only Storage nodes can be sources for storage references
        match source_node {
            PlanDagNode::Storage { .. } => Ok(()),
            _ => Err(ValidationError::InvalidConnection {
                source: source_id.to_string(),
                target: target_id.to_string(),
                reason: "Storage reference must originate from Storage node".to_string(),
            }),
        }
    }

    fn validate_resources(plan_dag: &PlanDag) -> Result<(), ValidationError> {
        // Check if total resource requirements are reasonable
        let total_memory: usize = plan_dag.nodes.iter()
            .map(|_| 100) // Simplified - would calculate actual requirements
            .sum();

        if total_memory > 10_000 { // 10GB limit
            return Err(ValidationError::ExcessiveResourceRequirements {
                required_memory_mb: total_memory,
                limit_memory_mb: 10_000,
            });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Cyclic dependency detected in Plan DAG")]
    CyclicDependency,
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Invalid connection from {source} to {target}: {reason}")]
    InvalidConnection { source: String, target: String, reason: String },
    #[error("Excessive resource requirements: {required_memory_mb}MB required, limit is {limit_memory_mb}MB")]
    ExcessiveResourceRequirements { required_memory_mb: usize, limit_memory_mb: usize },
    #[error("Empty graph name")]
    EmptyGraphName,
}
```

**Hierarchy Service Implementation**
```rust
// Project hierarchy management
pub struct ProjectHierarchyService {
    db: DatabaseConnection,
}

impl ProjectHierarchyService {
    pub async fn create_scenario(&self, parent_id: i32, name: String) -> Result<ProjectModel> {
        // 1. Copy parent project structure
        // 2. Copy all graph data (nodes, edges, layers)
        // 3. Create parent-child relationship
        // 4. Return new scenario project
    }

    pub async fn propagate_changes(&self, parent_id: i32, changes: Vec<Change>) -> Result<()> {
        // 1. Find all child projects
        // 2. Apply non-conflicting changes
        // 3. Flag conflicts for manual resolution
        // 4. Notify users of propagated changes
    }

    pub async fn get_hierarchy_tree(&self, root_id: i32) -> Result<HierarchyTree> {
        // 1. Build complete project tree
        // 2. Include change status for each node
        // 3. Return structured hierarchy
    }
}
```

**Unified GraphQL Schema with Subscriptions**
```graphql
# Core Types
type Project {
  id: ID!
  name: String!
  description: String
  parentProject: Project
  childProjects: [Project!]!
  hierarchyLevel: Int!
  isScenario: Boolean!
  changesSinceParent: [Change!]!
  planDagJson: String
  layercakeGraphs: [LayercakeGraph!]!
  collaborators: [User!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type LayercakeGraph {
  id: ID!
  name: String!
  projectId: ID!
  parentGraphId: ID
  generation: Int!
  graphType: GraphType!
  nodeCount: Int!
  edgeCount: Int!
  layerCount: Int!
  metadata: JSON
  createdAt: DateTime!
  updatedAt: DateTime!
}

type GraphNode {
  id: ID!
  projectId: ID!
  layercakeGraphId: ID
  nodeId: String!
  label: String!
  layerId: String
  properties: JSON
  x: Float
  y: Float
  createdAt: DateTime!
  updatedAt: DateTime!
}

type GraphEdge {
  id: ID!
  projectId: ID!
  layercakeGraphId: ID
  edgeId: String!
  sourceNodeId: String!
  targetNodeId: String!
  label: String
  properties: JSON
  createdAt: DateTime!
  updatedAt: DateTime!
}

type User {
  id: ID!
  name: String!
  email: String!
  isOnline: Boolean!
  currentProject: ID
  cursorPosition: CursorPosition
  lastSeen: DateTime!
}

type CursorPosition {
  x: Float!
  y: Float!
  nodeId: String
  editorType: EditorType!
}

enum EditorType {
  PLAN_DAG
  GRAPH_VISUAL
  GRAPH_SPREADSHEET
}

enum GraphType {
  ROOT
  COPY
  TRANSFORM
  MERGE
  SCENARIO
}

# Collaboration Types
type Conflict {
  id: ID!
  projectId: ID!
  operationA: GraphOperation!
  operationB: GraphOperation!
  conflictType: ConflictType!
  resolutionStrategy: ResolutionStrategy
  resolvedAt: DateTime
  resolvedBy: ID
}

type GraphOperation {
  id: ID!
  projectId: ID!
  layercakeGraphId: ID
  operationType: OperationType!
  entityType: EntityType!
  entityId: String!
  data: JSON!
  authorId: ID!
  timestamp: DateTime!
  causedConflicts: [Conflict!]!
}

enum OperationType {
  CREATE
  UPDATE
  DELETE
  MOVE
  BULK_UPDATE
}

enum EntityType {
  NODE
  EDGE
  LAYER
  PROJECT
  LAYERCAKE_GRAPH
  PLAN_DAG
}

enum ConflictType {
  CONCURRENT_EDIT
  DELETE_VS_UPDATE
  TYPE_MISMATCH
  CONSTRAINT_VIOLATION
}

enum ResolutionStrategy {
  LAST_WRITER_WINS
  MERGE_CHANGES
  MANUAL_RESOLUTION
  REJECT_OPERATION
}

# QUERIES - Replace REST API entirely
type Query {
  # Project Management
  projects(filter: ProjectFilter): [Project!]!
  project(id: ID!): Project
  projectHierarchy(rootId: ID!): ProjectHierarchy!

  # Graph Data
  layercakeGraphs(projectId: ID!): [LayercakeGraph!]!
  layercakeGraph(id: ID!): LayercakeGraph
  graphNodes(projectId: ID!, layercakeGraphId: ID, pagination: Pagination): GraphNodeConnection!
  graphEdges(projectId: ID!, layercakeGraphId: ID, pagination: Pagination): GraphEdgeConnection!

  # Collaboration
  activeUsers(projectId: ID!): [User!]!
  conflicts(projectId: ID!, resolved: Boolean): [Conflict!]!
  operations(projectId: ID!, since: DateTime): [GraphOperation!]!

  # Analysis
  graphMetrics(layercakeGraphId: ID!): GraphMetrics!
  findPaths(layercakeGraphId: ID!, source: String!, target: String!): [Path!]!
}

# MUTATIONS - Replace REST API entirely
type Mutation {
  # Project Operations
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: ID!, input: UpdateProjectInput!): Project!
  deleteProject(id: ID!): Boolean!
  createScenario(parentId: ID!, name: String!): Project!

  # Plan DAG Operations
  updatePlanDag(projectId: ID!, planDagJson: String!): Project!
  executePlanDag(projectId: ID!): ExecutionResult!

  # LayercakeGraph Operations
  createLayercakeGraph(input: CreateLayercakeGraphInput!): LayercakeGraph!
  copyLayercakeGraph(sourceId: ID!, targetName: String!): LayercakeGraph!
  transformLayercakeGraph(sourceId: ID!, transformConfig: JSON!, targetName: String!): LayercakeGraph!

  # Graph Data Operations (replacing REST endpoints)
  createNode(input: CreateNodeInput!): GraphNode!
  updateNode(id: ID!, input: UpdateNodeInput!): GraphNode!
  deleteNode(id: ID!): Boolean!
  createEdge(input: CreateEdgeInput!): GraphEdge!
  updateEdge(id: ID!, input: UpdateEdgeInput!): GraphEdge!
  deleteEdge(id: ID!): Boolean!

  # Bulk Operations
  bulkUpdateNodes(projectId: ID!, updates: [BulkNodeUpdate!]!): BulkUpdateResult!
  bulkCreateEdges(projectId: ID!, edges: [CreateEdgeInput!]!): BulkCreateResult!

  # Collaboration Operations
  joinProject(projectId: ID!): User!
  leaveProject(projectId: ID!): Boolean!
  updateCursor(projectId: ID!, position: CursorPositionInput!): Boolean!
  resolveConflict(conflictId: ID!, strategy: ResolutionStrategy!, data: JSON): Conflict!

  # MCP Tool Integration
  executeMcpTool(toolName: String!, parameters: JSON!): McpResult!
  registerMcpTool(toolDefinition: McpToolInput!): McpTool!
}

# SUBSCRIPTIONS - Real-time updates replace WebSocket
type Subscription {
  # Project Changes
  projectChanged(projectId: ID!): ProjectChange!

  # Graph Data Changes
  graphChanged(projectId: ID!, layercakeGraphId: ID): GraphChange!
  nodesChanged(projectId: ID!, layercakeGraphId: ID): NodeChange!
  edgesChanged(projectId: ID!, layercakeGraphId: ID): EdgeChange!

  # Collaboration Events
  userPresence(projectId: ID!): UserPresenceChange!
  operationBroadcast(projectId: ID!): GraphOperation!
  conflictDetected(projectId: ID!): Conflict!

  # System Events
  connectionStatus: ConnectionStatus!
  mcpToolUpdate: McpToolChange!
}

# Subscription Payload Types
type ProjectChange {
  changeType: ChangeType!
  project: Project!
  operation: GraphOperation
  author: User!
}

type GraphChange {
  changeType: ChangeType!
  layercakeGraph: LayercakeGraph!
  affectedNodes: [String!]!
  affectedEdges: [String!]!
  operation: GraphOperation!
  author: User!
}

type NodeChange {
  changeType: ChangeType!
  node: GraphNode
  operation: GraphOperation!
  author: User!
}

type EdgeChange {
  changeType: ChangeType!
  edge: GraphEdge
  operation: GraphOperation!
  author: User!
}

type UserPresenceChange {
  changeType: PresenceChangeType!
  user: User!
  previousPosition: CursorPosition
}

type ConnectionStatus {
  isConnected: Boolean!
  latency: Int
  queuedOperations: Int!
  lastSync: DateTime
}

enum ChangeType {
  CREATED
  UPDATED
  DELETED
  MOVED
}

enum PresenceChangeType {
  JOINED
  LEFT
  CURSOR_MOVED
  STATUS_CHANGED
}

# Input Types
input CreateProjectInput {
  name: String!
  description: String
  parentProjectId: ID
}

input UpdateProjectInput {
  name: String
  description: String
}

input CreateLayercakeGraphInput {
  projectId: ID!
  name: String!
  parentGraphId: ID
  graphType: GraphType!
}

input CreateNodeInput {
  projectId: ID!
  layercakeGraphId: ID
  nodeId: String!
  label: String!
  layerId: String
  properties: JSON
  x: Float
  y: Float
}

input UpdateNodeInput {
  label: String
  layerId: String
  properties: JSON
  x: Float
  y: Float
}

input CreateEdgeInput {
  projectId: ID!
  layercakeGraphId: ID
  edgeId: String!
  sourceNodeId: String!
  targetNodeId: String!
  label: String
  properties: JSON
}

input UpdateEdgeInput {
  label: String
  properties: JSON
}

input CursorPositionInput {
  x: Float!
  y: Float!
  nodeId: String
  editorType: EditorType!
}

input BulkNodeUpdate {
  nodeId: String!
  updates: UpdateNodeInput!
}

# Pagination and Filtering
input Pagination {
  first: Int
  after: String
  last: Int
  before: String
}

input ProjectFilter {
  name: String
  isScenario: Boolean
  parentId: ID
}

type GraphNodeConnection {
  edges: [GraphNodeEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type GraphNodeEdge {
  node: GraphNode!
  cursor: String!
}

type GraphEdgeConnection {
  edges: [GraphEdgeConnectionEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type GraphEdgeConnectionEdge {
  node: GraphEdge!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# Result Types
type ExecutionResult {
  executionId: String!
  status: ExecutionStatus!
  createdGraphs: [LayercakeGraph!]!
  generatedFiles: [String!]!
  executionTime: Int!
  errors: [ExecutionError!]
}

type BulkUpdateResult {
  successCount: Int!
  failureCount: Int!
  errors: [BulkError!]!
  updatedNodes: [GraphNode!]!
}

type BulkCreateResult {
  successCount: Int!
  failureCount: Int!
  errors: [BulkError!]!
  createdEdges: [GraphEdge!]!
}

type McpResult {
  success: Boolean!
  data: JSON
  errors: [String!]
}

enum ExecutionStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
  CANCELLED
}

scalar DateTime
scalar JSON
```

**Key Deliverables**
- ✅ Improved Plan DAG node architecture with clear data flow
- ✅ LayercakeGraph entity and service implementation
- ✅ Copy→Transform→Graph pipelines with proper UX flow
- ✅ Multiple independent copies from single graph support
- ✅ Plan DAG execution engine with path-based execution
- ✅ Visual distinction between operations and storage nodes
- ✅ Edit operation tracking for reproducibility

**Success Criteria**
- Copy operations flow data without immediate persistence
- Transform chains can be applied before creating final graphs
- Multiple independent copies can branch from single source graph
- Plan DAG execution follows data flow paths correctly
- Visual editor clearly distinguishes operations vs. storage
- LayercakeGraph instances only created when data reaches GraphNode
- Complex branching and merging pipelines work correctly

#### Month 7-8: Graph Visual Editor

**ReactFlow Graph Editor**
```typescript
// GraphVisualEditor Component
interface GraphNode {
  id: string;
  type: 'default' | 'group';
  position: { x: number; y: number };
  data: {
    label: string;
    layerId?: string;
    properties: Record<string, any>;
    isSelected: boolean;
    isHighlighted: boolean;
  };
  style: {
    backgroundColor?: string;
    borderColor?: string;
    color?: string;
  };
}

interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: 'default' | 'smoothstep' | 'step';
  data: {
    label?: string;
    properties: Record<string, any>;
  };
  style: {
    stroke?: string;
    strokeWidth?: number;
  };
}

// Graph Editor Features
const GraphVisualEditor: React.FC<GraphVisualEditorProps> = ({ projectId }) => {
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [edges, setEdges] = useState<GraphEdge[]>([]);
  const [selectedLayer, setSelectedLayer] = useState<string | null>(null);

  // Real-time collaboration
  const { onlineUsers, broadcastChange } = useCollaboration(projectId);

  // Graph operations
  const onNodeAdd = useCallback((node: Partial<GraphNode>) => {
    const operation = createNodeOperation(node);
    broadcastChange(operation);
    setNodes(prev => [...prev, operation.node]);
  }, [broadcastChange]);

  // Layer management
  const onLayerChange = useCallback((nodeId: string, layerId: string) => {
    const operation = updateNodeLayerOperation(nodeId, layerId);
    broadcastChange(operation);
    updateNodeInState(nodeId, { layerId });
  }, [broadcastChange]);
};
```

**Performance Optimizations**
```typescript
// Large graph handling
const useGraphPerformance = (nodes: GraphNode[], edges: GraphEdge[]) => {
  // Virtualization for large graphs
  const [visibleNodes, setVisibleNodes] = useState<GraphNode[]>([]);
  const [visibleEdges, setVisibleEdges] = useState<GraphEdge[]>([]);

  // Only render nodes in viewport
  const updateVisibleElements = useCallback((viewport: Viewport) => {
    const visible = nodes.filter(node => isInViewport(node, viewport));
    setVisibleNodes(visible);

    const connectedEdges = edges.filter(edge =>
      visible.some(n => n.id === edge.source || n.id === edge.target)
    );
    setVisibleEdges(connectedEdges);
  }, [nodes, edges]);

  // Debounced updates
  const debouncedUpdate = useMemo(
    () => debounce(updateVisibleElements, 100),
    [updateVisibleElements]
  );
};
```

**Key Deliverables**
- ✅ Complete Graph Visual Editor with ReactFlow
- ✅ Node/edge creation, editing, deletion
- ✅ Layer-aware visualization and editing
- ✅ Performance optimized for 10K+ nodes
- ✅ Integration with existing graph CRUD APIs

**Success Criteria**
- Graph editor responsive with large graphs (10K+ nodes)
- All CRUD operations work with existing backend
- Layer system functional and intuitive
- Visual editor maintains data consistency

### Phase 3: Unified Real-time Collaboration via GraphQL (Months 9-12)

#### Month 9-10: GraphQL Subscription Infrastructure

**GraphQL Subscription Server Implementation**
```rust
// Unified GraphQL subscription system - replaces WebSocket complexity
use async_graphql::*;
use tokio::sync::broadcast;
use std::collections::HashMap;
use uuid::Uuid;

// Subscription publisher for real-time events
pub struct SubscriptionPublisher {
    project_channels: Arc<RwLock<HashMap<i32, broadcast::Sender<ProjectEvent>>>>,
    user_sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    conflict_detector: Arc<ConflictDetector>,
}

#[derive(Debug, Clone)]
pub enum ProjectEvent {
    GraphChanged {
        project_id: i32,
        layercake_graph_id: Option<i32>,
        change_type: ChangeType,
        operation: GraphOperation,
        author: User,
    },
    UserPresenceChanged {
        project_id: i32,
        change_type: PresenceChangeType,
        user: User,
        previous_position: Option<CursorPosition>,
    },
    ConflictDetected {
        project_id: i32,
        conflict: Conflict,
    },
    ConnectionStatusChanged {
        is_connected: bool,
        latency: Option<i32>,
        queued_operations: i32,
    },
}

impl SubscriptionPublisher {
    pub fn new() -> Self {
        Self {
            project_channels: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            conflict_detector: Arc::new(ConflictDetector::new()),
        }
    }

    pub async fn subscribe_to_project(&self, project_id: i32) -> broadcast::Receiver<ProjectEvent> {
        let mut channels = self.project_channels.write().await;
        let channel = channels.entry(project_id)
            .or_insert_with(|| broadcast::channel(1000).0);
        channel.subscribe()
    }

    pub async fn publish_graph_change(
        &self,
        project_id: i32,
        operation: GraphOperation,
        author: User,
    ) -> Result<()> {
        // Check for conflicts before publishing
        if let Some(conflict) = self.conflict_detector.detect_conflict(&operation).await? {
            self.publish_conflict(project_id, conflict).await?;
        }

        let event = ProjectEvent::GraphChanged {
            project_id,
            layercake_graph_id: operation.layercake_graph_id,
            change_type: operation.operation_type.into(),
            operation,
            author,
        };

        self.publish_to_project(project_id, event).await
    }

    pub async fn publish_user_presence(
        &self,
        project_id: i32,
        user: User,
        change_type: PresenceChangeType,
        previous_position: Option<CursorPosition>,
    ) -> Result<()> {
        let event = ProjectEvent::UserPresenceChanged {
            project_id,
            change_type,
            user,
            previous_position,
        };

        self.publish_to_project(project_id, event).await
    }

    async fn publish_to_project(&self, project_id: i32, event: ProjectEvent) -> Result<()> {
        if let Some(channel) = self.project_channels.read().await.get(&project_id) {
            let _ = channel.send(event); // Ignore if no subscribers
        }
        Ok(())
    }

    async fn publish_conflict(&self, project_id: i32, conflict: Conflict) -> Result<()> {
        let event = ProjectEvent::ConflictDetected { project_id, conflict };
        self.publish_to_project(project_id, event).await
    }
}

// GraphQL Subscription resolvers
struct Subscription;

#[Subscription]
impl Subscription {
    // Replace WebSocket with GraphQL subscription
    async fn project_changed(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> Result<impl Stream<Item = ProjectChange>> {
        let publisher = ctx.data::<SubscriptionPublisher>()?;
        let project_id: i32 = project_id.parse()?;

        // Verify user has access to project
        let user = get_current_user(ctx)?;
        verify_project_access(&user, project_id).await?;

        let mut receiver = publisher.subscribe_to_project(project_id).await;

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if let ProjectEvent::GraphChanged { change_type, operation, author, .. } = event {
                    yield ProjectChange {
                        change_type,
                        project: load_project(project_id).await.unwrap(),
                        operation: Some(operation),
                        author,
                    };
                }
            }
        })
    }

    async fn graph_changed(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
        layercake_graph_id: Option<ID>,
    ) -> Result<impl Stream<Item = GraphChange>> {
        let publisher = ctx.data::<SubscriptionPublisher>()?;
        let project_id: i32 = project_id.parse()?;
        let graph_id: Option<i32> = layercake_graph_id.map(|id| id.parse()).transpose()?;

        let user = get_current_user(ctx)?;
        verify_project_access(&user, project_id).await?;

        let mut receiver = publisher.subscribe_to_project(project_id).await;

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                match event {
                    ProjectEvent::GraphChanged { layercake_graph_id, change_type, operation, author, .. } => {
                        // Filter by specific graph if requested
                        if graph_id.is_none() || graph_id == layercake_graph_id {
                            if let Ok(graph) = load_layercake_graph(layercake_graph_id.unwrap_or(0)).await {
                                yield GraphChange {
                                    change_type,
                                    layercake_graph: graph,
                                    affected_nodes: operation.get_affected_nodes(),
                                    affected_edges: operation.get_affected_edges(),
                                    operation,
                                    author,
                                };
                            }
                        }
                    },
                    _ => {} // Ignore non-graph events
                }
            }
        })
    }

    async fn user_presence(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> Result<impl Stream<Item = UserPresenceChange>> {
        let publisher = ctx.data::<SubscriptionPublisher>()?;
        let project_id: i32 = project_id.parse()?;

        let user = get_current_user(ctx)?;
        verify_project_access(&user, project_id).await?;

        let mut receiver = publisher.subscribe_to_project(project_id).await;

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if let ProjectEvent::UserPresenceChanged { change_type, user, previous_position, .. } = event {
                    yield UserPresenceChange {
                        change_type,
                        user,
                        previous_position,
                    };
                }
            }
        })
    }

    async fn conflict_detected(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> Result<impl Stream<Item = Conflict>> {
        let publisher = ctx.data::<SubscriptionPublisher>()?;
        let project_id: i32 = project_id.parse()?;

        let user = get_current_user(ctx)?;
        verify_project_access(&user, project_id).await?;

        let mut receiver = publisher.subscribe_to_project(project_id).await;

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if let ProjectEvent::ConflictDetected { conflict, .. } = event {
                    yield conflict;
                }
            }
        })
    }

    async fn connection_status(
        &self,
        ctx: &Context<'_>,
    ) -> Result<impl Stream<Item = ConnectionStatus>> {
        let publisher = ctx.data::<SubscriptionPublisher>()?;

        // Global connection status subscription
        Ok(async_stream::stream! {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                yield ConnectionStatus {
                    is_connected: true,
                    latency: Some(calculate_latency().await),
                    queued_operations: get_queued_operations_count().await,
                    last_sync: Some(chrono::Utc::now()),
                };
            }
        })
    }
}
```

**Enhanced Conflict Detection and Resolution**
```rust
// Improved operational transform integrated with GraphQL mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOperation {
    pub id: String,
    pub project_id: i32,
    pub layercake_graph_id: Option<i32>,
    pub operation_type: OperationType,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub data: serde_json::Value,
    pub author_id: String,
    pub timestamp: DateTime<Utc>,
    pub causality_vector: HashMap<String, u64>, // Vector clock for causality
    pub dependencies: Vec<String>, // Operation IDs this depends on
}

impl GraphOperation {
    pub fn get_affected_nodes(&self) -> Vec<String> {
        match self.entity_type {
            EntityType::NODE => vec![self.entity_id.clone()],
            EntityType::EDGE => {
                // Extract source and target from edge data
                if let Ok(edge_data) = serde_json::from_value::<EdgeData>(self.data.clone()) {
                    vec![edge_data.source_node_id, edge_data.target_node_id]
                } else {
                    vec![]
                }
            },
            _ => vec![],
        }
    }

    pub fn get_affected_edges(&self) -> Vec<String> {
        match self.entity_type {
            EntityType::EDGE => vec![self.entity_id.clone()],
            EntityType::NODE if self.operation_type == OperationType::DELETE => {
                // When deleting a node, find all connected edges
                // This would be populated by the mutation resolver
                vec![] // Simplified for now
            },
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    CREATE,
    UPDATE,
    DELETE,
    MOVE,
    BULK_UPDATE,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    NODE,
    EDGE,
    LAYER,
    PROJECT,
    LAYERCAKE_GRAPH,
    PLAN_DAG,
}

pub struct ConflictDetector {
    recent_operations: Arc<RwLock<HashMap<String, Vec<GraphOperation>>>>, // project_id -> operations
    conflict_rules: Vec<ConflictRule>,
}

#[derive(Debug, Clone)]
pub struct ConflictRule {
    pub name: String,
    pub detector: fn(&GraphOperation, &GraphOperation) -> Option<ConflictType>,
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            recent_operations: Arc::new(RwLock::new(HashMap::new())),
            conflict_rules: vec![
                ConflictRule {
                    name: "concurrent_edit".to_string(),
                    detector: |op1, op2| {
                        if op1.entity_type == op2.entity_type
                            && op1.entity_id == op2.entity_id
                            && op1.operation_type == OperationType::UPDATE
                            && op2.operation_type == OperationType::UPDATE
                            && !Self::is_causally_ordered(op1, op2)
                        {
                            Some(ConflictType::CONCURRENT_EDIT)
                        } else {
                            None
                        }
                    },
                },
                ConflictRule {
                    name: "delete_vs_update".to_string(),
                    detector: |op1, op2| {
                        if op1.entity_type == op2.entity_type
                            && op1.entity_id == op2.entity_id
                            && ((op1.operation_type == OperationType::DELETE && op2.operation_type == OperationType::UPDATE)
                                || (op1.operation_type == OperationType::UPDATE && op2.operation_type == OperationType::DELETE))
                        {
                            Some(ConflictType::DELETE_VS_UPDATE)
                        } else {
                            None
                        }
                    },
                },
            ],
        }
    }

    pub async fn detect_conflict(&self, new_op: &GraphOperation) -> Result<Option<Conflict>> {
        let recent_ops = self.recent_operations.read().await;
        let project_ops = recent_ops.get(&new_op.project_id.to_string()).unwrap_or(&vec![]);

        // Check against recent operations (last 5 minutes)
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        for existing_op in project_ops {
            if existing_op.timestamp < cutoff {
                continue;
            }

            // Apply conflict detection rules
            for rule in &self.conflict_rules {
                if let Some(conflict_type) = (rule.detector)(existing_op, new_op) {
                    return Ok(Some(Conflict {
                        id: Uuid::new_v4().to_string(),
                        project_id: new_op.project_id,
                        operation_a: existing_op.clone(),
                        operation_b: new_op.clone(),
                        conflict_type,
                        resolution_strategy: None,
                        resolved_at: None,
                        resolved_by: None,
                    }));
                }
            }
        }

        // Store operation for future conflict detection
        self.store_operation(new_op.clone()).await;
        Ok(None)
    }

    async fn store_operation(&self, operation: GraphOperation) {
        let mut recent_ops = self.recent_operations.write().await;
        let project_ops = recent_ops.entry(operation.project_id.to_string()).or_insert_with(Vec::new);
        project_ops.push(operation);

        // Keep only recent operations
        let cutoff = Utc::now() - chrono::Duration::minutes(5);
        project_ops.retain(|op| op.timestamp >= cutoff);
    }

    fn is_causally_ordered(op1: &GraphOperation, op2: &GraphOperation) -> bool {
        // Check if op2 depends on op1 using vector clocks
        op1.causality_vector.iter().all(|(author, clock1)| {
            op2.causality_vector.get(author).map_or(false, |clock2| clock2 >= clock1)
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub project_id: i32,
    pub operation_a: GraphOperation,
    pub operation_b: GraphOperation,
    pub conflict_type: ConflictType,
    pub resolution_strategy: Option<ResolutionStrategy>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    CONCURRENT_EDIT,
    DELETE_VS_UPDATE,
    TYPE_MISMATCH,
    CONSTRAINT_VIOLATION,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    LAST_WRITER_WINS,
    MERGE_CHANGES,
    MANUAL_RESOLUTION,
    REJECT_OPERATION,
}
```

**Frontend GraphQL Subscription Integration**
```typescript
// Unified Apollo Client setup with subscriptions
import { ApolloClient, InMemoryCache, createHttpLink, split } from '@apollo/client';
import { GraphQLWsLink } from '@apollo/client/link/subscriptions';
import { getMainDefinition } from '@apollo/client/utilities';
import { createClient } from 'graphql-ws';
import { setContext } from '@apollo/client/link/context';

// Create WebSocket link for subscriptions
const wsLink = new GraphQLWsLink(
  createClient({
    url: 'ws://localhost:4000/graphql',
    connectionParams: () => ({
      authToken: localStorage.getItem('auth-token'),
    }),
    shouldRetry: () => true,
    retryAttempts: 5,
    retryWait: async (attempt) => {
      await new Promise((resolve) => setTimeout(resolve, Math.pow(2, attempt) * 1000));
    },
  })
);

// Create HTTP link for queries and mutations
const httpLink = createHttpLink({
  uri: 'http://localhost:4000/graphql',
});

// Add authentication to HTTP requests
const authLink = setContext((_, { headers }) => {
  const token = localStorage.getItem('auth-token');
  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : '',
    },
  };
});

// Split links based on operation type
const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === 'OperationDefinition' &&
      definition.operation === 'subscription'
    );
  },
  wsLink,
  authLink.concat(httpLink)
);

// Apollo Client with enhanced cache configuration
const apolloClient = new ApolloClient({
  link: splitLink,
  cache: new InMemoryCache({
    typePolicies: {
      Project: {
        fields: {
          layercakeGraphs: {
            merge(existing = [], incoming) {
              return incoming;
            },
          },
          collaborators: {
            merge(existing = [], incoming) {
              return incoming;
            },
          },
        },
      },
      LayercakeGraph: {
        fields: {
          nodeCount: {
            merge: false, // Always replace
          },
          edgeCount: {
            merge: false,
          },
        },
      },
      Query: {
        fields: {
          graphNodes: {
            keyArgs: ['projectId', 'layercakeGraphId'],
            merge(existing, incoming) {
              if (!existing) return incoming;

              // Merge pagination results
              return {
                ...incoming,
                edges: [...(existing.edges || []), ...incoming.edges],
              };
            },
          },
        },
      },
    },
  }),
  defaultOptions: {
    watchQuery: {
      errorPolicy: 'all',
      notifyOnNetworkStatusChange: true,
    },
    query: {
      errorPolicy: 'all',
    },
    mutate: {
      errorPolicy: 'all',
    },
  },
});

// Unified collaboration hook using GraphQL subscriptions
const useCollaboration = (projectId: string) => {
  const { data: onlineUsers } = useSubscription(USER_PRESENCE_SUBSCRIPTION, {
    variables: { projectId },
    errorPolicy: 'all',
  });

  const { data: graphChanges } = useSubscription(GRAPH_CHANGED_SUBSCRIPTION, {
    variables: { projectId },
    errorPolicy: 'all',
    onSubscriptionData: ({ subscriptionData }) => {
      if (subscriptionData.data?.graphChanged) {
        // Apollo cache automatically updates - no manual state management needed!
        console.log('Graph changed:', subscriptionData.data.graphChanged);
      }
    },
  });

  const { data: conflicts } = useSubscription(CONFLICT_DETECTED_SUBSCRIPTION, {
    variables: { projectId },
    errorPolicy: 'all',
    onSubscriptionData: ({ subscriptionData }) => {
      if (subscriptionData.data?.conflictDetected) {
        showConflictDialog(subscriptionData.data.conflictDetected);
      }
    },
  });

  const [updateCursor] = useMutation(UPDATE_CURSOR_MUTATION);
  const [createNode] = useMutation(CREATE_NODE_MUTATION, {
    // Optimistic response for immediate UI feedback
    optimisticResponse: (variables) => ({
      createNode: {
        __typename: 'GraphNode',
        id: `temp-${Date.now()}`,
        ...variables.input,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    }),
    update: (cache, { data }) => {
      if (data?.createNode) {
        // Apollo cache automatically handles the update through subscriptions
        // No manual cache manipulation needed!
      }
    },
  });

  const performOperation = useCallback(async (operation: GraphOperationInput) => {
    try {
      switch (operation.type) {
        case 'CREATE_NODE':
          await createNode({ variables: { input: operation.data } });
          break;
        case 'UPDATE_NODE':
          await updateNode({ variables: { id: operation.entityId, input: operation.data } });
          break;
        case 'DELETE_NODE':
          await deleteNode({ variables: { id: operation.entityId } });
          break;
        // ... other operations
      }
    } catch (error) {
      // Apollo error handling with retry logic
      console.error('Operation failed:', error);
      if (error.networkError) {
        // Queue for retry when connection restored
        queueOperationForRetry(operation);
      }
    }
  }, [createNode, updateNode, deleteNode]);

  const updateUserCursor = useCallback((position: CursorPosition) => {
    updateCursor({
      variables: {
        projectId,
        position,
      },
      // Don't wait for response, fire-and-forget for smooth UX
      errorPolicy: 'ignore',
    });
  }, [projectId, updateCursor]);

  return {
    onlineUsers: onlineUsers?.userPresence?.user ? [onlineUsers.userPresence.user] : [],
    performOperation,
    updateUserCursor,
    // Connection state is managed by Apollo Client automatically
    connectionState: apolloClient.wsClient?.connection?.state || 'disconnected',
  };
};

// GraphQL subscription definitions
const USER_PRESENCE_SUBSCRIPTION = gql`
  subscription UserPresence($projectId: ID!) {
    userPresence(projectId: $projectId) {
      changeType
      user {
        id
        name
        isOnline
        cursorPosition {
          x
          y
          nodeId
          editorType
        }
      }
      previousPosition {
        x
        y
        nodeId
        editorType
      }
    }
  }
`;

const GRAPH_CHANGED_SUBSCRIPTION = gql`
  subscription GraphChanged($projectId: ID!) {
    graphChanged(projectId: $projectId) {
      changeType
      layercakeGraph {
        id
        name
        nodeCount
        edgeCount
      }
      affectedNodes
      affectedEdges
      operation {
        id
        operationType
        entityType
        entityId
        timestamp
      }
      author {
        id
        name
      }
    }
  }
`;

const CONFLICT_DETECTED_SUBSCRIPTION = gql`
  subscription ConflictDetected($projectId: ID!) {
    conflictDetected(projectId: $projectId) {
      id
      conflictType
      operationA {
        id
        operationType
        entityId
        data
      }
      operationB {
        id
        operationType
        entityId
        data
      }
    }
  }
`;
```

**Key Deliverables - GraphQL Unified Approach**
- ✅ GraphQL subscriptions replace WebSocket complexity
- ✅ Enhanced conflict detection with vector clocks
- ✅ Apollo Client manages all state (local + remote)
- ✅ Optimistic UI updates with automatic rollback
- ✅ Connection management with automatic retry
- ✅ Real-time user presence via subscriptions
- ✅ MCP integration through GraphQL mutations
- ✅ Offline operation queuing in Apollo cache

#### Month 10-11: Connection Management and Offline Support

**Apollo Client Connection Management**
```typescript
// Enhanced connection management with offline support
import { ApolloClient, from, ApolloLink } from '@apollo/client';
import { onError } from '@apollo/client/link/error';
import { RetryLink } from '@apollo/client/link/retry';
import { persistCache, LocalStorageWrapper } from 'apollo3-cache-persist';

// Offline operation queue
class OfflineOperationQueue {
  private queue: GraphOperationInput[] = [];
  private isProcessing = false;
  private apolloClient: ApolloClient<any>;

  constructor(client: ApolloClient<any>) {
    this.apolloClient = client;
    this.loadQueueFromStorage();
  }

  async queueOperation(operation: GraphOperationInput): Promise<void> {
    this.queue.push({
      ...operation,
      queuedAt: new Date().toISOString(),
      retryCount: 0,
    });
    await this.saveQueueToStorage();

    // Try to process immediately if online
    if (navigator.onLine) {
      this.processQueue();
    }
  }

  async processQueue(): Promise<void> {
    if (this.isProcessing || this.queue.length === 0) return;

    this.isProcessing = true;
    const processedOperations: string[] = [];

    try {
      for (const [index, operation] of this.queue.entries()) {
        try {
          await this.executeOperation(operation);
          processedOperations.push(index.toString());
        } catch (error) {
          console.warn('Failed to process queued operation:', error);

          // Increment retry count
          operation.retryCount = (operation.retryCount || 0) + 1;

          // Remove operation if exceeded max retries
          if (operation.retryCount >= 3) {
            console.error('Operation exceeded max retries, removing:', operation);
            processedOperations.push(index.toString());
          }
        }
      }
    } finally {
      // Remove processed operations
      this.queue = this.queue.filter((_, index) =>
        !processedOperations.includes(index.toString())
      );
      await this.saveQueueToStorage();
      this.isProcessing = false;
    }
  }

  private async executeOperation(operation: GraphOperationInput): Promise<void> {
    const mutation = this.getGraphQLMutation(operation);
    await this.apolloClient.mutate({
      mutation: mutation.query,
      variables: mutation.variables,
      errorPolicy: 'none', // Fail fast for queue processing
    });
  }

  private getGraphQLMutation(operation: GraphOperationInput) {
    switch (operation.type) {
      case 'CREATE_NODE':
        return {
          query: CREATE_NODE_MUTATION,
          variables: { input: operation.data },
        };
      case 'UPDATE_NODE':
        return {
          query: UPDATE_NODE_MUTATION,
          variables: { id: operation.entityId, input: operation.data },
        };
      case 'DELETE_NODE':
        return {
          query: DELETE_NODE_MUTATION,
          variables: { id: operation.entityId },
        };
      // ... handle other operation types
      default:
        throw new Error(`Unknown operation type: ${operation.type}`);
    }
  }

  private async saveQueueToStorage(): Promise<void> {
    try {
      localStorage.setItem('offline-operation-queue', JSON.stringify(this.queue));
    } catch (error) {
      console.warn('Failed to save operation queue to storage:', error);
    }
  }

  private loadQueueFromStorage(): void {
    try {
      const stored = localStorage.getItem('offline-operation-queue');
      if (stored) {
        this.queue = JSON.parse(stored);
      }
    } catch (error) {
      console.warn('Failed to load operation queue from storage:', error);
      this.queue = [];
    }
  }

  getQueueLength(): number {
    return this.queue.length;
  }

  clearQueue(): void {
    this.queue = [];
    this.saveQueueToStorage();
  }
}

type ConnectionState = 'connected' | 'connecting' | 'disconnected' | 'error';

interface GraphOperationInput {
  type: string;
  entityId: string;
  data: any;
  queuedAt?: string;
  retryCount?: number;
}
```

**Connection Status UI Components**
```typescript
// Connection status indicator component
const ConnectionStatus: React.FC = () => {
  const { connectionState, queueLength, lastSync, syncNow } = useConnection();
  const { data: connectionData } = useSubscription(CONNECTION_STATUS_SUBSCRIPTION);

  const getStatusColor = () => {
    switch (connectionState) {
      case 'connected': return 'green';
      case 'connecting': return 'yellow';
      case 'disconnected': return 'red';
      case 'error': return 'red';
      default: return 'gray';
    }
  };

  const getStatusText = () => {
    switch (connectionState) {
      case 'connected':
        const latency = connectionData?.connectionStatus?.latency;
        return latency ? `Connected (${latency}ms)` : 'Connected';
      case 'connecting': return 'Connecting...';
      case 'disconnected': return 'Offline';
      case 'error': return 'Connection Error';
      default: return 'Unknown';
    }
  };

  return (
    <Group spacing="xs" className="connection-status">
      <Badge color={getStatusColor()} variant="filled" size="sm">
        <Group spacing={4}>
          <div className={`connection-dot ${connectionState}`} />
          {getStatusText()}
        </Group>
      </Badge>

      {queueLength > 0 && (
        <Tooltip label={`${queueLength} operations queued for sync`}>
          <Badge color="yellow" variant="outline" size="sm">
            Queue: {queueLength}
          </Badge>
        </Tooltip>
      )}

      {connectionState === 'disconnected' && queueLength > 0 && (
        <Button size="xs" variant="outline" onClick={syncNow}>
          Sync Now
        </Button>
      )}

      {lastSync && (
        <Text size="xs" color="dimmed">
          Last sync: {formatDistanceToNow(lastSync)} ago
        </Text>
      )}
    </Group>
  );
};
```

**Success Criteria**
- Automatic reconnection with exponential backoff
- Offline operations queued and synced when connection restored
- Real-time updates responsive (< 100ms latency)
- System stable with 10+ concurrent users
- Graceful degradation during network issues
- Cache persistence survives browser refresh
- Clear connection status feedback to users

#### Month 11-12: Graph Spreadsheet Editor

**Mantine Table Implementation**
```typescript
// GraphSpreadsheetEditor Component
const GraphSpreadsheetEditor: React.FC<{ projectId: string }> = ({ projectId }) => {
  const [activeTab, setActiveTab] = useState<'nodes' | 'edges' | 'layers'>('nodes');
  const [selectedRows, setSelectedRows] = useState<string[]>([]);
  const [editingCell, setEditingCell] = useState<{ row: string; column: string } | null>(null);

  // Data management
  const { data: graphData, loading, error } = useGraphData(projectId);
  const [updateNode] = useUpdateNodeMutation();
  const [updateEdge] = useUpdateEdgeMutation();
  const [updateLayer] = useUpdateLayerMutation();

  // Bulk operations
  const handleBulkUpdate = useCallback(async (updates: BulkUpdate[]) => {
    const operations = updates.map(update => ({
      type: 'bulk_update',
      entity_type: activeTab,
      updates: update,
    }));

    await Promise.all(operations.map(op => executeOperation(op)));

    // Broadcast to collaborators
    broadcastOperation(operations);
  }, [activeTab, broadcastOperation]);

  // Column definitions
  const nodeColumns: ColumnDef<GraphNode>[] = [
    {
      accessorKey: 'node_id',
      header: 'Node ID',
      enableEditing: false,
    },
    {
      accessorKey: 'label',
      header: 'Label',
      enableEditing: true,
      cell: ({ getValue, row, column }) => (
        <EditableCell
          value={getValue()}
          onSave={(value) => handleCellUpdate(row.id, column.id, value)}
          isEditing={editingCell?.row === row.id && editingCell?.column === column.id}
        />
      ),
    },
    {
      accessorKey: 'layer_id',
      header: 'Layer',
      enableEditing: true,
      cell: ({ getValue, row }) => (
        <LayerSelectCell
          value={getValue()}
          layers={graphData?.layers || []}
          onChange={(layerId) => handleLayerChange(row.id, layerId)}
        />
      ),
    },
  ];

  return (
    <Tabs value={activeTab} onChange={setActiveTab}>
      <Tabs.List>
        <Tabs.Tab value="nodes">Nodes ({graphData?.nodes?.length || 0})</Tabs.Tab>
        <Tabs.Tab value="edges">Edges ({graphData?.edges?.length || 0})</Tabs.Tab>
        <Tabs.Tab value="layers">Layers ({graphData?.layers?.length || 0})</Tabs.Tab>
      </Tabs.List>

      <Tabs.Panel value="nodes">
        <DataTable
          columns={nodeColumns}
          data={graphData?.nodes || []}
          enableRowSelection
          onRowSelectionChange={setSelectedRows}
          enableBulkOperations
          onBulkUpdate={handleBulkUpdate}
        />
      </Tabs.Panel>

      {/* Similar panels for edges and layers */}
    </Tabs>
  );
};
```

**Advanced Spreadsheet Features**
```typescript
// Advanced editing capabilities
const useAdvancedSpreadsheet = () => {
  // Undo/Redo system
  const [history, setHistory] = useState<OperationHistory[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  const undo = useCallback(() => {
    if (historyIndex >= 0) {
      const operation = history[historyIndex];
      reverseOperation(operation);
      setHistoryIndex(prev => prev - 1);
    }
  }, [history, historyIndex]);

  const redo = useCallback(() => {
    if (historyIndex < history.length - 1) {
      const operation = history[historyIndex + 1];
      applyOperation(operation);
      setHistoryIndex(prev => prev + 1);
    }
  }, [history, historyIndex]);

  // Search and filter
  const [searchTerm, setSearchTerm] = useState('');
  const [filters, setFilters] = useState<ColumnFilter[]>([]);

  const filteredData = useMemo(() => {
    let filtered = data;

    // Apply search
    if (searchTerm) {
      filtered = filtered.filter(row =>
        Object.values(row).some(value =>
          String(value).toLowerCase().includes(searchTerm.toLowerCase())
        )
      );
    }

    // Apply column filters
    filters.forEach(filter => {
      filtered = filtered.filter(row =>
        applyFilter(row[filter.column], filter.value, filter.operator)
      );
    });

    return filtered;
  }, [data, searchTerm, filters]);

  // Data validation
  const validateCellUpdate = useCallback((rowId: string, column: string, value: any) => {
    const schema = getColumnSchema(column);
    return validateValue(value, schema);
  }, []);
};
```

**Key Deliverables**
- ✅ Complete Graph Spreadsheet Editor with three tabs
- ✅ Bulk edit operations with validation
- ✅ Search, filter, and sort functionality
- ✅ Undo/redo system with operation history
- ✅ Integration with real-time collaboration

**Success Criteria**
- Spreadsheet handles large datasets efficiently (10K+ rows)
- Bulk operations work reliably with existing APIs
- Data validation prevents integrity violations
- Real-time collaboration works in spreadsheet mode

### Phase 4: Enhanced Features (Months 13-16)

#### Month 13-14: Enhanced MCP Tools

**Extended MCP Tool Categories**
```rust
// Enhanced MCP tools building on existing foundation
impl LayercakeToolRegistry {
    async fn get_enhanced_tools(&self) -> Vec<Tool> {
        vec![
            // ✅ EXISTING TOOLS (14 tools already implemented)
            // Project: list_projects, create_project, get_project, delete_project
            // Plans: create_plan, execute_plan, get_plan_status
            // Graph: import_csv, export_graph, get_graph_data
            // Analysis: analyze_connectivity, find_paths

            // 🚧 NEW HIERARCHY TOOLS
            Tool::new("create_scenario", "Create child project scenario")
                .with_params(json!({
                    "parent_id": {"type": "integer"},
                    "name": {"type": "string"},
                    "description": {"type": "string", "optional": true}
                })),

            Tool::new("get_project_hierarchy", "Get project hierarchy tree")
                .with_params(json!({
                    "root_project_id": {"type": "integer"}
                })),

            Tool::new("propagate_changes", "Propagate changes to child projects")
                .with_params(json!({
                    "parent_id": {"type": "integer"},
                    "change_types": {"type": "array", "items": {"type": "string"}}
                })),

            // 🚧 NEW COLLABORATION TOOLS
            Tool::new("get_active_sessions", "Get active collaboration sessions")
                .with_params(json!({
                    "project_id": {"type": "integer"}
                })),

            Tool::new("resolve_conflicts", "Resolve edit conflicts")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "conflict_ids": {"type": "array", "items": {"type": "string"}},
                    "resolution_strategy": {"type": "string", "enum": ["accept_latest", "manual_merge"]}
                })),

            // 🚧 NEW ADVANCED ANALYSIS TOOLS
            Tool::new("analyze_graph_metrics", "Calculate comprehensive graph metrics")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "metrics": {"type": "array", "items": {"type": "string"}}
                })),

            Tool::new("detect_communities", "Detect communities in graph")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "algorithm": {"type": "string", "enum": ["louvain", "leiden", "modularity"]}
                })),

            Tool::new("find_critical_paths", "Find critical paths in graph")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "source_nodes": {"type": "array", "items": {"type": "string"}},
                    "target_nodes": {"type": "array", "items": {"type": "string"}}
                })),

            // 🚧 NEW BULK OPERATION TOOLS
            Tool::new("bulk_update_nodes", "Bulk update multiple nodes")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "updates": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "node_id": {"type": "string"},
                            "updates": {"type": "object"}
                        }
                    }}
                })),

            Tool::new("bulk_create_edges", "Bulk create multiple edges")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "edges": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "source": {"type": "string"},
                            "target": {"type": "string"},
                            "label": {"type": "string"},
                            "properties": {"type": "object"}
                        }
                    }}
                })),

            // 🚧 NEW VISUALIZATION TOOLS
            Tool::new("generate_layout", "Generate automatic graph layout")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "algorithm": {"type": "string", "enum": ["force", "hierarchical", "circular", "grid"]},
                    "options": {"type": "object"}
                })),

            Tool::new("export_visualization", "Export interactive visualization")
                .with_params(json!({
                    "project_id": {"type": "integer"},
                    "format": {"type": "string", "enum": ["html", "svg", "png", "pdf"]},
                    "layout": {"type": "object"}
                })),
        ]
    }
}
```

**AI Agent Workflow Optimization**
```rust
// Optimized tool workflows for AI agents
pub struct AgentWorkflowService {
    tool_registry: LayercakeToolRegistry,
}

impl AgentWorkflowService {
    // Common workflow: Project Analysis
    pub async fn analyze_project_workflow(&self, project_id: i32) -> McpResult<AnalysisReport> {
        // 1. Get project details
        let project = self.tool_registry.execute_tool("get_project",
            ToolExecutionContext::with_args(json!({"project_id": project_id}))
        ).await?;

        // 2. Get graph data
        let graph_data = self.tool_registry.execute_tool("get_graph_data",
            ToolExecutionContext::with_args(json!({
                "project_id": project_id,
                "include_nodes": true,
                "include_edges": true,
                "include_layers": true
            }))
        ).await?;

        // 3. Analyze connectivity
        let connectivity = self.tool_registry.execute_tool("analyze_connectivity",
            ToolExecutionContext::with_args(json!({"project_id": project_id}))
        ).await?;

        // 4. Calculate metrics
        let metrics = self.tool_registry.execute_tool("analyze_graph_metrics",
            ToolExecutionContext::with_args(json!({
                "project_id": project_id,
                "metrics": ["centrality", "clustering", "density", "components"]
            }))
        ).await?;

        // 5. Detect communities
        let communities = self.tool_registry.execute_tool("detect_communities",
            ToolExecutionContext::with_args(json!({
                "project_id": project_id,
                "algorithm": "louvain"
            }))
        ).await?;

        Ok(AnalysisReport {
            project: project.into(),
            graph_data: graph_data.into(),
            connectivity: connectivity.into(),
            metrics: metrics.into(),
            communities: communities.into(),
        })
    }

    // Workflow: Iterative Graph Development
    pub async fn iterative_development_workflow(&self,
        project_name: String,
        csv_data: String
    ) -> McpResult<DevelopmentResult> {
        // 1. Create project
        let project = self.tool_registry.execute_tool("create_project",
            ToolExecutionContext::with_args(json!({
                "name": project_name,
                "description": "Created by AI agent for iterative development"
            }))
        ).await?;

        let project_id = project.content[0].get("id").unwrap().as_i64().unwrap() as i32;

        // 2. Import CSV data
        let import_result = self.tool_registry.execute_tool("import_csv",
            ToolExecutionContext::with_args(json!({
                "project_id": project_id,
                "nodes_csv": csv_data
            }))
        ).await?;

        // 3. Analyze initial structure
        let initial_analysis = self.analyze_project_workflow(project_id).await?;

        // 4. Generate layout
        let layout = self.tool_registry.execute_tool("generate_layout",
            ToolExecutionContext::with_args(json!({
                "project_id": project_id,
                "algorithm": "force",
                "options": {"iterations": 100, "repulsion": 50}
            }))
        ).await?;

        Ok(DevelopmentResult {
            project_id,
            import_result,
            analysis: initial_analysis,
            layout,
        })
    }
}
```

**Key Deliverables**
- ✅ 25+ total MCP tools (14 existing + 11+ new)
- ✅ Workflow optimization for AI agents
- ✅ Enhanced analysis and visualization tools
- ✅ Bulk operation tools for efficiency
- ✅ Project hierarchy tools

**Success Criteria**
- Every UI operation available as MCP tool
- AI agent workflows execute efficiently
- Tool performance adequate for interactive use
- Clear documentation for tool discovery

#### Month 15-16: Advanced Visualizations

**3D Visualization Integration**
```typescript
// 3D Force Graph Integration
import ForceGraph3D from '3d-force-graph';

const Graph3DVisualization: React.FC<{ projectId: string }> = ({ projectId }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [graph3D, setGraph3D] = useState<any>(null);
  const { data: graphData } = useGraphData(projectId);

  useEffect(() => {
    if (containerRef.current && !graph3D) {
      const fg = ForceGraph3D()(containerRef.current)
        .graphData(convertToForceGraphData(graphData))
        .nodeAutoColorBy('layer_id')
        .nodeThreeObject(node => {
          const sprite = new SpriteText(node.label);
          sprite.material.depthWrite = false;
          sprite.color = node.color;
          sprite.textHeight = 8;
          return sprite;
        })
        .onNodeHover(node => {
          showNodeTooltip(node);
        })
        .onNodeClick(node => {
          selectNode(node.id);
        });

      setGraph3D(fg);
    }
  }, [containerRef, graph3D, graphData]);

  // Update graph when data changes
  useEffect(() => {
    if (graph3D && graphData) {
      graph3D.graphData(convertToForceGraphData(graphData));
    }
  }, [graph3D, graphData]);

  return (
    <div>
      <div ref={containerRef} style={{ width: '100%', height: '600px' }} />
      <Graph3DControls
        graph={graph3D}
        onLayoutChange={handleLayoutChange}
        onExport={handleExport}
      />
    </div>
  );
};
```

**Isoflow Network Diagrams**
```typescript
// Isoflow integration for network-style diagrams
import { IsoflowData, IsoflowRenderer } from 'isoflow';

const NetworkDiagramVisualization: React.FC<{ projectId: string }> = ({ projectId }) => {
  const { data: graphData } = useGraphData(projectId);
  const [isoflowData, setIsoflowData] = useState<IsoflowData | null>(null);

  useEffect(() => {
    if (graphData) {
      const converted = convertToIsoflowData(graphData);
      setIsoflowData(converted);
    }
  }, [graphData]);

  const convertToIsoflowData = (data: GraphData): IsoflowData => {
    return {
      nodes: data.nodes.map(node => ({
        id: node.node_id,
        label: node.label,
        type: 'server', // or determine from layer/properties
        x: node.position?.x || 0,
        y: node.position?.y || 0,
        z: 0,
      })),
      connections: data.edges.map(edge => ({
        id: edge.edge_id,
        source: edge.source_node_id,
        target: edge.target_node_id,
        label: edge.label,
        type: 'ethernet', // or determine from properties
      })),
    };
  };

  return (
    <div style={{ width: '100%', height: '600px' }}>
      {isoflowData && (
        <IsoflowRenderer
          data={isoflowData}
          onNodeSelect={handleNodeSelect}
          onConnectionSelect={handleConnectionSelect}
          showGrid={true}
          enableZoom={true}
        />
      )}
    </div>
  );
};
```

**Enhanced Export System**
```typescript
// Enhanced export with visualizations
const useEnhancedExport = () => {
  const exportWithVisualization = useCallback(async (
    projectId: string,
    format: 'html' | 'svg' | 'png' | 'pdf',
    visualization: 'force3d' | 'isoflow' | 'reactflow'
  ) => {
    const graphData = await getGraphData(projectId);

    switch (format) {
      case 'html':
        return generateInteractiveHTML(graphData, visualization);

      case 'svg':
        return generateSVGExport(graphData, visualization);

      case 'png':
        return generateImageExport(graphData, visualization, 'png');

      case 'pdf':
        return generatePDFExport(graphData, visualization);
    }
  }, []);

  const generateInteractiveHTML = async (data: GraphData, viz: string) => {
    const template = await loadVisualizationTemplate(viz);
    const html = template
      .replace('{{GRAPH_DATA}}', JSON.stringify(data))
      .replace('{{VISUALIZATION_TYPE}}', viz);

    return {
      content: html,
      filename: `graph-${viz}-${Date.now()}.html`,
      mimeType: 'text/html',
    };
  };

  return { exportWithVisualization };
};
```

**Key Deliverables**
- ✅ 3D force graph visualization integration
- ✅ Isoflow network diagram support
- ✅ Interactive HTML exports with embedded visualizations
- ✅ Enhanced export system with multiple visualization options
- ✅ Performance optimization for large graphs

**Success Criteria**
- 3D visualizations work smoothly with 1K+ node graphs
- Network diagrams render correctly for infrastructure graphs
- Interactive exports fully functional offline
- Visualization exports integrate with existing export system

### Phase 5: Production & Polish (Months 17-18)

#### Month 17: Performance Optimization

**Database Query Optimization**
```rust
// Optimized database queries for large graphs
impl GraphDataService {
    // Paginated node retrieval
    pub async fn get_nodes_paginated(&self,
        project_id: i32,
        page: u64,
        page_size: u64,
        filters: Option<NodeFilters>
    ) -> Result<PaginatedNodes> {
        let mut query = Node::find()
            .filter(node::Column::ProjectId.eq(project_id));

        if let Some(f) = filters {
            if let Some(layer_id) = f.layer_id {
                query = query.filter(node::Column::LayerId.eq(layer_id));
            }
            if let Some(search) = f.search {
                query = query.filter(node::Column::Label.contains(&search));
            }
        }

        let paginated = query
            .paginate(&self.db, page_size)
            .fetch_page(page)
            .await?;

        Ok(PaginatedNodes {
            nodes: paginated.data,
            total: paginated.number_of_items,
            page,
            page_size,
        })
    }

    // Efficient subgraph extraction
    pub async fn get_subgraph(&self,
        project_id: i32,
        node_ids: Vec<String>,
        depth: u32
    ) -> Result<SubGraph> {
        // Use recursive CTE for efficient subgraph queries
        let query = r#"
            WITH RECURSIVE subgraph(node_id, depth) AS (
                SELECT node_id, 0 FROM nodes
                WHERE project_id = ? AND node_id IN (?)
                UNION ALL
                SELECT DISTINCT e.target_node_id, s.depth + 1
                FROM edges e
                JOIN subgraph s ON e.source_node_id = s.node_id
                WHERE s.depth < ? AND e.project_id = ?
            )
            SELECT DISTINCT node_id FROM subgraph
        "#;

        let result = self.db.query_all(
            Statement::from_string(DatabaseBackend::Sqlite, query)
                .values([
                    project_id.into(),
                    node_ids.join(",").into(),
                    depth.into(),
                    project_id.into(),
                ])
        ).await?;

        // Fetch nodes and edges for subgraph
        // ... implementation details
    }
}
```

**Frontend Performance Optimization**
```typescript
// React performance optimizations
const OptimizedGraphEditor = React.memo<GraphEditorProps>(({ projectId }) => {
  // Virtual scrolling for large node lists
  const { virtualizer } = useVirtualizer({
    count: nodes.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => 50,
    overscan: 10,
  });

  // Debounced operations
  const debouncedSave = useMemo(
    () => debounce(async (operations: GraphOperation[]) => {
      await saveBatchOperations(operations);
    }, 500),
    []
  );

  // Memoized calculations
  const visibleNodes = useMemo(() => {
    return nodes.filter(node => isInViewport(node, viewport));
  }, [nodes, viewport]);

  const visibleEdges = useMemo(() => {
    const visibleNodeIds = new Set(visibleNodes.map(n => n.id));
    return edges.filter(edge =>
      visibleNodeIds.has(edge.source) || visibleNodeIds.has(edge.target)
    );
  }, [edges, visibleNodes]);

  // WebWorker for heavy computations
  const layoutWorker = useMemo(() => {
    return new Worker(new URL('../workers/layout.worker.ts', import.meta.url));
  }, []);

  const calculateLayout = useCallback((nodes: GraphNode[], edges: GraphEdge[]) => {
    return new Promise<LayoutResult>((resolve) => {
      layoutWorker.postMessage({ nodes, edges, algorithm: 'force' });
      layoutWorker.onmessage = (e) => resolve(e.data);
    });
  }, [layoutWorker]);

  return (
    <div ref={containerRef}>
      <ReactFlow
        nodes={visibleNodes}
        edges={visibleEdges}
        onNodesChange={debouncedSave}
        onEdgesChange={debouncedSave}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
      />
    </div>
  );
});
```

**WebSocket Performance**
```rust
// WebSocket connection pooling and optimization
pub struct OptimizedCollaborationService {
    connection_pool: Arc<RwLock<HashMap<i32, Vec<SessionConnection>>>>,
    message_buffer: Arc<Mutex<HashMap<i32, Vec<CollaborationMessage>>>>,
    flush_interval: Duration,
}

impl OptimizedCollaborationService {
    pub async fn start_message_batching(&self) {
        let buffer = self.message_buffer.clone();
        let pool = self.connection_pool.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.flush_interval);

            loop {
                interval.tick().await;

                let mut buffer_guard = buffer.lock().await;
                for (project_id, messages) in buffer_guard.drain() {
                    if !messages.is_empty() {
                        self.broadcast_batched_messages(project_id, messages).await;
                    }
                }
            }
        });
    }

    async fn broadcast_batched_messages(&self,
        project_id: i32,
        messages: Vec<CollaborationMessage>
    ) {
        let pool = self.connection_pool.read().await;
        if let Some(connections) = pool.get(&project_id) {
            let batched = CollaborationMessage::Batch { messages };
            let serialized = serde_json::to_string(&batched).unwrap();

            for connection in connections {
                let _ = connection.send(Message::Text(serialized.clone())).await;
            }
        }
    }
}
```

**Key Deliverables**
- ✅ Database queries optimized for large datasets
- ✅ Frontend rendering optimized for 10K+ nodes
- ✅ WebSocket performance improved with batching
- ✅ Memory usage optimization and leak prevention
- ✅ Load testing completed and documented

**Success Criteria**
- System handles 50K+ node graphs without performance degradation
- Real-time collaboration remains responsive with 20+ users
- Memory usage stable during long editing sessions
- Database queries respond in < 100ms for typical operations

#### Month 18: Production Deployment

**Docker Configuration**
```dockerfile
# Multi-stage Dockerfile for production
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY external-modules ./external-modules

# Build with release optimizations
RUN cargo build --release --features=all-apis

FROM node:18 as frontend-builder

WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci

COPY frontend/ ./
RUN npm run build

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and frontend assets
COPY --from=builder /app/target/release/layercake /usr/local/bin/
COPY --from=frontend-builder /app/frontend/dist /usr/local/share/layercake/frontend

# Create non-root user
RUN groupadd -r layercake && useradd -r -g layercake layercake
USER layercake

# Configuration
ENV RUST_LOG=info
ENV DATABASE_URL=sqlite:///data/layercake.db
ENV FRONTEND_DIR=/usr/local/share/layercake/frontend

EXPOSE 3000
VOLUME ["/data"]

CMD ["layercake", "serve", "--port", "3000", "--database", "/data/layercake.db"]
```

**Docker Compose for Development**
```yaml
# docker-compose.yml
version: '3.8'

services:
  layercake:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - layercake_data:/data
      - ./config:/config
    environment:
      - RUST_LOG=debug
      - DATABASE_URL=sqlite:///data/layercake.db
      - CORS_ORIGIN=http://localhost:5173
    depends_on:
      - postgres
    restart: unless-stopped

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=layercake
      - POSTGRES_USER=layercake
      - POSTGRES_PASSWORD=layercake_dev
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - layercake
    restart: unless-stopped

volumes:
  layercake_data:
  postgres_data:
```

**Production Configuration**
```yaml
# config/production.yaml
server:
  port: 3000
  host: "0.0.0.0"
  cors_origins:
    - "https://app.layercake.dev"
    - "https://layercake.dev"

database:
  url: "postgresql://layercake:${DATABASE_PASSWORD}@postgres:5432/layercake"
  max_connections: 20
  min_connections: 5
  acquire_timeout: 30s

collaboration:
  max_concurrent_users: 100
  message_batch_size: 50
  flush_interval: 100ms
  session_timeout: 1h

performance:
  max_nodes_per_project: 100000
  max_edges_per_project: 500000
  query_timeout: 30s
  cache_size: 1GB

security:
  session_secret: "${SESSION_SECRET}"
  jwt_secret: "${JWT_SECRET}"
  rate_limit_requests: 100
  rate_limit_window: 60s

logging:
  level: "info"
  format: "json"
  access_log: true

monitoring:
  metrics_enabled: true
  health_check_interval: 30s
  tracing_enabled: true
```

**Security Implementation**
```rust
// Production security middleware
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub fn create_production_app(db: DatabaseConnection) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .nest("/graphql", graphql_routes(db.clone()))
        .nest("/mcp", mcp_routes(db))
        .layer(
            ServiceBuilder::new()
                // Security layers
                .layer(SecurityHeadersLayer::new())
                .layer(CorsLayer::very_permissive())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
                .layer(TimeoutLayer::new(Duration::from_secs(30)))

                // Rate limiting
                .layer(RateLimitLayer::new(100, Duration::from_secs(60)))

                // Logging and tracing
                .layer(TraceLayer::new_for_http())
                .layer(AccessLogLayer::new())
        )
}

#[derive(Clone)]
pub struct SecurityHeadersLayer;

impl<S> Layer<S> for SecurityHeadersLayer {
    type Service = SecurityHeadersService<S>;

    fn layer(&self, service: S) -> Self::Service {
        SecurityHeadersService { inner: service }
    }
}

pub struct SecurityHeadersService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for SecurityHeadersService<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let future = self.inner.call(request);

        // Add security headers to response
        future.map(|response| {
            response.map(|mut resp| {
                resp.headers_mut().insert("X-Content-Type-Options", "nosniff".parse().unwrap());
                resp.headers_mut().insert("X-Frame-Options", "DENY".parse().unwrap());
                resp.headers_mut().insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
                resp.headers_mut().insert("Strict-Transport-Security",
                    "max-age=31536000; includeSubDomains".parse().unwrap());
                resp
            })
        })
    }
}
```

**Monitoring and Health Checks**
```rust
// Comprehensive health checks
#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    version: String,
    uptime: u64,
    database: DatabaseHealth,
    memory: MemoryHealth,
    collaboration: CollaborationHealth,
}

#[derive(Serialize)]
pub struct DatabaseHealth {
    connected: bool,
    query_time_ms: u64,
    connection_pool_size: usize,
    active_connections: usize,
}

pub async fn health_check(Extension(db): Extension<DatabaseConnection>) -> Json<HealthStatus> {
    let start_time = Instant::now();

    // Test database connectivity
    let db_health = test_database_health(&db).await;

    // Check memory usage
    let memory_health = get_memory_health();

    // Check collaboration service
    let collaboration_health = get_collaboration_health();

    let status = if db_health.connected {
        "healthy"
    } else {
        "unhealthy"
    };

    Json(HealthStatus {
        status: status.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: start_time.elapsed().as_secs(),
        database: db_health,
        memory: memory_health,
        collaboration: collaboration_health,
    })
}
```

**Key Deliverables**
- ✅ Production-ready Docker containerization
- ✅ Security hardening with middleware and headers
- ✅ Comprehensive monitoring and health checks
- ✅ Database backup and recovery procedures
- ✅ Load balancing and scaling configuration

**Success Criteria**
- System deployable in production environment
- Security review passes with no critical issues
- Monitoring provides adequate operational visibility
- Backup and recovery procedures tested and documented

## Resource Requirements & Timeline

### Team Structure

**Core Development Team (12-18 months)**
- **Backend Lead** (1.0 FTE): Rust, SeaORM, WebSocket, MCP enhancement
- **Frontend Lead** (1.0 FTE): React, ReactFlow, Mantine, real-time UI
- **DevOps/Full-stack** (0.5 FTE): Docker, deployment, performance, database

**Additional Resources (as needed)**
- **UX Designer** (0.25 FTE, months 6-12): UI design and user experience
- **Performance Engineer** (0.5 FTE, months 16-17): Optimization and scaling

### Technology Dependencies

**Backend Stack (Existing)**
```toml
# Core dependencies already in place
sea-orm = "0.12"           # Database ORM
axum = "0.7"               # HTTP server
tokio = "1.0"              # Async runtime
axum-mcp = { path = "..." } # MCP integration
async-graphql = "7.0"      # GraphQL server
```

**Frontend Stack (New)**
```json
{
  "dependencies": {
    "react": "^18.0.0",
    "typescript": "^5.0.0",
    "@mantine/core": "^7.0.0",
    "@apollo/client": "^3.8.0",
    "reactflow": "^11.0.0",
    "zustand": "^4.4.0"
  }
}
```

**Infrastructure**
```yaml
# Production infrastructure requirements
application_server:
  cpu: "4 cores"
  memory: "8GB"
  storage: "100GB SSD"

database_server:
  cpu: "4 cores"
  memory: "16GB"
  storage: "500GB SSD"

load_balancer:
  cpu: "2 cores"
  memory: "4GB"
```

### Risk Assessment

**LOW RISK** (Building on existing foundation)
- Backend API integration (APIs already complete)
- Database schema extensions (additive changes only)
- MCP tool enhancement (building on 95% complete implementation)
- Export system integration (system already complete)

**MEDIUM RISK** (New but standard technology)
- React frontend development (standard web development)
- WebSocket real-time collaboration (well-established patterns)
- Performance optimization (standard techniques)
- Docker deployment (standard containerization)

**HIGHER RISK** (Complex features)
- Large graph performance (10K+ nodes in browser)
- Real-time collaboration conflict resolution
- Project hierarchy change propagation
- 3D visualization performance

### Success Metrics

**Technical Performance**
- Graph editor responsive with 10K+ nodes (< 1s operations)
- Real-time collaboration latency < 100ms
- System uptime > 99% in production
- Memory usage stable during long sessions

**User Experience**
- Successful migration of existing projects: 100%
- Time to create graph visualization: < 50% of current YAML workflow
- User adoption of visual editors: > 80%
- Support ticket volume: < 20% of baseline

**Development Metrics**
- Code coverage: > 80% for new functionality
- Build time: < 5 minutes for full frontend + backend
- Test suite execution: < 2 minutes
- Documentation completeness: 100% for public APIs

## Updated Architecture Summary - Unified GraphQL + MCP

### **Major Improvements Made**

🚀 **Simplified Communication Architecture:**
- **REMOVED**: REST API complexity (15+ endpoints eliminated)
- **REMOVED**: WebSocket custom protocol complexity
- **REMOVED**: Zustand state management confusion
- **UNIFIED**: Single GraphQL API handles ALL operations (Query/Mutation/Subscription)
- **ENHANCED**: MCP integration via GraphQL mutations for seamless AI agent interaction

📊 **Single Source of Truth:**
- **Apollo Client Cache** manages ALL application state (local + remote)
- **GraphQL Subscriptions** replace WebSocket complexity with standard protocol
- **Optimistic UI Updates** with automatic rollback on conflict detection
- **Offline Operation Queue** with automatic sync when connection restored

🔍 **Enhanced Conflict Resolution:**
- **Vector Clock Causality** tracking for proper operation ordering
- **Real-time Conflict Detection** with CRDT-style resolution strategies
- **Manual Resolution UI** for complex conflicts that require user input
- **Operation Transform** system preserves user intent during concurrent edits

🔌 **Robust Connection Management:**
- **Automatic Reconnection** with exponential backoff
- **Cache Persistence** survives browser refresh and network interruptions
- **Connection Status UI** provides clear feedback to users
- **Graceful Degradation** maintains functionality during network issues

### **Technical Benefits Achieved**

1. **➕ Maintainability**: Single communication pattern (GraphQL) instead of 3 separate systems
2. **➕ Clarity**: Apollo Client as single source of truth eliminates state synchronization bugs
3. **➕ Implementability**: Standard GraphQL subscriptions replace custom WebSocket protocol
4. **➕ Extensibility**: New features only require GraphQL schema updates
5. **➕ Design Coherence**: Unified error handling and state management patterns

### **Production Readiness Improvements**

- **✅ Offline Support**: Operations queued and synced automatically
- **✅ Error Recovery**: Comprehensive error boundaries with retry logic
- **✅ Performance**: Apollo Client optimizations and caching strategies
- **✅ Scalability**: GraphQL subscriptions scale better than WebSocket broadcasts
- **✅ Testing**: Single communication layer easier to mock and test

## Conclusion

This **significantly enhanced** implementation plan transforms the original architecture into a **production-ready, maintainable system**. By eliminating REST API complexity and unifying all communication through GraphQL + MCP, the system achieves:

### Key Success Factors

1. **Unified Architecture**: Single GraphQL API eliminates communication complexity
2. **Real-time First**: GraphQL subscriptions provide reliable real-time updates
3. **Offline Resilience**: Robust connection management with operation queuing
4. **AI Integration**: Enhanced MCP tools with real-time GraphQL event integration
5. **Developer Experience**: Apollo Client provides excellent debugging and caching

### Expected Outcomes

**12-Month Milestone**: Production-ready visual editing platform with real-time collaboration
**18-Month Milestone**: Enterprise-grade system with advanced AI integration and performance optimization

**The unified GraphQL + MCP architecture delivers superior maintainability, reliability, and extensibility compared to the original mixed-protocol approach.** This positions layercake as a **modern, scalable platform** ready for enterprise deployment and AI agent integration.

The conservative timeline and incremental approach ensure reliable delivery while building toward the full vision outlined in the specification. This plan positions layercake as a leading interactive graph editing platform with strong AI collaboration capabilities.
