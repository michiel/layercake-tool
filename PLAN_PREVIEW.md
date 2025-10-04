# Plan: DataSource Preview Feature

## Overview

Add preview functionality to DataSource nodes to allow users to view the raw CSV data before it's processed into graphs. This helps users verify data quality and understand the structure of their source files.

## Current State

- **GraphNodes**: Already have preview functionality with force-graph visualization
- **DataSourceNodes**: Display metadata (file type, size, upload time, badges) but no preview
- **Image shows**: Two DataSource nodes (nodes.csv, links.csv) connected to a Merge node

## Requirements

### Functional Requirements

1. **Preview Button**
   - Add centered play button to DataSourceNode (similar to GraphNode)
   - Button should be visible in non-readonly mode
   - Tooltip: "Preview data source"

2. **Data Display**
   - Show CSV data in tabular format
   - Display first N rows (configurable, default 100)
   - Show column headers from CSV
   - Indicate total row count vs. displayed rows
   - Handle both node and edge CSV formats

3. **Data Fetching**
   - Query backend for CSV data by projectId and source file path
   - Support pagination or row limits to avoid loading massive files
   - Handle loading states and errors gracefully

4. **Preview Dialog**
   - Full-screen or large modal display
   - Table with scrollable rows and columns
   - Column headers should be sticky/frozen
   - Show metadata: filename, file size, row count, column count
   - Export button to download full CSV (optional enhancement)

### Non-Functional Requirements

1. **Performance**
   - Lazy load data only when preview is opened
   - Virtualized table rendering for large datasets
   - Cancel in-flight requests if dialog is closed

2. **UX Consistency**
   - Match GraphNode preview style and interaction patterns
   - Use same dialog component structure
   - Consistent button placement and styling

## Implementation Plan

### Phase 1: Backend GraphQL Support

**File**: Backend GraphQL schema (e.g., `backend/graphql/schema.graphql`)

1. Add query to fetch data source content:
```graphql
type DataSourceRow {
  rowNumber: Int!
  data: JSON!
}

type DataSourcePreview {
  filename: String!
  totalRows: Int!
  totalColumns: Int!
  columns: [String!]!
  rows: [DataSourceRow!]!
  fileSize: Int
  fileType: String
}

extend type Query {
  dataSourcePreview(
    projectId: Int!
    sourceId: String!
    limit: Int = 100
    offset: Int = 0
  ): DataSourcePreview!
}
```

2. Implement resolver to:
   - Locate CSV file by projectId and sourceId
   - Parse CSV headers
   - Read first N rows (with offset for pagination)
   - Return structured data

**Estimated effort**: 4-6 hours

### Phase 2: Frontend GraphQL Client

**File**: `frontend/src/graphql/dataSources.ts` (new file)

1. Create GraphQL query:
```typescript
export const GET_DATA_SOURCE_PREVIEW = gql`
  query GetDataSourcePreview(
    $projectId: Int!
    $sourceId: String!
    $limit: Int
    $offset: Int
  ) {
    dataSourcePreview(
      projectId: $projectId
      sourceId: $sourceId
      limit: $limit
      offset: $offset
    ) {
      filename
      totalRows
      totalColumns
      columns
      rows {
        rowNumber
        data
      }
      fileSize
      fileType
    }
  }
`
```

2. Define TypeScript interfaces:
```typescript
export interface DataSourceRow {
  rowNumber: number
  data: Record<string, any>
}

export interface DataSourcePreviewResponse {
  filename: string
  totalRows: number
  totalColumns: number
  columns: string[]
  rows: DataSourceRow[]
  fileSize?: number
  fileType?: string
}
```

**Estimated effort**: 1 hour

### Phase 3: Preview Component

**File**: `frontend/src/components/visualization/DataSourcePreview.tsx` (new file)

1. Create table component:
   - Use Mantine Table or DataTable component
   - Implement virtualized scrolling (react-window or @mantine/core Table with scroll)
   - Sticky header row
   - Alternating row colors for readability
   - Cell content truncation with tooltips for long values

2. Features:
   - Column resizing (optional)
   - Column sorting (client-side, optional)
   - Search/filter (optional, future enhancement)

**Estimated effort**: 3-4 hours

### Phase 4: Preview Dialog

**File**: `frontend/src/components/visualization/DataSourcePreviewDialog.tsx` (new file)

1. Create dialog wrapper (similar to GraphPreviewDialog):
   - Full-screen or large modal
   - Header with metadata (filename, row count, column count)
   - Close button
   - Footer with pagination controls (if implementing pagination)

2. Integration:
   - Load data on open
   - Handle loading/error states
   - Pass data to DataSourcePreview component

**Estimated effort**: 2-3 hours

### Phase 5: DataSourceNode Integration

**File**: `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx`

