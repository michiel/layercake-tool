# Layercake Plan DAG Design Specification

## Overview

This document defines the DAG (Directed Acyclic Graph) structure for Layercake execution plans, specifying the JSON representation, graph-based data model, and visual editing capabilities. Plans represent execution workflows that can branch and have multiple endpoints, moving beyond simple linear pipelines.

## Current Understanding

### Core Concepts

#### **Plan Data Format Strategy**
- **Internal Representation**: JSON with flat DAG structure (nodes[] + edges[])
- **Legacy Support**: YAML import/conversion (read-only)
- **Primary Interface**: Visual DAG editor with direct JSON import/export
- **Collaboration**: JSON Patch operations for real-time editing
- **Query Support**: GraphQL-friendly flat structure with strong typing

#### **DAG Execution Model**

1. **Execution Nodes**: Discrete operations in the execution graph
2. **Node Types**: Import, Transform, Export, Control
3. **Execution Flow**: Defined by edges, supports branching and multiple endpoints
4. **Render Context**: Accumulated metadata and configuration flowing through execution paths
5. **Parallel Execution**: Independent branches can execute concurrently

#### **Visual Editing**
- ReactFlow-based visual editor for DAG construction
- Direct JSON manipulation with schema validation
- Real-time preview of execution flow and render context
- Import/export of complete plan JSON

## Design Requirements

### ✅ Confirmed Specifications
- **Flat DAG Structure**: JSON with `nodes[]` and `edges[]` arrays for GraphQL compatibility
- **Execution Model**: Non-linear workflows with branching and multiple endpoints
- **JSON-First**: Native JSON representation with direct import/export
- **Visual Editing**: ReactFlow-based DAG editor as primary interface
- **Strong Typing**: GraphQL schema support with well-defined node/edge types

### 🎯 Design Goals

#### 1. **Graph Database Compatibility**
- Flat structure enables efficient querying and indexing
- Direct mapping to graph database storage models
- Strong typing for GraphQL schema generation

#### 2. **Execution Flexibility**
- Support for parallel execution branches
- Multiple export endpoints per plan
- Conditional execution flows
- Fan-out and fan-in patterns

#### 3. **Developer Experience**
- Direct JSON manipulation for power users
- Visual editing for intuitive workflow construction
- Schema validation with clear error messages
- Real-time collaboration via JSON Patch

## JSON DAG Structure

