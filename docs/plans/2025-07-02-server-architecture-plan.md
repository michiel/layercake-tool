# Layercake Server Architecture Plan
*Date: 2025-07-02*

## Overview
Transform layercake from a CLI-only tool to a full-featured server application with web interface, database persistence, and multiple API endpoints. The server will manage projects containing DAGs of plans with graph data (nodes, edges, layers) as persistent entities.

## Phase 1: Core Server Infrastructure

### 1.1 Database Layer (SeaORM + SQLite)
- **Project Entity**: Root container for all data
  - `id: i32` (primary key)
  - `name: String`
  - `description: Option<String>`
  - `created_at: DateTime`
  - `updated_at: DateTime`

- **Plan Entity**: Execution plans in DAG structure
  - `id: i32` (primary key)
  - `project_id: i32` (foreign key)
  - `name: String`
  - `yaml_content: String` (serialized plan YAML)
  - `dependencies: Vec<i32>` (plan IDs this depends on)
  - `status: PlanStatus` (pending, running, completed, failed)
  - `created_at: DateTime`
  - `updated_at: DateTime`

- **Graph Data Entities**:
  - **Node Entity**: Graph nodes
    - `id: i32` (primary key)
    - `project_id: i32` (foreign key)
    - `node_id: String` (business identifier)
    - `label: String`
    - `layer_id: Option<String>`
    - `properties: Json` (additional node data)
    
  - **Edge Entity**: Graph edges
    - `id: i32` (primary key)
    - `project_id: i32` (foreign key)
    - `source_node_id: String`
    - `target_node_id: String`
    - `properties: Json` (additional edge data)
    
  - **Layer Entity**: Graph layers
    - `id: i32` (primary key)
    - `project_id: i32` (foreign key)
    - `layer_id: String`
    - `name: String`
    - `color: Option<String>`
    - `properties: Json` (additional layer data)

### 1.2 Server Framework Setup
- Use **Axum** for HTTP server (aligns with Rust ecosystem)
- **tokio** for async runtime
- **tower** middleware for CORS, logging, authentication
- **sqlite** with SeaORM for database
- Database migrations with SeaORM CLI

### 1.3 New Command Structure
```rust
enum Commands {
    Run { /* existing */ },
    Init { /* existing */ },
    Generate { /* existing */ },
    Serve {
        #[clap(short, long, default_value = "3000")]
        port: u16,
        #[clap(short, long, default_value = "layercake.db")]
        database: String,
        #[clap(long)]
        cors_origin: Option<String>,
    },
    Migrate {
        #[clap(subcommand)]
        direction: MigrateDirection,
    }
}

enum MigrateDirection {
    Up,
    Down,
    Fresh,
}
```

## Phase 2: REST API Implementation

### 2.1 Project Management Endpoints
```
GET    /api/v1/projects              - List all projects
POST   /api/v1/projects              - Create new project
GET    /api/v1/projects/{id}         - Get project details
PUT    /api/v1/projects/{id}         - Update project
DELETE /api/v1/projects/{id}         - Delete project
```

### 2.2 Plan Management Endpoints
```
GET    /api/v1/projects/{id}/plans           - List project plans
POST   /api/v1/projects/{id}/plans           - Create new plan
GET    /api/v1/projects/{id}/plans/{plan_id} - Get plan details
PUT    /api/v1/projects/{id}/plans/{plan_id} - Update plan
DELETE /api/v1/projects/{id}/plans/{plan_id} - Delete plan
POST   /api/v1/projects/{id}/plans/{plan_id}/execute - Execute plan
GET    /api/v1/projects/{id}/plans/{plan_id}/status  - Get execution status
```

