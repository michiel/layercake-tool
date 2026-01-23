# Library System Upgrade Plan

## Overview

Upgrade the library system from a dataset-only repository to a multi-type resource library supporting datasets, sample projects, and project templates.

## Current State

- Library entity only stores datasets
- UI presents datasets in a simple list
- Sample projects are hardcoded and created via special GraphQL mutation
- No way to export/import projects as templates

## Goals

1. Support multiple library item types (datasets, projects, project templates)
2. Enable searching across all library items
3. Filter library items by type when accessing for specific purposes
4. Move sample projects into the library as 'project' type items
5. Support exporting existing projects as templates
6. Support importing project templates from library

## Architecture Changes

## Status

- ✅ **Stage 1** (data model + migration) implemented: `library_items` table with SeaORM entity, migration copying existing datasets, GraphQL schema updated, REST upload/download endpoints added.
- ⏳ **Stage 2** (sample project migration) pending – hardcoded bundles still active until export utility is wired into seeding.
- ✅ **Stage 3/4** core backend flow: export project → template ZIP, create project from library templates, GraphQL mutations and UI wiring in place. Additional polishing (partial exports, validation telemetry) still TBD.
- ✅ **Stage 5** first pass at refreshed library UI: filters, search, unified cards, dataset/template uploads, REST download integration. Filters and manage controls now sit side-by-side on desktop layouts, keeping the results grid visible without extra scrolling.
- ✅ **Stage 6** dataset creation page now supports importing datasets directly from the library.
- 🔄 **Stage 7** export polish in progress – the project overview now owns the export dialog with separate “Export” and “Export as template” tabs so users don’t have to hunt through Artefacts to publish or download bundles.

---

### 1. Data Model

#### Library Entity Schema

Update `library_sources` table to support multiple types with blob storage:

```sql
CREATE TABLE library_items (
  id INTEGER PRIMARY KEY,
  type TEXT NOT NULL,  -- 'dataset', 'project', 'project_template'
  name TEXT NOT NULL,
  description TEXT,
  tags TEXT DEFAULT '[]',  -- JSON array for categorisation
  metadata TEXT DEFAULT '{}',  -- JSON object for type-specific data
  content_blob BLOB NOT NULL,  -- Binary content (CSV, JSON, ZIP, etc.)
  content_size INTEGER,  -- Size in bytes
  content_type TEXT DEFAULT '',  -- MIME type
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)
```

**Changes from current schema:**
- Remove `graph_json` field (stored in blob instead)
- Remove `source_type` field (derived from type)
- Remove `file_path` field (stored as blob)
- Remove `csv_data` field (stored in blob)
- Add `content_blob` field for all content
- Add `type` field to distinguish item types

**Metadata field examples:**
- Dataset: `{"format": "csv", "row_count": 1000, "column_count": 5, "headers": [...]}`
- Project: `{"is_sample": true, "node_count": 10, "dataset_count": 3}`
- Project Template: `{"original_project_id": 123, "node_count": 10, "dataset_count": 3, "empty_datasets": true}`

#### Migration Strategy

1. Create new `library_items` table with BLOB storage
2. Migrate existing `library_sources` data:
   - Copy `csv_data` or `graph_json` to `content_blob`
   - Set `type='dataset'`
   - Convert `tags` to JSON array format
   - Compute `content_size` from blob length
3. Migrate hardcoded sample projects to `library_items` with type='project'
4. Drop old `library_sources` table
5. Update all code references to use new schema

**Migration considerations:**
- Handle both CSV and graph datasets from old schema
- Preserve all existing tags and metadata
- Verify blob integrity after migration
- Provide rollback script for safety

### 2. Backend Changes

#### Entity Updates

**File:** `layercake-core/src/database/entities/library_items.rs`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "library_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub item_type: String,  // 'dataset', 'project', 'project_template'
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(column_type = "Text", default_value = "[]")]
    pub tags: String,  // JSON array
    #[sea_orm(column_type = "Text", default_value = "{}")]
    pub metadata: String,  // JSON object
    #[sea_orm(column_type = "Binary(BlobSize::Blob(None))")]
    pub content_blob: Vec<u8>,  // Binary content
    pub content_size: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}
