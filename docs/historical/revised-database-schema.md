# Revised Database Schema

## Overview

This document presents the revised database schema that supports:
1. Plan DAG as JSON object in projects table
2. Dual-edit system (CRDT + reproducible operations)
3. Edit reproducibility mechanics
4. Enhanced MCP integration
5. User authentication and groups

## Core Schema Changes

### 1. Enhanced Projects Table

```sql
-- Updated projects table with Plan DAG support
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,

    -- Plan DAG stored as JSON object
    plan_dag TEXT NOT NULL DEFAULT '{"version":"1.0","nodes":[],"edges":[],"metadata":{}}',
    plan_dag_version TEXT NOT NULL DEFAULT '1.0',

    -- Group and user management
    group_id INTEGER NOT NULL,
    created_by INTEGER NOT NULL,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE RESTRICT,

    -- JSON validation constraint (SQLite 3.38+)
    CHECK (json_valid(plan_dag))
);

-- Indexes for performance
CREATE INDEX idx_projects_group ON projects(group_id);
CREATE INDEX idx_projects_created_by ON projects(created_by);
CREATE INDEX idx_projects_updated ON projects(updated_at);
```

### 2. Authentication and Authorization

```sql
-- Groups for multi-user organization
CREATE TABLE groups (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    description TEXT,

    -- Group settings
    settings TEXT DEFAULT '{}',  -- JSON: group-specific settings

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (json_valid(settings))
);

-- Users within groups
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT,

    -- User profile
    full_name TEXT,
    avatar_url TEXT,
    settings TEXT DEFAULT '{}',  -- JSON: user preferences

    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    last_login_at DATETIME,

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CHECK (json_valid(settings))
);

-- User group memberships
CREATE TABLE group_memberships (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,

    -- Role-based access (future RBAC expansion)
    role TEXT NOT NULL DEFAULT 'member',  -- 'admin', 'member', 'viewer'

    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,

    UNIQUE(group_id, user_id)
);

-- Indexes
CREATE INDEX idx_group_memberships_group ON group_memberships(group_id);
CREATE INDEX idx_group_memberships_user ON group_memberships(user_id);
```

### 3. Graph Data with CRDT Support

```sql
-- Enhanced graphs table for CRDT and references
CREATE TABLE graphs (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,

    -- Graph identification
    name TEXT NOT NULL,
    description TEXT,

    -- Graph type and references
    graph_type TEXT NOT NULL DEFAULT 'data',  -- 'data', 'reference', 'computed'
    parent_graph_id INTEGER,  -- For graph hierarchy/copying

    -- CRDT document state
    crdt_state TEXT,  -- Yjs document state (base64 encoded)
    crdt_vector TEXT, -- CRDT vector clock (base64 encoded)

    -- Graph metadata
    metadata TEXT DEFAULT '{}',  -- JSON: custom metadata

    -- Statistics
    node_count INTEGER DEFAULT 0,
    edge_count INTEGER DEFAULT 0,
    layer_count INTEGER DEFAULT 0,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_graph_id) REFERENCES graphs(id) ON DELETE SET NULL,

    CHECK (json_valid(metadata))
);

-- Indexes
CREATE INDEX idx_graphs_project ON graphs(project_id);
CREATE INDEX idx_graphs_parent ON graphs(parent_graph_id);
CREATE INDEX idx_graphs_type ON graphs(graph_type);
CREATE INDEX idx_graphs_updated ON graphs(updated_at);
```

### 4. Graph Data Tables (Enhanced)

