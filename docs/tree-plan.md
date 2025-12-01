# Tree Editor Implementation Plan

## Goals
- Visualise and edit dataset hierarchies defined by `graphJson.nodes[].belongs_to`.
- Support CRUD (add/delete/update) operations with cascading delete of descendants.
- Allow drag-and-drop reparenting plus reordering that updates `belongs_to`.
- Style nodes according to their assigned layer colours.
- Surface an inspector panel with editable fields: `id`, `label`, `comment`, `layer`.
- Persist edits back to `updateDataSetGraphData`.

## Target Stack
- **Tree engine:** `react-arborist` (virtualised, headless) composed with shadcn/ui classes.
- **State:** local normalized store (`Record<string, TreeNode>`, children map) derived from dataset `graphJson`.
- **Forms:** shadcn `Input`, `Textarea`, `Select` backed by React Hook Form for the inspector panel.

## UX Layout
1. **Left column:** scrollable tree (arborist) with toolbar for add/delete/root actions.
2. **Right column:** inspector drawer showing editable fields for the selected node plus layer colour previews.
3. **Status bar:** dirty indicator + "Save hierarchy" / "Discard changes".

## Core Behaviours
- **Delete node:** remove node and recursively delete descendants; warn if node has children. Update edges array if present.
- **Drag and drop:** arborist `onMove` hook updates `belongs_to` and sibling order. Restrict invalid drops (no cycles).
- **Add node:** toolbar button opens quick form (layer dropdown + parent selection). Auto-generate unique id (prefixed) or allow manual entry with validation.
- **Selection:** clicking a node populates inspector; unsaved edits apply to local store immediately.
- **Layer styling:** fetch dataset layers (from `graphJson.layers` or `ProjectLayersPage` cache) and map to Tailwind classes, falling back to neutral palette.

## Data Flow & Persistence
1. Fetch dataset via `GET_DATASOURCE` and parse `graphJson`.
2. Normalise nodes into:
   - `nodesById: Record<string, TreeNode>`
   - `childrenByParent: Record<string | null, string[]>`
3. After each edit, recompute arrays needed by backend:
   - `nodes` array (preserve other fields).
   - `edges` array (for hierarchy edges we synthesize from belongs_to if original file lacks them).
4. Persist via `UPDATE_DATASOURCE_GRAPH_DATA` with stringified JSON.
5. Optimistically update Apollo cache & dirty flag.

## Validation
- Ensure `id` uniqueness and non-empty labels.
- Prevent deleting required synthetic roots.
- When changing layer, verify chosen layer exists; otherwise allow inline creation?

## Implementation Steps
1. **Infra**
   - Add `react-arborist` dependency and Tailwind tokens for layer colours.
   - Extend dataset type definitions with helper interfaces (`HierarchyNode`).
2. **State hooks**
   - `useHierarchyStore(dataset)` builds `nodesById`, `roots`, `dirty` flag, and exposes methods (`addNode`, `deleteNode`, `moveNode`, `updateNode`).
   - Implement cascade delete + belongs_to updates in the hook.
3. **Tree UI**
   - Create `DatasetTreeEditor` component using arborist, custom node renderer, toolbar, and drag/drop handlers.
   - Style nodes using layer colours; show badges for partitions or missing parents.
4. **Inspector panel**
   - Build `HierarchyInspector` with form inputs, validation, and live preview of layer colours.
   - Wire to store updates on blur/change.
5. **Integration**
   - Add new "Hierarchy" tab inside `DataSetEditor` (beside Graph Data/Data Edit).
   - Hook save/discard buttons to `updateDataSetGraphData`.
6. **Persistence and Testing**
   - Ensure mutated graphJson updates dataset counts (nodeCount/edgeCount) if backend expects them.
   - Add React component tests for mutators + manual QA checklist.

## Open Questions
- Should edits also emit `graphEdges` entries (mirroring `Graph::get_hierarchy_edges`)? Proposed approach: regenerate edges client-side after each save.
- Do we need versioning/undo? Not in initial scope; rely on discard + dataset edit history.
- Layer dropdown options: use dataset `graphJson.layers` first, then fall back to project-level layers if dataset lacks definitions.
