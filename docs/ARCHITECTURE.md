# Layercake Tool Architecture

## Overview

Layercake is a graph visualization and transformation tool that processes CSV data representing nodes, edges, and layers to generate various output formats. The tool supports both CLI operations and server-based functionality with web interfaces and APIs.

## Current Architecture

**Single Binary**: `layercake` with server functionality enabled by default
**Commands**: `run`, `init`, `generate`, `serve`, `migrate`

### Core Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Data Loader   │───▶│  Graph Builder  │───▶│   Exporters     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CSV Files     │    │  Graph Model    │    │ Output Files    │
│ - nodes.csv     │    │ - Nodes         │    │ - DOT, GML      │
│ - edges.csv     │    │ - Edges         │    │ - JSON, CSV     │
│ - layers.csv    │    │ - Layers        │    │ - PlantUML      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### File Structure (Current)

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library exports
├── common.rs               # Shared utilities
├── data_loader.rs          # CSV data loading
├── graph.rs                # Graph data structures
├── plan.rs                 # Plan configuration
├── plan_execution.rs       # Plan processing
├── generate_commands.rs    # Template generation
└── export/                 # Export modules
    ├── mod.rs
    ├── to_dot.rs           # Graphviz DOT export
    ├── to_gml.rs           # GML export
    ├── to_json.rs          # JSON export
    ├── to_csv_*.rs         # CSV exports
    ├── to_plantuml.rs      # PlantUML export
    ├── to_mermaid.rs       # Mermaid export
    └── to_custom.rs        # Custom template export
```

### Data Flow

1. **Input Processing**: CSV files are loaded and parsed into internal data structures
2. **Graph Construction**: Nodes, edges, and layers are organized into a graph model
3. **Plan Execution**: YAML plan defines transformation and export steps
4. **Export Generation**: Multiple output formats generated based on plan configuration

### Key Data Structures

```rust
// Core graph structures
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: Option<String>,
    // Additional properties from CSV
}

pub struct Edge {
    pub source: String,
    pub target: String,
    // Additional properties from CSV
}