### **Flat DAG Plan Schema**

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
          "sources": {
            "nodes": "architecture/nodes.csv",
            "edges": "architecture/edges.csv", 
            "layers": "architecture/layers.csv"
          }
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
        "id": "group_by_layer",
        "type": "transform", 
        "label": "Group by Layer",
        "config": {
          "operation": "group",
          "group_by": "layer",
          "aggregate_edges": true
        },
        "render_context": {
          "layout": "layered",
          "show_aggregates": true
        },
        "position": { "x": 500, "y": 100 }
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
      },
      {
        "id": "export_mermaid", 
        "type": "export",
        "label": "Export Mermaid",
        "config": {
          "format": "mermaid",
          "output": "diagrams/flowchart.mmd"
        },
        "render_context": {
          "mermaid": {
            "theme": "default",
            "direction": "TD"
          }
        },
        "position": { "x": 600, "y": 250 }
      },
      {
        "id": "export_summary",
        "type": "export",
        "label": "Export Summary",
        "config": {
          "format": "json",
          "output": "reports/summary.json"
        },
        "render_context": {
          "include_metadata": true,
          "format_style": "pretty"
        },
        "position": { "x": 700, "y": 100 }
      },
      {
        "id": "export_custom_report",
        "type": "export",
        "label": "Export Custom Report",
        "config": {
          "format": "custom",
          "template": "reports/architecture-analysis.html.hbs",
          "output": "reports/architecture-analysis.html"
        },
        "render_context": {
          "report": {
            "title": "Architecture Analysis Report",
            "include_diagrams": true,
            "show_metrics": true
          }
        },
        "position": { "x": 500, "y": 350 }
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
        "id": "filter_to_group", 
        "source": "filter_active",
        "target": "group_by_layer",
        "type": "data_flow"
      },
      {
        "id": "filter_to_plantuml",
        "source": "filter_active", 
        "target": "export_plantuml",
        "type": "data_flow"
      },
      {
        "id": "group_to_mermaid",
        "source": "group_by_layer",
        "target": "export_mermaid", 
        "type": "data_flow"
      },
      {
        "id": "group_to_summary",
        "source": "group_by_layer",
        "target": "export_summary",
        "type": "data_flow"
      },
      {
        "id": "filter_to_custom",
        "source": "filter_active",
        "target": "export_custom_report",
        "type": "data_flow"
      }
    ]
  }
}
```

### **Efficient Graph Scope and Template Context Example**

```json
{
  "execution_state": {
    "execution_id": "exec_456",
    "plan_id": "plan_123",
    
    "graph_objects": {
      "graph_1": {
        "id": "graph_1",
        "hash": "abc123def456",
        "created_by_node": "import_data",
        "nodes": [
          {"id": "node_1", "label": "API Gateway", "layer": "gateway", "status": "active"},
          {"id": "node_2", "label": "User Service", "layer": "services", "status": "active"}, 
          {"id": "node_3", "label": "Legacy DB", "layer": "data", "status": "deprecated"}
        ],
        "edges": [
          {"id": "edge_1", "source": "node_1", "target": "node_2", "weight": 0.8}
        ],
        "layers": [
          {"id": "gateway", "label": "API Gateway", "color": "#blue"},
          {"id": "services", "label": "Services", "color": "#green"},
          {"id": "data", "label": "Data Layer", "color": "#red"}
        ],
        "metadata": {
          "node_count": 3,
          "edge_count": 1,
          "layer_count": 3
        }
      },
      "graph_2": {
        "id": "graph_2", 
        "hash": "def456ghi789",
        "created_by_node": "filter_active",
        "parent_graph": "graph_1",
        "nodes": [
          {"id": "node_1", "label": "API Gateway", "layer": "gateway", "status": "active"},
          {"id": "node_2", "label": "User Service", "layer": "services", "status": "active"}
        ],
        "edges": [
          {"id": "edge_1", "source": "node_1", "target": "node_2", "weight": 0.8}
        ],
        "layers": [
          {"id": "gateway", "label": "API Gateway", "color": "#blue"},
          {"id": "services", "label": "Services", "color": "#green"}
        ],
        "metadata": {
          "node_count": 2,
          "edge_count": 1, 
          "layer_count": 2,
          "transformation": "filter(status == 'active')"
        }
      }
    },
    
    "node_scopes": {
      "import_data": {
        "scope_id": "scope_1",
        "graph_ref": "graph_1",
        "render_context": {
          "graph_name": "Architecture Overview",
          "theme": "corporate",
          "base_layout": "hierarchical"
        }
      },
      "filter_active": {
        "scope_id": "scope_2",
        "graph_ref": "graph_2", 
        "render_context": {
          "graph_name": "Active Components"
        }
      },
      "export_mermaid": {
        "scope_id": "scope_3",
        "graph_ref": "graph_2",
        "render_context": {
          "mermaid": {
            "theme": "default",
            "direction": "TD"
          }
        }
      }
    },
    
    "template_context_for_export_mermaid": {
      "graph_ref": "graph_2",
      "resolved_metadata": {
        "graph_name": "Active Components",
        "theme": "corporate",
        "base_layout": "hierarchical", 
        "mermaid": {
          "theme": "default",
          "direction": "TD"
        }
      },
      "execution": {
        "node_id": "export_mermaid",
        "timestamp": "2025-01-15T10:30:00Z",
        "path": ["import_data", "filter_active", "export_mermaid"],
        "scope_id": "scope_3"
      }
    }
  }
}
```

### **Template Resolution at Export Time**

```json
{
  "export_template_context": {
    "execution_id": "exec_456",
    "node_id": "export_mermaid",
    "graph": {
      "nodes": [
        {"id": "node_1", "label": "API Gateway", "layer": "gateway", "status": "active"},
        {"id": "node_2", "label": "User Service", "layer": "services", "status": "active"}
      ],
      "edges": [
        {"id": "edge_1", "source": "node_1", "target": "node_2", "weight": 0.8}
      ],
      "layers": [
        {"id": "gateway", "label": "API Gateway", "color": "#blue"},
        {"id": "services", "label": "Services", "color": "#green"}
      ]
    },
    "metadata": {
      "graph_name": "Active Components",
      "theme": "corporate",
      "base_layout": "hierarchical",
      "mermaid": {
        "theme": "default",
        "direction": "TD"
      }
    },
    "execution": {
      "node_id": "export_mermaid",
      "timestamp": "2025-01-15T10:30:00Z",
      "path": ["import_data", "filter_active", "export_mermaid"],
      "scope_id": "scope_3",
      "graph_id": "graph_2",
      "graph_hash": "def456ghi789"
    }
  }
}
```

### **GraphQL Schema Types**

```typescript
type Plan = {
  id: ID!
  version: String!
  metadata: PlanMetadata!
  dag: ExecutionDAG!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type PlanMetadata = {
  name: String!
  description: String
  schemaVersion: String!
  tags: [String!]
}

type ExecutionDAG = {
  nodes: [ExecutionNode!]!
  edges: [ExecutionEdge!]!
}

type ExecutionNode = {
  id: ID!
  type: NodeType!
  label: String!
  config: JSON!
  renderContext: JSON
  position: Position
}

enum NodeType {
  IMPORT
  TRANSFORM  
  EXPORT
}

type ExecutionEdge = {
  id: ID!
  source: ID!
  target: ID!
  type: EdgeType!
  condition: String
}

enum EdgeType {
  DATA_FLOW
  DEPENDENCY
}

type Position = {
  x: Float!
  y: Float!
}

type ExecutionState = {
  id: ID!
  planId: ID!
  status: ExecutionStatus!
  startedAt: DateTime!
  completedAt: DateTime
  currentNode: ID
  graphObjects: [GraphObject!]!
  scopes: [GraphScope!]!
  errors: [ExecutionError!]!
}

enum ExecutionStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
  CANCELLED
}

