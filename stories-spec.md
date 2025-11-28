# Stories Import/Export Specification

## Overview

This specification defines the format for importing and exporting Stories and their Sequences from the Layercake application. Stories contain Sequences, which are ordered collections of graph edges with optional annotations.

## File Formats

Stories can be imported/exported in two formats:
1. **CSV** - Flat structure for simple import/export
2. **JSON** - Hierarchical structure preserving all metadata

---

## CSV Format

### File Structure

CSV files use a flat structure where each row represents a sequence item (edge) within a story and sequence.

### Columns

| Column | Type | Required | Description |
|--------|------|----------|-------------|
| `story_id` | Integer | Yes | Unique identifier for the story (used for updates, auto-generated for new imports) |
| `story_name` | String | Yes | Name of the story |
| `story_description` | String | No | Description of the story |
| `story_tags` | String | No | Comma-separated tags (e.g., "tag1,tag2,tag3") |
| `story_enabled_dataset_ids` | String | No | Comma-separated dataset IDs enabled for this story |
| `sequence_id` | Integer | Yes | Unique identifier for the sequence |
| `sequence_name` | String | Yes | Name of the sequence |
| `sequence_description` | String | No | Description of the sequence |
| `sequence_enabled_dataset_ids` | String | No | Comma-separated dataset IDs enabled for this sequence |
| `sequence_item_id` | Integer | Yes | Order position of this edge in the sequence (0-based) |
| `dataset_id` | Integer | Yes | ID of the dataset containing the edge |
| `edge_id` | String | Yes | Identifier of the edge in the graph |
| `note` | String | No | Annotation text for this edge |
| `note_position` | Enum | No | Where to display the note: "Source", "Target", or "Both" |

### CSV Example

```csv
story_id,story_name,story_description,story_tags,story_enabled_dataset_ids,sequence_id,sequence_name,sequence_description,sequence_enabled_dataset_ids,sequence_item_id,dataset_id,edge_id,note,note_position
1,"User Journey","Main user flow","onboarding,critical","101,102",10,"Login Flow","User authentication process","101",0,101,"edge_user_to_login","User initiates login","Source"
1,"User Journey","Main user flow","onboarding,critical","101,102",10,"Login Flow","User authentication process","101",1,101,"edge_login_to_auth","System validates credentials","Target"
1,"User Journey","Main user flow","onboarding,critical","101,102",10,"Login Flow","User authentication process","101",2,101,"edge_auth_to_dashboard","Redirect to dashboard","Both"
1,"User Journey","Main user flow","onboarding,critical","101,102",11,"Registration Flow","New user signup","101,102",0,102,"edge_signup_start","User clicks register","Source"
1,"User Journey","Main user flow","onboarding,critical","101,102",11,"Registration Flow","New user signup","101,102",1,102,"edge_form_submit","Form submission","Target"
```

### CSV Import Rules

1. **Story Grouping**: Rows with the same `story_id` and `story_name` belong to the same story
2. **Sequence Grouping**: Rows with the same `sequence_id` and `sequence_name` within a story belong to the same sequence
3. **Edge Ordering**: Edges within a sequence are ordered by `sequence_item_id` (ascending)
4. **ID Handling**:
   - On import, if `story_id` is 0 or negative, a new story is created
   - If `story_id` matches an existing story in the project, that story is updated
   - Same logic applies to `sequence_id`
5. **Dataset Validation**: All referenced `dataset_id` values must exist in the target project
6. **Edge Validation**: All `edge_id` values must exist in their referenced datasets

---

## JSON Format

### File Structure

JSON format provides a hierarchical structure preserving all metadata and relationships.

### Schema

