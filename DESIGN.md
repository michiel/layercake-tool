# Layercake Design Specification

## Summary

- Layercake is a tool for collecting graph data points, collecting them into a graph, allowing versioning and editing of the graph, and then exporting the full graph or selected subsets of the graph to various formats that can be configured multiple times, individually.
- Layercake is a single binary that can be used as a library, a CLI tool, or a web service. It is designed to be extensible and modular, allowing users to add their own data sources, exporters, and other components.
- Layercake started as a plan runner, where a YAML plan defines inputs (nodes, edges, layers in CSV format) and runs a pipeline execution for the plan, generating multiple outputs that are templated using handlebars (example: PlantUML output via feeding the graph data into a PlantUML handlebars template).
- Layercake is going to be running a server that can serve multiple projects, each with inputs, its own graph, transformations for outputs, and configured exporters, using the current plan runner as a base
- Data will be persisted in a sqlite database via seaorm (with PostgreSQL support for production)
- The server will expose a full set of tools for all operations via a MCP API for AI Agent and LLM interactions
- The server will also expose a REST API for web applications and other clients to interact with the data
- The server will also expose a GraphQL API for advanced querying and manipulation of the project, plan and graph data
- MCP, GraphQL and REST will all use a unified backend
- The project will remain a single binary, with the CLI tool and web service being two different modes of operation
- CLI one-off plan execution can be performed using the same data model, possibly with an in-memory SQL database and a default project initialized specifically for the run and then discarded
- Outputs will be written to a directory structure that can be configured per project, with the ability to override the output directory for one-off runs
- As a new capability, outputs will also be exposed directly as inputs to web components and other clients, allowing for dynamic updates and interactions (example: a react component rendering the graph and updating as the graph changes)
- Layercake will have a react frontend that can be used to interact with the server, allowing users to view and edit their projects, plans, and graphs as well as preview outputs and interact with the outputs dynamically

## API Architecture and Migration Strategy

### **Current Architecture Issues**

The current system has a single graph endpoint per project (`/api/v1/project/1/graph`) which doesn't support the hierarchical navigation required for project → plan → workflow → plan node inspection.

### **Target Architecture**

The target architecture supports three API interfaces with a unified backend:

1. **GraphQL API** (Primary for frontend)
2. **REST API** (Resource access and external integrations)
3. **MCP API** (AI Agent interactions)

### **Migration Path**

**Phase 1: Foundation (Current → Hybrid)**
- **Timeline**: 2-4 weeks
- **Goal**: Introduce GraphQL alongside existing REST
- **Actions**:
  - Implement GraphQL schema with hierarchical navigation support
  - Keep existing REST endpoints for backward compatibility
  - Add flat REST endpoints for direct resource access
  - Implement unified backend layer serving both REST and GraphQL

**Phase 2: Frontend Migration (Hybrid → GraphQL-first)**
- **Timeline**: 4-6 weeks
- **Goal**: Migrate frontend components to GraphQL
- **Actions**:
  - Update GraphVisualization component to support hierarchical context
  - Implement hierarchical navigation components (ProjectOverview → PlanSelector → WorkflowViewer)
  - Add real-time subscriptions for live execution monitoring
  - Maintain REST fallbacks for non-critical features

**Phase 3: Feature Expansion (GraphQL-first → Full Feature)**
- **Timeline**: 6-8 weeks
- **Goal**: Complete feature set with graph inspection
- **Actions**:
  - Implement graph inspection at every DAG plan node
  - Add MCP API for AI agent interactions
  - Implement advanced caching strategies
  - Add performance optimizations for large graphs

**Phase 4: Deprecation (Full Feature → Clean Architecture)**
- **Timeline**: 2-3 weeks
- **Goal**: Clean up legacy endpoints
- **Actions**:
  - Deprecate hierarchical REST endpoints
  - Keep minimal flat REST for simple integrations
  - Document final API surface
  - Optimize performance based on usage patterns

