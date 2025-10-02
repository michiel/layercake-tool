# Plan: Data Source Format Refactoring

## Overview

Refactor the data source system to separate file format (extension) from semantic type. Currently, the system infers the data type from the filename (e.g., "nodes.csv" → CSV Nodes), which is fragile and limits flexibility. The new system will allow users to upload files with supported extensions (CSV, TSV, JSON) and explicitly select the data type via an enumeration dropdown.

## Current State Analysis

### Backend (Rust)

**Database Schema** (`data_sources` table):
- `source_type: String` - Stores combined format + type (e.g., "csv_nodes", "json_graph")
- `filename: String` - Original filename with extension
- Inference logic in `DataSourceType::from_filename()` uses filename patterns

**Issues**:
1. Coupling between file format and semantic type
2. Fragile pattern matching (e.g., must contain "node" AND end with ".csv")
3. Cannot upload "mydata.csv" and specify it contains edges
4. No support for TSV format
5. Limited to specific naming conventions

### Frontend (TypeScript)

**Upload Flow** (`DataSourceUploader.tsx`):
- `determineFileType()` replicates backend pattern matching
- File type determined automatically from filename
- No user control over type selection
- Types: `'csv_nodes' | 'csv_edges' | 'csv_layers' | 'json_graph'`

**Issues**:
1. Duplicate type inference logic
2. No UI for explicit type selection
3. "unknown" type results in upload failure

## Target State

### Data Model Separation

**File Format** (Physical):
- CSV (Comma-Separated Values) - `.csv`
- TSV (Tab-Separated Values) - `.tsv`
- JSON (JavaScript Object Notation) - `.json`

**Data Type** (Semantic):
- NODES - Node/vertex data
- EDGES - Edge/relationship data
- LAYERS - Layer/group data
- GRAPH - Complete graph structure (JSON only)

### New Database Schema

```rust
pub struct Model {
    // ... existing fields ...
    pub file_format: String,     // NEW: "csv", "tsv", "json"
    pub data_type: String,        // CHANGED: "nodes", "edges", "layers", "graph"
    pub filename: String,          // UNCHANGED: original filename
    // ... existing fields ...
}
```

### GraphQL API Changes

```rust
#[derive(Enum)]
pub enum FileFormat {
    CSV,  // "CSV"
    TSV,  // "TSV"
    JSON, // "JSON"
}

#[derive(Enum)]
pub enum DataType {
    NODES,  // "NODES"
    EDGES,  // "EDGES"
    LAYERS, // "LAYERS"
    GRAPH,  // "GRAPH"
}

pub struct CreateDataSourceInput {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_content: String,       // Base64 encoded
    pub file_format: FileFormat,    // NEW: Explicit format
    pub data_type: DataType,        // NEW: Explicit type
}
```

## Implementation Status

**Current Progress**: Phase 1 - Backend Schema (Completed)

### Completed Tasks

✅ Migration m009 created with new columns
✅ Entity model updated with file_format and data_type fields
✅ New FileFormat and DataType enums created
✅ Validation methods added
✅ Service layer updated to accept new parameters
✅ TSV processing support added (delimiter parameter)
✅ GraphQL types updated with FileFormat and DataType enums
✅ GraphQL mutations updated to use new parameters
✅ All call sites updated (update_file, reprocess)
✅ Backend compiles successfully

### In Progress

🔄 Phase 3 - Frontend UI Updates

### Todo

- Update frontend DataSourceUploader.tsx with format detection and type dropdown
- Update frontend GraphQL mutations
- Update Plan DAG integration components
- Complete Phase 4 and 5

## Implementation Plan

### Phase 1: Backend Schema Migration

**Tasks**:
1. Create migration `m009_refactor_data_source_types.rs`
   - Add `file_format` column (TEXT, NOT NULL, default 'csv')
   - Add `data_type` column (TEXT, NOT NULL, default 'nodes')
   - Migrate existing data:
     - `csv_nodes` → format: `csv`, type: `nodes`
     - `csv_edges` → format: `csv`, type: `edges`
     - `csv_layers` → format: `csv`, type: `layers`
     - `json_graph` → format: `json`, type: `graph`
   - Drop old `source_type` column (in separate step for safety)

2. Update entity model (`data_sources.rs`)
   - Add `file_format: String` field
   - Rename/update `source_type` → `data_type: String`
   - Update `DataSourceType` enum to separate concerns:
     ```rust
     pub enum FileFormat {
         Csv,
         Tsv,
         Json,
     }

     pub enum DataType {
         Nodes,
         Edges,
         Layers,
         Graph,
     }
     ```
   - Remove `from_filename()` method (no longer needed)
   - Add validation: JSON format only supports GRAPH type