```json
{
  "version": "1.0",
  "exportedAt": "2025-11-28T12:00:00Z",
  "stories": [
    {
      "id": 1,
      "name": "User Journey",
      "description": "Main user flow",
      "tags": ["onboarding", "critical"],
      "enabledDatasetIds": [101, 102],
      "layerConfig": [
        {
          "sourceDatasetId": 101,
          "mode": "default"
        }
      ],
      "sequences": [
        {
          "id": 10,
          "name": "Login Flow",
          "description": "User authentication process",
          "enabledDatasetIds": [101],
          "edgeOrder": [
            {
              "datasetId": 101,
              "edgeId": "edge_user_to_login",
              "note": "User initiates login",
              "notePosition": "Source"
            },
            {
              "datasetId": 101,
              "edgeId": "edge_login_to_auth",
              "note": "System validates credentials",
              "notePosition": "Target"
            },
            {
              "datasetId": 101,
              "edgeId": "edge_auth_to_dashboard",
              "note": "Redirect to dashboard",
              "notePosition": "Both"
            }
          ]
        },
        {
          "id": 11,
          "name": "Registration Flow",
          "description": "New user signup",
          "enabledDatasetIds": [101, 102],
          "edgeOrder": [
            {
              "datasetId": 102,
              "edgeId": "edge_signup_start",
              "note": "User clicks register",
              "notePosition": "Source"
            },
            {
              "datasetId": 102,
              "edgeId": "edge_form_submit",
              "note": "Form submission",
              "notePosition": "Target"
            }
          ]
        }
      ]
    }
  ]
}
```

### JSON Field Definitions

#### Root Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | String | Yes | Format version (currently "1.0") |
| `exportedAt` | ISO DateTime | Yes | Timestamp when export was created |
| `stories` | Array[Story] | Yes | Array of story objects |

#### Story Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | Integer | Yes | Story ID (0 for new imports) |
| `name` | String | Yes | Story name |
| `description` | String | No | Story description |
| `tags` | Array[String] | No | Array of tag strings |
| `enabledDatasetIds` | Array[Integer] | No | Dataset IDs enabled for this story |
| `layerConfig` | Array[LayerConfig] | No | Layer rendering configuration |
| `sequences` | Array[Sequence] | Yes | Array of sequence objects |

#### LayerConfig Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `sourceDatasetId` | Integer | No | Dataset ID providing layer source (null for manual layers) |
| `mode` | String | Yes | Fallback style mode: "default", "light", or "dark" |

#### Sequence Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | Integer | Yes | Sequence ID (0 for new imports) |
| `name` | String | Yes | Sequence name |
| `description` | String | No | Sequence description |
| `enabledDatasetIds` | Array[Integer] | No | Dataset IDs enabled for this sequence |
| `edgeOrder` | Array[EdgeRef] | Yes | Ordered array of edge references |

#### EdgeRef Object

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `datasetId` | Integer | Yes | ID of dataset containing the edge |
| `edgeId` | String | Yes | Edge identifier |
| `note` | String | No | Annotation text |
| `notePosition` | Enum | No | Note display position: "Source", "Target", or "Both" |

### JSON Import Rules

1. **Story ID Handling**:
   - If `id` is 0, creates a new story
   - If `id` > 0 and matches existing story, updates that story
   - If `id` > 0 but doesn't exist, creates new story (ignoring the ID)

2. **Sequence ID Handling**:
   - Same rules as Story ID
   - Sequences are created/updated within their parent story

3. **Validation**:
   - All `datasetId` references must exist in the target project
   - All `edgeId` values must exist in their respective datasets
   - Layer config `sourceDatasetId` must be null or valid dataset ID

4. **Merging Behavior**:
   - On update, existing sequences not in import are preserved
   - To replace all sequences, delete the story first and import as new

---

## API Endpoints

### GraphQL Mutations

#### Export Story

```graphql
mutation ExportStory($storyId: Int!, $format: ExportFormat!) {
  exportStory(storyId: $storyId, format: $format) {
    filename
    content  # Base64-encoded file content
    mimeType
  }
}

enum ExportFormat {
  CSV
  JSON
}
```

#### Import Story

