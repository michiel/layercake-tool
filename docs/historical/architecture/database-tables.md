# Database Architecture: Dual-Table Design

**Status**: Production
**Last Updated**: 2025-10-01
**Author**: Architecture Team

## Overview

The Layercake system uses a **dual-table architecture** to separate concerns between **graph domain data** (what to execute) and **workflow UI metadata** (how to present and orchestrate). This document explains why this design is correct and how the tables interact.

## Table Categories

### 1. Graph Tables (Domain Data)

**Purpose**: Store actual graph content for execution and export.

**Tables**:
- `nodes` - Graph vertices with properties
- `edges` - Graph relationships
- `layers` - Visual grouping layers

**Schema: `nodes`**:
```sql
CREATE TABLE nodes (
    id SERIAL PRIMARY KEY,
    project_id INT NOT NULL,
    node_id VARCHAR(255) NOT NULL,
    label VARCHAR(255) NOT NULL,
    layer_id VARCHAR(255),
    properties TEXT  -- JSON
);
```

**Schema: `edges`**:
```sql
CREATE TABLE edges (
    id SERIAL PRIMARY KEY,
    project_id INT NOT NULL,
    source_node_id VARCHAR(255) NOT NULL,
    target_node_id VARCHAR(255) NOT NULL,
    properties TEXT  -- JSON
);
```

**Usage**:
- **CSV Import**: `layercake-core/src/services/graph_service.rs`
- **Graph Execution**: `build_graph_from_project()` method
- **Export Generation**: Used by layercake renderer
- **GraphQL Queries**: `nodes`, `edges`, `searchNodes`

**Lifecycle**:
1. User uploads CSV files
2. Data imported into `nodes`/`edges` tables
3. Rarely modified after import
4. Referenced by Plan DAG nodes during workflow design
5. Used for graph execution and export

### 2. Plan DAG Tables (Workflow Metadata)

**Purpose**: Store visual workflow editor canvas metadata for real-time collaboration.

**Tables**:
- `plan_dag_nodes` - Workflow canvas nodes with positions and configs
- `plan_dag_edges` - Visual connections between workflow nodes

**Schema: `plan_dag_nodes`**:
```sql
CREATE TABLE plan_dag_nodes (
    id VARCHAR(255) PRIMARY KEY,  -- UUID
    plan_id INT NOT NULL,
    node_type VARCHAR(50) NOT NULL,  -- 'data_source', 'transform', etc.
    position_x REAL NOT NULL,
    position_y REAL NOT NULL,
    metadata_json TEXT NOT NULL,  -- {label, description}
    config_json TEXT NOT NULL,    -- Node-specific configuration
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_plan_dag_nodes_plan_id ON plan_dag_nodes(plan_id);
```

**Schema: `plan_dag_edges`**:
```sql
CREATE TABLE plan_dag_edges (
    id VARCHAR(255) PRIMARY KEY,  -- UUID
    plan_id INT NOT NULL,
    source_node_id VARCHAR(255) NOT NULL,
    target_node_id VARCHAR(255) NOT NULL,
    metadata_json TEXT NOT NULL,  -- {label, dataType}
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

CREATE INDEX idx_plan_dag_edges_plan_id ON plan_dag_edges(plan_id);
```

**Usage**:
- **Visual Editor**: `frontend/src/components/editors/PlanVisualEditor/`
- **Real-time Collaboration**: GraphQL subscriptions with JSON Patch deltas
- **CQRS Pattern**: `PlanDagCommandService` (writes) + `usePlanDagCQRS` (reads)
- **Workflow Orchestration**: Defines data flow between operations

**Lifecycle**:
1. User opens visual workflow editor
2. Drags nodes onto canvas (creates `plan_dag_nodes`)
3. Connects nodes (creates `plan_dag_edges`)
4. Configures nodes (updates `config_json`)
5. Real-time sync via WebSocket subscriptions
6. Frequently modified during editing sessions

## Key Differences

| Aspect | Graph Tables | Plan DAG Tables |
|--------|--------------|-----------------|
| **Domain** | Graph data (execution) | Workflow UI (orchestration) |
| **Structure** | Simple relational | Rich metadata with positions |
| **Primary Key** | Auto-increment integer | String UUID |
| **Timestamps** | `TIMESTAMPTZ` | `TIMESTAMP` |
| **Mutability** | Import-once, rarely changed | Frequently changed (drag/drop) |
| **Real-time** | No subscriptions | JSON Patch delta subscriptions |
| **Relationships** | Foreign keys to projects | Foreign keys to plans |
| **Purpose** | What to execute | How to orchestrate |

## Data Flow Diagram

```
┌──────────────┐
│  CSV Upload  │
└──────┬───────┘
       │
       ▼
┌──────────────────────────────┐
│  Graph Tables (Domain Data)   │
│  - nodes                      │
│  - edges                      │
│  - layers                     │
└──────┬───────────────────────┘
       │ Referenced by
       │ (node_id)
       ▼
┌──────────────────────────────┐
│  Plan DAG Tables (UI Metadata)│
│  - plan_dag_nodes             │
│    {                          │
│      node_type: "data_source",│
│      config: {                │
│        source_node_id: "n123" │  ◄── References graph nodes
│      },                       │
│      position: {x: 100, y: 50}│
│    }                          │
│  - plan_dag_edges             │
└──────┬───────────────────────┘
       │
       ▼
┌──────────────────────────────┐
│  ReactFlow Visual Editor     │
│  (Real-time Collaboration)   │
└──────────────────────────────┘
       │
       ▼
┌──────────────────────────────┐
│  Graph Execution Engine      │
│  (Uses graph tables + plan)  │
└──────────────────────────────┘
```

