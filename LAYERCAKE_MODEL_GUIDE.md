# Layercake Graph Model Guide

This guide documents the layercake graph model, including node types, edge types, their attributes, and the rules governing the graph structure.

## Overview

Layercake uses a two-level graph model:

1. **Plan DAG** - A directed acyclic graph defining data processing workflows. Nodes represent operations (data sources, transformations, outputs) and edges represent data flow.

2. **Graph Data** - The actual graph structures processed by the Plan DAG. These contain domain-specific nodes and edges with attributes like labels, layers, and weights.

## Plan DAG Node Types

The Plan DAG supports 10 distinct node types:

| Node Type | Purpose | Aliases |
|-----------|---------|---------|
| `DataSetNode` | Source data ingestion from uploaded CSV/TSV/JSON files | - |
| `GraphNode` | Graph processing and execution | - |
| `TransformNode` | Graph transformation operations | - |
| `FilterNode` | Graph filtering operations | - |
| `MergeNode` | Graph merging operations | - |
| `GraphArtefactNode` | Rendered output for computed graphs | OutputNode, Output |
| `TreeArtefactNode` | Tree structure outputs | - |
| `ProjectionNode` | Graph projection operations | - |
| `StoryNode` | Story/narrative outputs | - |
| `SequenceArtefactNode` | Sequence diagram outputs | - |

### Node Attribute Reference

All Plan DAG nodes share these core attributes:

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | String | Yes | Unique identifier (format: `{type}_{uuid12}`, e.g., `graph_42b0374af121`) |
| `node_type` | String | Yes | One of the 10 node types above |
| `position` | `{x, y}` | Yes | Canvas position (floats, range: -100,000 to +100,000) |
| `metadata` | Object | Yes | Contains `label` (required) and `description` (optional) |
| `config` | Object | Yes | Node-type specific configuration (max 100KB JSON) |

#### Node Metadata Structure

```json
{
  "label": "My Node Label",
  "description": "Optional description of this node"
}
```

- `label`: Required, max 200 characters, no HTML/script injection
- `description`: Optional markdown text

#### Node Configuration by Type

**DataSetNode**:
```json
{
  "dataSetId": 123
}
```

**GraphNode**:
```json
{
  "metadata": {}
}
```

**GraphArtefactNode**:
```json
{
  "renderTarget": "Mermaid",
  "renderConfig": {
    "orientation": "LR",
    "containNodes": false,
    "applyLayers": true
  }
}
```

### Execution State

Nodes track execution progress through these states:

| State | Description |
|-------|-------------|
| `NotStarted` | Initial state, never executed |
| `Pending` | Queued for execution |
| `Processing` | Currently executing |
| `Completed` | Execution succeeded |
| `Error` | Execution failed (check `error_message`) |

#### DataSet Execution Metadata

When a `DataSetNode` executes:

```json
{
  "data_set_id": 180,
  "filename": "data.csv",
  "status": "processed",
  "processed_at": "2026-01-21T10:30:00Z",
  "execution_state": "completed",
  "error_message": null
}
```

#### Graph Execution Metadata

When a `GraphNode` executes:

```json
{
  "graph_id": 1001128,
  "graph_data_id": 1001128,
  "node_count": 45,
  "edge_count": 48,
  "execution_state": "completed",
  "computed_date": "2026-01-21T04:32:32.863776+00:00",
  "error_message": null,
  "annotations": "Processing notes in markdown"
}
```

## Plan DAG Edge Types

Edges connect nodes and define data flow direction.

### Edge Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | String | Yes | Unique identifier (format: `edge_{uuid12}`) |
| `source` | String | Yes | Source node ID |
| `target` | String | Yes | Target node ID |
| `metadata` | Object | Yes | Contains `label` and `data_type` |

### Edge Metadata Structure

```json
{
  "label": "Data Flow",
  "data_type": "GraphData"
}
```

### Data Types

| Data Type | Description |
|-----------|-------------|
| `GraphData` | Standard graph structure data flowing between nodes |
| `GraphReference` | Reference to an external graph structure |
| `SequenceData` | Sequence diagram data |

## Graph Data Model

The actual graph structures contain nodes and edges with rich attributes.

### Graph Node Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | String | Yes | Unique external identifier |
| `label` | String | No | Display label |
| `layer` | String | No | Layer assignment for styling/grouping |
| `weight` | Float | No | Importance metric |
| `is_partition` | Boolean | No | Marks node as a container (default: false) |
| `belongs_to` | String | No | Parent partition node ID (creates hierarchy) |
| `comment` | String | No | Documentation/notes |
| `attributes` | JSON | No | Custom key-value attributes |

### Graph Edge Attributes

| Attribute | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | String | Yes | Unique identifier |
| `source` | String | Yes | Source node external_id |
| `target` | String | Yes | Target node external_id |
| `label` | String | No | Edge label |
| `layer` | String | No | Layer assignment |
| `weight` | Float | No | Edge weight/importance |
| `comment` | String | No | Documentation |
| `attributes` | JSON | No | Custom key-value attributes |

## Partition Hierarchy

Layercake supports hierarchical node organisation through partitions.

### What Are Partitions?

Partition nodes act as containers that group related nodes. This creates a tree-like hierarchy within the graph:

