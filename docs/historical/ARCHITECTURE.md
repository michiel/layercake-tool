# Layercake Tool Architecture

## Overview

Layercake is a graph visualization and transformation tool that processes CSV data representing nodes, edges, and layers to generate various output formats. The tool supports both CLI operations and server-based functionality with web interfaces and APIs.

## Current Architecture

**Single Binary**: `layercake` with server functionality enabled by default
**Commands**: `run`, `init`, `generate`, `serve`, `db init`, `db migrate`
**APIs**: REST (✅), GraphQL (✅), MCP (🚧)

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

### ✅ Phase 3: Complete REST API (COMPLETED)
1. ✅ CSV import/export functionality implemented
2. ✅ Plan execution API with full transformation support
3. ✅ Business logic services layer
4. ✅ OpenAPI documentation with Swagger UI
5. ✅ Database commands (`layercake db init`, `layercake db migrate`)

### ✅ Phase 4: GraphQL API (COMPLETED)  
1. ✅ GraphQL API implementation (100% complete - server integration resolved)
2. ✅ Schema, types, queries, mutations fully implemented
3. ✅ GraphQL Playground and introspection available

### 🚧 Phase 5: MCP and Advanced Features (IN PROGRESS)
1. 🔄 MCP (Model Context Protocol) integration
2. ⏳ GraphQL subscriptions for real-time updates  
3. ⏳ WebSocket real-time updates
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

## Phase 4: Multi-API Architecture (In Progress)

### Unified Backend Design

The next evolution adds GraphQL and MCP APIs while maintaining the existing REST API, all sharing a unified service layer:

```
┌─────────────────────────────────────────────────────────────────┐
│                     API Layer (Multiple Interfaces)            │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   REST API      │   GraphQL API   │        MCP API              │
│   (✅ Complete) │   (🚧 Progress) │     (🚧 Progress)           │
│                 │                 │                             │
│ • OpenAPI Docs  │ • Schema Types  │ • Tools/Functions           │
│ • Swagger UI    │ • Resolvers     │ • Resources                 │
│ • CRUD Ops      │ • Subscriptions │ • AI Integration            │
└─────────────────┴─────────────────┴─────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│               Unified Business Logic Layer                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Project   │  │    Plan     │  │     Graph Data          │  │
│  │   Service   │  │   Service   │  │     Service             │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │   Import    │  │   Export    │  │   Transformation       │  │
│  │   Service   │  │   Service   │  │     Service             │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                     Data Layer                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │              SeaORM Entities & Database                    ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Feature Flags Architecture

```toml
[features]
default = ["server", "rest"]
cli = []
server = ["rest", "dep:axum", "dep:tokio", ...]
rest = ["server", "dep:utoipa", "dep:utoipa-swagger-ui"]
graphql = ["server", "dep:async-graphql", "dep:async-graphql-axum"]
mcp = ["server", "dep:mcp-sdk", "dep:tokio-tungstenite"]
all-apis = ["rest", "graphql", "mcp"]
```

### Multi-API File Structure (Planned)

```
src/
├── main.rs                 # Enhanced CLI + multi-API server entry
├── lib.rs                  # Updated library exports
├── server/                 # Server implementation (enhanced)
│   ├── mod.rs
│   ├── app.rs             # Multi-API app configuration
│   └── handlers/          # REST handlers (existing)
│       ├── mod.rs
│       ├── projects.rs
│       ├── plans.rs
│       └── graph_data.rs
├── graphql/               # GraphQL implementation (new)
│   ├── mod.rs
│   ├── schema.rs          # Combined GraphQL schema
│   ├── types/             # GraphQL type definitions
│   │   ├── mod.rs
│   │   ├── project.rs
│   │   ├── plan.rs
│   │   ├── node.rs
│   │   ├── edge.rs
│   │   └── layer.rs
│   ├── mutations/         # Mutation resolvers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── plans.rs
│   │   └── graph_data.rs
│   ├── queries/           # Query resolvers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── plans.rs
│   │   └── graph_data.rs
│   └── context.rs         # GraphQL context
├── mcp/                   # MCP implementation (new)
│   ├── mod.rs
│   ├── server.rs          # MCP server
│   ├── tools/             # MCP tool implementations
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── graph_data.rs
│   │   └── transformations.rs
│   ├── resources/         # MCP resource handlers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   └── graph_data.rs
│   └── prompts/           # MCP prompt templates
│       ├── mod.rs
│       └── graph_analysis.rs
├── services/              # Enhanced service layer (existing + enhanced)
│   ├── mod.rs
│   ├── project_service.rs # Enhanced with trait abstraction
│   ├── plan_service.rs    # Enhanced with trait abstraction
│   ├── import_service.rs  # Existing
│   ├── export_service.rs  # Existing
│   └── graph_service.rs   # Existing
└── (existing modules...)   # All existing functionality preserved
```

### Cross-API Data Models

Shared data structures with feature-specific derives:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]           // REST API
#[cfg_attr(feature = "graphql", derive(SimpleObject))]     // GraphQL
pub struct ProjectData {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### API Endpoints Summary

#### REST API (✅ Implemented)
- **Base**: `/api/v1/`
- **Documentation**: `/docs` (Swagger UI), `/api-docs/openapi.json`
- **Features**: Full CRUD, CSV import/export, plan execution

#### GraphQL API (✅ Implemented)
- **Endpoint**: `/graphql`
- **Features**: Flexible queries, mutations, GraphQL Playground
- **Tools**: GraphQL Playground, introspection, schema documentation

#### MCP API (🚧 In Progress)  
- **Endpoint**: `/mcp` (WebSocket upgrade)
- **Protocol**: Model Context Protocol over WebSocket
- **Features**: AI tool integration, resource access, prompt templates, graph analysis tools

This multi-API architecture provides maximum flexibility for different use cases while maintaining consistency through a unified backend service layer.

## GraphQL Implementation Progress

### ✅ Completed GraphQL Features

**Schema Design**: Complete type system with relationships
```graphql
type Project {
  id: Int!
  name: String!
  description: String
  createdAt: DateTime!
  updatedAt: DateTime!
  # Relationships
  plans: [Plan!]!
  nodes: [Node!]!
  edges: [Edge!]!
  layers: [Layer!]!
}

