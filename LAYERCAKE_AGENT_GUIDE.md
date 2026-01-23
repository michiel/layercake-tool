# Layercake Agent Guide

This guide explains how to use the `layercake` CLI tool's query interface to interact with layercake databases. This is intended for AI agents working alongside users who are viewing the same data through the web interface.

## Overview

Layercake is a graph-based planning and data flow visualisation tool. The `layercake query` command provides a REPL-style interface for programmatic access to:
- Projects and plans
- Datasets and their executions
- Graph nodes representing computations
- Edges representing data flow
- Graph artefacts (rendered outputs in various formats)

## Basic Command Structure

```bash
layercake query \
  --database <path> \
  --entity <entity-type> \
  --action <action-type> \
  --project <project-id> \
  [--plan <plan-id>] \
  [--payload-json '<json>'] \
  [--payload-file <path>] \
  [--pretty]
```

### Required Parameters
- `--database`: Path to the SQLite database (typically `layercake.db`)
- `--entity`: What you're working with (datasets, plans, nodes, edges, exports)
- `--action`: What you want to do (list, get, create, update, delete, move, download)
- `--project`: Project ID (required for most operations)

### Optional Parameters
- `--plan`: Plan ID (optional if working with the default plan)
- `--payload-json`: Inline JSON payload as a string
- `--payload-file`: Path to file containing JSON payload
- `--pretty`: Format output as human-readable JSON
- `--dry-run`: Validate payload without executing the action (Phase 1.6)
- `--session`: Session identifier for future auth support

## Quick Reference

| Entity | Available Actions | Purpose |
|--------|------------------|---------|
| `datasets` | list, get | Input data sources |
| `plans` | list, get | Workflow containers |
| `nodes` | list, get, create, update, delete, move, traverse, search, batch, clone | DAG nodes (datasets, graphs, artefacts) |
| `edges` | create, update, delete | Connections between nodes |
| `exports` | download | Graph visualisation exports |
| `schema` | get, list | Introspect available types and actions (Phase 1.4) |
| `analysis` | get | Graph structure analysis (Phase 2.3) |
| `annotations` | create, list, get, update, delete | Key-value annotations for nodes and edges (Phase 2.4) |

## Entities and Available Actions

### Datasets
**Purpose**: Input data sources that feed into graph computations.

**Actions:**
- `list` - List all datasets in a project
- `get` - Retrieve specific dataset by ID

**Examples:**
```bash
# List all datasets in project 34
layercake query --database layercake.db \
  --entity datasets --action list --project 34 --pretty

# Get specific dataset
layercake query --database layercake.db \
  --entity datasets --action get --project 34 \
  --payload-json '{"id":180}' --pretty
```

**Response includes:**
- Dataset metadata (ID, filename, processing state)
- Execution state (completed, failed, pending)
- Error messages if execution failed

### Plans
**Purpose**: Container for organising related graphs and workflows.

**Actions:**
- `list` - List all plans in a project
- `get` - Retrieve specific plan by ID

**Examples:**
```bash
# List all plans in project 34
layercake query --database layercake.db \
  --entity plans --action list --project 34 --pretty

# Get specific plan
layercake query --database layercake.db \
  --entity plans --action get --project 34 \
  --payload-json '{"id":37}' --pretty
```

**Response includes:**
- Plan metadata (name, description, status, version)
- Creation and update timestamps
- Dependencies and tags

### Nodes
**Purpose**: Nodes in the plan DAG (Directed Acyclic Graph). Can be datasets, graphs, or graph artefacts.

**Node Types:**
- `DataSetNode` - References an input dataset
- `GraphNode` - Represents a computation/transformation
- `GraphArtefactNode` - Rendered output in a specific format

**Actions:**
- `list` - List all nodes and edges in a plan (returns full DAG, supports filtering - Phase 1.1)
- `get` - Get a single node by ID with execution metadata enrichment (Phase 1.2)
- `create` - Create a new node
- `update` - Update an existing node
- `delete` - Delete a node
- `move` - Change a node's position
- `traverse` - Traverse graph from a starting node (Phase 1.3)
- `search` - Search nodes by text and topology (Phase 2.2)
- `batch` - Execute multiple operations atomically (Phase 2.1)
- `clone` - Clone a node with optional position/label changes (Phase 2.5)