1. Add preview button:
   - Position in center (similar to GraphNode)
   - IconPlayerPlay or IconEye icon
   - Tooltip: "Preview data source"
   - Only show in non-readonly mode

2. Add state and query:
```typescript
const [showPreview, setShowPreview] = useState(false)

const { data: previewData } = useQuery<{ dataSourcePreview: DataSourcePreviewResponse }>(
  GET_DATA_SOURCE_PREVIEW,
  {
    variables: {
      projectId: projectId || 0,
      sourceId: props.id,
      limit: 100
    },
    skip: !showPreview,
  }
)
```

3. Add dialog component:
```tsx
<DataSourcePreviewDialog
  opened={showPreview}
  onClose={() => setShowPreview(false)}
  data={previewData?.dataSourcePreview || null}
  title={`Data Source: ${data.metadata.label}`}
/>
```

**Estimated effort**: 2-3 hours

### Phase 6: Testing & Polish

1. **Unit Tests**
   - Test DataSourcePreview component rendering
   - Test pagination logic
   - Test error states

2. **Integration Tests**
   - Test preview button click flow
   - Test data loading
   - Test dialog open/close

3. **Manual Testing**
   - Test with small CSV files (< 100 rows)
   - Test with large CSV files (> 10,000 rows)
   - Test with wide CSV files (many columns)
   - Test with different CSV formats (nodes vs edges)
   - Test error scenarios (missing file, invalid CSV)

4. **Polish**
   - Loading skeletons
   - Empty state messaging
   - Error messaging improvements
   - Accessibility (keyboard navigation, ARIA labels)

**Estimated effort**: 3-4 hours

## File Changes Summary

### New Files
- `backend/graphql/resolvers/dataSource.ts` (or add to existing resolver file)
- `frontend/src/graphql/dataSources.ts`
- `frontend/src/components/visualization/DataSourcePreview.tsx`
- `frontend/src/components/visualization/DataSourcePreviewDialog.tsx`

### Modified Files
- `backend/graphql/schema.graphql` (add query)
- `frontend/src/components/editors/PlanVisualEditor/nodes/DataSourceNode.tsx`

## Alternative Approaches

### Option A: Simple Text Preview
Instead of a table, show raw CSV text in a code editor component with syntax highlighting.

**Pros**: Simpler implementation, shows exact file content
**Cons**: Less user-friendly for large files, harder to read

### Option B: Reuse Graph Preview
Convert CSV data to a simple graph visualization for preview.

**Pros**: Consistent with GraphNode preview
**Cons**: Doesn't match the actual data format, confusing for users

### Option C: Inline Preview
Show first 5 rows directly in the node (expand on click).

**Pros**: Quick glance without opening dialog
**Cons**: Limited space, clutters the node

**Recommendation**: Stick with main plan (table dialog) for best UX.

## Implementation Order

1. **Backend GraphQL** (Phase 1) - Required for all frontend work
2. **Frontend GraphQL** (Phase 2) - Set up data fetching
3. **Preview Component** (Phase 3) - Build table display
4. **Preview Dialog** (Phase 4) - Wrap table in modal
5. **Node Integration** (Phase 5) - Add button and wire up
6. **Testing & Polish** (Phase 6) - Final touches

## Success Criteria

- [ ] Preview button appears on all DataSource nodes in non-readonly mode
- [ ] Clicking preview opens dialog with CSV data in table format
- [ ] Table shows first 100 rows with all columns
- [ ] Table has sticky headers and scrollable content
- [ ] Dialog shows metadata (filename, row count, column count)
- [ ] Loading state displays while fetching data
- [ ] Error state displays if data fetch fails
- [ ] Preview works for both node and edge CSV files
- [ ] Performance is acceptable for files up to 10,000 rows
- [ ] Dialog can be closed via X button or ESC key

## Total Estimated Effort

**15-21 hours** (approximately 2-3 days of development)

Breakdown:
- Backend: 4-6 hours
- Frontend components: 7-10 hours
- Integration: 2-3 hours
- Testing & polish: 3-4 hours

## Future Enhancements

1. **Full Pagination**: Support browsing entire file, not just first 100 rows
2. **Column Filtering**: Filter rows by column values
3. **Export**: Download filtered/sorted data
4. **Statistics**: Show column statistics (min, max, unique values, null count)
5. **Data Validation**: Highlight invalid/missing data
6. **Diff View**: Compare two data sources side-by-side
7. **Edit Support**: Allow inline editing of CSV data (advanced feature)

## Notes

- This plan assumes CSV files are stored on the backend filesystem
- If files are stored differently (S3, database), adjust backend implementation
- Consider caching preview data on backend to avoid re-parsing large files
- May need to handle different CSV dialects (comma, semicolon, tab-delimited)
- Character encoding issues (UTF-8 vs others) should be handled gracefully