### **API Endpoint Design Comparison**

**Hierarchical REST Endpoints (Phase 1-2):**
```
GET /api/v1/project/{project_id}                                    # Project metadata
GET /api/v1/project/{project_id}/plans                             # All plans in project
GET /api/v1/project/{project_id}/plan/{plan_id}                    # Specific plan DAG
GET /api/v1/project/{project_id}/plan/{plan_id}/executions         # Plan executions
GET /api/v1/project/{project_id}/plan/{plan_id}/execution/{execution_id}  # Execution state
GET /api/v1/project/{project_id}/plan/{plan_id}/plan-node/{plan_node_id}/graph      # Graph at plan node
```

**Flat REST Endpoints (Phase 3-4):**
```
GET /api/v1/projects                           # List all projects
GET /api/v1/projects/{project_id}              # Single project
GET /api/v1/plans/{plan_id}                    # Single plan
GET /api/v1/executions/{execution_id}          # Single execution
GET /api/v1/graphs/{graph_id}                  # Single graph object
GET /api/v1/plan-nodes/{plan_node_id}/graph              # Graph at specific plan node
```

**GraphQL Endpoint (All Phases):**
```
POST /graphql                                  # Single endpoint for all queries
WS   /graphql                                  # WebSocket for subscriptions
```

**Advantages of Hybrid Approach:**
- **Semantic clarity**: Hierarchical URLs for human-readable resource relationships
- **Direct access**: Flat URLs for efficient resource access
- **Flexibility**: GraphQL for complex queries and real-time updates
- **Migration safety**: Gradual transition with backward compatibility

## Plan Data, Presentation and Editing Format

### **DAG-Based Plan Structure**

Plans represent execution workflows as Directed Acyclic Graphs (DAGs) that can branch and have multiple endpoints, moving beyond simple linear pipelines.

#### **Plan Data Format Strategy**
- **Internal Representation**: JSON with flat DAG structure (nodes[] + edges[])
- **Legacy Support**: YAML import/conversion (read-only)
- **Primary Interface**: Visual DAG editor with direct JSON import/export
- **Collaboration**: JSON Patch operations for real-time editing
- **Query Support**: GraphQL-friendly flat structure with strong typing

#### **Flat DAG Plan Schema**

```json
{
  "version": "2.0",
  "metadata": {
    "name": "Architecture Analysis Plan", 
    "description": "Multi-branch execution with parallel exports",
    "created_at": "2025-01-15T10:30:00Z",
    "schema_version": "2.0.0"
  },
  "dag": {
    "nodes": [
      {
        "id": "import_data",
        "type": "import",
        "label": "Import Graph Data",
        "config": {
          "imports": [
            {
              "id": "arch_csv",
              "type": "csv_files",
              "sources": {
                "nodes": "architecture/nodes.csv",
                "edges": "architecture/edges.csv", 
                "layers": "architecture/layers.csv"
              }
            }
          ],
          "merge_strategy": "append"
        },
        "render_context": {
          "graph_name": "Architecture Overview",
          "theme": "corporate",
          "base_layout": "hierarchical"
        },
        "position": { "x": 100, "y": 100 }
      },
      {
        "id": "filter_active",
        "type": "transform",
        "label": "Filter Active Components",
        "config": {
          "operation": "filter",
          "rules": {
            "nodes": "status == 'active'",
            "edges": "weight > 0.3"
          }
        },
        "render_context": {
          "graph_name": "Active Components"
        },
        "position": { "x": 300, "y": 100 }
      },
      {
        "id": "export_plantuml",
        "type": "export",
        "label": "Export PlantUML",
        "config": {
          "format": "plantuml",
          "output": "diagrams/components.puml"
        },
        "render_context": {
          "plantuml": {
            "skin": "corporate",
            "direction": "top to bottom"
          }
        },
        "position": { "x": 400, "y": 250 }
      }
    ],
    "edges": [
      {
        "id": "import_to_filter",
        "source": "import_data",
        "target": "filter_active",
        "type": "data_flow"
      },
      {
        "id": "filter_to_plantuml",
        "source": "filter_active", 
        "target": "export_plantuml",
        "type": "data_flow"
      }
    ]
  }
}
```

