# Changes

## v0.1.7 (Upcoming)

### 🚀 New Features
- **Database Commands**: Added `layercake db init` and `layercake db migrate` commands with SQLite defaults
- **OpenAPI Documentation**: Integrated utoipa for OpenAPI 3.0 specification and Swagger UI
  - Interactive documentation available at `/docs` endpoint
  - OpenAPI specification available at `/api-docs/openapi.json`
- **Enhanced Server Logging**: HTTP routes are now logged during server startup

### 🏗️ Server Architecture (Major)
- **Single Binary**: Evolved from CLI tool to unified binary with server functionality enabled by default
- **Complete REST API**: Implemented comprehensive API for projects, plans, and graph data management
  - Project CRUD operations (`/api/v1/projects`)
  - Plan management and execution (`/api/v1/projects/{id}/plans`)
  - Graph data operations for nodes, edges, and layers
  - CSV import/export endpoints with bulk operations
- **Database Layer**: Added SeaORM-based persistence with SQLite support
  - Automatic database migrations
  - Structured entities for projects, plans, nodes, edges, and layers
- **Business Logic Services**: Created service layer for import, export, and graph operations
  - Full integration with existing export engine
  - Support for all graph transformations and export formats
- **Backward Compatibility**: All existing CLI functionality preserved

### 🐛 Bug Fixes
- Fixed SQLite database URL format for proper file creation (added `mode=rwc` parameter)

### 🔧 Technical Improvements
- Enhanced server startup with comprehensive route logging
- Added proper CORS configuration for web interface compatibility
- Improved error handling and validation across API endpoints

### 📚 Documentation
- Updated architecture documentation with server implementation details
- Added comprehensive API endpoint documentation
- Enhanced development guides and project structure documentation