```graphql
mutation ImportStory($projectId: Int!, $format: ImportFormat!, $content: String!) {
  importStory(projectId: $projectId, format: $format, content: $content) {
    importedStories {
      id
      name
      sequenceCount
    }
    createdCount
    updatedCount
    errors
  }
}

enum ImportFormat {
  CSV
  JSON
}
```

---

## Use Cases

### 1. Export Story for Backup
```bash
# Export single story to JSON
exportStory(storyId: 1, format: JSON)
```

### 2. Share Story Between Projects
```bash
# Export from Project A
exportStory(storyId: 5, format: JSON)

# Import to Project B (will create new story with new ID)
importStory(projectId: 2, format: JSON, content: "...")
```

### 3. Bulk Edit in Spreadsheet
```bash
# Export to CSV
exportStory(storyId: 3, format: CSV)

# Edit in Excel/Google Sheets
# Import updated CSV
importStory(projectId: 1, format: CSV, content: "...")
```

### 4. Version Control
- Export stories to JSON
- Commit to Git
- Track changes to narrative structure over time

---

## Error Handling

### Validation Errors

| Error | Description | Resolution |
|-------|-------------|------------|
| `INVALID_DATASET_ID` | Referenced dataset doesn't exist | Ensure all datasets exist in target project |
| `INVALID_EDGE_ID` | Edge not found in dataset | Verify edge IDs match dataset graph structure |
| `DUPLICATE_SEQUENCE_ITEM` | Multiple edges with same `sequence_item_id` | Ensure unique ordering within each sequence |
| `INVALID_NOTE_POSITION` | Note position not in enum | Use "Source", "Target", or "Both" |
| `MALFORMED_CSV` | CSV parsing error | Check CSV format and encoding (UTF-8) |
| `MALFORMED_JSON` | JSON parsing error | Validate JSON structure |
| `STORY_NOT_FOUND` | Story ID doesn't exist for export | Verify story exists |
| `PERMISSION_DENIED` | User lacks access to project | Check user permissions |

### Partial Import

- If some rows/objects fail validation, import continues for valid items
- Response includes both success counts and error messages
- Failed items are reported with row numbers (CSV) or paths (JSON)

---

## Implementation Notes

### Export Process

1. Fetch story with all sequences from database
2. For CSV: Flatten hierarchical structure into rows
3. For JSON: Serialize directly to JSON schema
4. Return base64-encoded content with appropriate MIME type

### Import Process

1. Parse file content (CSV or JSON)
2. Validate all references (datasets, edges)
3. Create/update stories and sequences in transaction
4. Return summary with counts and errors

### Frontend Integration

- Add "Export Story" button to story detail page
- Add "Import Story" button to stories list page
- File upload dialog with format selection
- Download generated file with appropriate filename

### Performance Considerations

- Limit maximum edges per sequence: 10,000
- Limit maximum sequences per story: 1,000
- Use streaming for large exports
- Batch database operations during import

---

## Future Enhancements

1. **Bulk Export**: Export multiple stories in one file
2. **Project Export**: Export all stories for a project
3. **Template Stories**: Mark stories as templates for reuse
4. **Diff View**: Show changes before confirming import
5. **Incremental Import**: Merge mode that doesn't duplicate sequences
6. **Excel Format**: Direct .xlsx import/export
7. **Validation Preview**: Pre-import validation report

---

## Appendix: Example Files

### Minimal CSV

```csv
story_id,story_name,sequence_id,sequence_name,sequence_item_id,dataset_id,edge_id
0,"New Story",0,"New Sequence",0,101,"edge_1"
0,"New Story",0,"New Sequence",1,101,"edge_2"
```

### Minimal JSON

```json
{
  "version": "1.0",
  "exportedAt": "2025-11-28T12:00:00Z",
  "stories": [
    {
      "id": 0,
      "name": "New Story",
      "sequences": [
        {
          "id": 0,
          "name": "New Sequence",
          "edgeOrder": [
            { "datasetId": 101, "edgeId": "edge_1" },
            { "datasetId": 101, "edgeId": "edge_2" }
          ]
        }
      ]
    }
  ]
}
```
