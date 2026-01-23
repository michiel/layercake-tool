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
- `--session`: Session identifier for future auth support

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
- `list` - List all nodes and edges in a plan (returns full DAG)
- `create` - Create a new node
- `update` - Update an existing node
- `delete` - Delete a node
- `move` - Change a node's position

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

## Tips for Agents

1. **Always use `--pretty`** when exploring to make responses readable
2. **Check execution state** before exporting - only completed graphs can be exported
3. **Node IDs are unique strings** like `graph_42b0374af121` - use them exactly as returned
4. **Position coordinates** are floats representing canvas position in the web UI
5. **Config fields are JSON strings** - when reading, parse them; when writing, stringify them
6. **The `list` action on nodes returns the entire DAG** - both nodes and edges
7. **Edge data_type matters** - `GraphData` vs `GraphReference` affects visualisation
8. **Graph artefacts don't execute** - they're render configurations, not computations

## Error Handling

Common errors:
- Missing required `--project` parameter
- Invalid JSON in `--payload-json`
- Node or edge IDs that don't exist
- Attempting to export a graph that hasn't completed execution
- Invalid node types or action combinations

All errors return `"status": "error"` with a `"message"` field explaining the issue.

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
2. Use `nodes list` to get the full structure
3. Identify specific nodes they're interested in
4. Perform targeted queries or modifications
5. Confirm changes by re-querying or asking the user to check their UI

Remember: You're a collaborative tool working alongside the user's web interface. Always confirm before making destructive changes like deletions.