### **Execution Model**

#### **Node Types**
- **IMPORT**: Entry points that load graph data from external sources
- **TRANSFORM**: Operations that modify graph data (filter, group, aggregate, etc.)
- **EXPORT**: Terminal nodes that generate output files/content

#### **Execution Flow**
- **Parallel Execution**: Independent branches can execute concurrently
- **Multiple Endpoints**: Fan-out to different export formats
- **Render Context Flow**: Metadata accumulates along execution paths

#### **DAG Validation Rules**
- No cycles allowed in the execution graph
- Import nodes must have no incoming edges
- Export nodes must have no outgoing edges  
- Transform nodes must have both incoming and outgoing edges
- All nodes must be reachable from at least one import node

### **Import Node Configuration**

Import nodes support multiple data sources that can be combined into a single graph scope:

**Import Types:**
- `csv_files`: CSV files from filesystem (nodes.csv, edges.csv, layers.csv)
- `csv_upload`: Uploaded CSV files via web interface
- `gml_file`: GML (Graph Modeling Language) file from filesystem
- `gml_upload`: Uploaded GML file via web interface
- `http_endpoint`: Dynamic data from HTTP/REST API
- `database_query`: Direct database query results

**Merge Strategies:**
- `append`: Add all imported data to existing graph
- `merge`: Smart merge based on node/edge IDs (updates existing, adds new)
- `replace`: Replace entire graph with imported data
- `union`: Combine graphs with conflict resolution

**Example Configuration:**
```json
{
  "imports": [
    {
      "id": "base_architecture",
      "type": "csv_files",
      "sources": {
        "nodes": "data/nodes.csv",
        "edges": "data/edges.csv",
        "layers": "data/layers.csv"
      }
    },
    {
      "id": "service_registry",
      "type": "http_endpoint",
      "sources": {
        "url": "https://api.company.com/services/graph",
        "method": "GET",
        "headers": {
          "Authorization": "Bearer ${SERVICE_TOKEN}"
        },
        "format": "json"
      }
    }
  ],
  "merge_strategy": "merge",
  "conflict_resolution": "keep_last"
}
```

### **Export Node Configuration**

**Built-in Exporters (no template parameter):**
- `plantuml`: Uses internal PlantUML generation logic
- `mermaid`: Uses internal Mermaid generation logic  
- `dot`: Uses internal DOT/Graphviz generation logic
- `json`: Uses internal JSON serialization
- `csv`: Uses internal CSV export for nodes/edges/layers

**Custom Exporter (requires template parameter):**
- `custom`: Uses user-provided Handlebars template
- Receives full graph context + resolved render context

```json
{
  "format": "custom",
  "template": "reports/analysis.html.hbs",
  "output": "reports/analysis.html"
}
```

## Hierarchical Navigation and Graph Inspection

### **Navigation Hierarchy**

The system supports hierarchical navigation from project level down to individual graph inspection:

```
Project → Plan → Workflow (DAG) → Node Inspection → Graph Visualization
```

### **GraphQL Schema**

