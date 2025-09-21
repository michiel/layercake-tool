# Layercake Implementation Plan V2 - Conservative Approach

## Executive Summary

This implementation plan provides a **realistic and technically feasible** roadmap for transforming layercake into an interactive graph editing platform with **Plan DAG architecture**. Based on comprehensive analysis of the existing codebase, this approach builds incrementally on the robust foundation already in place (~70% complete) while implementing the critical Plan DAG with LayercakeGraph objects.

**Key Strategic Decision**: Evolve from YAML plans to Plan DAG with LayercakeGraph objects, enabling graph projections via copy→transform→graph pipelines while maintaining backward compatibility.

### Current State Assessment

**✅ ALREADY IMPLEMENTED (Significant Foundation)**:
- **Complete REST API**: Full CRUD operations for projects, plans, nodes, edges, layers
- **GraphQL API**: Query/mutation system with comprehensive schema
- **MCP Integration**: ~95% complete with 14 tools, resources, and prompts
- **Database Layer**: SeaORM entities with proper relations and migrations
- **Export System**: All formats (JSON, CSV, DOT, GML, PlantUML, Mermaid, Custom)
- **Plan Execution**: Robust YAML-based transformation pipeline
- **CLI Interface**: Complete command-line functionality
- **Server Infrastructure**: Axum-based HTTP server with CORS, health checks