```

#### GraphQL Types

**File:** `layercake-core/src/graphql/types/library.rs`

```graphql
enum LibraryItemType {
  DATASET
  PROJECT
  PROJECT_TEMPLATE
}

type LibraryItem {
  id: Int!
  type: LibraryItemType!
  name: String!
  description: String
  tags: [String!]!
  metadata: JSON!
  contentSize: Int
  createdAt: String!
  updatedAt: String!
}

# Note: content_blob is not exposed via GraphQL
# Content is retrieved via separate download endpoint

input LibraryItemFilter {
  types: [LibraryItemType!]
  tags: [String!]
  searchQuery: String
}

type Query {
  libraryItems(filter: LibraryItemFilter): [LibraryItem!]!
  libraryItem(id: Int!): LibraryItem
}

type Mutation {
  uploadLibraryItem(input: UploadLibraryItemInput!): LibraryItem!
  deleteLibraryItem(id: Int!): Boolean!
  exportProjectAsTemplate(projectId: Int!): LibraryItem!
  createProjectFromLibrary(libraryItemId: Int!, name: String): Project!
}
```

**HTTP Endpoints for Blob Content:**

Since BLOBs are not efficiently transferred via GraphQL, provide REST endpoints:

```
GET  /api/library/:id/download    # Download blob content
POST /api/library/upload           # Upload blob content (multipart/form-data)
```

These endpoints:
- Stream large blobs efficiently
- Set appropriate Content-Type headers
- Support Content-Disposition for downloads
- Validate authentication/authorization

#### Export Project as Template

**File:** `layercake-core/src/app_context.rs`

Add method to export project as template:

```rust
pub async fn export_project_as_template(
    &self,
    project_id: i32,
) -> Result<LibraryItem> {
    // 1. Load project with full DAG
    // 2. Load all datasets referenced in DAG
    // 3. Create empty versions of datasets (schema only)
    // 4. Create ZIP archive in memory with:
    //    - project_metadata.json (name, description, tags)
    //    - dag.json (full DAG structure)
    //    - datasets/ folder with empty dataset files
    // 5. Get ZIP bytes
    // 6. Insert into library_items with:
    //    - type='project_template'
    //    - content_blob = ZIP bytes
    //    - content_size = byte length
    // 7. Return library item
}
```

**ZIP structure:**
```
project_template.zip
├── manifest.json          # Version, type, metadata
├── project.json           # Name, description, tags
├── dag.json              # Full DAG structure
└── datasets/
    ├── dataset_1.csv     # Empty CSV with headers only
    ├── dataset_2.csv
    └── ...