```sql
-- Enhanced nodes table
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,

    -- Node data (existing structure preserved)
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    layer_id TEXT,
    is_partition BOOLEAN DEFAULT FALSE,
    belongs_to TEXT,
    weight INTEGER DEFAULT 1,
    comment TEXT,

    -- Additional properties
    properties TEXT DEFAULT '{}',  -- JSON: extensible properties

    -- Lineage tracking for parent-child propagation
    lineage_id TEXT,  -- UUID linking nodes across graph hierarchy
    source_node_id INTEGER,  -- Original node this was copied from

    -- CRDT integration
    crdt_item_id TEXT,  -- CRDT item identifier

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_node_id) REFERENCES nodes(id) ON DELETE SET NULL,

    UNIQUE(graph_id, node_id),
    CHECK (json_valid(properties))
);

-- Enhanced edges table
CREATE TABLE edges (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,

    -- Edge data (existing structure preserved)
    edge_id TEXT NOT NULL,
    source_node_id TEXT NOT NULL,
    target_node_id TEXT NOT NULL,
    label TEXT DEFAULT '',
    layer_id TEXT,
    weight INTEGER DEFAULT 1,
    comment TEXT,

    -- Additional properties
    properties TEXT DEFAULT '{}',  -- JSON: extensible properties

    -- Lineage tracking
    lineage_id TEXT,  -- UUID linking edges across graph hierarchy
    source_edge_id INTEGER,  -- Original edge this was copied from

    -- CRDT integration
    crdt_item_id TEXT,  -- CRDT item identifier

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_edge_id) REFERENCES edges(id) ON DELETE SET NULL,

    UNIQUE(graph_id, edge_id),
    CHECK (json_valid(properties))
);

-- Enhanced layers table
CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,

    -- Layer data (existing structure preserved)
    layer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    background_color TEXT,
    text_color TEXT,
    border_color TEXT,

    -- Additional properties
    properties TEXT DEFAULT '{}',  -- JSON: extensible properties

    -- Lineage tracking
    lineage_id TEXT,  -- UUID linking layers across graph hierarchy
    source_layer_id INTEGER,  -- Original layer this was copied from

    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (source_layer_id) REFERENCES layers(id) ON DELETE SET NULL,

    UNIQUE(graph_id, layer_id),
    CHECK (json_valid(properties))
);

-- Indexes for graph data
CREATE INDEX idx_nodes_graph ON nodes(graph_id);
CREATE INDEX idx_nodes_node_id ON nodes(graph_id, node_id);
CREATE INDEX idx_nodes_lineage ON nodes(lineage_id);
CREATE INDEX idx_nodes_source ON nodes(source_node_id);

CREATE INDEX idx_edges_graph ON edges(graph_id);
CREATE INDEX idx_edges_edge_id ON edges(graph_id, edge_id);
CREATE INDEX idx_edges_source_target ON edges(graph_id, source_node_id, target_node_id);
CREATE INDEX idx_edges_lineage ON edges(lineage_id);

CREATE INDEX idx_layers_graph ON layers(graph_id);
CREATE INDEX idx_layers_layer_id ON layers(graph_id, layer_id);
CREATE INDEX idx_layers_lineage ON layers(lineage_id);
```

### 5. Edit Tracking and Reproducibility