**Examples:**
```bash
# List all nodes and edges in plan 37
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 --pretty

# Create a graph node
layercake query --database layercake.db \
  --entity nodes --action create --project 34 --plan 37 \
  --payload-json '{
    "nodeType": "GraphNode",
    "position": {"x": 500, "y": 500},
    "metadata": {"label": "My Graph"},
    "config": {"metadata": {}}
  }' --pretty

# Update a node's label
layercake query --database layercake.db \
  --entity nodes --action update --project 34 --plan 37 \
  --payload-json '{
    "nodeId": "graph_42b0374af121",
    "metadata": {"label": "Updated Label"}
  }' --pretty

# Move a node
layercake query --database layercake.db \
  --entity nodes --action move --project 34 --plan 37 \
  --payload-json '{
    "nodeId": "graph_42b0374af121",
    "position": {"x": 600, "y": 1200}
  }' --pretty

# Delete a node
layercake query --database layercake.db \
  --entity nodes --action delete --project 34 --plan 37 \
  --payload-json '{
    "nodeId": "graph_42b0374af121"
  }' --pretty

# Filter nodes by type (Phase 1.1)
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 \
  --payload-json '{"nodeType":"GraphNode"}' --pretty

# Filter nodes by label pattern (Phase 1.1)
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 \
  --payload-json '{"labelPattern":"copilot"}' --pretty

# Filter nodes by position bounds (Phase 1.1)
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 \
  --payload-json '{
    "bounds":{"minX":0,"maxX":500,"minY":0,"maxY":500}
  }' --pretty

# Get single node with enriched metadata (Phase 1.2)
layercake query --database layercake.db \
  --entity nodes --action get --project 34 --plan 37 \
  --payload-json '{"nodeId":"graph_42b0374af121"}' --pretty

# Traverse downstream from a dataset (Phase 1.3)
layercake query --database layercake.db \
  --entity nodes --action traverse --project 34 --plan 37 \
  --payload-json '{
    "startNode":"dataset_fb5f819c7089",
    "direction":"downstream",
    "maxDepth":3
  }' --pretty

# Traverse upstream from an artefact (Phase 1.3)
layercake query --database layercake.db \
  --entity nodes --action traverse --project 34 --plan 37 \
  --payload-json '{
    "startNode":"graphartefact_af32487bb03c",
    "direction":"upstream",
    "maxDepth":5
  }' --pretty

# Find path between two nodes (Phase 1.3)
layercake query --database layercake.db \
  --entity nodes --action traverse --project 34 --plan 37 \
  --payload-json '{
    "startNode":"dataset_fb5f819c7089",
    "endNode":"graphartefact_af32487bb03c",
    "findPath":true
  }' --pretty

# Search nodes by text (Phase 2.2)
layercake query --database layercake.db \
  --entity nodes --action search --project 34 --plan 37 \
  --payload-json '{
    "query":"copilot",
    "fields":["label","description"]
  }' --pretty

# Find isolated nodes (Phase 2.2)
layercake query --database layercake.db \
  --entity nodes --action search --project 34 --plan 37 \
  --payload-json '{
    "query":"",
    "edgeFilter":"isolated"
  }' --pretty

# Clone a node (Phase 2.5)
layercake query --database layercake.db \
  --entity nodes --action clone --project 34 --plan 37 \
  --payload-json '{
    "nodeId":"graph_42b0374af121",
    "position":{"x":800,"y":600},
    "updateLabel":"Copilot 003 (copy)"
  }' --pretty

# Batch operations - create nodes and edge (Phase 2.1)
layercake query --database layercake.db \
  --entity nodes --action batch --project 34 --plan 37 \
  --payload-json '{
    "operations": [
      {
        "op": "createNode",
        "id": "temp_node_1",
        "data": {
          "nodeType": "GraphNode",
          "position": {"x": 1500, "y": 500},
          "metadata": {"label": "Batch Node 1"},
          "config": {"metadata": {}}
        }
      },
      {
        "op": "createNode",
        "id": "temp_node_2",
        "data": {
          "nodeType": "GraphNode",
          "position": {"x": 1700, "y": 500},
          "metadata": {"label": "Batch Node 2"},
          "config": {"metadata": {}}
        }
      },
      {
        "op": "createEdge",
        "data": {
          "source": "$temp_node_1",
          "target": "$temp_node_2",
          "metadata": {"label": "Connection", "data_type": "GraphData"}
        }
      }
    ],
    "atomic": false
  }' --pretty

# Validate node creation without executing (Phase 1.6)
layercake query --database layercake.db \
  --entity nodes --action create --project 34 --plan 37 \
  --payload-json '{
    "nodeType":"GraphNode",
    "position":{"x":100,"y":200},
    "metadata":{"label":"Test"},
    "config":{}
  }' --dry-run --pretty
```

