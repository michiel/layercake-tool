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
**Status**: Complete

## Stage 2: Backend GraphQL Types
**Goal**: Define GraphQL types for Story and Sequence
**Status**: Complete

## Stage 3: Backend GraphQL Queries
**Goal**: Add query resolvers for stories and sequences
**Status**: Complete

## Stage 4: Backend GraphQL Mutations
**Goal**: Add mutation resolvers for stories and sequences
**Status**: Complete

## Stage 5: Frontend GraphQL Definitions
**Goal**: Define frontend GraphQL queries and mutations
**Status**: Complete

## Stage 6: Navigation & Routing
**Goal**: Add Stories to navigation and set up routes
**Status**: Complete

## Stage 7: Stories List Page
**Goal**: Create the Stories listing page
**Status**: Complete

## Stage 8: Story Detail Page - Structure
**Goal**: Create the Story detail page shell with tabs
**Status**: Complete

## Stage 9: Story Details Tab
**Goal**: Implement the Details tab with metadata and dataset selection
**Status**: Complete

## Stage 10: Story Layers Tab
**Goal**: Implement the Layers tab for layer configuration
**Status**: Partial (placeholder UI, full implementation pending)

## Stage 11: Story Sequences Tab
**Goal**: Implement the Sequences tab with sequence listing
**Status**: Complete

## Stage 12: Sequence Editor Dialog - Structure
**Goal**: Create the sequence editor dialog shell
**Status**: Complete

## Stage 13: Sequence Editor - Dataset Selection
**Goal**: Implement dataset selection panel in sequence editor
**Status**: Complete

## Stage 14: Sequence Editor - Edge List
**Goal**: Implement the draggable edge sequence list
**Status**: Complete

## Stage 15: Integration & Polish
**Goal**: Final integration and UX improvements
**Status**: Complete (basic implementation)

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
- [x] Stories appear in left navigation under Workbench
- [x] Can create, edit, delete stories
- [x] Story Details tab allows editing name/description/tags
- [x] Story Details tab allows selecting which datasets are available
- [ ] Story Layers tab allows configuring layer colors and sources (placeholder only)
- [x] Story Sequences tab lists sequences with add/edit/delete
- [x] Sequence editor allows enabling/disabling datasets
- [x] Sequence editor shows draggable edge list
- [x] Edge list shows source node, edge info, target node
- [x] Edges can be reordered via drag-and-drop
- [x] All data persists correctly
- [x] Deleting a story cascades to delete its sequences

---

## Files Created/Modified

### Backend
- `layercake-core/src/database/migrations/m20251122_000003_create_stories.rs`
- `layercake-core/src/database/migrations/m20251122_000004_create_sequences.rs`
- `layercake-core/src/database/entities/stories.rs`
- `layercake-core/src/database/entities/sequences.rs`
- `layercake-core/src/graphql/types/story.rs`
- `layercake-core/src/graphql/types/sequence.rs`
- `layercake-core/src/graphql/mutations/story.rs`
- `layercake-core/src/graphql/mutations/sequence.rs`
- `layercake-core/src/graphql/queries/mod.rs` (modified)

### Frontend
- `frontend/src/graphql/stories.ts`
- `frontend/src/graphql/sequences.ts`
- `frontend/src/pages/StoriesPage.tsx`
- `frontend/src/pages/StoryPage.tsx`
- `frontend/src/components/stories/StorySequencesTab.tsx`
- `frontend/src/components/stories/SequenceEditorDialog.tsx`
- `frontend/src/App.tsx` (modified)

---

## Future Enhancements (Out of Scope)
- Sequence playback/presentation mode
- Notes attached to sequence edges
- Sequence export (PDF, presentation)
- Sequence sharing/collaboration
- Version history for sequences
- Full layer configuration UI in Story Layers tab