pub struct Layer {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

// Plan configuration
pub struct Plan {
    pub name: Option<String>,
    pub description: Option<String>,
    pub data: DataConfig,
    pub exports: Vec<ExportConfig>,
}
```

## Server Architecture (Implemented)

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Layercake Server                        │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   REST API  │  │ GraphQL API │  │   MCP API   │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │
│  │   Services  │  │ Plan Engine │  │  WebSocket  │            │
│  └─────────────┘  └─────────────┘  └─────────────┘            │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────────────────────────────┐  │
│  │  Database   │  │           Graph Engine              │  │
│  │  (SQLite)   │  │     (Existing Export System)       │  │
│  └─────────────┘  └─────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Database Schema (SeaORM Entities)

```sql
-- Projects: Root containers for all data
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Plans: Execution plans in DAG structure  
CREATE TABLE plans (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    yaml_content TEXT NOT NULL,
    dependencies TEXT, -- JSON array of plan IDs
    status TEXT NOT NULL, -- pending, running, completed, failed
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Graph data entities
CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    label TEXT NOT NULL,
    layer_id TEXT,
    properties TEXT, -- JSON
    FOREIGN KEY (project_id) REFERENCES projects(id),
    UNIQUE(project_id, node_id)
);

CREATE TABLE edges (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    source_node_id TEXT NOT NULL,
    target_node_id TEXT NOT NULL,
    properties TEXT, -- JSON
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

CREATE TABLE layers (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL,
    layer_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT,
    properties TEXT, -- JSON
    FOREIGN KEY (project_id) REFERENCES projects(id),
    UNIQUE(project_id, layer_id)
);
```

### API Design

#### REST Endpoints
```
GET    /api/v1/projects              # List projects
POST   /api/v1/projects              # Create project
GET    /api/v1/projects/{id}         # Get project
PUT    /api/v1/projects/{id}         # Update project
DELETE /api/v1/projects/{id}         # Delete project

GET    /api/v1/projects/{id}/plans   # List plans
POST   /api/v1/projects/{id}/plans   # Create plan
PUT    /api/v1/projects/{id}/plans/{plan_id} # Update plan
DELETE /api/v1/projects/{id}/plans/{plan_id} # Delete plan
POST   /api/v1/projects/{id}/plans/{plan_id}/execute # Execute plan

GET    /api/v1/projects/{id}/nodes   # List nodes
POST   /api/v1/projects/{id}/nodes   # Bulk create/update nodes
DELETE /api/v1/projects/{id}/nodes   # Bulk delete nodes

GET    /api/v1/projects/{id}/edges   # List edges  
POST   /api/v1/projects/{id}/edges   # Bulk create/update edges
DELETE /api/v1/projects/{id}/edges   # Bulk delete edges

GET    /api/v1/projects/{id}/layers  # List layers
POST   /api/v1/projects/{id}/layers  # Bulk create/update layers
DELETE /api/v1/projects/{id}/layers  # Bulk delete layers

POST   /api/v1/projects/{id}/import/csv # Import CSV files
GET    /api/v1/projects/{id}/export/{format} # Export graph data
```

### Server File Structure (Implemented)

```
src/
├── main.rs                 # Enhanced CLI + server entry
├── lib.rs                  # Updated library exports
├── server/                 # Server implementation (implemented)
│   ├── mod.rs
│   ├── app.rs             # Axum app configuration
│   └── handlers/          # HTTP request handlers
│       ├── mod.rs
│       ├── projects.rs    # Project CRUD operations
│       ├── plans.rs       # Plan management and execution
│       └── graph_data.rs  # Graph data, CSV import/export
├── database/              # Database layer (implemented)
│   ├── mod.rs
│   ├── entities/          # SeaORM entities (implemented)
│   │   ├── mod.rs
│   │   ├── projects.rs    # Project model
│   │   ├── plans.rs       # Plan model
│   │   ├── nodes.rs       # Node model
│   │   ├── edges.rs       # Edge model
│   │   └── layers.rs      # Layer model
│   ├── migrations/        # Database migrations (implemented)
│   └── connection.rs      # Database setup (implemented)
├── services/              # Business logic layer (implemented)
│   ├── mod.rs
│   ├── import_service.rs  # CSV import functionality
│   ├── export_service.rs  # Graph export with transformations
│   └── graph_service.rs   # Graph building from database entities
└── (existing modules...)   # All existing functionality preserved
    ├── common.rs           # Shared utilities
    ├── data_loader.rs      # CSV data loading
    ├── graph.rs            # Graph data structures
    ├── plan.rs             # Plan configuration
    ├── plan_execution.rs   # Plan processing
    ├── generate_commands.rs # Template generation
    └── export/             # Export modules (unchanged)
```

## Design Principles

### Backward Compatibility
- All existing CLI functionality preserved
- Existing plan files continue to work
- CSV import/export maintained
- No breaking changes to current APIs

### Modularity
- Clear separation between CLI and server code
- Feature flags for optional server dependencies
- Reusable core graph engine
- Pluggable export system

### Data Persistence
- SQLite for lightweight, file-based storage
- In-memory option for development/testing
- SeaORM for type-safe database operations
- Migration system for schema evolution

### API Design
- RESTful endpoints following OpenAPI standards
- GraphQL for complex queries and subscriptions
- MCP integration for AI tool compatibility
- Unified business logic across all APIs

### Performance
- Async/await throughout server code
- Connection pooling for database access
- Streaming responses for large datasets
- WebSocket for real-time updates

### Security
- Input validation on all endpoints
- SQL injection prevention via ORM
- CORS configuration for web interface
- Rate limiting for public APIs

## Implementation Status

### ✅ Phase 1: Foundation (COMPLETED)
1. ✅ Server dependencies integrated with default features
2. ✅ Database entities and migrations implemented
3. ✅ Basic server command (`layercake serve`) working
4. ✅ All existing CLI functionality preserved

### ✅ Phase 2: Core APIs (COMPLETED)
1. ✅ REST endpoints for CRUD operations implemented
2. ✅ Plan execution API with full transformation support
3. ✅ CSV import/export via API endpoints
4. ✅ Integration with existing export engine

### 🚧 Phase 3: Advanced Features (PLANNED)
1. ⏳ GraphQL API implementation
2. ⏳ WebSocket real-time updates
3. ⏳ MCP integration
4. ⏳ Web-based graph editor

## Key Implementation Details

### Single Binary Architecture
- The project now builds a single `layercake` binary with server functionality enabled by default
- All CLI commands remain functional: `run`, `init`, `generate`, `serve`, `migrate`
- Server can be started with `layercake serve` command

### Services Layer
Three core services bridge database operations with the existing graph engine:

- **ImportService**: Handles bulk CSV imports with error collection and validation
- **ExportService**: Integrates with existing export engine for graph transformations and format conversion
- **GraphService**: Converts database entities to Graph structures for seamless compatibility

### Export Engine Integration
Full integration with existing export system supporting:
- All existing formats: DOT, GML, JSON, Mermaid, PlantUML, CSV
- Graph transformations: invert, partition limits, label modifications
- Plan execution with YAML configuration parsing
- Transformation pipeline with graph integrity verification

This architecture successfully evolved from CLI tool to full-featured server application while maintaining complete backward compatibility and leveraging the existing robust graph processing engine.