type GraphScope = {
  id: ID!
  nodeId: ID!
  graphRef: ID!
  renderContext: JSON!
}

type GraphObject = {
  id: ID!
  hash: String!
  createdByNode: ID!
  parentGraph: ID
  nodes: [GraphNode!]!
  edges: [GraphEdge!]!
  layers: [GraphLayer!]!
  metadata: GraphMetadata!
}

type GraphMetadata = {
  nodeCount: Int!
  edgeCount: Int!
  layerCount: Int!
  transformation: String
  createdAt: DateTime!
}

type ExecutionError = {
  nodeId: ID!
  errorType: String!
  message: String!
  timestamp: DateTime!
  stackTrace: [String!]
  context: JSON
}
```

## Execution Model Specifications

### 1. **Node Type Definitions**
```
IMPORT: Entry points that load graph data from external sources
TRANSFORM: Operations that modify graph data (filter, group, aggregate, etc.)
EXPORT: Terminal nodes that generate output files/content
```

### 2. **Render Context Flow Rules**
```
1. Context flows along execution paths from import to export nodes
2. Each node can contribute additional context properties
3. Downstream context overrides upstream for matching keys  
4. Deep merge for nested objects (format-specific configurations)
5. Export nodes receive accumulated context from their execution path
```

### 3. **Execution Node Behavior**
```
IMPORT: Load source data + establish initial render context
TRANSFORM: Process input graph + merge render context changes
EXPORT: Generate output using input graph + resolved render context
```

### 4. **DAG Validation Rules**
```
- No cycles allowed in the execution graph
- Import nodes must have no incoming edges
- Export nodes must have no outgoing edges  
- Transform nodes must have both incoming and outgoing edges
- All nodes must be reachable from at least one import node
```

### 5. **Parallel Execution Model**
```
- Independent branches can execute concurrently
- Multiple export endpoints enable different output formats
- Fan-out: One node can feed multiple downstream nodes
- Fan-in: Multiple nodes can feed into one downstream node
- Execution completes when all terminal nodes finish
```

### 6. **Export Node Configuration**
```
Built-in Exporters (no template parameter):
- format: "plantuml" - Uses internal PlantUML generation logic
- format: "mermaid" - Uses internal Mermaid generation logic  
- format: "dot" - Uses internal DOT/Graphviz generation logic
- format: "json" - Uses internal JSON serialization
- format: "csv" - Uses internal CSV export for nodes/edges/layers
- format: "gml" - Uses internal GML (Graph Modeling Language) export
- format: "graphml" - Uses internal GraphML export