**Node Payload Structure:**

For creating nodes (`CliPlanNodeInput`):
```json
{
  "nodeType": "GraphNode|DataSetNode|GraphArtefactNode",
  "position": {"x": 100.0, "y": 200.0},
  "metadata": {
    "label": "Node Label",
    "description": "Optional description"
  },
  "config": {
    // Node-type specific configuration
    // For GraphNode: {"metadata": {}}
    // For DataSetNode: {"dataSetId": 123}
    // For GraphArtefactNode: {"renderTarget": "Mermaid", "renderConfig": {...}}
  }
}
```

For updating nodes (`NodeUpdatePayload`):
```json
{
  "nodeId": "graph_42b0374af121",
  "metadata": {
    "label": "New Label",
    "description": "New description"
  },
  "config": "{...}"  // Optional: JSON string of config
}
```

**Node Filtering Payload (Phase 1.1):**

For filtering nodes in `list` action:
```json
{
  "nodeType": "GraphNode",  // Optional: Filter by node type
  "labelPattern": "copilot",  // Optional: Case-insensitive substring match on label
  "executionState": "completed",  // Optional: "completed", "failed", "pending", "processing"
  "bounds": {  // Optional: Filter by position rectangle
    "minX": 0,
    "maxX": 500,
    "minY": 0,
    "maxY": 500
  }
}
```

**Graph Traversal Payload (Phase 1.3):**

For `traverse` action:
```json
{
  "startNode": "dataset_fb5f819c7089",  // Required: Starting node ID
  "direction": "downstream",  // Optional: "upstream", "downstream", "both" (default: "downstream")
  "maxDepth": 3,  // Optional: Maximum traversal depth (default: 10)
  "endNode": "graphartefact_af32487bb03c",  // Optional: For path finding
  "findPath": true,  // Optional: If true with endNode, find shortest path
  "includeConnections": true,  // Optional: Include connection metadata
  "radius": 2  // Optional: For radial traversal (future feature)
}
```

**Search Payload (Phase 2.2):**

For `search` action:
```json
{
  "query": "copilot",  // Required: Search text (empty string for topology-only search)
  "fields": ["label", "description"],  // Optional: Fields to search (default: ["label"])
  "edgeFilter": "isolated"  // Optional: "isolated", "noIncoming", "noOutgoing"
}
```

**Batch Operations Payload (Phase 2.1):**

For `batch` action:
```json
{
  "operations": [
    {
      "op": "createNode",  // Operation type: createNode, createEdge, updateNode, updateEdge, deleteNode, deleteEdge
      "id": "temp_node_1",  // Optional: Temporary ID for referencing in subsequent ops
      "data": {
        // Operation-specific payload (same as individual action payloads)
      }
    }
  ],
  "atomic": false  // Optional: If true, rollback all on any failure (not fully implemented)
}
```

Use `$tempId` syntax in subsequent operations to reference nodes created earlier in the batch:
```json
{
  "operations": [
    {"op": "createNode", "id": "temp1", "data": {...}},
    {"op": "createEdge", "data": {"source": "$temp1", "target": "existing_node", ...}}
  ]
}
```

**Clone Payload (Phase 2.5):**

For `clone` action:
```json
{
  "nodeId": "graph_42b0374af121",  // Required: Source node ID
  "position": {"x": 800, "y": 600},  // Optional: New position (default: offset from original)
  "updateLabel": "Copilot 003 (copy)"  // Optional: New label (default: appends " (copy)")
}
```

**Analysis Payload (Phase 2.3):**

For `analysis` entity `get` action:
```json
{
  "analysisType": "stats",  // Required: "stats", "bottlenecks", "cycles"
  "threshold": 3  // Optional: For bottlenecks, minimum degree to report (default: 3)
}
```

**Schema Introspection Payload (Phase 1.4):**

For `schema` entity:
```json
// Get node schema
{
  "type": "node",  // Required: "node" or "edge"
  "nodeType": "GraphNode"  // Optional: Specific node type (omit for general node schema)
}

// List node types
{
  "type": "nodeTypes"  // Returns array of available node type names
}

// List available actions
{
  "type": "actions",  // Returns actions for specified entity
  "entity": "nodes"  // Required: "nodes", "edges", "datasets", "plans", "exports", "schema", "analysis", "annotations"
}
```

**Annotation Payloads (Phase 2.4):**

For creating annotations:
```json
{
  "targetId": "graph_42b0374af121",  // Required: Node or edge ID
  "targetType": "node",  // Required: "node" or "edge"
  "key": "status",  // Required: Annotation key
  "value": "reviewed"  // Required: Annotation value
}
```

