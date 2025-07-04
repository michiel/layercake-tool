# TODO

## Current Status: Phase 1 - Frontend Foundation Implementation

### ✅ Phase 0: Backend Infrastructure - COMPLETED
- [x] Server dependencies and SeaORM database entities
- [x] Database migrations and connection management
- [x] Complete REST API with Project/Plan/Graph CRUD endpoints
- [x] GraphQL API with full schema and resolvers
- [x] MCP API implementation with axum-mcp framework
- [x] OpenAPI documentation with Swagger UI
- [x] Static file serving and development infrastructure

### 🚧 Phase 1: Frontend Foundation - IN PROGRESS

#### ✅ Phase 1.1: Project Management UI - COMPLETED
- [x] React frontend project structure with Vite
- [x] TypeScript configuration and TanStack React Query
- [x] Project Management UI with full CRUD operations
- [x] Reusable UI components (Button, Input, Modal, etc.)
- [x] Custom React hooks for API integration
- [x] Production build pipeline with backend integration

#### ✅ Phase 1.2: Plan Editor - COMPLETED  
- [x] Plan Editor with JSON/YAML support
- [x] CodeEditor component with syntax highlighting
- [x] Plan templates and format switching
- [x] Plan CRUD operations with execution controls
- [x] Navigation integration between Projects and Plans
- [x] Advanced form validation and error handling

#### 🚧 Phase 1.3: Graph Visualization - IN PROGRESS
- [ ] Basic graph visualization component
- [ ] Node and edge rendering with D3.js
- [ ] Interactive controls (pan, zoom, selection)
- [ ] Graph data integration with backend APIs
- [ ] Real-time updates and collaboration features

#### Phase 1.4: Advanced Features - PENDING
- [ ] Spreadsheet-like data editor for bulk operations
- [ ] Real-time collaboration with SSE
- [ ] Export preview and template system
- [ ] Performance optimization for large graphs

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