3. Update GraphQL types (`data_source.rs`)
   - Add `FileFormat` and `DataType` enums with GraphQL attributes
   - Update `CreateDataSourceInput` to include both fields
   - Update `DataSource` type to expose both fields
   - Add `UpdateDataSourceInput` support for changing type

**Validation Rules**:
- CSV/TSV formats: Support NODES, EDGES, LAYERS types
- JSON format: Only supports GRAPH type
- File extension must match declared format

### Phase 2: Backend Service Layer

**Tasks**:
1. Update `DataSourceService::create_from_file()`
   - Accept `file_format` and `data_type` parameters
   - Validate format matches file extension
   - Validate type is compatible with format
   - Remove filename-based type inference

2. Update file processors
   - `process_csv_nodes()` → `process_csv()` with type parameter
   - Add `process_tsv()` method (similar to CSV but tab-delimited)
   - Update `process_json_graph()` to validate complete graph structure
   - Consolidate CSV/TSV parsing logic

3. Add format detection utility
   - Validate file extension matches declared format
   - Detect delimiter for CSV vs TSV
   - Validate JSON structure

**Processing Logic**:
```rust
match (file_format, data_type) {
    (FileFormat::Csv, DataType::Nodes) => process_csv_nodes(),
    (FileFormat::Csv, DataType::Edges) => process_csv_edges(),
    (FileFormat::Csv, DataType::Layers) => process_csv_layers(),
    (FileFormat::Tsv, DataType::Nodes) => process_tsv_nodes(),
    (FileFormat::Tsv, DataType::Edges) => process_tsv_edges(),
    (FileFormat::Tsv, DataType::Layers) => process_tsv_layers(),
    (FileFormat::Json, DataType::Graph) => process_json_graph(),
    _ => Err("Invalid format/type combination"),
}
```

### Phase 3: Frontend UI Updates

**Tasks**:
1. Update `DataSourceUploader.tsx`
   - Remove automatic type detection
   - Add format detection from file extension
   - Add `Select` dropdown for data type selection
   - Show validation errors for invalid combinations
   - Update form schema:
     ```typescript
     interface UploadForm {
       name: string
       description: string
       file: File
       dataType: 'NODES' | 'EDGES' | 'LAYERS' | 'GRAPH'  // NEW
     }
     ```

2. Add UI components
   - Format badge (auto-detected, read-only)
   - Data type selector (dropdown)
   - Validation feedback:
     - ✓ CSV/TSV + Nodes/Edges/Layers
     - ✓ JSON + Graph
     - ✗ JSON + Nodes (invalid)
     - ✗ CSV + Graph (invalid)

3. Update GraphQL mutations (`datasources.ts`)
   - Add `fileFormat` and `dataType` to input
   - Update TypeScript types
   - Add format/type validation helpers

4. Update data source display components
   - `DataSourcesPage.tsx`: Show format + type separately
   - `DataSourceEditor.tsx`: Allow editing type (with validation)
   - Add badges for visual distinction:
     - Format badge (blue): "CSV", "TSV", "JSON"
     - Type badge (green): "Nodes", "Edges", "Layers", "Graph"

### Phase 4: Plan DAG Integration

**Tasks**:
1. Update `DataSourceNodeConfigForm.tsx`
   - Display both format and type
   - Filter data sources by compatible types
   - Show validation when incompatible type selected

2. Update `DataSourceSelectionDialog.tsx`
   - Group by type or format (user preference)
   - Show format + type in list items
   - Add filtering by type

3. Update node configuration validation
   - Ensure selected data source type matches expected input
   - Provide clear error messages

### Phase 5: Testing & Validation

**Tasks**:
1. Backend tests
   - Migration tests (data preservation)
   - Format/type validation
   - CSV/TSV parsing
   - Invalid combination rejection

2. Frontend tests
   - Upload form validation
   - Type selection interaction
   - Format detection

3. Integration tests
   - End-to-end upload flow
   - Type filtering in Plan DAG
   - Data source editing

4. Manual testing checklist
   - [ ] Upload CSV with Nodes type
   - [ ] Upload CSV with Edges type
   - [ ] Upload CSV with Layers type
   - [ ] Upload TSV with all types
   - [ ] Upload JSON with Graph type
   - [ ] Reject JSON with non-Graph type
   - [ ] Reject CSV with Graph type
   - [ ] Edit existing data source type
   - [ ] Filter by type in Plan DAG
   - [ ] Migrate existing data sources

## Migration Strategy

### Data Migration