For listing annotations:
```json
{
  "targetId": "graph_42b0374af121",  // Optional: Filter by specific target
  "key": "status"  // Optional: Filter by key
}
// Omit both to list all annotations in the plan
```

For getting/updating/deleting annotations:
```json
{
  "id": 1,  // Required: Annotation ID
  "value": "approved"  // Required for update: New value
}
```

### Edges
**Purpose**: Connections between nodes representing data flow or references.

**Edge Types:**
- `GraphData` - Data flows from source to target
- `GraphReference` - Target references/uses source

**Actions:**
- `create` - Create a new edge
- `update` - Update an existing edge
- `delete` - Delete an edge

**Examples:**
```bash
# Create an edge
layercake query --database layercake.db \
  --entity edges --action create --project 34 --plan 37 \
  --payload-json '{
    "source": "dataset_fb5f819c7089",
    "target": "graph_42b0374af121",
    "metadata": {
      "label": "Data",
      "data_type": "GraphData"
    }
  }' --pretty

# Update edge metadata
layercake query --database layercake.db \
  --entity edges --action update --project 34 --plan 37 \
  --payload-json '{
    "edgeId": "edge_d8381d18a7a2",
    "metadata": {
      "label": "Updated Label"
    }
  }' --pretty

# Delete an edge
layercake query --database layercake.db \
  --entity edges --action delete --project 34 --plan 37 \
  --payload-json '{
    "edgeId": "edge_d8381d18a7a2"
  }' --pretty
```

**Edge Payload Structure:**

For creating edges (`CliPlanEdgeInput`):
```json
{
  "source": "source_node_id",
  "target": "target_node_id",
  "metadata": {
    "label": "Edge Label",
    "data_type": "GraphData|GraphReference"
  }
}
```

### Exports
**Purpose**: Download rendered graph visualisations.

**Actions:**
- `download` - Export a graph in a specific format

**Supported Formats:**
- `Mermaid` - Mermaid diagram syntax
- `DOT` - Graphviz DOT format
- `JSON` - Raw graph data
- `CSV` - Tabular representation

**Examples:**
```bash
# Download graph as Mermaid with default config
layercake query --database layercake.db \
  --entity exports --action download \
  --payload-json '{
    "graphId": 1001128,
    "format": "Mermaid"
  }' --pretty

# Download with custom render config
layercake query --database layercake.db \
  --entity exports --action download \
  --payload-json '{
    "graphId": 1001128,
    "format": "Mermaid",
    "renderConfig": {
      "orientation": "LR",
      "containNodes": false,
      "applyLayers": true
    }
  }' --output-file graph.mmd --pretty

# Export as DOT
layercake query --database layercake.db \
  --entity exports --action download \
  --payload-json '{
    "graphId": 1001128,
    "format": "DOT",
    "renderConfig": {
      "targetOptions": {
        "graphviz": {
          "layout": "dot",
          "ranksep": 1.3,
          "nodesep": 0.3
        }
      }
    }
  }' --output-file graph.dot
```

**Render Configuration Options:**

```json
{
  "orientation": "TB|LR|RL|BT",
  "containNodes": true,
  "applyLayers": true,
  "builtInStyles": "light|dark",
  "addNodeCommentsAsNotes": false,
  "notePosition": "left|right|top|bottom",
  "useEdgeWeight": true,
  "useNodeWeight": true,
  "targetOptions": {
    "mermaid": {
      "display": "full|compact",
      "look": "default|handDrawn"
    },
    "graphviz": {
      "layout": "dot|neato|fdp|circo|twopi",
      "ranksep": 1.3,
      "nodesep": 0.3,
      "splines": true,
      "overlap": false,
      "commentStyle": "label|note"
    }
  }
}
```

### Schema (Phase 1.4)
**Purpose**: Introspect available node types, actions, and payload structures.

**Actions:**
- `get` - Get schema for a specific node type or edge
- `list` - List available node types or actions

**Examples:**
```bash
# Get schema for GraphNode
layercake query --database layercake.db \
  --entity schema --action get \
  --payload-json '{"type":"node","nodeType":"GraphNode"}' --pretty

# Get edge schema
layercake query --database layercake.db \
  --entity schema --action get \
  --payload-json '{"type":"edge"}' --pretty

# List available node types
layercake query --database layercake.db \
  --entity schema --action list \
  --payload-json '{"type":"nodeTypes"}' --pretty

# List available actions for nodes entity
layercake query --database layercake.db \
  --entity schema --action list \
  --payload-json '{"type":"actions","entity":"nodes"}' --pretty
```