```graphql
type Project {
  id: ID!
  name: String!
  description: String
  plans: [Plan!]!
  activeExecutions: [ExecutionState!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type Plan {
  id: ID!
  project: Project!
  name: String!
  dag: ExecutionDAG!
  executions: [ExecutionState!]!
  # Get graph at any plan node in the DAG
  graphAtPlanNode(planNodeId: ID!, executionId: ID): GraphObject
  # Get all inspectable points in the plan
  inspectionPoints(executionId: ID): [GraphInspectionPoint!]!
}

type ExecutionState {
  id: ID!
  plan: Plan!
  status: ExecutionStatus!
  graphObjects: [GraphObject!]!
  planNodeScopes: [GraphScope!]!
  currentPlanNode: ID
  # Get all available graphs for inspection
  inspectableGraphs: [GraphInspectionPoint!]!
  # Navigation breadcrumbs
  executionPath: [ExecutionPathPlanNode!]!
}

type GraphInspectionPoint {
  planNodeId: ID!
  planNodeName: String!
  planNodeType: PlanNodeType!
  graphRef: ID!
  graphObject: GraphObject!
  renderContext: JSON!
  canInspect: Boolean!
  executionStatus: PlanNodeExecutionStatus!
  lastUpdated: DateTime
}

type ExecutionPathPlanNode {
  planNodeId: ID!
  planNodeName: String!
  status: PlanNodeExecutionStatus!
  startedAt: DateTime
  completedAt: DateTime
  graphRef: ID
}

type GraphScope {
  id: ID!
  planNodeId: ID!
  graphRef: ID!
  renderContext: JSON!
}

enum PlanNodeType {
  IMPORT
  TRANSFORM
  EXPORT
}

enum PlanNodeExecutionStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
  SKIPPED
}
```

### **GraphQL Queries for Hierarchical Navigation**

**Project Overview:**
```graphql
query GetProject($projectId: ID!) {
  project(id: $projectId) {
    id
    name
    description
    plans {
      id
      name
      dag {
        nodes {
          id
          type
          label
        }
      }
      activeExecutions: executions(status: [RUNNING, PENDING]) {
        id
        status
        currentPlanNode
        startedAt
      }
    }
  }
}
```

**Graph at Specific Plan Node:**
```graphql
query GetGraphAtPlanNode($projectId: ID!, $planId: ID!, $planNodeId: ID!, $executionId: ID) {
  project(id: $projectId) {
    plan(id: $planId) {
      graphAtPlanNode(planNodeId: $planNodeId, executionId: $executionId) {
        id
        hash
        createdByPlanNode
        nodes {
          id
          label
          layer
          x
          y
          weight
        }
        edges {
          id
          source
          target
          weight
          layer
        }
        layers {
          id
          name
          color
          description
        }
        metadata {
          nodeCount
          edgeCount
          layerCount
          transformation
          createdAt
        }
      }
    }
  }
}
```

### **Live Execution Monitoring**

**Real-time Updates via GraphQL Subscriptions:**
```typescript
subscription ExecutionUpdates($executionId: ID!) {
  executionStateChanged(executionId: $executionId) {
    id
    status
    currentPlanNode
    planNodeScopes {
      planNodeId
      graphRef
      renderContext
    }
    errors {
      planNodeId
      message
      timestamp
    }
  }
}
```

### **Graph Inspection Specifications**

**Inspection Points Definition:**
1. **Import Nodes**: Always inspectable - show imported graph data
2. **Transform Nodes**: Inspectable after execution - show transformed graph
3. **Export Nodes**: Inspectable during/after execution - show final graph used for export
4. **Failed Nodes**: Always inspectable - show input graph for debugging

**Inspection Capabilities:**
```typescript
interface GraphInspectionCapabilities {
  // What can be inspected
  canViewGraph: boolean;          // Graph visualization available
  canViewMetadata: boolean;       // Graph metadata available
  canViewRenderContext: boolean;  // Render context available
  canViewTransformations: boolean; // Applied transformations
  canViewErrors: boolean;         // Error information if failed
  
  // Interactive capabilities
  canEditGraph: boolean;          // Graph data can be modified
  canReExecute: boolean;          // Node can be re-executed
  canExportGraph: boolean;        // Graph can be exported separately
  
  // Data availability
  graphDataSize: number;          // Size of graph data
  lastUpdated: DateTime;          // When graph was last updated
  cacheStatus: 'fresh' | 'stale' | 'loading';
}
```

