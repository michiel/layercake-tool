# TODO

## Current Status: Phase 4 - GraphQL and MCP APIs Implementation

### ✅ Phase 1: Core Server Infrastructure - COMPLETED
- [x] Add server dependencies (now default features)
- [x] Create SeaORM database entities (Project, Plan, Node, Edge, Layer)
- [x] Implement database migrations
- [x] Add `layercake serve` command (enabled by default)
- [x] Create basic Axum server with health check
- [x] Implement database connection and initialization

### ✅ Phase 2: Basic REST API - COMPLETED
- [x] Project CRUD endpoints
- [x] Plan CRUD endpoints  
- [x] Graph data CRUD endpoints (nodes, edges, layers)
- [x] Plan execution API (stub)
- [x] CSV import/export endpoints (stubs)

### ✅ Phase 3: Complete REST API Implementation - COMPLETED
- [x] Implement CSV import functionality
- [x] Implement export functionality integration with existing engine
- [x] Implement plan execution API with existing engine
- [x] Add business logic services layer
- [x] Enhanced error handling and validation
- [x] Complete OpenAPI documentation with Swagger UI
- [x] Database commands (`layercake db init`, `layercake db migrate`)
- [x] Fix OpenAPI schema references and Swagger UI integration

### ✅ Phase 4: GraphQL API Implementation - COMPLETED
#### GraphQL Implementation
- [x] Add async-graphql dependencies with feature flags
- [x] Design comprehensive GraphQL schema (Projects, Plans, Nodes, Edges, Layers)
- [x] Implement GraphQL types and resolvers
- [x] Add GraphQL queries (projects, plans, graph data)
- [x] Add GraphQL mutations (CRUD operations, bulk operations)
- [x] Integrate GraphQL with existing service layer
- [x] Fix GraphQL server integration (axum version compatibility resolved)
- [x] Add GraphQL endpoint to server (`/graphql`)
- [x] Add GraphQL Playground/introspection
- [ ] Add GraphQL subscriptions (real-time updates)
- [ ] Test GraphQL functionality end-to-end

### ✅ Phase 5: MCP API Implementation - COMPLETED (Initial)

#### MCP (Model Context Protocol) Implementation - First Version
- [x] Add MCP SDK dependencies with feature flags
- [x] Design MCP tools for graph operations
- [x] Implement MCP server using axum-mcp framework
- [x] Add MCP endpoints to server (`/mcp`, `/mcp/sse`)
- [x] Test MCP functionality with server startup
- [x] Migrate from custom MCP to axum-mcp framework

### 🚧 Phase 6: MCP Implementation Reconciliation - EXECUTING

**Plans**: 
- [Reconciliation Analysis](docs/plans/2025-07-03-mcp-reconciliation-plan.md)  
- [Implementation Plan](docs/plans/2025-07-03-mcp-implementation-plan.md) ⭐ **ACTIVE**

#### Phase 1: Preparation & Backup (2-3 hours)
- [ ] Create backup branches for both MCP implementations
- [ ] Document current state and feature inventory
- [ ] Create comprehensive migration checklist

#### Phase 2: Core Framework Migration (6-8 hours)  
- [ ] Replace src/mcp/ base with axum-mcp framework
- [ ] Migrate and unify tool implementations from both versions
- [ ] Update imports and server integration

#### Phase 3: Layercake Integration (6-8 hours)
- [ ] Configure layercake:// URI scheme for resources
- [ ] Implement graph data resource providers
- [ ] Create graph analysis prompt templates

#### Phase 4: Advanced Analysis Tools (4-6 hours)
- [ ] Port connectivity analysis algorithms
- [ ] Port advanced graph analysis tools (path finding, statistics)

#### Phase 5: Integration & Testing (4-6 hours)
- [ ] Comprehensive testing and validation
- [ ] Claude Desktop compatibility verification
- [ ] Performance benchmarking

#### Missing Features from Legacy Implementation
- [x] ~~**Resources System**: URI-based resource access~~ ✅ **COMPLETED in axum-mcp**
- [x] ~~**Prompts System**: Graph analysis prompts with dynamic content~~ ✅ **COMPLETED in axum-mcp**
- [ ] **Advanced Analysis**: Connectivity analysis, path finding algorithms
- [ ] **Graph Statistics**: Connected components, degree distribution
- [ ] **WebSocket Support**: Native WebSocket transport compatibility (low priority)

#### axum-mcp Framework Status ✅ **MAJOR PROGRESS**
- [x] Create comprehensive roadmap for axum-mcp framework
- [x] ~~Fix Axum integration compilation issues~~ ✅ **COMPLETED**
- [x] ~~Implement ResourceRegistry trait~~ ✅ **COMPLETED** 
- [x] ~~Implement PromptRegistry trait~~ ✅ **COMPLETED**

**Framework Status Update**: axum-mcp is now ~90% complete with **all critical MCP features implemented**

#### Unified Backend Enhancement
- [ ] Enhance service layer abstraction for multi-API support
- [ ] Add shared data models with cross-API compatibility
- [ ] Implement feature flags (rest, graphql, mcp, all-apis)
- [ ] Add cross-API consistency tests

### Testing & Documentation
- [ ] Unit tests for all services
- [ ] Integration tests for API endpoints
- [ ] Update architecture documentation
- [ ] API documentation generation

## Future Enhancements

### Inline handlebars helpers

The handlerbars helpers for rendering recursive structures (`hierarchy_tree`),
are defined in Rust code. Inlining these as handlebars code would allow for
greater flexibility in Custom export renderings