```

#### Import Project from Library

**File:** `layercake-core/src/app_context.rs`

Add method to create project from library item:

```rust
pub async fn create_project_from_library(
    &self,
    library_item_id: i32,
    project_name: Option<String>,
) -> Result<Project> {
    // 1. Load library item with content_blob
    // 2. Verify type is 'project' or 'project_template'
    // 3. Extract ZIP archive from content_blob bytes
    // 4. Parse manifest.json and validate structure
    // 5. Create new project with name (or use template name)
    // 6. Import all datasets from ZIP
    // 7. Import DAG structure from dag.json
    // 8. Return new project
}
```

### 3. Frontend Changes

#### Library Page Updates

**File:** `frontend/src/components/library/LibrarySourcesPage.tsx`

Rename to `LibraryPage.tsx` and update to:

1. **Filter Controls:**
   - Type filter (All, Datasets, Projects, Templates)
   - Tag filter (similar to project tags)
   - Search bar (name/description)

2. **Item List:**
   - Group by type or show mixed with type badges
   - Icon per type (IconDatabase, IconFolderPlus, IconTemplate)
   - Show metadata summary (size, date, item count)
   - Actions per type:
     - Dataset: Use in new dataset, Delete
     - Project: Create from template, Delete
     - Template: Create project, Delete

3. **Upload Section:**
   - Support uploading datasets (CSV)
   - Support uploading project templates (ZIP)
   - Drag-and-drop support

#### Dataset Creation Integration

**File:** `frontend/src/pages/DatasetCreationPage.tsx`

Add "From Library" option:
1. Show modal with library items filtered by type='dataset'
2. Select dataset to seed new dataset
3. Import data and proceed with creation flow

#### Project Template Export

**File:** `frontend/src/pages/ProjectDetailPage.tsx` or new export dialog

Add export button to project overview:
1. "Export as Template" button
2. Dialog shows:
   - Template name (pre-filled with project name)
   - Description
   - Tags
   - Preview of what will be included (DAG nodes, empty datasets)
   - Confirm export
3. Creates library item with type='project_template'
4. Shows success notification with link to library

#### Project Creation from Library

Update project creation flows:

**HomePage and ProjectsPage:**
- Add "From Library Template" button alongside "New Project" and "Import Sample"
- Shows modal with library items filtered by type in ['project', 'project_template']
- Select template and provide new project name
- Creates project from template

### 4. Storage Strategy

#### Blob Storage

All library content stored as BLOBs directly in SQLite database:

**Benefits:**
- Single source of truth (no file/DB sync issues)
- Atomic transactions (content + metadata updated together)
- Simplified backup (just database file)
- No file path management needed
- Built-in ACID guarantees

**Content types and formats:**
- **Datasets**: Raw CSV/JSON bytes
- **Projects**: ZIP archive bytes (containing project structure)
- **Project Templates**: ZIP archive bytes (containing template structure)

**Size considerations:**
- SQLite BLOB limit: 2GB (more than sufficient for templates)
- Recommend max upload size: 100MB for usability
- content_size field tracks actual size for UI display

**Content retrieval:**
- Metadata queries via GraphQL (fast, no BLOB transfer)
- Content download via separate HTTP endpoint when needed
- Stream large BLOBs to avoid memory issues

## Implementation Stages

### Stage 1: Data Model & Migration (Backend Focus)

**Goal:** Update database schema and migrate existing data

**Tasks:**
1. Create migration for `library_items` table
2. Update entity models in `layercake-core/src/database/entities/`
3. Create migration script to copy `library_sources` → `library_items`
4. Update GraphQL types in `layercake-core/src/graphql/types/library.rs`
5. Update queries/mutations to use new schema
6. Write tests for migration

**Success Criteria:**
- All existing library datasets migrated successfully
- GraphQL queries return library items with type field
- Tests pass

### Stage 2: Sample Projects Migration

**Goal:** Move hardcoded samples into library

**Tasks:**
1. Create sample project export utility
2. Export each hardcoded sample as library item (type='project')
3. Update `createSampleProject` mutation to use library
4. Update frontend to query library items instead of `sampleProjects` query
5. Remove hardcoded sample definitions

**Success Criteria:**
- Sample projects visible in library
- Can create projects from library samples
- Legacy sample code removed

### Stage 3: Project Template Export

**Goal:** Enable exporting projects as templates

**Tasks:**
1. Implement `export_project_as_template` in app_context
2. Create ZIP archive builder
3. Create empty dataset generator (schema preservation)
4. Add GraphQL mutation `exportProjectAsTemplate`
5. Add export UI to project detail page
6. Add download option for exported template

**Success Criteria:**
- Can export project as template
- Template ZIP contains correct structure
- Template appears in library with type='project_template'

### Stage 4: Project Template Import

**Goal:** Enable creating projects from templates

**Tasks:**
1. Implement `create_project_from_library` in app_context
2. Create ZIP extraction and validation
3. Add GraphQL mutation `createProjectFromLibrary`
4. Update project creation UI to include library option
5. Add template preview/metadata display

**Success Criteria:**
- Can create project from library template
- DAG structure preserved
- Empty datasets created correctly
- New project fully functional

### Stage 5: Library UI Enhancement

**Goal:** Rich library browsing and management

**Tasks:**
1. Rename LibrarySourcesPage → LibraryPage
2. Add type filter controls
3. Add tag filter integration
4. Add search functionality
5. Update item cards with type-specific icons and actions
6. Add upload support for templates
7. Improve drag-and-drop UI

**Success Criteria:**
- Can filter library by type
- Can search across all items
- Upload works for both datasets and templates
- UI clearly distinguishes item types

### Stage 6: Dataset Creation Integration

**Goal:** Use library datasets when creating new datasets

**Tasks:**
1. Update DatasetCreationPage to show library option
2. Add library item picker filtered by type='dataset'
3. Implement data seeding from library dataset
4. Add preview before import

**Success Criteria:**
- Can select dataset from library when creating new dataset
- Data imports correctly
- Preview shows accurate data

## Technical Considerations

### File Size Limits

- Set maximum upload size for templates (e.g., 100MB)
- Show progress indicator for large uploads
- Validate ZIP structure before accepting

### Security

- Validate ZIP contents before extraction (no path traversal)
- Scan for malicious content
- Limit extraction to safe directory
- Validate JSON structure in manifest files

### Performance

- Index library_items table on (type, tags, name)
- Lazy-load item previews
- Stream large file uploads
- Cache frequently accessed templates

### Versioning

Include version field in manifest.json for future compatibility:
```json
{
  "manifest_version": "1.0",
  "created_with": "layercake-0.1.0",
  "type": "project_template"
}
```

### Backwards Compatibility

- Keep dataset import from CSV working during migration
- Support old API endpoints temporarily
- Provide migration guide for users

## Testing Strategy

### Unit Tests

- Library item CRUD operations
- ZIP archive creation/extraction
- Dataset schema extraction
- Template validation

### Integration Tests

- End-to-end project export → import
- Sample project migration
- Library filtering and search
- Multi-type library queries

### Manual Testing

- Export complex project with multiple datasets
- Import template and verify DAG structure
- Upload malformed ZIP (error handling)
- Filter library by multiple criteria
- Create project from each sample type

## Documentation Updates

### User Documentation

1. **Library User Guide**
   - How to use the library
   - Uploading datasets and templates
   - Creating projects from templates

2. **Template Creation Guide**
   - How to export a project as template
   - Best practices for templates
   - What gets included/excluded

3. **Migration Guide**
   - Changes for existing users
   - How existing library items are affected

### Developer Documentation

1. **Library Architecture**
   - Data model overview
   - Storage strategy
   - Extension points

2. **Template Format Specification**
   - ZIP structure
   - manifest.json schema
   - Versioning strategy

## Future Enhancements

### Phase 2 Features (Not in Initial Implementation)

1. **Template Marketplace**
   - Share templates with other users
   - Rating and review system
   - Template categories and curation

2. **Version Control for Templates**
   - Track template versions
   - Update notifications
   - Rollback capability

3. **Advanced Filtering**
   - Filter by metadata fields
   - Custom tag hierarchies
   - Saved filter presets

4. **Template Customisation**
   - Configure template during import
   - Map datasets to different sources
   - Customise node parameters

5. **Cloud Storage Integration**
   - Store large templates in cloud
   - Share templates via URL
   - Import from external sources

## Open Questions

1. Should we support partial template exports (select which nodes/datasets to include)? (NO PARTIAL TEMPLATE EXPORTS)
2. Should templates include graph outputs or only source datasets? (ONLY SOURCE DATASETS)
3. How to handle template versioning when core format changes? (add a metadata.json file in the export root that contains a layercake project format version field, set it to 1)
4. Should we validate DAG compatibility before import? (YES)
5. Maximum size limit for project templates? (NO, but add a warning for project templates over 1MB)

## Success Metrics

- Migration completes without data loss
- All sample projects work via library
- Can export and re-import complex projects successfully
- Library page load time < 2s with 100+ items
- Zero security vulnerabilities in ZIP handling