## Frontend Architecture

### **Component Hierarchy**

```typescript
// Navigation hierarchy
ProjectOverview -> PlanSelector -> WorkflowViewer -> GraphVisualization
                                     ↓
                                PlanNodeInspector -> GraphVisualization
```

### **Updated GraphVisualization Component**

```typescript
interface GraphVisualizationProps {
  // Current props
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: GraphLayer[];
  
  // New props for hierarchical context
  context: 'project' | 'plan' | 'workflow' | 'plan-node-inspection';
  dataSource: {
    type: 'rest' | 'graphql';
    endpoint?: string;
    query?: string;
    variables?: Record<string, any>;
  };
  
  // Navigation context
  projectId: string;
  planId?: string;
  planNodeId?: string;
  executionId?: string;
  
  // Inspection capabilities
  enablePlanNodeInspection?: boolean;
  onPlanNodeInspect?: (planNodeId: string) => void;
  
  // DAG-specific props
  showExecutionPath?: boolean;
  highlightActiveNodes?: boolean;
  executionState?: ExecutionState;
  
  // Navigation callbacks
  onNavigateToProject?: () => void;
  onNavigateToPlan?: () => void;
  onNavigateToPlanNode?: (planNodeId: string) => void;
}
```

### **React Frontend Architecture**

- React frontend with TypeScript for type safety and modern development experience
- Modular component architecture with dynamic loading for large editors (ReactFlow, Isoflow)
- Component registry system enables pluggable editor components
- Development mode: separate React dev server with API proxy for hot reload
- Production mode: single binary serves embedded HTML shell loading CDN assets
- Frontend assets deployed to CDN (GitHub Pages/jsdelivr) via CI/CD for global distribution

### **URL Structure and Routing**

**Frontend Routes:**
```
/projects                                        # Project list
/projects/{project_id}                          # Project overview
/projects/{project_id}/plans                    # Plan list
/projects/{project_id}/plans/{plan_id}         # Plan DAG workflow
/projects/{project_id}/plans/{plan_id}/plan-nodes/{plan_node_id}  # Graph at plan node
/projects/{project_id}/plans/{plan_id}/executions/{execution_id}  # Live execution
/projects/{project_id}/plans/{plan_id}/executions/{execution_id}/plan-nodes/{plan_node_id}  # Graph at plan node in execution
```

## Data Editing Interfaces

- Multi-mode plan editor supporting three editing approaches:
  - Rich text YAML editor with syntax highlighting (legacy compatibility)
  - Visual ReactFlow editor for DAG-based plan construction
  - JSON patch editor for incremental real-time editing
- Spreadsheet-like interface for bulk data editing with tabs for layers, nodes, and edges
- Excel-like functionality including keyboard shortcuts, bulk operations, and data validation
- Real-time data synchronization across all editing interfaces via GraphQL subscriptions

## Graph Operations

- Graph versioning system with diff capabilities for tracking changes over time
- Advanced graph transformations and filtering for subset selection
- Connectivity analysis and path finding algorithms for graph insights
- Interactive graph visualization with drag-and-drop editing capabilities
- Performance optimization for large graphs (10,000+ nodes) with virtualization

### **Efficient Graph Object Management**

**Graph Object Storage:**
- Each transformation node creates a new unique graph object
- Graph objects are stored separately and referenced by ID
- Graph objects include hash for deduplication and caching
- Parent graph relationship tracked for lineage

**Benefits:**
- Eliminates graph data duplication in JSON structures
- Enables efficient caching and storage
- Supports graph object reuse across multiple export nodes
- Facilitates incremental execution with cached graph states

### **Caching Strategy**

```
- Project metadata: Cache for 5 minutes
- Plan DAG structure: Cache for 1 hour (invalidate on plan changes)
- Execution state: Cache for 10 seconds (real-time updates)
- Graph objects: Cache by hash indefinitely (immutable)
- Render context: Cache for 30 seconds (can change during execution)
```

