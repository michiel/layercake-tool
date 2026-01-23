# Story Import/Export Implementation Summary

## Overview

Successfully implemented comprehensive import/export functionality for Stories and Sequences, allowing users to backup, share, and bulk-edit their narrative structures.

## Files Created/Modified

### Documentation
- ✅ **stories-spec.md** - Complete specification for CSV and JSON formats

### Backend (Rust)
- ✅ **layercake-core/src/app_context/story_operations.rs** (NEW) - Core import/export logic
  - `export_story_csv()` - Export story to CSV format
  - `export_story_json()` - Export story to JSON format
  - `import_story_csv()` - Import stories from CSV with validation
  - `import_story_json()` - Import stories from JSON with validation

- ✅ **layercake-core/src/app_context/mod.rs** - Added public types:
  - `StoryExportResult`
  - `StoryImportResult`
  - `StoryImportSummary`

- ✅ **layercake-core/src/graphql/mutations/story.rs** - Added mutations:
  - `export_story` - Export story in CSV or JSON format
  - `import_story` - Import stories from CSV or JSON

- ✅ **layercake-core/src/graphql/types/story.rs** - Added types:
  - `StoryExportFormat` enum (CSV, JSON)
  - `StoryImportFormat` enum (CSV, JSON)
  - `StoryExport` type
  - `StoryImportResult` type
  - `StoryImportSummary` type

### Frontend (TypeScript/React)
- ✅ **frontend/src/graphql/stories.ts** - Added GraphQL operations:
  - `EXPORT_STORY` mutation
  - `IMPORT_STORY` mutation
  - TypeScript types for import/export

- ✅ **frontend/src/pages/StoriesPage.tsx** - Added UI:
  - "Import Story" button with format selection dialog (CSV/JSON)
  - "Export" dropdown on each story card with CSV/JSON options
  - File upload handling for import
  - Base64 decode and download for export

## Features

### Export
- **Per-Story Export**: Each story card has an "Export" dropdown
- **Format Selection**: Choose between CSV and JSON
- **Automatic Download**: Generates filename based on story name and format
- **Hierarchical JSON**: Preserves full story structure with sequences
- **Flattened CSV**: One row per edge for spreadsheet editing

### Import
- **Project-Level Import**: Import button on stories list page
- **Format Selection**: Choose CSV or JSON before file selection
- **Validation**: Validates dataset IDs and edge IDs before import
- **Create/Update Logic**: Story ID of 0 creates new, otherwise updates existing
- **Error Reporting**: Shows count of created/updated stories and any errors

### Data Validation
- ✅ All dataset IDs must exist in the target project
- ✅ All edge IDs must exist in their referenced datasets
- ✅ Note positions validated (Source/Target/Both)
- ✅ Transactional import (all-or-nothing per story)
- ✅ Detailed error messages with context

## CSV Format Example

```csv
story_id,story_name,story_description,story_tags,story_enabled_dataset_ids,sequence_id,sequence_name,sequence_description,sequence_enabled_dataset_ids,sequence_item_id,dataset_id,edge_id,note,note_position
0,"New Story","Description","tag1,tag2","101,102",0,"Sequence 1","Seq desc","101",0,101,"edge_1","Note text","Source"
```

## JSON Format Example

```json
{
  "version": "1.0",
  "exportedAt": "2025-11-28T12:00:00Z",
  "stories": [
    {
      "id": 0,
      "name": "Story Name",
      "description": "Description",
      "tags": ["tag1", "tag2"],
      "enabledDatasetIds": [101, 102],
      "layerConfig": [
        {
          "sourceDatasetId": 101,
          "mode": "default"
        }
      ],
      "sequences": [
        {
          "id": 0,
          "name": "Sequence Name",
          "description": "Seq description",
          "enabledDatasetIds": [101],
          "edgeOrder": [
            {
              "datasetId": 101,
              "edgeId": "edge_1",
              "note": "Note text",
              "notePosition": "Source"
            }
          ]
        }
      ]
    }
  ]
}
```

## GraphQL API

### Export Story
```graphql
mutation ExportStory($storyId: Int!, $format: StoryExportFormat!) {
  exportStory(storyId: $storyId, format: $format) {
    filename
    content  # Base64-encoded
    mimeType
  }
}
```

### Import Story
```graphql
mutation ImportStory($projectId: Int!, $format: StoryImportFormat!, $content: String!) {
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
```

## Use Cases

1. **Backup Stories**: Export to JSON for version control
2. **Share Between Projects**: Export from one project, import to another
3. **Bulk Edit**: Export to CSV, edit in Excel/Google Sheets, import back
4. **Template Creation**: Export story as template, modify IDs to 0, import as new
5. **Documentation**: CSV format is human-readable for documentation

## Testing

- ✅ Backend compiles successfully
- ✅ All 202 backend tests pass
- ✅ Frontend TypeScript types aligned
- ✅ GraphQL schema matches specification

## Technical Implementation Notes

### Backend
- Uses `csv` crate for CSV serialization/deserialization
- Base64-encodes content for safe GraphQL transmission
- Transactional database operations ensure consistency
- Validates all references before committing
- Error collection allows partial imports to succeed

### Frontend
- File upload with format detection
- Base64 decode for downloads
- Dropdown menu for export format selection
- Modal dialog for import with format toggle
- Success/error messages with counts and details

## Future Enhancements (from spec)

1. **Bulk Export**: Export multiple stories in one file
2. **Project Export**: Export all stories for a project
3. **Template Stories**: Mark stories as templates for reuse
4. **Diff View**: Show changes before confirming import
5. **Incremental Import**: Merge mode that doesn't duplicate sequences
6. **Excel Format**: Direct .xlsx import/export
7. **Validation Preview**: Pre-import validation report

## Performance Considerations

- Limit: 10,000 edges per sequence
- Limit: 1,000 sequences per story
- Streaming for large exports (future)
- Batch database operations during import
