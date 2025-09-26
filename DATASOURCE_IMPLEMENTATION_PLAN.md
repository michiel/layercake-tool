# DataSource Implementation Plan

## Overview

Based on the SPECIFICATION.md changes, we need to implement a simplified DataSource system that allows projects to manage file-based data imports and use them as source nodes in Plan DAGs.

## Requirements Analysis

### From SPECIFICATION.md & Modifications:
- Each Project has DataSources (project-level entities)
- DataSources store raw import data (`blob` attribute) via file upload only
- DataSources contain processed graph data (`graph_json` attribute with `{nodes:[], edges:[], layers:[]}`)
- DataSources can be referenced as nodes in PlanDAG
- DataSources connect to GraphNodes via edges (one-to-many relationship)
- DataSources need dedicated CRUD management pages with upload/download functionality

### Simplified Scope:
- **File-based only**: No REST endpoints or SQL queries
- **Limited formats**: Only CSV files (nodes.csv, edges.csv, layers.csv) and raw JSON (graph.json format)
- **Upload/Download**: Simple file upload with immediate processing and download options
- **Auto-processing**: Immediate graph_json generation on file upload

### Current State:
- ✅ Frontend DataSourceNode component (basic)
- ✅ TypeScript interfaces for DataSourceNodeConfig
- ✅ PlanDAG integration ready
- ❌ Missing: Database schema, backend services, management UI

## Implementation Phases

### Phase 1: Database Schema & Migrations
**Status: ✅ Completed**

#### 1.1 Create DataSources Entity
- Create `data_sources.rs` entity
- Fields:
  - `id: i32` (primary key)
  - `project_id: i32` (foreign key to projects)
  - `name: String` (user-friendly name)
  - `description: Option<String>`
  - `source_type: String` ('csv_nodes', 'csv_edges', 'csv_layers', 'json_graph')
  - `filename: String` (original uploaded filename)
  - `blob: Vec<u8>` (raw uploaded file data)
  - `graph_json: String` (processed graph data as JSON)
  - `status: String` ('active', 'processing', 'error')
  - `error_message: Option<String>` (processing error details)
  - `file_size: i64` (size of uploaded file in bytes)
  - `processed_at: Option<ChronoDateTimeUtc>` (when processing completed)
  - `created_at: ChronoDateTimeUtc`
  - `updated_at: ChronoDateTimeUtc`

#### 1.2 Update Related Entities
- Add DataSource relations to `projects.rs`
- Update `mod.rs` to include data_sources
- Create migration for data_sources table

#### 1.3 Update PlanDAG Node Config
- Modify `DataSourceNodeConfig` to reference `data_source_id`
- Update validation to ensure referenced DataSource exists

### Phase 2: Backend Services & GraphQL
**Status: ✅ Completed**

#### 2.1 DataSource Service
- Create `data_source_service.rs`
- CRUD operations: create, read, update, delete
- File processing: CSV parsing and JSON graph generation
  - CSV nodes parser: id, label, layer, x, y, metadata
  - CSV edges parser: id, source, target, label, metadata
  - CSV layers parser: id, label, color, metadata
  - JSON graph validator: {nodes:[], edges:[], layers:[]} format
- Download operations: serve raw blob and graph_json
- Validation: ensure processed graph_json format compliance

#### 2.2 GraphQL Schema & File Upload
- Add DataSource type to GraphQL schema
- Queries: `dataSource`, `dataSources(projectId)`, `downloadDataSource`
- Mutations: `createDataSourceFromFile`, `updateDataSource`, `deleteDataSource`, `uploadDataSourceFile`
- File upload support via GraphQL Upload scalar
- Download endpoints for raw file and processed JSON

#### 2.3 MCP Tools
- Add DataSource management to MCP tools
- Enable AI agents to create/manage DataSources

### Phase 3: Frontend Management Interface
**Status: Not Started**

#### 3.1 DataSource Management Pages
- Create `DataSourcesPage.tsx` - main management interface with data grid
- Create `DataSourceEditor.tsx` - create/edit individual DataSources
- Create `DataSourceUploader.tsx` - file upload component with drag-and-drop
- Add navigation routes and menu items