```
File (partition)
├── Function A (belongs_to: File)
├── Function B (belongs_to: File)
└── Class (partition, belongs_to: File)
    ├── Method 1 (belongs_to: Class)
    └── Method 2 (belongs_to: Class)
```

### Partition Rules

1. **Partition nodes cannot be edge endpoints** - Edges represent data flow between non-partition nodes only
2. **Hierarchy via `belongs_to`** - Child nodes reference their parent partition
3. **No cycles** - The `belongs_to` chain cannot form cycles
4. **References must exist** - A `belongs_to` value must reference an existing node

### Detecting Partition Structure

A graph has partition structure if:
- Any node has `is_partition = true`, OR
- Any node has a non-empty `belongs_to` value

## Validation Rules

### Node ID Rules

- Format: Alphanumeric with underscores only
- Pattern: `^[a-zA-Z0-9_]+$`
- Maximum length: 50 characters
- Must be unique within the graph/plan

### Label Rules

- Cannot be empty
- Maximum 200 characters
- HTML/script content is sanitised

### Position Rules

- X and Y must be finite numbers (not infinity or NaN)
- Range: -100,000.0 to +100,000.0

### Edge Rules

1. **No self-loops** - Source and target must be different nodes
2. **Endpoints must exist** - Both source and target nodes must exist
3. **No partition endpoints** - Edges cannot connect to partition nodes
4. **Unique IDs** - All edge IDs must be unique

### Plan DAG Limits

| Limit | Value |
|-------|-------|
| Maximum nodes per Plan DAG | 1,000 |
| Maximum edges per Plan DAG | 5,000 |
| Metadata JSON size | 50KB |
| Config JSON size | 100KB |

### Layer Rules

- All nodes must reference existing layers
- Layer IDs must exist in the graph's layer collection
- Edges referencing non-existent layers generate warnings (non-fatal)

## Integrity Checking

The `verify_graph_integrity()` function enforces:

1. All edges reference existing nodes
2. Edges never connect to partition nodes
3. `belongs_to` references point to existing nodes
4. No cycles in `belongs_to` chains
5. All nodes assigned to valid layers
6. Unique node and edge IDs

Integrity violations return detailed error messages identifying the specific issue.

## Common Data Flow Patterns

### Standard Processing Pipeline

```
DataSetNode (source)
    │
    ▼ [GraphData]
GraphNode (processing)
    │
    ├─▶ [GraphReference] GraphArtefactNode (Mermaid output)
    ├─▶ [GraphReference] GraphArtefactNode (DOT output)
    └─▶ [GraphReference] TreeArtefactNode (tree view)
```

### Transformation Pipeline

```
DataSetNode
    │
    ▼ [GraphData]
TransformNode
    │
    ▼ [GraphData]
FilterNode
    │
    ▼ [GraphData]
GraphNode
    │
    ▼ [GraphReference]
GraphArtefactNode
```

### Merge Pipeline

```
DataSetNode A ──┐
                │ [GraphData]
                ▼
           MergeNode
                ▲
                │ [GraphData]
DataSetNode B ──┘
```

## Graph Transforms

When processing graphs, these transforms can be applied:

### Partition-Aware Transforms

| Transform | Parameter | Description |
|-----------|-----------|-------------|
| `PartitionDepthLimit` | `max_partition_depth` | Limits nesting depth in hierarchy |
| `PartitionWidthLimit` | `max_partition_width` | Aggregates children exceeding width limit |
| `GenerateHierarchy` | - | Converts `belongs_to` to explicit edges |

### General Transforms

| Transform | Description |
|-----------|-------------|
| `DropUnconnectedNodes` | Removes isolated nodes (option: `exclude_partition_nodes`) |

## Canonical IDs

Layercake uses canonical IDs to uniquely identify resources across the system:

| Resource | Format | Example |
|----------|--------|---------|
| Plan | `plan:{project_id}:{plan_id}` | `plan:34:37` |
| Plan Node | `plannode:{project_id}:{plan_id}:{node_id}` | `plannode:34:37:graph_42b0374af121` |
| Edge | `edge:{project_id}:{plan_id}:{edge_id}` | `edge:34:37:edge_d8381d18a7a2` |

## Graph Data Storage

Graphs are stored with a source type discriminator:

| Source Type | Description | Key Fields |
|-------------|-------------|------------|
| `dataset` | Uploaded file data | `file_format`, `filename`, `blob`, `file_size` |
| `computed` | Result of DAG execution | `dag_node_id`, `source_hash`, `computed_date` |
| `manual` | User-created graph | Standard fields only |

Common fields for all source types:
- `node_count` / `edge_count` - Graph dimensions
- `annotations` - JSON array of markdown strings
- `status` - 'active', 'processing', or 'error'
- `error_message` - Error details if status is 'error'
- `metadata` - Custom JSON metadata

## Best Practices

### Node Naming

- Use descriptive labels that indicate purpose
- Keep labels under 50 characters for readability
- Use descriptions for additional context

### Graph Organisation

- Use partitions to group related nodes logically
- Keep partition depth shallow (2-3 levels) for clarity
- Assign meaningful layer names for styling

### Edge Design

- Use `GraphData` for actual data flow
- Use `GraphReference` for output/rendering relationships
- Add labels to edges when the relationship isn't obvious

### Validation

- Use `--dry-run` to validate changes before execution
- Check for cycles before adding edges that might create them
- Verify partition structure integrity after modifications