**Schema Response Example:**
```json
{
  "type": "node",
  "nodeType": "GraphNode",
  "description": "A computation node that processes data",
  "fields": {
    "nodeType": {"type": "string", "required": true, "example": "GraphNode"},
    "position": {"type": "Position", "required": true, "example": {"x": 100.0, "y": 200.0}},
    "metadata": {"type": "object", "required": true},
    "config": {"type": "object", "required": true}
  }
}
```

### Analysis (Phase 2.3)
**Purpose**: Analyze graph structure, find bottlenecks, detect cycles, and compute statistics.

**Actions:**
- `get` - Get graph analysis result based on analysis type

**Analysis Types:**
- `stats` - Get node/edge counts, degree statistics, connectivity metrics
- `bottlenecks` - Find nodes with high fan-in/fan-out
- `cycles` - Detect circular dependencies in the graph

**Examples:**
```bash
# Get graph statistics
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"stats"}' --pretty

# Find bottleneck nodes
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"bottlenecks","threshold":3}' --pretty

# Detect cycles
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"cycles"}' --pretty
```

**Analysis Response Example:**
```json
{
  "stats": {
    "node_count": 15,
    "edge_count": 18,
    "avg_degree": 2.4,
    "max_in_degree": 5,
    "max_out_degree": 3,
    "isolated_nodes": 1,
    "source_nodes": 3,
    "sink_nodes": 4
  },
  "bottlenecks": [
    {
      "node_id": "graph_42b0374af121",
      "label": "Copilot 003",
      "in_degree": 1,
      "out_degree": 5,
      "reason": "High fan-out"
    }
  ],
  "cycles": []
}
```

### Annotations (Phase 2.4)
**Purpose**: Attach key-value metadata annotations to nodes and edges for documentation, status tracking, or custom metadata.

**Actions:**
- `create` - Create a new annotation
- `list` - List annotations (for a target or entire plan)
- `get` - Get a specific annotation by ID
- `update` - Update annotation value
- `delete` - Delete an annotation

**Examples:**
```bash
# Create annotation on a node
layercake query --database layercake.db \
  --entity annotations --action create \
  --project 34 --plan 37 \
  --payload-json '{
    "targetId":"graph_42b0374af121",
    "targetType":"node",
    "key":"status",
    "value":"reviewed"
  }' --pretty

# Create annotation on an edge
layercake query --database layercake.db \
  --entity annotations --action create \
  --project 34 --plan 37 \
  --payload-json '{
    "targetId":"edge_d8381d18a7a2",
    "targetType":"edge",
    "key":"confidence",
    "value":"0.95"
  }' --pretty

# List annotations for a specific target
layercake query --database layercake.db \
  --entity annotations --action list \
  --project 34 --plan 37 \
  --payload-json '{"targetId":"graph_42b0374af121"}' --pretty

# List all annotations in a plan
layercake query --database layercake.db \
  --entity annotations --action list \
  --project 34 --plan 37 \
  --payload-json '{}' --pretty

# List annotations filtered by key
layercake query --database layercake.db \
  --entity annotations --action list \
  --project 34 --plan 37 \
  --payload-json '{"key":"status"}' --pretty

# Get specific annotation by ID
layercake query --database layercake.db \
  --entity annotations --action get \
  --payload-json '{"id":1}' --pretty

# Update annotation value
layercake query --database layercake.db \
  --entity annotations --action update \
  --payload-json '{"id":1,"value":"approved"}' --pretty

# Delete annotation
layercake query --database layercake.db \
  --entity annotations --action delete \
  --payload-json '{"id":1}' --pretty
```

**Annotation Response Example:**
```json
{
  "id": 1,
  "project_id": 34,
  "plan_id": 37,
  "target_id": "graph_42b0374af121",
  "target_type": "node",
  "key": "status",
  "value": "reviewed",
  "created_at": "2026-01-23T10:30:00Z",
  "updated_at": "2026-01-23T10:30:00Z"
}
```

**Use Cases:**
- **Status Tracking**: Mark nodes as "reviewed", "approved", "deprecated"
- **Documentation**: Add notes, links, or context to specific nodes/edges
- **Quality Metrics**: Track confidence scores, error rates, or other metrics
- **Custom Metadata**: Any key-value data that doesn't fit in the standard node/edge structure
- **Workflow State**: Track progress through multi-stage processes

## Response Format