```sql
-- CRDT operations for real-time collaboration
CREATE TABLE crdt_operations (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,

    -- CRDT operation data
    operation_data BLOB NOT NULL,  -- Yjs operation (binary)
    vector_clock TEXT NOT NULL,    -- CRDT vector clock

    -- Operation metadata
    user_id INTEGER NOT NULL,
    session_id TEXT NOT NULL,
    client_id TEXT NOT NULL,

    -- Timestamps
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    applied_at DATETIME,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Manual edits for reproducibility
CREATE TABLE manual_edits (
    id TEXT PRIMARY KEY,  -- UUID
    graph_id INTEGER NOT NULL,

    -- Edit operation data
    operation_type TEXT NOT NULL,  -- 'node_create', 'node_update', etc.
    entity_id TEXT NOT NULL,       -- node_id, edge_id, layer_id
    operation_data TEXT NOT NULL,  -- JSON: complete GraphEditOperation

    -- Context and applicability
    context_data TEXT NOT NULL,    -- JSON: EditContext
    dag_state_hash TEXT NOT NULL,  -- DAG state when edit was made
    applicability_signature TEXT NOT NULL,  -- Quick validation hash

    -- Edit lifecycle
    is_active BOOLEAN DEFAULT TRUE,
    last_applied_at DATETIME,
    deactivated_at DATETIME,
    deactivation_reason TEXT,

    -- User tracking
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE,

    CHECK (json_valid(operation_data)),
    CHECK (json_valid(context_data))
);

-- Edit replay summaries
CREATE TABLE edit_replay_summaries (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,

    -- Replay results
    applied_edits TEXT NOT NULL,      -- JSON: array of edit IDs
    failed_edits TEXT NOT NULL,       -- JSON: array of {editId, reason}
    inapplicable_edits TEXT NOT NULL, -- JSON: array of edit IDs

    -- Metadata
    replay_timestamp DATETIME NOT NULL,
    dag_state_hash TEXT NOT NULL,
    triggered_by INTEGER,  -- User who triggered replay

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (triggered_by) REFERENCES users(id) ON DELETE SET NULL,

    CHECK (json_valid(applied_edits)),
    CHECK (json_valid(failed_edits)),
    CHECK (json_valid(inapplicable_edits))
);

-- Indexes for edit tracking
CREATE INDEX idx_crdt_operations_graph ON crdt_operations(graph_id, created_at);
CREATE INDEX idx_crdt_operations_user ON crdt_operations(user_id, created_at);

CREATE INDEX idx_manual_edits_graph_active ON manual_edits(graph_id, is_active);
CREATE INDEX idx_manual_edits_dag_state ON manual_edits(dag_state_hash);
CREATE INDEX idx_manual_edits_created ON manual_edits(created_at);

CREATE INDEX idx_replay_summaries_graph ON edit_replay_summaries(graph_id, replay_timestamp);
```

### 6. MCP Integration Tables

```sql
-- MCP sessions for AI agent tracking
CREATE TABLE mcp_sessions (
    id TEXT PRIMARY KEY,  -- Session UUID

    -- Session identification
    transport_type TEXT NOT NULL,  -- 'stdio', 'http', 'websocket'
    client_name TEXT,
    client_version TEXT,

    -- Authentication
    user_id INTEGER,  -- NULL for unauthenticated sessions
    group_id INTEGER, -- Inherited from user or specified

    -- Session state
    is_active BOOLEAN DEFAULT TRUE,
    capabilities TEXT DEFAULT '{}',  -- JSON: client capabilities

    -- Timestamps
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_activity_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at DATETIME,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE SET NULL,

    CHECK (json_valid(capabilities))
);

-- MCP tool invocations for audit and analysis
CREATE TABLE mcp_tool_invocations (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,

    -- Tool invocation data
    tool_name TEXT NOT NULL,
    arguments TEXT NOT NULL,  -- JSON: tool arguments

    -- Results
    result_status TEXT NOT NULL,  -- 'success', 'error', 'timeout'
    result_data TEXT,            -- JSON: tool result or error

    -- Performance
    execution_time_ms INTEGER,

    -- Context
    project_id INTEGER,  -- If tool operates on a project
    graph_id INTEGER,    -- If tool operates on a graph

    -- Timestamps
    invoked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,

    FOREIGN KEY (session_id) REFERENCES mcp_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE SET NULL,

    CHECK (json_valid(arguments)),
    CHECK (json_valid(result_data))
);

-- MCP resource access tracking
CREATE TABLE mcp_resource_access (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,

    -- Resource identification
    resource_uri TEXT NOT NULL,
    resource_type TEXT NOT NULL,  -- 'project', 'graph', 'plan_dag', etc.

    -- Access details
    access_type TEXT NOT NULL,    -- 'read', 'list', 'subscribe'
    access_result TEXT NOT NULL, -- 'granted', 'denied', 'error'

    -- Context
    project_id INTEGER,
    graph_id INTEGER,

    accessed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (session_id) REFERENCES mcp_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE SET NULL
);

-- Indexes for MCP tables
CREATE INDEX idx_mcp_sessions_user ON mcp_sessions(user_id, started_at);
CREATE INDEX idx_mcp_sessions_active ON mcp_sessions(is_active, last_activity_at);

CREATE INDEX idx_mcp_tool_invocations_session ON mcp_tool_invocations(session_id, invoked_at);
CREATE INDEX idx_mcp_tool_invocations_tool ON mcp_tool_invocations(tool_name, invoked_at);
CREATE INDEX idx_mcp_tool_invocations_project ON mcp_tool_invocations(project_id, invoked_at);

CREATE INDEX idx_mcp_resource_access_session ON mcp_resource_access(session_id, accessed_at);
CREATE INDEX idx_mcp_resource_access_resource ON mcp_resource_access(resource_uri, accessed_at);
```