Custom Exporter (requires template parameter):
- format: "custom" - Uses user-provided Handlebars template
- template: "path/to/template.hbs" - Required for custom format
- Receives full graph context + resolved render context
- Supports any output format via templating

Config Structure:
{
  "format": "plantuml|mermaid|dot|json|csv|gml|graphml|custom",
  "template": "path/to/template.hbs", // Required only for format: "custom"
  "output": "path/to/output/file"
}

Example Configurations:
// Built-in exporter
{"format": "plantuml", "output": "diagrams/arch.puml"}

// Custom exporter  
{"format": "custom", "template": "reports/analysis.html.hbs", "output": "reports/analysis.html"}
```

## Implementation Recommendations

### **Phase 1: Core DAG Engine**
1. Define JSON schema for flat DAG validation
2. Implement execution path resolution algorithm
3. Create render context flow engine with deep merge
4. Build basic node execution framework (import/transform/export)

### **Phase 2: GraphQL Integration**
1. Generate GraphQL schema from DAG types
2. Implement efficient DAG querying and mutations
3. Add real-time subscriptions for plan changes
4. Create plan validation and execution APIs

### **Phase 3: Visual Editor**
1. ReactFlow integration with flat DAG visualization
2. Drag-and-drop node creation and connection
3. Real-time preview of execution paths and context flow
4. Direct JSON editing with schema validation

### **Phase 4: Advanced Features**
1. JSON Patch support for collaborative editing
2. Parallel execution engine with dependency resolution
3. Performance optimization for large DAGs (1000+ nodes)
4. Advanced transformation operations (merge, split, aggregate)

### **Migration Strategy from YAML**
```
1. Parse legacy YAML plans to extract transformation sequences
2. Convert linear transformations to flat DAG nodes and edges
3. Map YAML render options to new render context model
4. Validate generated JSON against new schema
5. Provide YAML import tool with validation feedback
```

## Execution Flow Examples

### **Simple Linear Execution**
```
[import_data] → [filter_active] → [export_plantuml]

Nodes: 3, Edges: 2, Endpoints: 1
```

### **Multi-Format Export Fan-Out**
```
[import_data] → [filter_active] → [export_plantuml]
                                 ↘ [export_mermaid] 
                                 ↘ [export_json]

Nodes: 5, Edges: 4, Endpoints: 3
```

### **Complex Multi-Branch Workflow**
```
[import_data] → [filter_active] → [export_overview]
              ↘ [group_layers] → [export_summary]
                              ↘ [transform_aggregates] → [export_detailed]
                                                       ↘ [export_metrics]

Nodes: 7, Edges: 7, Endpoints: 4
```

### **Parallel Processing with Fan-In**
```
[import_nodes] ↘                    ↗ [merge_data] → [export_combined]
[import_edges] → [validate_integrity] 
[import_layers] ↗                   ↘ [export_validation_report]