All responses follow this structure:

```json
{
  "status": "ok|error",
  "entity": "datasets|plans|nodes|edges|exports",
  "action": "list|get|create|update|delete|move|download",
  "project": 34,
  "plan": 37,
  "result": { /* action-specific data */ },
  "message": "error message if status is error"
}
```

## Understanding Canonical IDs

Layercake uses canonical IDs to uniquely identify resources:
- Plans: `plan:34:37` (project 34, plan 37)
- Plan Nodes: `plannode:34:37:graph_42b0374af121` (project 34, plan 37, node ID)
- Edges: `edge:34:37:edge_d8381d18a7a2` (project 34, plan 37, edge ID)

## Common Workflows

### 1. Exploring a Plan's Structure

```bash
# Get plan metadata
layercake query --database layercake.db \
  --entity plans --action get --project 34 \
  --payload-json '{"id":37}' --pretty

# List all nodes and edges
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 --pretty
```

### 2. Finding a Specific Graph Node

When a user references a node like `plannode:34:37:graph_42b0374af121`:
- Project ID: 34
- Plan ID: 37
- Node ID: `graph_42b0374af121`

```bash
# Get the full DAG to see this node's context
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 --pretty \
  | jq '.result.nodes[] | select(.node.id == "graph_42b0374af121")'
```

### 3. Understanding Graph Execution State

Graph nodes have execution metadata:
```json
{
  "graph_execution": {
    "execution_state": "completed|failed|pending",
    "graph_id": 1001128,
    "graph_data_id": 1001128,
    "node_count": 45,
    "edge_count": 48,
    "computed_date": "2026-01-21T04:32:32.863776+00:00",
    "error_message": null,
    "annotations": null
  }
}
```

Use `graph_id` from the execution metadata when exporting.

### 4. Tracing Data Flow

Follow edges to understand how data flows:
1. Start with a DataSetNode
2. Find edges where `source` matches the dataset node ID
3. Follow to GraphNode targets
4. Continue following edges to GraphArtefactNodes

Example from the data:
```
dataset_fb5f819c7089 (Dataset 182)
  └─→ edge_d8381d18a7a2 (GraphData)
      └─→ graph_42b0374af121 (Copilot 003)
          ├─→ edge_f5326c897735 (GraphReference)
          │   └─→ graphartefact_af32487bb03c (Mermaid LR)
          ├─→ edge_35e665887300 (GraphReference)
          │   └─→ graphartefact_372ee7a65e55 (Mermaid TB)
          └─→ edge_a8a507f8ea6b (GraphReference)
              └─→ graphartefact_a80a459dfeb5 (DOT)
```

### 5. Exporting a Graph Visualisation

```bash
# First, get the graph_id from the node's execution metadata
layercake query --database layercake.db \
  --entity nodes --action list --project 34 --plan 37 --pretty \
  | jq '.result.nodes[] | select(.node.id == "graph_42b0374af121") | .node.graph_execution.graph_id'

# Then export using that graph_id (1001128)
layercake query --database layercake.db \
  --entity exports --action download \
  --payload-json '{
    "graphId": 1001128,
    "format": "Mermaid",
    "renderConfig": {
      "orientation": "LR",
      "containNodes": false
    }
  }' --output-file copilot-003.mmd --pretty
```

### 6. Finding All Nodes Downstream from a Dataset (Phase 1.3)

```bash
# Traverse downstream to see what computations use this dataset
layercake query --database layercake.db \
  --entity nodes --action traverse --project 34 --plan 37 \
  --payload-json '{
    "startNode":"dataset_fb5f819c7089",
    "direction":"downstream",
    "maxDepth":10
  }' --pretty

# This returns all nodes and edges in the downstream subgraph
# Use jq to extract just the node IDs:
# ... | jq -r '.result.nodes[].node.id'
```

### 7. Searching for Specific Nodes (Phase 2.2)

```bash
# Find all nodes with "copilot" in their label
layercake query --database layercake.db \
  --entity nodes --action search --project 34 --plan 37 \
  --payload-json '{
    "query":"copilot",
    "fields":["label"]
  }' --pretty

# Find isolated nodes (no connections)
layercake query --database layercake.db \
  --entity nodes --action search --project 34 --plan 37 \
  --payload-json '{
    "query":"",
    "edgeFilter":"isolated"
  }' --pretty

# Find nodes with no incoming edges (sources)
layercake query --database layercake.db \
  --entity nodes --action search --project 34 --plan 37 \
  --payload-json '{
    "query":"",
    "edgeFilter":"noIncoming"
  }' --pretty
```