**Step 1**: Add new columns with defaults
```sql
ALTER TABLE data_sources ADD COLUMN file_format TEXT NOT NULL DEFAULT 'csv';
ALTER TABLE data_sources ADD COLUMN data_type TEXT NOT NULL DEFAULT 'nodes';
```

**Step 2**: Migrate existing data
```sql
-- Migrate csv_nodes
UPDATE data_sources SET file_format = 'csv', data_type = 'nodes'
WHERE source_type = 'csv_nodes';

-- Migrate csv_edges
UPDATE data_sources SET file_format = 'csv', data_type = 'edges'
WHERE source_type = 'csv_edges';

-- Migrate csv_layers
UPDATE data_sources SET file_format = 'csv', data_type = 'layers'
WHERE source_type = 'csv_layers';

-- Migrate json_graph
UPDATE data_sources SET file_format = 'json', data_type = 'graph'
WHERE source_type = 'json_graph';
```

**Step 3**: Drop old column (after verification)
```sql
ALTER TABLE data_sources DROP COLUMN source_type;
```

### Rollback Plan

If issues arise:
1. Keep `source_type` column during transition
2. Populate it from new columns: `format + '_' + type`
3. Allow rollback to previous version
4. Remove old column only after stable release

## File Format Specifications

### CSV/TSV Requirements

**Nodes**:
- Required columns: `id`, `label`
- Optional columns: `layer`, `x`, `y`, `description`, `color`

**Edges**:
- Required columns: `id`, `source`, `target`
- Optional columns: `label`, `description`, `weight`, `color`

**Layers**:
- Required columns: `id`, `label`
- Optional columns: `color`, `description`, `z_index`

**Delimiter**:
- CSV: Comma (`,`)
- TSV: Tab (`\t`)

### JSON Requirements

**Graph**:
- Must contain complete graph structure:
  ```json
  {
    "nodes": [...],
    "edges": [...],
    "layers": [...]  // optional
  }
  ```

## UI/UX Considerations

### Upload Flow

1. User clicks "Upload Data Source"
2. User selects file (CSV, TSV, or JSON)
3. System auto-detects format from extension
4. User provides:
   - Name (required)
   - Description (optional)
   - **Data Type dropdown** (required, validated against format)
5. System validates combination
6. User confirms upload
7. System processes and stores

### Validation Messages

- **Invalid combination**: "JSON files can only contain Graph data. Please select a CSV or TSV file for Nodes/Edges/Layers data."
- **Format mismatch**: "File extension .csv doesn't match selected format TSV"
- **Missing type**: "Please select the data type for this file"

### Visual Design

```
┌─────────────────────────────────────┐
│ Upload Data Source                  │
├─────────────────────────────────────┤
│ File: mydata.csv                    │
│ Format: [CSV] (auto-detected)       │
│                                     │
│ Data Type: [Dropdown ▼]            │
│   • Nodes                           │
│   • Edges                           │
│   • Layers                          │
│   • Graph (disabled for CSV)        │
│                                     │
│ Name: [___________________]         │
│ Description: [___________]          │
│                                     │
│           [Cancel] [Upload]         │
└─────────────────────────────────────┘
```

## Success Criteria

1. ✅ Users can upload any CSV/TSV/JSON file
2. ✅ Users explicitly select data type
3. ✅ System validates format/type compatibility
4. ✅ TSV format supported
5. ✅ Existing data sources migrated successfully
6. ✅ No breaking changes to Plan DAG functionality
7. ✅ Clear error messages for invalid combinations
8. ✅ Backward compatibility during migration period

## Risk Mitigation

**Risk**: Data loss during migration
- **Mitigation**: Multi-step migration, keep old column initially

**Risk**: Breaking existing integrations
- **Mitigation**: Maintain compatibility layer, staged rollout

**Risk**: User confusion with type selection
- **Mitigation**: Clear validation, helpful error messages, tooltips

**Risk**: Invalid existing data
- **Mitigation**: Validation script before migration, manual review

## Timeline Estimate

- Phase 1 (Backend Schema): 2-3 days
- Phase 2 (Backend Service): 2-3 days
- Phase 3 (Frontend UI): 3-4 days
- Phase 4 (Integration): 2 days
- Phase 5 (Testing): 2-3 days

**Total**: 11-15 days

## Dependencies

- SeaORM migration system
- async-graphql enum support
- Mantine UI Select component
- Apollo Client mutation updates

## Notes

- Consider adding format/type presets for common use cases
- Future: Auto-detect type from CSV headers (optional convenience)
- Future: Support for additional formats (Parquet, Arrow, etc.)
- Future: Inline editing of data in browser