## MCP API Integration

### **MCP Tool Categories**

**Plan Execution:**
- `execute_plan(plan_id, mode="full|partial|incremental")` 
- `execute_plan_node(plan_id, plan_node_id)`
- `get_execution_status(execution_id)`
- `cancel_execution(execution_id)`

**Plan Debugging:**
- `get_execution_trace(execution_id)`
- `get_plan_node_output(execution_id, plan_node_id)` 
- `get_error_details(execution_id, plan_node_id)`
- `validate_plan(plan_id)`

**Plan Management:**
- `create_plan(plan_data)`
- `update_plan(plan_id, changes)`
- `get_plan(plan_id)`
- `list_plans()`

### **Error Handling and Debugging**

```json
{
  "execution_id": "exec_123",
  "plan_node_id": "filter_active", 
  "error_type": "transformation_error",
  "message": "Filter expression 'status == active' failed: unknown field 'status'",
  "timestamp": "2025-01-15T10:30:15Z",
  "input_graph": {
    "node_count": 150,
    "edge_count": 200,
    "available_fields": ["id", "label", "layer", "weight"]
  },
  "stack_trace": [...],
  "execution_path": ["import_data", "filter_active"],
  "upstream_outputs": {
    "import_data": {
      "status": "success",
      "output_hash": "abc123"
    }
  }
}
```

## Deployment Model

- Single binary deployment containing both backend and frontend
- CDN-first asset delivery with local fallback for offline operation
- Automatic cache busting using Git commit hashes for asset versioning
- Cross-platform support (Linux, macOS, Windows) with hybrid TLS architecture
- Development workflow enables seamless frontend/backend development with hot reload

## Implementation Priorities

### **Phase 1: Core DAG Engine (2-4 weeks)**
1. Define JSON schema for flat DAG validation
2. Implement execution path resolution algorithm
3. Create render context flow engine with deep merge
4. Build basic node execution framework (import/transform/export)

### **Phase 2: API Foundation (4-6 weeks)**
1. Implement unified backend serving REST, GraphQL, and MCP
2. Add hierarchical REST endpoints for backward compatibility
3. Implement GraphQL schema with hierarchical navigation
4. Add real-time subscriptions for execution monitoring

### **Phase 3: Frontend Migration (6-8 weeks)**
1. ReactFlow integration with flat DAG visualization
2. Implement hierarchical navigation components
3. Update GraphVisualization for multi-context support
4. Add graph inspection capabilities at every DAG node

### **Phase 4: Advanced Features (6-8 weeks)**
1. JSON Patch support for collaborative editing
2. Parallel execution engine with dependency resolution
3. Performance optimization for large DAGs (1000+ nodes)
4. Advanced transformation operations (merge, split, aggregate)

### **Phase 5: Production Readiness (2-3 weeks)**
1. Optimize caching strategies based on usage patterns
2. Complete MCP API implementation
3. Deprecate legacy endpoints
4. Performance testing and optimization

## Conclusion

This design supports the evolution from simple linear pipelines to complex multi-branch workflows while maintaining strong typing, efficient data access patterns, and comprehensive graph inspection capabilities at every level of the execution hierarchy.

**Key Advantages:**
1. **Hierarchical Navigation**: Full drilling capability from project to individual node graphs
2. **Real-time Monitoring**: Live execution updates with graph inspection
3. **Flexible API Architecture**: Gradual migration from REST to GraphQL with MCP integration
4. **Efficient Data Management**: Graph object referencing eliminates duplication
5. **Developer Experience**: Visual editing with JSON-first approach and strong typing
6. **Scalability**: Performance optimizations for large graphs and complex workflows

The flat DAG structure provides a robust foundation for complex graph transformation workflows with strong GraphQL integration, enabling powerful querying, real-time collaboration, and visual editing capabilities.