### 2.3 Graph Data Endpoints
```
GET    /api/v1/projects/{id}/nodes     - List project nodes
POST   /api/v1/projects/{id}/nodes     - Create/update nodes (bulk)
DELETE /api/v1/projects/{id}/nodes     - Delete nodes (bulk)

GET    /api/v1/projects/{id}/edges     - List project edges
POST   /api/v1/projects/{id}/edges     - Create/update edges (bulk)
DELETE /api/v1/projects/{id}/edges     - Delete edges (bulk)

GET    /api/v1/projects/{id}/layers    - List project layers
POST   /api/v1/projects/{id}/layers    - Create/update layers (bulk)
DELETE /api/v1/projects/{id}/layers    - Delete layers (bulk)

POST   /api/v1/projects/{id}/import/csv - Import from CSV files
GET    /api/v1/projects/{id}/export/{format} - Export graph data
```

## Phase 3: GraphQL API Implementation

### 3.1 Schema Design
```graphql
type Project {
  id: ID!
  name: String!
  description: String
  plans: [Plan!]!
  nodes: [Node!]!
  edges: [Edge!]!
  layers: [Layer!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type Plan {
  id: ID!
  name: String!
  yamlContent: String!
  dependencies: [Plan!]!
  status: PlanStatus!
  project: Project!
}

type Node {
  id: ID!
  nodeId: String!
  label: String!
  layer: Layer
  properties: JSON
  project: Project!
}

type Edge {
  id: ID!
  sourceNodeId: String!
  targetNodeId: String!
  sourceNode: Node!
  targetNode: Node!
  properties: JSON
  project: Project!
}

type Layer {
  id: ID!
  layerId: String!
  name: String!
  color: String
  properties: JSON
  project: Project!
}

enum PlanStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
}
```

### 3.2 Query/Mutation Design
```graphql
type Query {
  projects: [Project!]!
  project(id: ID!): Project
  planExecutionStatus(projectId: ID!, planId: ID!): ExecutionStatus
}

type Mutation {
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: ID!, input: UpdateProjectInput!): Project!
  deleteProject(id: ID!): Boolean!
  
  createPlan(projectId: ID!, input: CreatePlanInput!): Plan!
  updatePlan(id: ID!, input: UpdatePlanInput!): Plan!
  executePlan(id: ID!): ExecutionResult!
  
  importGraphData(projectId: ID!, input: GraphDataInput!): ImportResult!
  updateGraphData(projectId: ID!, input: GraphDataInput!): UpdateResult!
}
```

## Phase 4: MCP (Model Context Protocol) Integration

### 4.1 MCP Server Implementation
- Implement MCP server that exposes layercake functionality
- Provide tools for:
  - Project management
  - Plan execution
  - Graph data querying and manipulation
  - Export generation

### 4.2 MCP Tools
```json
{
  "tools": [
    {
      "name": "layercake_create_project",
      "description": "Create a new layercake project",
      "parameters": { "name": "string", "description": "string" }
    },
    {
      "name": "layercake_import_csv",
      "description": "Import graph data from CSV files",
      "parameters": { "project_id": "number", "nodes_csv": "string", "edges_csv": "string", "layers_csv": "string" }
    },
    {
      "name": "layercake_execute_plan",
      "description": "Execute a plan in a project",
      "parameters": { "project_id": "number", "plan_id": "number" }
    },
    {
      "name": "layercake_export_graph",
      "description": "Export graph in specified format",
      "parameters": { "project_id": "number", "format": "string" }
    }
  ]
}
```

## Phase 5: Web Interface Foundation

### 5.1 Static Web Assets
- Serve static files for web interface
- React/TypeScript frontend (separate build process)
- WebSocket support for real-time plan execution updates

### 5.2 Web API Endpoints
```
GET    /                              - Serve web interface
GET    /static/*                      - Static assets
GET    /ws                            - WebSocket for real-time updates
```

## Implementation Roadmap

### Milestone 1: Basic Server + Database (Week 1-2)
1. Add server dependencies to Cargo.toml
2. Create SeaORM entities and migrations
3. Implement basic `layercake serve` command
4. Database initialization and migration system
5. Health check endpoint