type Plan {
  id: Int!
  projectId: Int!
  name: String!
  yamlContent: String!
  dependencies: [Int!]
  status: String!
  createdAt: DateTime!
  updatedAt: DateTime!
  # Relationships
  project: Project
}
# ... additional types for Node, Edge, Layer
```

**Query Operations**: Full read access to all entities
- `projects` - List all projects
- `project(id)` - Get specific project
- `plans(projectId)` - Get plans for project
- `nodes(projectId)` - Get nodes for project
- `edges(projectId)` - Get edges for project
- `layers(projectId)` - Get layers for project
- `graphData(projectId)` - Get complete graph data
- `searchNodes(projectId, query)` - Search nodes by label

**Mutation Operations**: Full CRUD capabilities
- Project mutations: `createProject`, `updateProject`, `deleteProject`
- Plan mutations: `createPlan`, `updatePlan`, `deletePlan`, `executePlan`
- Bulk operations: `createNodes`, `createEdges`, `createLayers`

**Service Integration**: Unified backend with REST API
- Shared GraphQL context with database connection
- Service layer integration (ImportService, ExportService, GraphService)
- Consistent data models across REST and GraphQL

### ✅ Completed GraphQL Features

**Server Integration**: ✅ Resolved axum version compatibility by implementing custom GraphQL handlers
**Schema Builder**: ✅ Fixed GraphQL schema context injection using direct Schema::build approach  
**Endpoint Setup**: ✅ Added `/graphql` endpoint with proper POST handler for GraphQL API
**GraphQL Playground**: ✅ Added development UI at `/graphql` (GET request) with playground source
**Service Integration**: ✅ Full integration with unified backend service layer

### 🚧 Remaining GraphQL Tasks

1. **Subscriptions**: Real-time updates for plan execution status
2. **Testing**: End-to-end GraphQL query/mutation testing
3. **Performance**: Query complexity analysis and rate limiting

### GraphQL File Structure (Implemented)

```
src/graphql/
├── mod.rs                    # ✅ Module exports
├── schema.rs                 # ✅ Combined schema definition  
├── context.rs                # ✅ GraphQL context
├── types/                    # ✅ GraphQL type definitions
│   ├── mod.rs               # ✅
│   ├── project.rs           # ✅ Project type & resolvers
│   ├── plan.rs              # ✅ Plan type & resolvers
│   ├── node.rs              # ✅ Node type & resolvers
│   ├── edge.rs              # ✅ Edge type & resolvers
│   ├── layer.rs             # ✅ Layer type & resolvers
│   └── scalars.rs           # ✅ DateTime, JSON scalars
├── queries/                 # ✅ Query resolvers
│   └── mod.rs               # ✅ All read operations
├── mutations/               # ✅ Mutation resolvers
│   └── mod.rs               # ✅ All write operations
```

The GraphQL implementation represents a significant architectural milestone, providing flexible querying capabilities that complement the existing REST API while maintaining data consistency through the shared service layer.

## MCP Implementation Progress

### 🚧 MCP (Model Context Protocol) Integration

The MCP implementation will provide AI tool integration capabilities, allowing AI assistants and other tools to interact with graph data through a standardized protocol.

### Planned MCP Features

**MCP Tools**: Graph operations exposed as callable tools for AI assistants
```json
{
  "name": "create_project",
  "description": "Create a new graph project",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "description": {"type": "string"}
    }
  }
}
```

**MCP Resources**: Graph data and metadata accessible as resources
```json
{
  "uri": "layercake://project/123/graph",
  "name": "Project Graph Data",
  "description": "Complete graph structure for project 123",
  "mimeType": "application/json"
}
```

**MCP Prompts**: Pre-built analysis templates for graph insights
```json
{
  "name": "analyze_graph_structure",
  "description": "Analyze graph connectivity and structure",
  "arguments": [
    {"name": "project_id", "type": "number"}
  ]
}
```

### MCP Endpoint Design

- **Endpoint**: `/mcp` (HTTP upgrade to WebSocket)
- **Protocol**: Model Context Protocol v2024-11-05
- **Transport**: WebSocket with JSON-RPC 2.0 messaging
- **Authentication**: Optional API key or session-based

### MCP Tools Design

**Project Management Tools**:
- `list_projects` - Get all available projects
- `create_project` - Create new project
- `get_project` - Get project details
- `delete_project` - Remove project

**Graph Data Tools**:
- `import_csv` - Import graph data from CSV
- `export_graph` - Export graph in various formats
- `get_graph_data` - Retrieve nodes, edges, layers
- `analyze_connectivity` - Analyze graph structure
- `find_paths` - Find paths between nodes

**Plan Execution Tools**:
- `create_plan` - Create transformation plan
- `execute_plan` - Run graph transformations
- `get_plan_status` - Check execution status

### MCP File Structure (Planned)

```
src/mcp/
├── mod.rs                    # MCP module exports
├── server.rs                 # MCP WebSocket server
├── protocol.rs               # MCP protocol implementation
├── tools/                    # MCP tool implementations
│   ├── mod.rs
│   ├── projects.rs          # Project management tools
│   ├── graph_data.rs        # Graph data tools
│   ├── plans.rs             # Plan execution tools
│   └── analysis.rs          # Graph analysis tools
├── resources/               # MCP resource handlers
│   ├── mod.rs
│   ├── projects.rs          # Project resources
│   └── graph_data.rs        # Graph data resources
├── prompts/                 # MCP prompt templates
│   ├── mod.rs
│   └── graph_analysis.rs    # Graph analysis prompts
└── handlers/                # Request/response handlers
    ├── mod.rs
    ├── tools.rs
    ├── resources.rs
    └── prompts.rs
```