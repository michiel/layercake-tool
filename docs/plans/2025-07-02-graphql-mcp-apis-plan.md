# GraphQL and MCP APIs Implementation Plan

**Date**: 2025-07-02  
**Status**: Planning  
**Priority**: Medium  
**Dependencies**: REST API (✅ Completed), Server Architecture (✅ Completed)

## Overview

This plan outlines the implementation of GraphQL and Model Context Protocol (MCP) APIs alongside the existing REST API, leveraging a unified backend service layer to ensure consistency and maintainability across all API interfaces.

## Architecture Goals

### Unified Backend Design
```
┌─────────────────────────────────────────────────────────────────┐
│                     API Layer (Multiple Interfaces)            │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   REST API      │   GraphQL API   │        MCP API              │
│   (Existing)    │   (New)         │        (New)                │
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

## Phase 1: GraphQL API Implementation

### 1.1 Dependencies and Setup

**New Dependencies** (add to `Cargo.toml`):
```toml
# GraphQL dependencies (optional, feature-gated)
async-graphql = { version = "7.0", optional = true }
async-graphql-axum = { version = "7.0", optional = true }
```

**Feature Configuration**:
```toml
[features]
graphql = [
    "dep:async-graphql", 
    "dep:async-graphql-axum",
    "server"  # GraphQL requires server features
]
```

### 1.2 GraphQL Schema Design

**Core Types**:
```graphql
type Project {
  id: Int!
  name: String!
  description: String
  createdAt: DateTime!
  updatedAt: DateTime!
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
  status: PlanStatus!
  createdAt: DateTime!
  updatedAt: DateTime!
  project: Project!
}

type Node {
  id: Int!
  projectId: Int!
  nodeId: String!
  label: String!
  layerId: String
  properties: JSON
  project: Project!
}

type Edge {
  id: Int!
  projectId: Int!
  sourceNodeId: String!
  targetNodeId: String!
  properties: JSON
  project: Project!
}