#### 3.2 Integration with Project Structure
- Add DataSources tab to project dashboard
- Show DataSource count and status in project overview
- Link DataSources to PlanDAG usage

#### 3.3 File Upload & Download Workflows
- File upload with drag-and-drop support
- Supported formats: .csv (nodes/edges/layers), .json (graph format)
- Immediate processing with progress indicator
- Download buttons for raw file and processed JSON
- Error handling and validation feedback
- File size limits and format validation

### Phase 4: PlanDAG Integration Enhancement
**Status: Partially Complete**

#### 4.1 Enhanced DataSourceNode
- Improve DataSourceNode component to show DataSource details
- Add DataSource selection dialog
- Display import status and data preview
- Show connection indicators to GraphNodes

#### 4.2 Edge Validation
- Ensure DataSource nodes can only connect to GraphNodes
- Validate that referenced DataSource exists
- Update edge creation validation logic

#### 4.3 Data Flow Visualization
- Show data lineage from DataSource through GraphNodes
- Display data transformation pipeline
- Add data freshness indicators

### Phase 5: Advanced Features
**Status: Not Started**

#### 5.1 File Management Enhancements
- File versioning (keep history of uploads)
- File validation and preview before processing
- Batch file upload for multiple DataSources
- File format auto-detection

#### 5.2 Processing & Validation
- Advanced CSV parsing options (custom delimiters, headers)
- Schema validation for uploaded files
- Data quality checks and warnings
- Processing status notifications

#### 5.3 Collaboration Features
- Multi-user DataSource editing with locking
- File upload conflict resolution
- Shared DataSource templates and examples
- Activity log for DataSource changes

## Technical Specifications

### Database Schema
```sql
CREATE TABLE data_sources (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    source_type TEXT NOT NULL, -- 'csv_nodes', 'csv_edges', 'csv_layers', 'json_graph'
    filename TEXT NOT NULL, -- original uploaded filename
    blob BLOB NOT NULL, -- raw uploaded file data
    graph_json TEXT NOT NULL, -- processed JSON with {nodes:[], edges:[], layers:[]}
    status TEXT NOT NULL DEFAULT 'active', -- 'active', 'processing', 'error'
    error_message TEXT, -- processing error details if status='error'
    file_size INTEGER NOT NULL, -- size in bytes
    processed_at TIMESTAMP, -- when processing completed
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_data_sources_project_id ON data_sources(project_id);
CREATE INDEX idx_data_sources_status ON data_sources(status);
CREATE INDEX idx_data_sources_type ON data_sources(source_type);
```

### GraphQL Schema Addition
```graphql
scalar Upload

type DataSource {
    id: Int!
    projectId: Int!
    name: String!
    description: String
    sourceType: String! # 'csv_nodes', 'csv_edges', 'csv_layers', 'json_graph'
    filename: String!
    graphJson: String!
    status: String! # 'active', 'processing', 'error'
    errorMessage: String
    fileSize: Int!
    processedAt: DateTime
    createdAt: DateTime!
    updatedAt: DateTime!

    # Relations
    project: Project!
    planDagNodes: [PlanDagNode!]!
}

input CreateDataSourceInput {
    projectId: Int!
    name: String!
    description: String
    file: Upload! # uploaded file
}

input UpdateDataSourceInput {
    name: String
    description: String
    file: Upload # optional file re-upload
}

extend type Query {
    dataSource(id: Int!): DataSource
    dataSources(projectId: Int!): [DataSource!]!
    downloadDataSourceRaw(id: Int!): String! # returns download URL
    downloadDataSourceJson(id: Int!): String! # returns download URL
}

extend type Mutation {
    createDataSourceFromFile(input: CreateDataSourceInput!): DataSource!
    updateDataSource(id: Int!, input: UpdateDataSourceInput!): DataSource!
    deleteDataSource(id: Int!): Boolean!
    reprocessDataSource(id: Int!): DataSource! # reprocess existing file
}

extend type Subscription {
    dataSourceUpdated(projectId: Int!): DataSource!
}
```

