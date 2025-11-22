# Merge Datasets Implementation Plan

## Overview
Add a "Merge" button to the Data sets page that combines multiple selected datasets into a single new dataset.

## Stage 1: Frontend Merge Dialog
**Goal**: Add Merge button and dialog UI to DataSetsPage
**Status**: Complete

### Tasks
1. Add `IconGitMerge` import from tabler-icons-react
2. Add state for merge modal: `mergeModalOpen`, `mergeName`, `sumWeights`, `deleteMerged`
3. Add Merge button next to Export button (disabled when < 2 datasets selected)
4. Create merge dialog with:
   - Name input (default: first selected dataset name)
   - "Sum weights on aggregation" checkbox
   - "Delete merged datasets after merge" checkbox
   - Cancel and Merge buttons

## Stage 2: GraphQL Mutation
**Goal**: Add backend mutation to merge datasets
**Status**: Complete

### Tasks
1. Add `MERGE_DATASOURCES` mutation to `frontend/src/graphql/datasets.ts`
2. Input type: `MergeDataSetsInput` with fields:
   - `projectId: Int!`
   - `dataSetIds: [Int!]!`
   - `name: String!`
   - `sumWeights: Boolean!`
   - `deleteMerged: Boolean!`
3. Returns the newly created merged DataSet

## Stage 3: Backend Mutation Handler
**Goal**: Implement merge logic in Rust backend
**Status**: Complete

### Tasks
1. Add `MergeDataSetsInput` input type to GraphQL schema
2. Add `merge_data_sets` mutation resolver
3. Implement merge logic:
   - Load graphJson from all selected datasets
   - Merge nodes arrays (deduplicate by id, handle weight aggregation)
   - Merge edges arrays (deduplicate by source+target, handle weight aggregation)
   - Merge layers arrays (deduplicate by id)
   - Create new dataset with merged graphJson
   - Optionally delete source datasets
4. Return the new merged dataset

## Stage 4: Frontend Integration
**Goal**: Wire up frontend to backend mutation
**Status**: Complete

### Tasks
1. Add `useMutation` hook for `MERGE_DATASOURCES`
2. Implement `handleMerge` function
3. Show success/error notifications
4. Refetch datasets after merge
5. Clear selection after successful merge

## Success Criteria
- [x] Merge button appears next to Export, disabled when < 2 datasets selected
- [x] Merge dialog shows with name input (defaulting to first dataset name)
- [x] Sum weights checkbox works
- [x] Delete merged checkbox works
- [x] Merged dataset appears in list
- [x] Source datasets deleted if option selected
