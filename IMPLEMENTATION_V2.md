# Layercake Implementation Plan V2 - Conservative Approach

## Executive Summary

This implementation plan provides a **realistic and technically feasible** roadmap for transforming layercake into an interactive graph editing platform. Based on comprehensive analysis of the existing codebase, this approach builds incrementally on the robust foundation already in place (~70% complete) rather than attempting a complete rewrite.

**Key Strategic Decision**: Build visual editors over the existing YAML-based system, extending rather than replacing the proven architecture.

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

### Recommended Implementation Strategy

**Timeline**: 12-18 months (conservative estimate building on existing foundation)
**Risk Level**: LOW-MEDIUM (leveraging proven architecture)
**Team Size**: 2-3 developers

## Technical Architecture Overview

### System Design Principles

1. **Incremental Enhancement**: Build on existing 70% complete foundation
2. **Visual Layer Addition**: Add ReactFlow editors over current YAML system
3. **Backward Compatibility**: Maintain all existing functionality
4. **Modular Development**: Ship working features in 3-month cycles
5. **Performance First**: Optimize for large graphs (10K+ nodes)

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

### Current Schema (Already Implemented)

```sql
-- ✅ EXISTING TABLES
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE plans (
    id INTEGER PRIMARY KEY,
    project_id INTEGER REFERENCES projects(id),
    name TEXT NOT NULL,
    yaml_content TEXT NOT NULL,
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

### Phase 1 Extensions (Project Hierarchy)

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

#### Month 3-4: Plan Visual Editor

**ReactFlow Plan Editor Implementation**
```typescript
// PlanVisualEditor Component Architecture
interface PlanNode {
  id: string;
  type: 'input' | 'transform' | 'output';
  position: { x: number; y: number };
  data: {
    label: string;
    config: PlanNodeConfig;
    status: 'pending' | 'running' | 'completed' | 'error';
  };
}

interface PlanEdge {
  id: string;
  source: string;
  target: string;
  type: 'default' | 'dependency';
}

// Custom Node Types
const nodeTypes = {
  inputNode: InputNodeComponent,      // CSV files, REST endpoints, SQL
  transformNode: TransformNodeComponent, // Graph transformations
  outputNode: OutputNodeComponent,    // Export formats
  graphNode: GraphNodeComponent,      // Graph references
};
```

**YAML Integration**
```typescript
// Real-time YAML synchronization
const usePlanYAMLSync = (planId: string) => {
  const [yamlContent, setYamlContent] = useState<string>('');
  const [flowElements, setFlowElements] = useState<(PlanNode | PlanEdge)[]>([]);

  // Convert ReactFlow elements to YAML
  const elementsToYAML = useCallback((elements: (PlanNode | PlanEdge)[]) => {
    // Use existing plan.rs structures
    return convertToYAML(elements);
  }, []);

  // Convert YAML to ReactFlow elements
  const yamlToElements = useCallback((yaml: string) => {
    // Parse using existing YAML structures
    return parseYAMLToPlan(yaml);
  }, []);
};
```

**Key Deliverables**
- ✅ Complete Plan Visual Editor with ReactFlow
- ✅ All existing YAML plan features supported visually
- ✅ Real-time YAML preview and synchronization
- ✅ Drag-drop plan construction interface
- ✅ Node configuration popup editors

**Success Criteria**
- Visual editor generates valid YAML plans
- All existing plan execution functionality works
- Plan editor responsive with complex plans (20+ nodes)
- Visual editor integrates seamlessly with existing CLI

### Phase 2: Graph Hierarchy & Collaboration (Months 5-8)

#### Month 5-6: Project Hierarchy Implementation

**Database Schema Extensions**
```rust
// Extended Project Entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub parent_project_id: Option<i32>,  // 🚧 NEW
    pub hierarchy_level: i32,            // 🚧 NEW
    pub is_scenario: bool,               // 🚧 NEW
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

// New Relations
impl Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentProjectId",
        to = "Column::Id"
    )]
    ParentProject,
    #[sea_orm(has_many = "Entity")]
    ChildProjects,
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
- ✅ Database schema supports project hierarchy
- ✅ Project copying and scenario creation
- ✅ Change propagation from parent to children
- ✅ GraphQL API extended for hierarchy operations
- ✅ Frontend hierarchy visualization

**Success Criteria**
- Parent/child relationships work correctly
- Scenario creation copies all graph data
- Change propagation respects user preferences
- Hierarchy visible and manageable in UI

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