### 7. Export and Plan Execution Tracking

```sql
-- Plan execution history
CREATE TABLE plan_executions (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,

    -- Execution details
    execution_type TEXT NOT NULL,  -- 'manual', 'automatic', 'scheduled'
    plan_dag_snapshot TEXT NOT NULL,  -- JSON: Plan DAG at execution time

    -- Results
    status TEXT NOT NULL,  -- 'running', 'completed', 'failed', 'cancelled'
    error_message TEXT,

    -- Performance
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    execution_time_ms INTEGER,

    -- Output tracking
    generated_graphs TEXT DEFAULT '[]',  -- JSON: array of graph IDs created
    generated_files TEXT DEFAULT '[]',   -- JSON: array of output files

    -- User context
    triggered_by INTEGER,
    session_id TEXT,  -- MCP session if triggered by agent

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (triggered_by) REFERENCES users(id) ON DELETE SET NULL,

    CHECK (json_valid(plan_dag_snapshot)),
    CHECK (json_valid(generated_graphs)),
    CHECK (json_valid(generated_files))
);

-- Export operations tracking
CREATE TABLE export_operations (
    id INTEGER PRIMARY KEY,
    graph_id INTEGER NOT NULL,
    execution_id INTEGER,  -- Link to plan execution if applicable

    -- Export details
    format TEXT NOT NULL,
    output_path TEXT NOT NULL,
    file_size INTEGER,

    -- Configuration
    render_config TEXT DEFAULT '{}',
    graph_config TEXT DEFAULT '{}',

    -- Status
    status TEXT NOT NULL,  -- 'pending', 'completed', 'failed'
    error_message TEXT,

    -- Performance
    started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    export_time_ms INTEGER,

    -- User context
    triggered_by INTEGER,

    FOREIGN KEY (graph_id) REFERENCES graphs(id) ON DELETE CASCADE,
    FOREIGN KEY (execution_id) REFERENCES plan_executions(id) ON DELETE SET NULL,
    FOREIGN KEY (triggered_by) REFERENCES users(id) ON DELETE SET NULL,

    CHECK (json_valid(render_config)),
    CHECK (json_valid(graph_config))
);

-- Indexes for execution tracking
CREATE INDEX idx_plan_executions_project ON plan_executions(project_id, started_at);
CREATE INDEX idx_plan_executions_status ON plan_executions(status, started_at);
CREATE INDEX idx_plan_executions_triggered_by ON plan_executions(triggered_by, started_at);

CREATE INDEX idx_export_operations_graph ON export_operations(graph_id, started_at);
CREATE INDEX idx_export_operations_execution ON export_operations(execution_id);
CREATE INDEX idx_export_operations_status ON export_operations(status, started_at);
```

## Migration Scripts

### Migration from Current Schema