### Updated DataSourceNodeConfig
```typescript
export interface DataSourceNodeConfig {
    dataSourceId: number; // Reference to DataSource entity
    displayMode: 'summary' | 'detailed' | 'preview';
    outputGraphRef: string; // Target graph reference

    // Legacy support (to be deprecated after migration)
    inputType?: 'CSVNodesFromFile' | 'CSVEdgesFromFile' | 'CSVLayersFromFile';
    source?: string;
    dataType?: 'Nodes' | 'Edges' | 'Layers';
}

// File processing types
export type DataSourceType = 'csv_nodes' | 'csv_edges' | 'csv_layers' | 'json_graph';

export interface ProcessedGraphData {
    nodes: GraphNode[];
    edges: GraphEdge[];
    layers: GraphLayer[];
}

// CSV format specifications
export interface CSVNodeRow {
    id: string;
    label: string;
    layer?: string;
    x?: number;
    y?: number;
    [key: string]: any; // additional metadata
}

export interface CSVEdgeRow {
    id: string;
    source: string;
    target: string;
    label?: string;
    [key: string]: any; // additional metadata
}

export interface CSVLayerRow {
    id: string;
    label: string;
    color?: string;
    [key: string]: any; // additional metadata
}
```

## Dependencies & Prerequisites

1. **Database Migration System** - Need migration framework for schema changes
2. **File Upload Handling** - For blob storage of uploaded files
3. **GraphQL Upload Support** - For file uploads in GraphQL mutations (graphql-upload crate)
4. **CSV Parsing** - CSV processing library (csv crate)
5. **File Download** - HTTP endpoints for serving files
6. **Frontend File Handling** - File upload components (react-dropzone)

## Risk Assessment

### High Risk
- **File Size Limits** - Large CSV/JSON files may impact database performance
- **Processing Time** - Large files may need async processing to avoid timeouts
- **Data Migration** - Existing projects may have legacy DataSource references

### Medium Risk
- **File Format Validation** - Need robust CSV parsing and error handling
- **Storage Space** - Blob storage may grow quickly with multiple file uploads
- **UI Responsiveness** - File upload/processing should not block UI

### Low Risk
- **Simplified Scope** - File-only approach reduces complexity
- **Type Safety** - Well-defined file formats and interfaces
- **Incremental Rollout** - Can implement phases independently

## Success Criteria

### Phase 1 Complete ✅
- [x] DataSources table exists with file-focused schema
- [x] Can create/read DataSources via backend services
- [x] File processing works for CSV and JSON formats
- [x] PlanDAG nodes can reference DataSource IDs

### Phase 2 Complete ✅
- [x] GraphQL file upload mutations work
- [x] CSV parsing generates correct graph_json
- [x] JSON validation ensures proper format
- [x] Download endpoints serve raw and processed files
- [ ] MCP tools can manage DataSources (TODO: Next phase)

### Phase 3 Complete
- [ ] DataSources management page with file upload
- [ ] Drag-and-drop file upload component
- [ ] Download buttons for raw and processed files
- [ ] Error handling for invalid files
- [ ] DataSources integrated into project dashboard

### Phase 4 Complete
- [ ] Enhanced DataSourceNode with file info display
- [ ] DataSource selection dialog in PlanDAG
- [ ] Proper edge validation between DataSource and GraphNodes
- [ ] File processing status indicators

### Final Success
- [ ] Complete file-based DataSource management
- [ ] Robust CSV/JSON processing with error handling
- [ ] Seamless upload/download workflows
- [ ] Production-ready file handling and validation

## Next Steps

1. **Immediate**: Create database entity with file-focused schema
2. **Week 1**: Implement file processing service and GraphQL mutations
3. **Week 2**: Build file upload/download UI components
4. **Week 3**: Enhance PlanDAG integration with DataSource selection
5. **Week 4**: Add validation, error handling, and polish

This simplified plan focuses on file-based DataSource management with immediate processing and download capabilities, avoiding the complexity of external data connections while providing a solid foundation for future enhancements.