**📊 Foundation Statistics**:
- **Database**: 5 entities (projects, plans, nodes, edges, layers) with full relations
- **REST API**: 15+ endpoints covering all CRUD operations
- **GraphQL**: Complete schema with queries and mutations
- **MCP Tools**: 14 implemented tools across 4 categories
- **Export Formats**: 8 different output formats supported
- **CLI Commands**: 6 main commands with subcommands
- **⚠️ CRITICAL EVOLUTION NEEDED**: Transform plans table from YAML storage to Plan DAG with LayercakeGraph objects

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
│   SeaORM DB     │  Change Tracking│      File System               │
│                 │                 │                                │
│ ✅ COMPLETE     │ 🚧 EXTEND       │ ✅ COMPLETE                     │
│ • Projects      │ • Edit History  │ • Export Files                 │
│ • Plans (YAML)  │ • User Sessions │ • Import Sources               │
│ • Nodes/Edges   │ • Operation Log │ • Templates                    │
│ • Layers        │ • Conflict Res  │ • Cache                        │
│ • Migrations    │                 │                                │
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
// Plan DAG Node Architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanDagNode {
    InputNode {
        id: String,
        label: String,
        input_type: InputType,
        config: InputConfig,
        position: Position,
    },
    GraphNode {
        id: String,
        label: String,
        graph_id: i32,  // References graphs table
        metadata: GraphMetadata,
        position: Position,
    },
    CopyNode {
        id: String,
        label: String,
        source_graph_id: String,  // Source GraphNode ID
        target_graph_id: String,  // Target GraphNode ID
        copy_config: CopyConfig,
        position: Position,
    },
    TransformNode {
        id: String,
        label: String,
        source_graph_id: String,
        target_graph_id: String,
        transform_config: TransformConfig, // Existing YAML transform options
        position: Position,
    },
    MergeNode {
        id: String,
        label: String,
        source_graph_ids: Vec<String>,
        target_graph_id: String,
        merge_config: MergeConfig,
        position: Position,
    },
    OutputNode {
        id: String,
        label: String,
        source_graph_id: String,
        export_format: String, // "GML", "PlantUML", etc.
        export_config: ExportConfig,
        filename: String,
        position: Position,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    CSVNodesFromFile { file_path: String },
    CSVEdgesFromFile { file_path: String },
    CSVLayersFromFile { file_path: String },
    RestEndpoint { url: String, auth: Option<String> },
    SqlQuery { connection: String, query: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDag {
    pub nodes: Vec<PlanDagNode>,
    pub edges: Vec<PlanDagEdge>,
    pub metadata: PlanDagMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,  // Source node ID
    pub target: String,  // Target node ID
    pub edge_type: PlanDagEdgeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanDagEdgeType {
    DataFlow,      // Data flows from source to target
    Dependency,    // Target depends on source completion
    Transformation, // Source transforms into target
}
```

### LayercakeGraph Objects in Plan DAG

```rust
// LayercakeGraph represents graph instances within Plan DAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayercakeGraph {
    pub id: i32,
    pub project_id: i32,
    pub graph_name: String,
    pub parent_graph_id: Option<i32>,  // For graph hierarchy
    pub generation: i32,               // 0=root, 1=child, 2=grandchild
    pub graph_type: LayercakeGraphType,
    pub metadata: GraphMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayercakeGraphType {
    Root,          // Original imported graph
    Copy,          // Exact copy of another graph
    Transform,     // Result of transformation
    Merge,         // Result of merging multiple graphs
    Scenario,      // User-created scenario/projection
}

// Example Plan DAG with LayercakeGraph projections:
// InputNode(CSV) → GraphNode(graph1: "Current State")
//                     ↓
//                  CopyNode → GraphNode(graph2: "Scenario A")
//                     ↓
//                  TransformNode → GraphNode(graph3: "Scenario A Optimized")
//                     ↓
//                  OutputNode(PlantUML)
```

### Database Schema Evolution

```sql
-- 🚧 EVOLVE EXISTING PROJECTS TABLE
ALTER TABLE projects ADD COLUMN plan_dag_json TEXT; -- JSON Plan DAG

-- 🚧 NEW: LayercakeGraph instances (separate from Plan DAG)
CREATE TABLE layercake_graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    graph_name TEXT NOT NULL,
    parent_graph_id INTEGER REFERENCES layercake_graphs(id),
    generation INTEGER DEFAULT 0,
    graph_type TEXT NOT NULL, -- 'root', 'copy', 'transform', 'merge', 'scenario'
    metadata TEXT,            -- JSON metadata
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 🚧 EVOLVE: Link nodes/edges/layers to LayercakeGraph instances
ALTER TABLE nodes ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id);
ALTER TABLE edges ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id);
ALTER TABLE layers ADD COLUMN layercake_graph_id INTEGER REFERENCES layercake_graphs(id);

-- 🚧 NEW: Track edit operations for reproducibility
CREATE TABLE graph_edit_operations (
    id INTEGER PRIMARY KEY,
    layercake_graph_id INTEGER REFERENCES layercake_graphs(id),
    operation_type TEXT NOT NULL, -- 'create_node', 'update_node', 'delete_node', etc.
    operation_data TEXT NOT NULL, -- JSON operation details
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    is_reproducible BOOLEAN DEFAULT TRUE,
    user_session_id TEXT
);
```

### Phase 1 Extensions (Plan DAG Migration)

```sql
-- 🚧 NEW COLUMNS TO ADD
ALTER TABLE projects ADD COLUMN parent_project_id INTEGER REFERENCES projects(id);
ALTER TABLE projects ADD COLUMN hierarchy_level INTEGER DEFAULT 0;
ALTER TABLE projects ADD COLUMN is_scenario BOOLEAN DEFAULT FALSE;

-- 🚧 NEW TABLES FOR COLLABORATION
CREATE TABLE user_sessions (
    id INTEGER PRIMARY KEY,
    session_id TEXT UNIQUE NOT NULL,
    user_name TEXT,
    project_id INTEGER REFERENCES projects(id),
    last_seen TIMESTAMP,
    cursor_position TEXT -- JSON
);

CREATE TABLE change_operations (
    id INTEGER PRIMARY KEY,
    session_id TEXT REFERENCES user_sessions(session_id),
    project_id INTEGER REFERENCES projects(id),
    operation_type TEXT NOT NULL, -- 'create', 'update', 'delete'
    entity_type TEXT NOT NULL,    -- 'node', 'edge', 'layer', 'plan'
    entity_id TEXT NOT NULL,
    operation_data TEXT NOT NULL, -- JSON
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    applied BOOLEAN DEFAULT FALSE
);
```

## Implementation Phases

### Phase 1: Frontend Foundation (Months 1-4)

#### Month 1-2: Tauri Desktop Application

**React Application Setup**
```typescript
// Frontend Technology Stack
React 18+ with TypeScript
Mantine UI v7 (component library)
Apollo Client v3 (GraphQL)
ReactFlow v11 (visual editing)
Zustand (state management)
Vite (build tool)
Tauri v2 (desktop wrapper)
```

**Project Structure**
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
│   │   └── common/
│   │       ├── Layout/               # App layout
│   │       ├── Navigation/           # Navigation components
│   │       └── Loading/              # Loading states
│   ├── hooks/
│   │   ├── useGraphQL/               # GraphQL integration
│   │   ├── useWebSocket/             # Real-time updates
│   │   └── useProjectHierarchy/      # Hierarchy management
│   ├── stores/
│   │   ├── projectStore.ts           # Project state
│   │   ├── graphStore.ts             # Graph data state
│   │   └── collaborationStore.ts     # Real-time collaboration
│   └── utils/
│       ├── graphql/                  # GraphQL queries/mutations
│       ├── validation/               # Data validation
│       └── export/                   # Export utilities
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

// GraphNode component for LayercakeGraph objects
const GraphNodeComponent: React.FC<NodeProps> = ({ data, id }) => {
  const { layercakeGraphId, graphMetadata, label } = data;
  const [graphStats, setGraphStats] = useState<GraphStats | null>(null);

  // Load graph statistics
  useEffect(() => {
    if (layercakeGraphId) {
      loadGraphStats(layercakeGraphId).then(setGraphStats);
    }
  }, [layercakeGraphId]);

  const handleEditGraph = () => {
    // Open GraphVisualEditor or GraphSpreadsheetEditor
    openGraphEditor(layercakeGraphId);
  };

  return (
    <div className="plan-dag-graph-node">
      <Handle type="target" position={Position.Top} />

      <div className="node-header">
        <LayersIcon size={16} />
        <span className="node-type">LayercakeGraph</span>
      </div>

      <div className="node-content">
        <h4>{label}</h4>
        {graphStats && (
          <div className="graph-stats">
            <span>{graphStats.nodeCount} nodes</span>
            <span>{graphStats.edgeCount} edges</span>
            <span>{graphStats.layerCount} layers</span>
          </div>
        )}
        {graphMetadata?.parentGraphId && (
          <div className="graph-hierarchy">
            <span>Child of Graph {graphMetadata.parentGraphId}</span>
          </div>
        )}
      </div>

      <div className="node-actions">
        <Button size="xs" onClick={handleEditGraph}>
          Edit Graph
        </Button>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </div>
  );
};

// Copy→Transform→Graph Pipeline Example
const createCopyTransformPipeline = async (
  sourceGraphId: number,
  transformConfig: TransformConfig
) => {
  // 1. Create target LayercakeGraph for copy
  const copyGraph = await createLayercakeGraph({
    projectId,
    graphName: `${sourceGraph.name} - Copy`,
    parentGraphId: sourceGraphId,
    graphType: 'copy',
  });

  // 2. Create target LayercakeGraph for transform result
  const transformGraph = await createLayercakeGraph({
    projectId,
    graphName: `${sourceGraph.name} - Transformed`,
    parentGraphId: copyGraph.id,
    graphType: 'transform',
  });

  // 3. Add Plan DAG nodes
  const copyNode: PlanDagNode = {
    id: generateId(),
    type: 'copy',
    position: { x: 200, y: 100 },
    data: {
      label: 'Copy Graph',
      config: { sourceGraphId, targetGraphId: copyGraph.id },
      status: 'pending',
    },
  };

  const transformNode: PlanDagNode = {
    id: generateId(),
    type: 'transform',
    position: { x: 400, y: 100 },
    data: {
      label: 'Transform Graph',
      config: {
        sourceGraphId: copyGraph.id,
        targetGraphId: transformGraph.id,
        transformConfig,
      },
      status: 'pending',
    },
  };

  const graphNode: PlanDagNode = {
    id: generateId(),
    type: 'graph',
    position: { x: 600, y: 100 },
    data: {
      label: transformGraph.graphName,
      layercakeGraphId: transformGraph.id,
      graphMetadata: {
        parentGraphId: copyGraph.id,
        generation: transformGraph.generation,
      },
      status: 'pending',
    },
  };

  return [copyNode, transformNode, graphNode];
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

    async fn execute_node(&self, node: &PlanDagNode, plan_dag: &PlanDag) -> Result<()> {
        match node {
            PlanDagNode::InputNode { id, input_type, config, .. } => {
                self.execute_input_node(id, input_type, config).await
            },
            PlanDagNode::CopyNode { source_graph_id, target_graph_id, .. } => {
                let source_id = self.resolve_graph_node_id(source_graph_id, plan_dag)?;
                let target_id = self.resolve_graph_node_id(target_graph_id, plan_dag)?;
                self.graph_service.copy_graph_data(source_id, target_id).await
            },
            PlanDagNode::TransformNode { source_graph_id, target_graph_id, transform_config, .. } => {
                let source_id = self.resolve_graph_node_id(source_graph_id, plan_dag)?;
                let target_id = self.resolve_graph_node_id(target_graph_id, plan_dag)?;
                self.graph_service.apply_transformation(source_id, target_id, transform_config.clone()).await
            },
            PlanDagNode::OutputNode { source_graph_id, export_format, export_config, filename, .. } => {
                let graph_id = self.resolve_graph_node_id(source_graph_id, plan_dag)?;
                self.execute_export(graph_id, export_format, export_config, filename).await
            },
            PlanDagNode::GraphNode { .. } => {
                // GraphNodes are references, no execution needed
                Ok(())
            },
            PlanDagNode::MergeNode { source_graph_ids, target_graph_id, merge_config, .. } => {
                let source_ids: Result<Vec<i32>> = source_graph_ids.iter()
                    .map(|id| self.resolve_graph_node_id(id, plan_dag))
                    .collect();
                let target_id = self.resolve_graph_node_id(target_graph_id, plan_dag)?;
                self.execute_merge(source_ids?, target_id, merge_config).await
            },
        }
    }

    fn resolve_graph_node_id(&self, node_id: &str, plan_dag: &PlanDag) -> Result<i32> {
        // Find GraphNode in Plan DAG and return its layercake_graph_id
        plan_dag.nodes.iter()
            .find_map(|node| {
                if let PlanDagNode::GraphNode { id, graph_id, .. } = node {
                    if id == node_id {
                        Some(*graph_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("GraphNode not found: {}", node_id))
    }
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

**GraphQL Schema Extensions**
```graphql
extend type Project {
  parentProject: Project
  childProjects: [Project!]!
  hierarchyLevel: Int!
  isScenario: Boolean!
  changesSinceParent: [Change!]!
}

extend type Mutation {
  createScenario(parentId: ID!, name: String!): Project!
  propagateChangesToChildren(projectId: ID!, changes: [ChangeInput!]!): PropagationResult!
  mergeFromParent(projectId: ID!, changeIds: [ID!]!): MergeResult!
}

type PropagationResult {
  successfulUpdates: [Project!]!
  conflicts: [Conflict!]!
  errors: [PropagationError!]!
}
```

**Key Deliverables**
- ✅ LayercakeGraph entity and service implementation
- ✅ Graph copying and transformation pipelines
- ✅ Plan DAG execution engine
- ✅ Graph hierarchy management within Plan DAG
- ✅ Edit operation tracking for reproducibility
- ✅ GraphQL API for LayercakeGraph operations

**Success Criteria**
- LayercakeGraph instances properly separate from Plan DAG metadata
- Copy→Transform→Graph pipelines execute correctly
- Graph data copying preserves all nodes, edges, layers
- Plan DAG execution follows topological order
- Edit operations tracked for reproducibility

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

### Phase 3: Real-time Collaboration (Months 9-12)

#### Month 9-10: WebSocket Infrastructure

**WebSocket Server Implementation**
```rust
// WebSocket collaboration handler
use axum::extract::ws::{WebSocket, Message};
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CollaborationMessage {
    UserJoined { session_id: String, user_name: String, project_id: i32 },
    UserLeft { session_id: String },
    OperationBroadcast { operation: GraphOperation, author: String },
    CursorUpdate { session_id: String, position: CursorPosition },
    ConflictNotification { operation_id: String, conflict: Conflict },
}

pub struct CollaborationService {
    db: DatabaseConnection,
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    broadcast: broadcast::Sender<CollaborationMessage>,
}

impl CollaborationService {
    pub async fn handle_websocket(&self, socket: WebSocket, session_id: String) {
        let mut rx = self.broadcast.subscribe();

        let (sender, mut receiver) = socket.split();

        // Handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                if let Ok(Message::Text(text)) = msg {
                    self.handle_message(text, &session_id).await;
                }
            }
        });

        // Broadcast outgoing messages
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Ok(text) = serde_json::to_string(&msg) {
                    let _ = sender.send(Message::Text(text)).await;
                }
            }
        });
    }
}
```

**Operational Transform System**
```rust
// Simple operational transform for graph operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphOperation {
    CreateNode { node_id: String, node: NodeData, position: Position },
    UpdateNode { node_id: String, updates: NodeUpdates },
    DeleteNode { node_id: String },
    CreateEdge { edge_id: String, source: String, target: String, edge: EdgeData },
    UpdateEdge { edge_id: String, updates: EdgeUpdates },
    DeleteEdge { edge_id: String },
    CreateLayer { layer_id: String, layer: LayerData },
    UpdateLayer { layer_id: String, updates: LayerUpdates },
    DeleteLayer { layer_id: String },
}