```sql
-- Migration script: Add Plan DAG support to existing projects
ALTER TABLE projects ADD COLUMN plan_dag TEXT NOT NULL DEFAULT '{"version":"1.0","nodes":[],"edges":[],"metadata":{}}';
ALTER TABLE projects ADD COLUMN plan_dag_version TEXT NOT NULL DEFAULT '1.0';
ALTER TABLE projects ADD COLUMN group_id INTEGER;
ALTER TABLE projects ADD COLUMN created_by INTEGER;

-- Create default group for existing projects
INSERT INTO groups (id, name, description) VALUES (1, 'Default', 'Default group for existing projects');

-- Assign existing projects to default group
UPDATE projects SET group_id = 1, created_by = 1 WHERE group_id IS NULL;

-- Add foreign key constraints (if not using immediate constraints)
-- This would need to be done in a separate migration with proper constraint handling

-- Migrate existing graph data to new structure
-- Add lineage tracking columns
ALTER TABLE nodes ADD COLUMN lineage_id TEXT;
ALTER TABLE nodes ADD COLUMN source_node_id INTEGER;
ALTER TABLE nodes ADD COLUMN properties TEXT DEFAULT '{}';
ALTER TABLE nodes ADD COLUMN crdt_item_id TEXT;

ALTER TABLE edges ADD COLUMN lineage_id TEXT;
ALTER TABLE edges ADD COLUMN source_edge_id INTEGER;
ALTER TABLE edges ADD COLUMN properties TEXT DEFAULT '{}';
ALTER TABLE edges ADD COLUMN crdt_item_id TEXT;

ALTER TABLE layers ADD COLUMN lineage_id TEXT;
ALTER TABLE layers ADD COLUMN source_layer_id INTEGER;
ALTER TABLE layers ADD COLUMN properties TEXT DEFAULT '{}';

-- Generate lineage IDs for existing data
UPDATE nodes SET lineage_id = hex(randomblob(16)) WHERE lineage_id IS NULL;
UPDATE edges SET lineage_id = hex(randomblob(16)) WHERE lineage_id IS NULL;
UPDATE layers SET lineage_id = hex(randomblob(16)) WHERE lineage_id IS NULL;
```

## SeaORM Entity Updates

### Updated Projects Entity

```rust
// src/database/entities/projects.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub description: Option<String>,

    // Plan DAG as JSON
    pub plan_dag: String,
    pub plan_dag_version: String,

    // Group and user references
    pub group_id: i32,
    pub created_by: i32,

    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::graphs::Entity")]
    Graphs,
    #[sea_orm(has_many = "super::plan_executions::Entity")]
    PlanExecutions,
    #[sea_orm(belongs_to = "super::groups::Entity")]
    Group,
    #[sea_orm(belongs_to = "super::users::Entity")]
    CreatedBy,
}

impl Related<super::graphs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Graphs.def()
    }
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Group.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CreatedBy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Parse Plan DAG from JSON
    pub fn get_plan_dag(&self) -> Result<crate::plan_dag::PlanDAG, serde_json::Error> {
        serde_json::from_str(&self.plan_dag)
    }

    /// Update Plan DAG JSON
    pub fn set_plan_dag(&mut self, plan_dag: &crate::plan_dag::PlanDAG) -> Result<(), serde_json::Error> {
        self.plan_dag = serde_json::to_string(plan_dag)?;
        Ok(())
    }
}
```

### New MCP Entities

```rust
// src/database/entities/mcp_sessions.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "mcp_sessions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub transport_type: String,
    pub client_name: Option<String>,
    pub client_version: Option<String>,
    pub user_id: Option<i32>,
    pub group_id: Option<i32>,
    pub is_active: bool,
    pub capabilities: String,
    pub started_at: ChronoDateTimeUtc,
    pub last_activity_at: ChronoDateTimeUtc,
    pub ended_at: Option<ChronoDateTimeUtc>,
}

// Relations and implementations...
```

## Benefits of Revised Schema

### 1. **Plan DAG Integration**
- Plan DAG stored as JSON in projects table per specification
- Versioning support for schema evolution
- Efficient querying and updates

### 2. **Comprehensive Edit Tracking**
- Dual-edit system support (CRDT + reproducible)
- Complete audit trail for all changes
- Intelligent applicability validation

### 3. **Multi-user Support**
- Group-based organization
- User authentication and authorization
- Session tracking for collaboration

### 4. **MCP Integration**
- Complete tool invocation audit trail
- Resource access tracking
- Session management for AI agents

### 5. **Performance Optimization**
- Strategic indexing for common queries
- JSON validation constraints
- Efficient relationship modeling

### 6. **Maintainability**
- Clear migration path from existing schema
- SeaORM entity updates preserve existing patterns
- Extensible design for future requirements

This revised schema provides a solid foundation for the enhanced layercake system while maintaining compatibility with existing functionality and providing clear upgrade paths.