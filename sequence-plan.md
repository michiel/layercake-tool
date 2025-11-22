# Story & Sequence Feature Implementation Plan

## Overview
Add a new project-level Story entity that allows users to create narrative sequences from graph edges. Stories enable curated walkthroughs of graph data with configurable datasets and layers.

## Data Model

### Story Entity
```
stories
├── id: Int (PK)
├── project_id: Int (FK -> projects)
├── name: String
├── description: String?
├── tags: JSON (string array)
├── enabled_dataset_ids: JSON (int array) - datasets available in this story
├── layer_config: JSON - layer colors, sources, visibility settings
├── created_at: DateTime
└── updated_at: DateTime
```

### Sequence Entity
```
sequences
├── id: Int (PK)
├── story_id: Int (FK -> stories)
├── name: String
├── description: String?
├── enabled_dataset_ids: JSON (int array) - subset of story datasets enabled for this sequence
├── edge_order: JSON - array of { datasetId: Int, edgeId: String } references
├── created_at: DateTime
└── updated_at: DateTime
```

---

## Stage 1: Database Migration
**Goal**: Create stories and sequences tables
**Status**: Not Started

### Tasks
1. Create migration file `m20241122_000001_create_stories_table.rs`
   - stories table with all columns
   - Index on project_id
2. Create migration file `m20241122_000002_create_sequences_table.rs`
   - sequences table with all columns
   - Index on story_id
   - Foreign key to stories with CASCADE delete
3. Generate SeaORM entities for stories and sequences
4. Update migration mod.rs to include new migrations

### Files
- `layercake-core/migration/src/m20241122_000001_create_stories_table.rs`
- `layercake-core/migration/src/m20241122_000002_create_sequences_table.rs`
- `layercake-core/src/database/entities/stories.rs`
- `layercake-core/src/database/entities/sequences.rs`
- `layercake-core/src/database/entities/mod.rs`

---

## Stage 2: Backend GraphQL Types
**Goal**: Define GraphQL types for Story and Sequence
**Status**: Not Started

### Tasks
1. Create `story.rs` GraphQL types:
   - `Story` output type
   - `CreateStoryInput` input type
   - `UpdateStoryInput` input type
   - `StoryLayerConfig` type for layer settings
2. Create `sequence.rs` GraphQL types:
   - `Sequence` output type
   - `CreateSequenceInput` input type
   - `UpdateSequenceInput` input type
   - `SequenceEdgeRef` type for edge references
3. Add types to mod.rs exports

### Files
- `layercake-core/src/graphql/types/story.rs`
- `layercake-core/src/graphql/types/sequence.rs`
- `layercake-core/src/graphql/types/mod.rs`

---

## Stage 3: Backend GraphQL Queries
**Goal**: Add query resolvers for stories and sequences
**Status**: Not Started

### Tasks
1. Create `story.rs` query file:
   - `stories(projectId: Int!)` - list all stories for a project
   - `story(id: Int!)` - get single story by ID
2. Create `sequence.rs` query file:
   - `sequences(storyId: Int!)` - list all sequences for a story
   - `sequence(id: Int!)` - get single sequence by ID
3. Add queries to schema

### Files
- `layercake-core/src/graphql/queries/story.rs`
- `layercake-core/src/graphql/queries/sequence.rs`
- `layercake-core/src/graphql/queries/mod.rs`

---

## Stage 4: Backend GraphQL Mutations
**Goal**: Add mutation resolvers for stories and sequences
**Status**: Not Started

### Tasks
1. Create `story.rs` mutation file:
   - `createStory(input: CreateStoryInput!)` - create new story
   - `updateStory(id: Int!, input: UpdateStoryInput!)` - update story
   - `deleteStory(id: Int!)` - delete story (cascades to sequences)
2. Create `sequence.rs` mutation file:
   - `createSequence(input: CreateSequenceInput!)` - create new sequence
   - `updateSequence(id: Int!, input: UpdateSequenceInput!)` - update sequence
   - `deleteSequence(id: Int!)` - delete sequence
3. Add mutations to schema

### Files
- `layercake-core/src/graphql/mutations/story.rs`
- `layercake-core/src/graphql/mutations/sequence.rs`
- `layercake-core/src/graphql/mutations/mod.rs`