### 8. Analysing Graph Structure (Phase 2.3)

```bash
# Get overall statistics
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"stats"}' --pretty

# Find potential bottlenecks (nodes with high connectivity)
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"bottlenecks","threshold":3}' --pretty

# Detect circular dependencies
layercake query --database layercake.db \
  --entity analysis --action get --project 34 --plan 37 \
  --payload-json '{"analysisType":"cycles"}' --pretty
```

### 9. Batch Node Creation (Phase 2.1)

```bash
# Create multiple related nodes and edges in one operation
layercake query --database layercake.db \
  --entity nodes --action batch --project 34 --plan 37 \
  --payload-file batch_operations.json --pretty

# batch_operations.json contains:
# {
#   "operations": [
#     {"op": "createNode", "id": "temp1", "data": {...}},
#     {"op": "createNode", "id": "temp2", "data": {...}},
#     {"op": "createEdge", "data": {"source": "$temp1", "target": "$temp2", ...}}
#   ]
# }
```

### 10. Cloning a Node (Phase 2.5)

```bash
# Clone a node with a new position and updated label
layercake query --database layercake.db \
  --entity nodes --action clone --project 34 --plan 37 \
  --payload-json '{
    "nodeId":"graph_42b0374af121",
    "position":{"x":900,"y":650},
    "updateLabel":"Copilot 003 (experiment)"
  }' --pretty
```

### 11. Validating Changes Before Execution (Phase 1.6)

```bash
# Validate a node creation payload without actually creating it
layercake query --database layercake.db \
  --entity nodes --action create --project 34 --plan 37 \
  --payload-json '{
    "nodeType":"GraphNode",
    "position":{"x":100,"y":200},
    "metadata":{"label":"Test Node"},
    "config":{"metadata":{}}
  }' --dry-run --pretty

# If validation fails, you'll get helpful error messages:
# {
#   "status": "error",
#   "message": "Validation failed: missing required field 'position'"
# }
```

### 12. Using Annotations for Metadata (Phase 2.4)

```bash
# Add status annotation to a node
layercake query --database layercake.db \
  --entity annotations --action create \
  --project 34 --plan 37 \
  --payload-json '{
    "targetId":"graph_42b0374af121",
    "targetType":"node",
    "key":"reviewed",
    "value":"true"
  }' --pretty

# Add priority annotation
layercake query --database layercake.db \
  --entity annotations --action create \
  --project 34 --plan 37 \
  --payload-json '{
    "targetId":"graph_42b0374af121",
    "targetType":"node",
    "key":"priority",
    "value":"high"
  }' --pretty

# List all annotations for a node to see its metadata
layercake query --database layercake.db \
  --entity annotations --action list \
  --project 34 --plan 37 \
  --payload-json '{"targetId":"graph_42b0374af121"}' --pretty

# Find all high-priority items across the plan
layercake query --database layercake.db \
  --entity annotations --action list \
  --project 34 --plan 37 \
  --payload-json '{"key":"priority"}' --pretty \
  | jq '.[] | select(.value == "high")'
```

## Tips for Agents

1. **Always use `--pretty`** when exploring to make responses readable
2. **Check execution state** before exporting - only completed graphs can be exported
3. **Node IDs are unique strings** like `graph_42b0374af121` - use them exactly as returned
4. **Position coordinates** are floats representing canvas position in the web UI
5. **Config fields are JSON strings** - when reading, parse them; when writing, stringify them
6. **The `list` action on nodes returns the entire DAG** - both nodes and edges
7. **Edge data_type matters** - `GraphData` vs `GraphReference` affects visualisation
8. **Graph artefacts don't execute** - they're render configurations, not computations
9. **Use `get` instead of `list` for single nodes** (Phase 1.2) - it's more efficient and includes enriched metadata
10. **Use filtering on `list`** (Phase 1.1) to narrow results instead of filtering in memory
11. **Use `traverse` for exploration** (Phase 1.3) - more efficient than loading full DAG and walking manually
12. **Use schema introspection** (Phase 1.4) to discover available node types and required fields
13. **Use `--dry-run` to validate** (Phase 1.6) payloads before executing, especially for batch operations
14. **Batch operations use `$tempId` syntax** (Phase 2.1) to reference newly created nodes within the same batch
15. **Search with empty query** (Phase 2.2) to find nodes by topology (isolated, noIncoming, noOutgoing)
16. **Check for cycles** (Phase 2.3) before adding edges that might create circular dependencies
17. **Clone nodes for experimentation** (Phase 2.5) instead of manually recreating complex configurations
18. **Use annotations** (Phase 2.4) for flexible metadata without schema changes - perfect for status tracking, notes, and custom properties