Nodes: 6, Edges: 6, Endpoints: 2
```

## Detailed Specifications

### **1. Node and Graph Element Referencing**
```
- Plan nodes: Referenced by unique string ID within the plan's node set
- Graph elements: nodes, edges, layers each have unique string IDs within their respective collections
- Cross-references: Plan node IDs and graph element IDs are independent namespaces
- Persistence: IDs must be stable across plan modifications and execution runs
```

### **2. Template Rendering Context**
```
Export nodes receive complete graph context:
{
  "nodes": [...],     // Full array of graph nodes with all properties
  "edges": [...],     // Full array of graph edges with all properties  
  "layers": [...],    // Full array of graph layers with all properties
  "metadata": {...},  // Accumulated render context from execution path
  "execution": {      // Execution-specific context
    "node_id": "export_plantuml",
    "timestamp": "2025-01-15T10:30:00Z",
    "path": ["import_data", "filter_active", "export_plantuml"]
  }
}
```

### **3. Graph Scope Transformation Model**
```
Graph Object Management:
- Each transformation node creates a new unique graph object
- Graph objects are stored separately and referenced by ID
- Graph objects include hash for deduplication and caching
- Parent graph relationship tracked for lineage

Scope Management: 
- Node scopes reference graph objects by ID (not embedded)
- Multiple scopes can reference the same graph object (shared state)
- Render context stored separately from graph data
- Export nodes resolve graph object at template rendering time

Benefits:
- Eliminates graph data duplication in JSON structures
- Enables efficient caching and storage
- Supports graph object reuse across multiple export nodes
- Facilitates incremental execution with cached graph states
```

### **4. Interactive Execution Model**
```
Execution Modes:
- Full execution: Run entire DAG from import nodes to all export endpoints
- Partial execution: Run from specified node to downstream endpoints  
- Single node: Execute individual node with cached inputs
- Incremental: Re-execute only nodes affected by changes

Change Propagation:
- Node modification triggers re-execution of that node + all downstream nodes
- Graph data changes propagate through transformation scopes
- Render context changes affect all downstream export nodes
```

### **5. MCP Integration Requirements**
```
MCP Tool Categories:

plan_execution:
  - execute_plan(plan_id, mode="full|partial|incremental") 
  - execute_node(plan_id, node_id)
  - get_execution_status(execution_id)
  - cancel_execution(execution_id)

plan_debugging:
  - get_execution_trace(execution_id)
  - get_node_output(execution_id, node_id) 
  - get_error_details(execution_id, node_id)
  - validate_plan(plan_id)

plan_management:
  - create_plan(plan_data)
  - update_plan(plan_id, changes)
  - get_plan(plan_id)
  - list_plans()
```

### **6. Error Handling and Debugging**
```
Error Context Structure:
{
  "execution_id": "exec_123",
  "node_id": "filter_active", 
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

Debugging Tools:
- Step-through execution with breakpoints
- Graph data inspection at each transformation scope
- Render context tracking through execution paths
- Performance profiling for large graphs
- Execution replay from cached intermediate states
```

### **7. Efficiency and Performance Optimizations**
```
Graph Object Storage:
- Unique graph objects stored once with content-based hashing
- Duplicate graphs detected and reused via hash comparison
- Graph objects can be persisted to disk/database for large executions
- Lazy loading of graph data only when needed for template rendering

Memory Management:
- Execution state contains only graph references, not full graph data
- Template context resolved on-demand by dereferencing graph objects
- Garbage collection of unreferenced graph objects after execution
- Streaming export for large graphs without full memory loading

Caching Strategy:
- Graph object hashes enable cache hits across different executions
- Incremental execution reuses cached graph objects when nodes unchanged
- Template rendering results can be cached by graph hash + metadata hash
- Database queries optimized using graph object references and indexes
```

## MCP Tool Definitions

### **Plan Execution Tools**

```json
{
  "name": "execute_plan",
  "description": "Execute a plan with specified execution mode",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_id": {"type": "string", "description": "Unique plan identifier"},
      "mode": {
        "type": "string", 
        "enum": ["full", "partial", "incremental"],
        "description": "Execution mode"
      },
      "start_node": {"type": "string", "description": "Starting node for partial execution"},
      "variables": {"type": "object", "description": "Runtime variables for execution"}
    },
    "required": ["plan_id", "mode"]
  }
}