pub struct OperationTransform;

impl OperationTransform {
    pub fn resolve_conflict(op1: &GraphOperation, op2: &GraphOperation) -> ConflictResolution {
        match (op1, op2) {
            // Same node operations - use timestamp priority
            (GraphOperation::UpdateNode { node_id: id1, .. },
             GraphOperation::UpdateNode { node_id: id2, .. }) if id1 == id2 => {
                ConflictResolution::UseLatest
            },

            // Delete vs Update - delete wins
            (GraphOperation::DeleteNode { node_id: id1 },
             GraphOperation::UpdateNode { node_id: id2, .. }) if id1 == id2 => {
                ConflictResolution::UseFirst
            },

            // Non-conflicting operations
            _ => ConflictResolution::ApplyBoth,
        }
    }
}
```

**Frontend WebSocket Integration**
```typescript
// Real-time collaboration hook
const useCollaboration = (projectId: string) => {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [onlineUsers, setOnlineUsers] = useState<UserSession[]>([]);
  const [operations, setOperations] = useState<GraphOperation[]>([]);

  const connect = useCallback(() => {
    const ws = new WebSocket(`ws://localhost:3000/ws/${projectId}`);

    ws.onmessage = (event) => {
      const message: CollaborationMessage = JSON.parse(event.data);

      switch (message.type) {
        case 'UserJoined':
          setOnlineUsers(prev => [...prev, message.user]);
          break;

        case 'OperationBroadcast':
          // Apply operation to local state
          applyOperation(message.operation);
          break;

        case 'ConflictNotification':
          // Show conflict resolution UI
          showConflictDialog(message.conflict);
          break;
      }
    };

    setSocket(ws);
  }, [projectId]);

  const broadcastOperation = useCallback((operation: GraphOperation) => {
    if (socket?.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({
        type: 'OperationBroadcast',
        operation,
        timestamp: Date.now(),
      }));
    }
  }, [socket]);

  return { onlineUsers, broadcastOperation, connect };
};
```

**Key Deliverables**
- ✅ WebSocket server for real-time communication
- ✅ Simple operational transform system
- ✅ User presence and awareness indicators
- ✅ Conflict detection and resolution
- ✅ Frontend real-time collaboration hooks

**Success Criteria**
- Multiple users can edit simultaneously
- Conflicts resolved gracefully without data loss
- Real-time updates responsive (< 100ms latency)
- System stable with 10+ concurrent users

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

## Conclusion

This implementation plan provides a **realistic and achievable** roadmap that builds on layercake's substantial existing foundation. By leveraging the ~70% complete backend infrastructure and focusing on adding visual editing capabilities, this approach significantly reduces risk while delivering the core requirements.

### Key Success Factors

1. **Incremental Development**: Build on proven existing systems rather than replace them
2. **Performance First**: Optimize for large graphs throughout development
3. **User Experience**: Maintain backward compatibility while adding visual capabilities
4. **Collaboration Focus**: Simple but effective real-time editing features
5. **AI Integration**: Enhance existing MCP tools for comprehensive agent support

### Expected Outcomes

**12-Month Milestone**: Complete visual editing platform with basic collaboration
**18-Month Milestone**: Production-ready system with advanced features and optimization

The conservative timeline and incremental approach ensure reliable delivery while building toward the full vision outlined in the specification. This plan positions layercake as a leading interactive graph editing platform with strong AI collaboration capabilities.