# TODO

## Current Status: Phase 3 - Complete REST API Implementation

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

### 🚧 Phase 3: Complete REST API Implementation - IN PROGRESS
- [ ] Implement CSV import functionality
- [ ] Implement export functionality integration with existing engine
- [ ] Implement plan execution API with existing engine
- [ ] Add business logic services layer
- [ ] Enhanced error handling and validation
- [ ] Complete API documentation

### Advanced Features (Phase 3)
- [ ] GraphQL API implementation
- [ ] MCP (Model Context Protocol) integration
- [ ] WebSocket support for real-time updates
- [ ] Web interface static file serving

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