type Layer {
  id: Int!
  projectId: Int!
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

**Queries**:
```graphql
type Query {
  # Project queries
  projects: [Project!]!
  project(id: Int!): Project
  
  # Plan queries
  plans(projectId: Int!): [Plan!]!
  plan(id: Int!): Plan
  
  # Graph data queries
  nodes(projectId: Int!): [Node!]!
  edges(projectId: Int!): [Edge!]!
  layers(projectId: Int!): [Layer!]!
  
  # Advanced queries
  graphData(projectId: Int!): GraphData!
  searchNodes(projectId: Int!, query: String!): [Node!]!
}

type GraphData {
  nodes: [Node!]!
  edges: [Edge!]!
  layers: [Layer!]!
}
```

**Mutations**:
```graphql
type Mutation {
  # Project mutations
  createProject(input: CreateProjectInput!): Project!
  updateProject(id: Int!, input: UpdateProjectInput!): Project!
  deleteProject(id: Int!): Boolean!
  
  # Plan mutations
  createPlan(input: CreatePlanInput!): Plan!
  updatePlan(id: Int!, input: UpdatePlanInput!): Plan!
  deletePlan(id: Int!): Boolean!
  executePlan(id: Int!): PlanExecutionResult!
  
  # Graph data mutations
  createNodes(projectId: Int!, nodes: [CreateNodeInput!]!): [Node!]!
  createEdges(projectId: Int!, edges: [CreateEdgeInput!]!): [Edge!]!
  createLayers(projectId: Int!, layers: [CreateLayerInput!]!): [Layer!]!
  
  # Bulk operations
  importCsv(projectId: Int!, csvData: CsvImportInput!): ImportResult!
  exportGraph(projectId: Int!, format: ExportFormat!): ExportResult!
}
```

**Subscriptions** (Real-time updates):
```graphql
type Subscription {
  planExecutionUpdates(planId: Int!): PlanExecutionUpdate!
  projectUpdates(projectId: Int!): ProjectUpdate!
}
```

### 1.3 Implementation Structure

**File Organization**:
```
src/
├── graphql/
│   ├── mod.rs                    # GraphQL module exports
│   ├── schema.rs                 # Combined schema definition
│   ├── types/                    # GraphQL type definitions
│   │   ├── mod.rs
│   │   ├── project.rs           # Project type & resolvers
│   │   ├── plan.rs              # Plan type & resolvers
│   │   ├── node.rs              # Node type & resolvers
│   │   ├── edge.rs              # Edge type & resolvers
│   │   ├── layer.rs             # Layer type & resolvers
│   │   └── scalars.rs           # Custom scalar types (JSON, DateTime)
│   ├── mutations/               # Mutation resolvers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── plans.rs
│   │   └── graph_data.rs
│   ├── queries/                 # Query resolvers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   ├── plans.rs
│   │   └── graph_data.rs
│   ├── subscriptions/           # Subscription resolvers
│   │   ├── mod.rs
│   │   └── plan_execution.rs
│   └── context.rs               # GraphQL context (database, services)
```

### 1.4 Service Layer Integration

**GraphQL Context**:
```rust
#[derive(Clone)]
pub struct GraphQLContext {
    pub db: DatabaseConnection,
    pub project_service: Arc<ProjectService>,
    pub plan_service: Arc<PlanService>, 
    pub graph_service: Arc<GraphService>,
    pub import_service: Arc<ImportService>,
    pub export_service: Arc<ExportService>,
}
```

**Resolver Implementation Pattern**:
```rust
#[Object]
impl Project {
    async fn plans(&self, ctx: &Context<'_>) -> Result<Vec<Plan>> {
        let context = ctx.data::<GraphQLContext>()?;
        context.plan_service.get_plans_by_project(self.id).await
    }
}
```

## Phase 2: MCP (Model Context Protocol) API Implementation

### 2.1 MCP Overview

MCP is a protocol for AI tools to interact with external systems. It provides:
- **Tools**: Functions AI can call to perform actions
- **Resources**: Data sources AI can read from
- **Prompts**: Templates AI can use for interactions

### 2.2 MCP Tools Implementation

**Core Tools**:
```json
{
  "tools": [
    {
      "name": "create_project",
      "description": "Create a new graph visualization project",
      "inputSchema": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "description": {"type": "string"}
        },
        "required": ["name"]
      }
    },
    {
      "name": "import_graph_data",
      "description": "Import nodes, edges, and layers from CSV data",
      "inputSchema": {
        "type": "object", 
        "properties": {
          "projectId": {"type": "integer"},
          "csvData": {"type": "object"}
        },
        "required": ["projectId", "csvData"]
      }
    },
    {
      "name": "execute_plan",
      "description": "Execute a transformation plan and generate outputs",
      "inputSchema": {
        "type": "object",
        "properties": {
          "planId": {"type": "integer"}
        },
        "required": ["planId"]
      }
    },
    {
      "name": "export_graph",
      "description": "Export graph in specified format (DOT, GML, JSON, etc.)",
      "inputSchema": {
        "type": "object",
        "properties": {
          "projectId": {"type": "integer"},
          "format": {"type": "string", "enum": ["dot", "gml", "json", "mermaid", "plantuml"]}
        },
        "required": ["projectId", "format"]
      }
    }
  ]
}
```

**Resource Endpoints**:
```json
{
  "resources": [
    {
      "uri": "layercake://projects",
      "name": "Projects List", 
      "description": "List of all graph visualization projects",
      "mimeType": "application/json"
    },
    {
      "uri": "layercake://project/{id}",
      "name": "Project Details",
      "description": "Detailed information about a specific project",
      "mimeType": "application/json"
    },
    {
      "uri": "layercake://project/{id}/graph",
      "name": "Graph Data",
      "description": "Complete graph data (nodes, edges, layers) for a project", 
      "mimeType": "application/json"
    }
  ]
}
```

### 2.3 MCP Implementation Structure

**Dependencies**:
```toml
# MCP dependencies (optional, feature-gated)
mcp-sdk = { version = "0.1", optional = true }
tokio-tungstenite = { version = "0.21", optional = true }
```

**File Organization**:
```
src/
├── mcp/
│   ├── mod.rs                    # MCP module exports
│   ├── server.rs                 # MCP server implementation
│   ├── tools/                    # MCP tool implementations
│   │   ├── mod.rs
│   │   ├── projects.rs           # Project management tools
│   │   ├── graph_data.rs         # Graph data tools
│   │   └── transformations.rs    # Plan execution tools
│   ├── resources/               # MCP resource handlers
│   │   ├── mod.rs
│   │   ├── projects.rs
│   │   └── graph_data.rs
│   └── prompts/                 # MCP prompt templates
│       ├── mod.rs
│       └── graph_analysis.rs
```

## Phase 3: Unified Service Layer Enhancement

### 3.1 Service Layer Abstraction

**Enhanced Service Traits**:
```rust
#[async_trait]
pub trait ProjectService {
    async fn create_project(&self, input: CreateProjectInput) -> Result<Project>;
    async fn get_project(&self, id: i32) -> Result<Option<Project>>;
    async fn list_projects(&self) -> Result<Vec<Project>>;
    async fn update_project(&self, id: i32, input: UpdateProjectInput) -> Result<Project>;
    async fn delete_project(&self, id: i32) -> Result<()>;
}

#[async_trait] 
pub trait PlanService {
    async fn create_plan(&self, input: CreatePlanInput) -> Result<Plan>;
    async fn execute_plan(&self, id: i32) -> Result<PlanExecutionResult>;
    async fn get_execution_status(&self, id: i32) -> Result<PlanStatus>;
    // ... other methods
}
```

### 3.2 Cross-API Consistency

**Shared Data Models**:
```rust
// Common input/output types used across REST, GraphQL, and MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
#[cfg_attr(feature = "graphql", derive(SimpleObject))]
pub struct ProjectData {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## Phase 4: Integration and Testing

### 4.1 Server Integration

**Enhanced App Configuration**:
```rust
pub async fn create_app(
    db: DatabaseConnection, 
    cors_origin: Option<&str>,
    enable_graphql: bool,
    enable_mcp: bool
) -> Result<Router> {
    let mut app = Router::new()
        .route("/health", get(health::health_check))
        .nest("/api/v1", rest_routes());
    
    if enable_graphql {
        let schema = build_graphql_schema(db.clone()).await?;
        app = app.nest("/graphql", graphql_routes(schema));
    }
    
    if enable_mcp {
        app = app.nest("/mcp", mcp_routes(db.clone()));
    }
    
    // ... rest of configuration
}
```

### 4.2 Feature Flags

**Cargo.toml Configuration**:
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

### 4.3 Testing Strategy

**Unit Tests**:
- Service layer tests (business logic)
- GraphQL resolver tests
- MCP tool function tests

**Integration Tests**:
- Cross-API consistency tests
- End-to-end workflow tests
- Performance benchmarks

**API Documentation**:
- REST: OpenAPI/Swagger (✅ Existing)
- GraphQL: GraphQL Playground/introspection
- MCP: MCP protocol documentation

## Implementation Timeline

### Phase 1: GraphQL (2-3 days)
1. **Day 1**: Dependencies, basic schema, query resolvers
2. **Day 2**: Mutation resolvers, service integration
3. **Day 3**: Subscriptions, testing, documentation

### Phase 2: MCP (2-3 days)  
1. **Day 1**: MCP server setup, basic tools
2. **Day 2**: Resource handlers, advanced tools
3. **Day 3**: Integration testing, documentation

### Phase 3: Integration (1-2 days)
1. **Day 1**: Unified service layer, feature flags
2. **Day 2**: Testing, performance optimization

## Success Criteria

1. **Functional Parity**: All REST API functionality available through GraphQL and MCP
2. **Performance**: GraphQL queries perform comparably to REST endpoints
3. **Consistency**: Data models and business logic unified across all APIs
4. **Documentation**: Comprehensive documentation for all three API types
5. **Testing**: 90%+ test coverage for new GraphQL and MCP functionality

## Risks and Mitigation

**Risks**:
- Complexity increase in codebase
- Performance overhead from multiple API layers
- Inconsistencies between API implementations

**Mitigation**:
- Strong service layer abstraction
- Comprehensive integration tests
- Performance monitoring and benchmarking
- Clear separation of concerns with feature flags

## Future Enhancements

1. **Real-time Features**: WebSocket subscriptions for live updates
2. **Advanced GraphQL**: DataLoader for N+1 query optimization
3. **MCP Extensions**: Custom prompt templates for graph analysis
4. **API Versioning**: Support for multiple API versions
5. **Rate Limiting**: Per-API rate limiting and throttling