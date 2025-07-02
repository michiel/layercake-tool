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

### 🚧 Phase 5: MCP API Implementation - IN PROGRESS

#### MCP (Model Context Protocol) Implementation
- [ ] Add MCP SDK dependencies with feature flags
- [ ] Design MCP tools for graph operations
- [ ] Implement MCP resource handlers
- [ ] Add MCP server WebSocket support
- [ ] Create MCP prompt templates for graph analysis
- [ ] Integrate MCP with existing service layer
- [ ] Add MCP endpoint to server (`/mcp`)
- [ ] Test MCP functionality with AI clients

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