### Milestone 2: REST API Core (Week 2-3)
1. Project CRUD operations
2. Plan CRUD operations
3. CSV import functionality
4. Basic export endpoints
5. Plan execution API

### Milestone 3: Graph Data Management (Week 3-4)
1. Node/Edge/Layer CRUD operations
2. Bulk operations for graph data
3. Data validation and constraints
4. Graph data export in multiple formats

### Milestone 4: GraphQL Implementation (Week 4-5)
1. GraphQL schema implementation
2. Resolvers for all entities
3. Subscriptions for real-time updates
4. GraphQL playground integration

### Milestone 5: MCP Integration (Week 5-6)
1. MCP server implementation
2. Tool definitions and handlers
3. Integration with existing CLI functionality
4. Documentation and examples

### Milestone 6: Web Interface Preparation (Week 6-7)
1. Static file serving
2. WebSocket implementation
3. API documentation
4. CORS and security headers

## File Structure Changes

```
src/
├── main.rs                 # Enhanced with serve command
├── lib.rs                  # Updated exports
├── server/                 # New server module
│   ├── mod.rs
│   ├── app.rs             # Axum app setup
│   ├── handlers/          # Request handlers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── plans.rs
│   │   ├── graph_data.rs
│   │   └── health.rs
│   ├── graphql/           # GraphQL implementation
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── resolvers.rs
│   └── websocket.rs       # WebSocket handlers
├── database/              # Database layer
│   ├── mod.rs
│   ├── entities/          # SeaORM entities
│   │   ├── mod.rs
│   │   ├── project.rs
│   │   ├── plan.rs
│   │   ├── node.rs
│   │   ├── edge.rs
│   │   └── layer.rs
│   ├── migrations/        # Database migrations
│   └── connection.rs      # Database connection setup
├── mcp/                   # MCP server implementation
│   ├── mod.rs
│   ├── server.rs
│   └── tools.rs
├── services/              # Business logic
│   ├── mod.rs
│   ├── project_service.rs
│   ├── plan_service.rs
│   ├── graph_service.rs
│   └── execution_service.rs
└── (existing modules...)
```

## Dependencies to Add

```toml
# Server framework
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "fs"] }

# Database
sea-orm = { version = "0.12", features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
sea-orm-migration = "0.12"

# GraphQL
async-graphql = "7.0"
async-graphql-axum = "7.0"

# WebSocket
axum-extra = { version = "0.9", features = ["typed-header"] }
tokio-tungstenite = "0.21"

# MCP
serde_json = "1.0" # (already exists)
uuid = { version = "1.0", features = ["v4"] }

# Additional utilities
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono"] }
```

## Configuration Management

Create `server_config.yaml`:
```yaml
server:
  host: "127.0.0.1"
  port: 3000
  cors_origins: ["http://localhost:3000", "http://localhost:5173"]

database:
  url: "sqlite:layercake.db"
  max_connections: 10

mcp:
  enabled: true
  tools: ["project", "plan", "graph", "export"]

logging:
  level: "info"
  format: "json"
```

## Security Considerations

1. **Input Validation**: Validate all inputs, especially YAML content and file paths
2. **Rate Limiting**: Implement rate limiting for API endpoints
3. **CORS**: Configurable CORS settings for web interface
4. **Authentication**: Prepare hooks for future authentication system
5. **SQL Injection**: Use SeaORM's query builder to prevent SQL injection
6. **File Access**: Restrict file system access for export/import operations

## Testing Strategy

1. **Unit Tests**: Test all service layer functions
2. **Integration Tests**: Test API endpoints
3. **Database Tests**: Test migrations and entity operations
4. **GraphQL Tests**: Test schema and resolvers
5. **MCP Tests**: Test MCP tool implementations
6. **Load Tests**: Test server performance under load

This plan provides a comprehensive roadmap for transforming layercake into a full-featured server application while maintaining backward compatibility with existing CLI functionality.