---

## Stage 5: Frontend GraphQL Definitions
**Goal**: Define frontend GraphQL queries and mutations
**Status**: Not Started

### Tasks
1. Create `stories.ts` with:
   - `LIST_STORIES` query
   - `GET_STORY` query
   - `CREATE_STORY` mutation
   - `UPDATE_STORY` mutation
   - `DELETE_STORY` mutation
   - TypeScript interfaces for Story types
2. Create `sequences.ts` with:
   - `LIST_SEQUENCES` query
   - `GET_SEQUENCE` query
   - `CREATE_SEQUENCE` mutation
   - `UPDATE_SEQUENCE` mutation
   - `DELETE_SEQUENCE` mutation
   - TypeScript interfaces for Sequence types

### Files
- `frontend/src/graphql/stories.ts`
- `frontend/src/graphql/sequences.ts`

---

## Stage 6: Navigation & Routing
**Goal**: Add Stories to navigation and set up routes
**Status**: Not Started

### Tasks
1. Add Stories navigation item to left panel (under Workbench section)
   - Icon: `IconBook` or `IconScript`
   - Route: `/projects/:projectId/stories`
2. Add routes to App.tsx:
   - `/projects/:projectId/stories` - Stories list page
   - `/projects/:projectId/stories/:storyId` - Story detail page
3. Update any navigation components that need Stories link

### Files
- `frontend/src/components/layout/Sidebar.tsx` (or equivalent nav component)
- `frontend/src/App.tsx`

---

## Stage 7: Stories List Page
**Goal**: Create the Stories listing page
**Status**: Not Started

### Tasks
1. Create `StoriesPage.tsx`:
   - Breadcrumbs navigation
   - Page header with "New Story" button
   - Table/list of stories with columns: Name, Description, Sequences count, Updated
   - Row actions: Open, Delete
   - Empty state when no stories
   - Delete confirmation modal
2. Wire up GraphQL queries and mutations

### Files
- `frontend/src/pages/StoriesPage.tsx`

---

## Stage 8: Story Detail Page - Structure
**Goal**: Create the Story detail page shell with tabs
**Status**: Not Started

### Tasks
1. Create `StoryPage.tsx`:
   - Breadcrumbs navigation
   - Page header with story name
   - Tabs component with three tabs: Details, Layers, Sequences
   - Loading and error states
2. Set up tab routing/state management

### Files
- `frontend/src/pages/StoryPage.tsx`

---

## Stage 9: Story Details Tab
**Goal**: Implement the Details tab with metadata and dataset selection
**Status**: Not Started

### Tasks
1. Create `StoryDetailsTab.tsx`:
   - Form fields: Name, Description, Tags (comma-separated input)
   - Save button for metadata changes
2. Create `StoryDatasetSelection.tsx`:
   - Fetch project datasets
   - Checkbox list of datasets
   - Enable/disable datasets for this story
   - Show dataset info (name, node/edge counts)
   - Auto-save on change or explicit save button

### Files
- `frontend/src/components/stories/StoryDetailsTab.tsx`
- `frontend/src/components/stories/StoryDatasetSelection.tsx`

---

## Stage 10: Story Layers Tab
**Goal**: Implement the Layers tab for layer configuration
**Status**: Not Started

### Tasks
1. Create `StoryLayersTab.tsx`:
   - Similar UI to Configure Graph Artefact Layers dialog
   - Layer list from enabled datasets
   - For each layer:
     - Enable/disable toggle
     - Color picker
     - Source dataset selection (dropdown)
   - Save layer configuration
2. Reuse existing layer configuration components where possible

### Files
- `frontend/src/components/stories/StoryLayersTab.tsx`

---

## Stage 11: Story Sequences Tab
**Goal**: Implement the Sequences tab with sequence listing
**Status**: Not Started

### Tasks
1. Create `StorySequencesTab.tsx`:
   - "New Sequence" button
   - List/table of sequences: Name, Edge count, Updated
   - Row actions: Edit (opens editor), Delete
   - Empty state
   - Delete confirmation modal
2. Wire up sequence CRUD operations