## Example Workflow

### Step 1: Import Graph Data

```sql
-- User uploads nodes.csv
INSERT INTO nodes (project_id, node_id, label, layer_id, properties)
VALUES
    (1, 'customer_001', 'Alice', 'customers', '{"age": 30}'),
    (1, 'customer_002', 'Bob', 'customers', '{"age": 25}'),
    (1, 'product_001', 'Widget', 'products', '{"price": 9.99}');

-- User uploads edges.csv
INSERT INTO edges (project_id, source_node_id, target_node_id, properties)
VALUES
    (1, 'customer_001', 'product_001', '{"quantity": 2}'),
    (1, 'customer_002', 'product_001', '{"quantity": 1}');
```

### Step 2: Design Workflow in Visual Editor

```sql
-- Create plan for project
INSERT INTO plans (project_id, name, version)
VALUES (1, 'Customer Analysis', '1.0.0');

-- Add DataSource node to canvas (references graph nodes)
INSERT INTO plan_dag_nodes (id, plan_id, node_type, position_x, position_y, metadata_json, config_json, created_at, updated_at)
VALUES (
    'node-uuid-1',
    1,
    'data_source',
    100.0,
    50.0,
    '{"label": "Customer Data", "description": "Load customer nodes"}',
    '{"source_type": "graph_nodes", "layer_id": "customers"}',  -- References graph data!
    NOW(),
    NOW()
);

-- Add Transform node
INSERT INTO plan_dag_nodes (id, plan_id, node_type, position_x, position_y, metadata_json, config_json, created_at, updated_at)
VALUES (
    'node-uuid-2',
    1,
    'transform',
    300.0,
    50.0,
    '{"label": "Filter Adults", "description": "age >= 18"}',
    '{"filter_expression": "node.properties.age >= 18"}',
    NOW(),
    NOW()
);

-- Connect them visually
INSERT INTO plan_dag_edges (id, plan_id, source_node_id, target_node_id, metadata_json, created_at, updated_at)
VALUES (
    'edge-uuid-1',
    1,
    'node-uuid-1',
    'node-uuid-2',
    '{"label": "Customers", "dataType": "GraphData"}',
    NOW(),
    NOW()
);
```

### Step 3: Execution

When user executes the workflow:
1. System reads `plan_dag_nodes` and `plan_dag_edges` (workflow structure)
2. DataSource node config references `layer_id: "customers"`
3. System queries `nodes` table: `WHERE layer_id = 'customers'`
4. Loads actual graph data (Alice, Bob)
5. Applies transform (filter adults)
6. Outputs result

## Why Consolidation Would Be Harmful

### ❌ Anti-Pattern: Merging Tables

**Bad Idea**: Combining `nodes` and `plan_dag_nodes` into single table

**Problems**:
1. **Domain Confusion**: Mixing execution data with UI metadata
2. **Performance**: Every canvas drag would require graph data load
3. **Scalability**: 10,000 graph nodes pollute workflow editor
4. **Collaboration**: Real-time canvas edits conflict with graph imports
5. **Versioning**: Can't version workflows independently of data

### ✅ Correct Pattern: Separation of Concerns

**Graph Tables**: Domain data (what)
**Plan DAG Tables**: Workflow orchestration (how)

This follows established patterns:
- **MVC**: Model (graph) vs View (plan DAG)
- **Clean Architecture**: Domain layer vs Presentation layer
- **DDD**: Entities (graph) vs Value Objects (positions)

## Migration History

- **m001**: Create initial tables (projects, plans, nodes, edges, layers)
- **m002**: Add Plan DAG tables (plan_dag_nodes, plan_dag_edges)
- **m006**: Add foreign key constraints and indexes (referential integrity)

## Related Documentation

- [CQRS Pattern](./cqrs-architecture.md) - Command/Query separation
- [Real-time Collaboration](./collaboration.md) - JSON Patch subscriptions
- [GraphQL API](./graphql-api.md) - API structure

## FAQ

**Q: Why use UUID for plan_dag_nodes but integer for nodes?**
A: Plan DAG nodes are client-generated (collaborative editing), while graph nodes are server-generated (CSV import).

**Q: Can a plan_dag_node exist without referencing graph nodes?**
A: Yes. Transform, Merge, Copy, and Output nodes are purely orchestration nodes with no graph data reference.

**Q: What happens if I delete a node from graph tables?**
A: Plan DAG nodes may reference deleted nodes (via config). System should validate references before execution.

**Q: Why different timestamp types?**
A: Historical inconsistency. Graph tables use `TIMESTAMPTZ` (correct), Plan DAG uses `TIMESTAMP`. Should standardise to `TIMESTAMPTZ`.

## Future Improvements

1. **Standardise Timestamps**: Migrate Plan DAG tables to use `TIMESTAMPTZ`
2. **Add Reference Validation**: Foreign key from plan_dag_node.config.source_node_id → nodes.node_id
3. **Soft Deletes**: Add `deleted_at` to preserve workflow history
4. **Version Control**: Add plan_version table to track workflow changes
5. **Audit Log**: Track all plan_dag mutations for collaboration debugging