## Error Handling

Common errors:
- Missing required `--project` parameter
- Invalid JSON in `--payload-json`
- Node or edge IDs that don't exist
- Attempting to export a graph that hasn't completed execution
- Invalid node types or action combinations

All errors return `"status": "error"` with a `"message"` field explaining the issue.

### Improved Error Messages (Phase 1.5)

The query interface now provides contextual error messages with suggestions:

**Missing Field Error:**
```json
{
  "status": "error",
  "message": "Missing required field 'nodeId' in payload. Expected payload structure: {\"nodeId\": \"string\"}"
}
```

**Invalid Node Type Error:**
```json
{
  "status": "error",
  "message": "Invalid nodeType 'InvalidNode'. Available types: DataSetNode, GraphNode, GraphArtefactNode, TreeArtefactNode, ProjectionNode, StoryNode. Use schema introspection to learn more: --entity schema --action list --payload-json '{\"type\":\"nodeTypes\"}'"
}
```

**Invalid Action Error:**
```json
{
  "status": "error",
  "message": "Invalid action 'invalid' for entity 'nodes'. Available actions: list, get, create, update, delete, move, traverse, search, batch, clone. Use schema introspection: --entity schema --action list --payload-json '{\"type\":\"actions\",\"entity\":\"nodes\"}'"
}
```

**Deserialization Error:**
```json
{
  "status": "error",
  "message": "Failed to parse payload: missing field `position` at line 1 column 45. Check your JSON syntax and ensure all required fields are present."
}
```

## Integration with Web UI

When working with a user viewing the web interface:
- You both operate on the same database
- Changes you make via CLI appear immediately in their UI (on refresh)
- Node positions you set affect where they appear on their canvas
- Graph execution happens asynchronously - check execution_state
- The user can see canonical IDs in the web UI tooltips/URLs

## Performance Considerations

- The `nodes list` action returns the full DAG - this can be large for complex plans
- Use `jq` or similar tools to filter large responses
- Graph exports can be large - use `--output-file` to write directly to disk
- Database is SQLite - safe for concurrent reads, but writes should be sequential

## File Locations

Based on codebase exploration:
- CLI implementation: `layercake-cli/src/query.rs`
- Payload types: `layercake-cli/src/query_payloads.rs`
- Core services: `layercake-core/src/services/cli_graphql_helpers.rs`
- Default database: `layercake.db` (41MB in this project)

## Next Steps

To use this tool effectively:
1. Ask the user for their current project and plan IDs
2. Use **schema introspection** (Phase 1.4) to discover available actions and node types if needed
3. Use **filtering** (Phase 1.1) or **search** (Phase 2.2) to find relevant nodes instead of loading the full DAG
4. Use **get** (Phase 1.2) for single nodes to get enriched metadata
5. Use **traverse** (Phase 1.3) to explore relationships efficiently
6. Use **analysis** (Phase 2.3) to understand graph structure and identify issues
7. Use **--dry-run** (Phase 1.6) to validate changes before executing
8. Use **batch operations** (Phase 2.1) for complex multi-step changes
9. Confirm changes by re-querying or asking the user to check their UI

Remember: You're a collaborative tool working alongside the user's web interface. Always confirm before making destructive changes like deletions.

## Testing and Examples

A comprehensive test suite demonstrating all features is available in `test_query_interface.sh` at the root of the project. Run it to see examples of all Phase 1 and Phase 2 capabilities:

```bash
bash test_query_interface.sh
```

The test suite includes:
- Node filtering by type, label, position, and execution state
- Single node retrieval with metadata enrichment
- Graph traversal (upstream, downstream, path finding)
- Schema introspection examples
- Validation and dry-run examples
- Batch operations with temporary ID references
- Search and discovery (text search, topology filters)
- Graph analysis (stats, bottlenecks, cycles)
- Node cloning

## Feature Phases

**Phase 1: Essential Query Improvements (Completed)**
- 1.1: Node Query Filters
- 1.2: Single Node GET
- 1.3: Graph Traversal
- 1.4: Schema Introspection
- 1.5: Improved Error Messages
- 1.6: Validation and Dry-Run

**Phase 2: Productivity Enhancements (Completed)**
- 2.1: Batch Operations
- 2.2: Search and Discovery
- 2.3: Graph Analysis
- 2.4: Annotations
- 2.5: Clone Operations