{
  "name": "execute_node", 
  "description": "Execute a single node with cached inputs",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_id": {"type": "string"},
      "node_id": {"type": "string"},
      "use_cache": {"type": "boolean", "default": true}
    },
    "required": ["plan_id", "node_id"]
  }
}

{
  "name": "get_execution_status",
  "description": "Get current status and progress of plan execution", 
  "inputSchema": {
    "type": "object",
    "properties": {
      "execution_id": {"type": "string"}
    },
    "required": ["execution_id"]
  }
}
```

### **Plan Debugging Tools**

```json
{
  "name": "get_execution_trace",
  "description": "Get detailed execution trace with timing and data flow",
  "inputSchema": {
    "type": "object", 
    "properties": {
      "execution_id": {"type": "string"},
      "include_graph_data": {"type": "boolean", "default": false},
      "include_render_context": {"type": "boolean", "default": true}
    },
    "required": ["execution_id"]
  }
}

{
  "name": "get_node_output",
  "description": "Get the output graph and context from a specific node execution",
  "inputSchema": {
    "type": "object",
    "properties": {
      "execution_id": {"type": "string"},
      "node_id": {"type": "string"},
      "format": {"type": "string", "enum": ["json", "summary"], "default": "json"}
    },
    "required": ["execution_id", "node_id"]
  }
}

{
  "name": "get_error_details",
  "description": "Get comprehensive error information for failed node",
  "inputSchema": {
    "type": "object",
    "properties": {
      "execution_id": {"type": "string"},
      "node_id": {"type": "string"},
      "include_context": {"type": "boolean", "default": true}
    },
    "required": ["execution_id", "node_id"]
  }
}

{
  "name": "validate_plan",
  "description": "Validate plan structure and configuration",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_id": {"type": "string"},
      "check_dependencies": {"type": "boolean", "default": true},
      "check_templates": {"type": "boolean", "default": true}
    },
    "required": ["plan_id"]
  }
}
```

### **Plan Management Tools**

```json
{
  "name": "create_plan",
  "description": "Create a new execution plan",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_data": {"type": "object", "description": "Complete plan JSON structure"},
      "validate": {"type": "boolean", "default": true}
    },
    "required": ["plan_data"]
  }
}

{
  "name": "update_plan",
  "description": "Update existing plan using JSON Patch operations",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_id": {"type": "string"},
      "changes": {"type": "array", "description": "JSON Patch operations"},
      "incremental_execution": {"type": "boolean", "default": true}
    },
    "required": ["plan_id", "changes"]
  }
}

{
  "name": "get_plan",
  "description": "Retrieve complete plan definition",
  "inputSchema": {
    "type": "object",
    "properties": {
      "plan_id": {"type": "string"},
      "include_execution_history": {"type": "boolean", "default": false}
    },
    "required": ["plan_id"]
  }
}
```

## Conclusion

The flat DAG structure provides a robust foundation for complex graph transformation workflows with strong GraphQL integration. The JSON-first approach enables powerful querying, real-time collaboration, and visual editing. Key implementation priorities are:

1. **Core DAG engine** with execution path resolution
2. **Render context flow** with conflict resolution 
3. **GraphQL schema** with efficient querying
4. **Visual editor** with ReactFlow integration
5. **Parallel execution** with dependency management

This design supports the evolution from simple linear pipelines to complex multi-branch workflows while maintaining strong typing and efficient data access patterns.