### Files
- `frontend/src/components/stories/StorySequencesTab.tsx`

---

## Stage 12: Sequence Editor Dialog - Structure
**Goal**: Create the sequence editor dialog shell
**Status**: Not Started

### Tasks
1. Create `SequenceEditorDialog.tsx`:
   - Dialog with title (New Sequence / Edit: {name})
   - Name input field
   - Description textarea
   - Two-panel layout:
     - Left: Dataset selection (checkboxes)
     - Right: Edge sequence list
   - Save and Cancel buttons
2. Handle create vs edit modes

### Files
- `frontend/src/components/stories/SequenceEditorDialog.tsx`

---

## Stage 13: Sequence Editor - Dataset Selection
**Goal**: Implement dataset selection panel in sequence editor
**Status**: Not Started

### Tasks
1. Create `SequenceDatasetPanel.tsx`:
   - List of datasets enabled in the parent story
   - Checkbox for each dataset to enable/disable in this sequence
   - Show dataset name and edge count
   - When dataset toggled, update available edges in sequence panel

### Files
- `frontend/src/components/stories/SequenceDatasetPanel.tsx`

---

## Stage 14: Sequence Editor - Edge List
**Goal**: Implement the draggable edge sequence list
**Status**: Not Started

### Tasks
1. Create `SequenceEdgeList.tsx`:
   - Draggable/sortable list (use dnd-kit or similar)
   - Each row displays:
     - Source node label (left column)
     - Edge info with comments (center, wide)
     - Target node label (right column)
   - Add edge button/mechanism (select from available edges)
   - Remove edge from sequence
2. Create `SequenceEdgeItem.tsx`:
   - Single edge row component
   - Drag handle
   - Display source → edge → target
   - Remove button
3. Create `AddEdgeDialog.tsx`:
   - Browse/search edges from enabled datasets
   - Select edge to add to sequence

### Files
- `frontend/src/components/stories/SequenceEdgeList.tsx`
- `frontend/src/components/stories/SequenceEdgeItem.tsx`
- `frontend/src/components/stories/AddEdgeDialog.tsx`

---

## Stage 15: Integration & Polish
**Goal**: Final integration and UX improvements
**Status**: Not Started

### Tasks
1. Add loading states throughout
2. Add error handling and notifications
3. Test full workflow: Create story → Configure datasets → Configure layers → Create sequence → Add edges
4. Ensure proper data persistence
5. Add empty states and helpful messages
6. Verify cascade delete works (story deletion removes sequences)

---

## Data Flow Summary

```
Project
└── Stories (1:many)
    ├── enabled_dataset_ids[] - which datasets are available
    ├── layer_config{} - layer colors/visibility/sources
    └── Sequences (1:many)
        ├── enabled_dataset_ids[] - subset of story datasets
        └── edge_order[] - ordered list of {datasetId, edgeId}
```

### Edge Reference Structure
```typescript
interface SequenceEdgeRef {
  datasetId: number;  // Reference to the dataset
  edgeId: string;     // Edge ID within that dataset's graphJson
}
```

### Layer Config Structure
```typescript
interface StoryLayerConfig {
  layers: Array<{
    layerId: string;
    enabled: boolean;
    color: string;
    sourceDatasetId: number | null;
  }>;
}
```

---

## Success Criteria
- [ ] Stories appear in left navigation under Workbench
- [ ] Can create, edit, delete stories
- [ ] Story Details tab allows editing name/description/tags
- [ ] Story Details tab allows selecting which datasets are available
- [ ] Story Layers tab allows configuring layer colors and sources
- [ ] Story Sequences tab lists sequences with add/edit/delete
- [ ] Sequence editor allows enabling/disabling datasets
- [ ] Sequence editor shows draggable edge list
- [ ] Edge list shows source node, edge info, target node
- [ ] Edges can be reordered via drag-and-drop
- [ ] All data persists correctly
- [ ] Deleting a story cascades to delete its sequences

---

## Future Enhancements (Out of Scope)
- Sequence playback/presentation mode
- Notes attached to sequence edges
- Sequence export (PDF, presentation)
- Sequence sharing/collaboration
- Version history for sequences
