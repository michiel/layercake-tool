# Graph Editor Layers Panel Implementation Plan

**Date:** 2025-10-10
**Status:** Planning
**Priority:** High

## Executive Summary

Add a comprehensive layers panel to the graph editor that provides layer management, visibility control, filtering, and editing capabilities. The panel will be a collapsible sidebar on the right side of the graph editor.

## Overview

### Current State
- Graph editor displays LayercakeGraphEditor with ReactFlow canvas
- Layers are loaded from GraphQL but only used for node/edge styling
- No UI for interacting with layers
- No layer visibility controls or filtering

### Target State
- Two-panel layout: main graph canvas + collapsible layers panel
- Layers panel shows all layers with colors, statistics, and controls
- Layer visibility toggle (show/hide individual layers)
- Layer filtering (show only selected layers)
- Layer property editing (name, colors)
- Layer selection/highlighting
- Layer statistics (node/edge counts)
- Add/delete layer functionality
- Responsive layout with panel resize

## Features Breakdown

### 1. View Layer List ✨
**User Story:** As a user, I want to see all layers in my graph with their visual properties

**Requirements:**
- Display layer name
- Show layer color swatch/indicator
- Display background color, border color, text color
- Show layer ID for reference
- Visual hierarchy (grouped by usage)

**UI Components:**
- Layer list with card/item components
- Color swatches for each color property
- Scrollable list for many layers

---

### 2. Show/Hide Layers 👁️
**User Story:** As a user, I want to toggle layer visibility to focus on specific parts of my graph

**Requirements:**
- Checkbox or eye icon to toggle visibility
- Hidden layers: nodes/edges not rendered or visually dimmed
- Visibility state persists during session
- "Show All" / "Hide All" bulk actions
- Visual feedback when layers are hidden

**Implementation:**
- Add `visible` boolean to layer state
- Filter nodes/edges before rendering based on layer visibility
- Update ReactFlow nodes/edges when visibility changes

---

### 3. Edit Layer Properties ✏️
**User Story:** As a user, I want to edit layer colors and names

**Requirements:**
- Edit layer name (inline or modal)
- Change background color (color picker)
- Change border color (color picker)
- Change text color (color picker)
- Save changes to backend (GraphQL mutation)
- Real-time preview of changes
- Validation (name required, valid hex colors)

**UI Components:**
- Inline editing for name (click to edit)
- Color picker dropdowns
- Save/Cancel buttons
- Loading state during save

**Backend:**
- Mutation: `updateLayerProperties(layerId, name, properties)`

---

### 4. Select/Highlight Layer 🎯
**User Story:** As a user, I want to click a layer to select/highlight all nodes in that layer

**Requirements:**
- Click layer to select all associated nodes
- Visual highlight on selected nodes
- Selection persists until cleared or another layer selected
- "Clear Selection" action
- Show selection count

**Implementation:**
- Filter nodes by layer ID
- Use ReactFlow's selection state
- Add visual highlight (border/glow) to selected nodes
- Selection indicator in layer panel

---

### 5. Layer Statistics 📊
**User Story:** As a user, I want to see how many nodes and edges belong to each layer

**Requirements:**
- Display node count per layer
- Display edge count per layer
- Update counts when graph changes
- Visual indicator for empty layers
- Total graph statistics summary

**Implementation:**
- Calculate counts from graph data
- Memoize calculations for performance
- Display as badges or text in layer items

---

### 7. Add/Delete Layers ➕❌
**User Story:** As a user, I want to create new layers and delete unused ones

**Requirements:**
- "Add Layer" button
- Create new layer with default properties
- Delete layer (with confirmation if nodes exist)
- Cannot delete layer if nodes/edges are using it
- Automatically assign default colors to new layers

**UI Components:**
- Add Layer dialog/form
- Delete confirmation modal
- Validation and error messages

**Backend:**
- Mutation: `createLayer(graphId, name, properties)`
- Mutation: `deleteLayer(layerId)`
- Check for node/edge usage before delete

---

### 8. Filter Graph View 🔍
**User Story:** As a user, I want to filter the graph to show only selected layers

**Requirements:**
- Multi-select layers for filtering
- Show only nodes/edges from selected layers
- "Filter Mode" toggle
- Display active filter count
- Clear all filters action
- Distinct from hide/show (filtering is temporary focus)

**Implementation:**
- Maintain separate filter state from visibility
- Apply filters on top of visibility
- Visual indicator in panel (filter icon/badge)
- Filter persists during session but not saved

---

## Technical Architecture

### Component Structure
```
GraphEditorPage
├── Breadcrumbs
└── AppShell / Flex Layout
    ├── LayercakeGraphEditor (Main Canvas)
    │   ├── ReactFlow
    │   ├── Controls
    │   ├── MiniMap
    │   └── Background
    └── LayersPanel (Collapsible Sidebar)
        ├── LayersPanelHeader
        │   ├── Title
        │   ├── Search/Filter input
        │   └── Bulk actions (Show All, Hide All)
        ├── LayerStatsSummary
        │   ├── Total layers count
        │   ├── Total nodes count
        │   └── Total edges count
        ├── LayersList
        │   └── LayerListItem (for each layer)
        │       ├── Visibility toggle
        │       ├── Color swatches
        │       ├── Layer name (editable)
        │       ├── Statistics badges
        │       ├── Selection indicator
        │       └── Actions menu (Edit, Delete)
        └── LayersPanelFooter
            ├── Add Layer button
            └── Filter controls
```

### State Management

**Local Component State:**
```typescript
interface LayersState {
  layers: Layer[];
  visibilityMap: Map<string, boolean>; // layerId -> visible
  filterSet: Set<string>; // layerIds to filter
  filterMode: boolean; // Is filtering active?
  selectedLayerId: string | null; // For highlighting
  editingLayerId: string | null; // For inline editing
}
```

**GraphQL Operations:**
```graphql
# Query (already exists)
query GetGraphDetails($id: Int!) {
  graph(id: $id) {
    layers {
      id
      layerId
      name
      color
      properties
    }
    # ... nodes and edges
  }
}

# Mutations (to be created)
mutation UpdateLayerProperties(
  $layerId: String!
  $name: String!
  $properties: JSON!
) {
  updateLayerProperties(
    layerId: $layerId
    name: $name
    properties: $properties
  ) {
    id
    layerId
    name
    properties
  }
}

mutation CreateLayer(
  $graphId: Int!
  $layerId: String!
  $name: String!
  $properties: JSON!
) {
  createLayer(
    graphId: $graphId
    layerId: $layerId
    name: $name
    properties: $properties
  ) {
    id
    layerId
    name
    properties
  }
}

mutation DeleteLayer($layerId: String!) {
  deleteLayer(layerId: $layerId) {
    success
    message
  }
}
```

### Data Flow

1. **Load Graph Data:**
   - GraphQL query fetches graph with layers, nodes, edges
   - Initialize visibility map (all visible)
   - Calculate layer statistics

2. **Layer Visibility Toggle:**
   - Update visibility map
   - Filter nodes/edges before passing to ReactFlow
   - Re-render graph

3. **Layer Filtering:**
   - Update filter set
   - Apply filter on top of visibility
   - Re-render graph

4. **Layer Selection:**
   - Find all nodes with matching layer ID
   - Set ReactFlow selection state
   - Highlight selected nodes

5. **Layer Editing:**
   - Local state update for immediate feedback
   - GraphQL mutation to persist
   - Optimistic update with rollback on error

6. **Add/Delete Layer:**
   - Validation (check for node usage on delete)
   - GraphQL mutation
   - Refetch or optimistically update cache

---

## Implementation Phases

### Phase 1: Basic Panel UI (3-4 hours)
**Goal:** Display layers panel with basic layout and layer list

**Tasks:**
1. Create `LayersPanel.tsx` component
2. Add Mantine Grid/Flex layout to GraphEditorPage
3. Implement collapsible sidebar (Drawer or custom)
4. Create `LayerListItem.tsx` component
5. Display layer name, color swatches
6. Calculate and display layer statistics
7. Style panel to match application theme

**Acceptance Criteria:**
- Panel displays on right side of graph editor
- Can collapse/expand panel
- Shows all layers with colors and statistics
- Responsive layout (graph adjusts when panel opens/closes)

---

### Phase 2: Visibility Controls (2-3 hours)
**Goal:** Implement show/hide functionality for layers

**Tasks:**
1. Add visibility state management (Map<layerId, boolean>)
2. Add visibility toggle (eye icon or checkbox) to LayerListItem
3. Filter nodes and edges based on visibility before rendering
4. Implement "Show All" / "Hide All" bulk actions
5. Add visual indicator for hidden layers (grayed out)
6. Ensure performance with memoization

**Acceptance Criteria:**
- Clicking eye icon toggles layer visibility
- Hidden layers: associated nodes/edges not rendered in graph
- Bulk actions work correctly
- Graph re-renders smoothly without lag

---

### Phase 3: Layer Statistics & Selection (2 hours)
**Goal:** Display statistics and implement layer selection/highlighting

**Tasks:**
1. Calculate node/edge counts per layer (memoized)
2. Display counts as badges in LayerListItem
3. Add "Select Layer" button/action
4. Implement selection logic (find nodes by layer, set ReactFlow selection)
5. Add visual highlight for selected nodes
6. Add "Clear Selection" action
7. Display selection count in panel

**Acceptance Criteria:**
- Statistics show correct node/edge counts
- Clicking "Select" highlights all layer nodes
- Selection visually distinct in graph
- Can clear selection

---

### Phase 4: Layer Property Editing (4-5 hours)
**Goal:** Enable editing layer name and colors

**Tasks:**
1. Create GraphQL mutation: `updateLayerProperties`
2. Implement backend resolver and service method
3. Add inline name editing to LayerListItem (click to edit)
4. Create color picker components for properties
5. Implement edit mode UI (Save/Cancel)
6. Add validation (name required, valid hex colors)
7. Implement optimistic updates
8. Add loading state and error handling
9. Update graph rendering with new colors immediately

**Backend Tasks:**
- Create mutation in schema
- Implement resolver
- Update layers entity in database
- Return updated layer

**Acceptance Criteria:**
- Can edit layer name inline
- Color pickers work for all three color properties
- Changes save to database
- Graph immediately reflects color changes
- Validation prevents invalid input
- Error messages display on failure

---

### Phase 5: Layer Filtering (2-3 hours)
**Goal:** Implement filter mode to focus on specific layers

**Tasks:**
1. Add filter state (Set<layerId>)
2. Add filter mode toggle
3. Add multi-select checkboxes to layer items
4. Implement "Filter" button to apply selected filters
5. Apply filters on top of visibility
6. Display active filter count badge
7. Add "Clear Filters" action
8. Visual indicator for filtered state

**Acceptance Criteria:**
- Can select multiple layers for filtering
- Filter mode shows only nodes/edges from selected layers
- Filter indicator shows active filter count
- Can clear filters
- Filters work alongside visibility controls

---

### Phase 6: Add/Delete Layers (4-5 hours)
**Goal:** Allow creating new layers and deleting unused ones

**Tasks:**
1. Create GraphQL mutations: `createLayer`, `deleteLayer`
2. Implement backend resolvers and service methods
3. Add "Add Layer" button and dialog/form
4. Implement new layer form (name, default colors)
5. Generate unique layerId automatically
6. Implement delete confirmation modal
7. Check for node/edge usage before allowing delete
8. Add error handling (can't delete if in use)
9. Update graph list after add/delete
10. Handle edge cases (empty graph, last layer)

**Backend Tasks:**
- Create mutations in schema
- Implement create/delete logic
- Validate layer usage on delete
- Handle cascading if allowed
- Return appropriate errors

**Acceptance Criteria:**
- "Add Layer" creates new layer with default properties
- New layer appears in list immediately
- Can delete unused layers
- Cannot delete layers with nodes/edges (show error)
- Confirmation modal prevents accidental deletion
- Graph data refreshes after operations

---

## UI/UX Design Guidelines

### Layout
- **Panel Width:** 320-400px (resizable optional)
- **Panel Position:** Right side of screen
- **Collapse:** Icon button to collapse/expand
- **Responsive:** On small screens, panel overlays graph

### Visual Design
- **Color Swatches:** 24x24px squares with border
- **Layer Items:** Card-like appearance with hover state
- **Icons:** Tabler Icons for consistency
- **Spacing:** Use Mantine spacing scale
- **Typography:** Match application font hierarchy

### Interactions
- **Click layer name:** Select/highlight nodes in that layer
- **Click eye icon:** Toggle visibility
- **Double-click name:** Enter edit mode
- **Drag (optional):** Reorder layers (future)

### Feedback
- **Loading:** Spinner or skeleton during mutations
- **Success:** Subtle animation or toast notification
- **Error:** Red alert or inline error message
- **Empty State:** "No layers" message with "Add Layer" CTA

---

## Backend Requirements

### GraphQL Schema Updates

```graphql
extend type Mutation {
  updateLayerProperties(
    layerId: String!
    name: String!
    properties: JSON!
  ): Layer!

  createLayer(
    graphId: Int!
    layerId: String!
    name: String!
    properties: JSON!
  ): Layer!

  deleteLayer(layerId: String!): DeleteLayerResult!
}

type DeleteLayerResult {
  success: Boolean!
  message: String
}
```

### Service Methods

```rust
// In layer_service.rs or graph_service.rs

pub async fn update_layer_properties(
    db: &DatabaseConnection,
    layer_id: &str,
    name: String,
    properties: serde_json::Value,
) -> Result<Layer> {
    // Validate properties
    // Update database
    // Return updated layer
}

pub async fn create_layer(
    db: &DatabaseConnection,
    graph_id: i32,
    layer_id: String,
    name: String,
    properties: serde_json::Value,
) -> Result<Layer> {
    // Validate uniqueness of layer_id
    // Insert new layer
    // Return created layer
}

pub async fn delete_layer(
    db: &DatabaseConnection,
    layer_id: &str,
) -> Result<bool> {
    // Check if layer is used by nodes/edges
    // If not used, delete layer
    // Return success/error
}

pub async fn check_layer_usage(
    db: &DatabaseConnection,
    layer_id: &str,
) -> Result<(usize, usize)> {
    // Count nodes using this layer
    // Count edges using this layer
    // Return (node_count, edge_count)
}
```

---

## Performance Considerations

### Optimization Strategies

1. **Memoization:**
   - Memoize layer statistics calculations
   - Memoize filtered nodes/edges
   - Use `useMemo` for expensive computations

2. **Virtualization:**
   - If >50 layers, implement virtual scrolling
   - Use react-window or similar

3. **Debouncing:**
   - Debounce search/filter input (300ms)
   - Debounce color picker changes for real-time preview

4. **GraphQL Optimization:**
   - Use optimistic updates for better UX
   - Consider subscriptions for multi-user collaboration
   - Implement proper cache invalidation

5. **ReactFlow Performance:**
   - Use `memo` for LayerListItem components
   - Avoid unnecessary re-renders with stable callbacks
   - Batch node/edge updates when changing visibility

---

## Testing Strategy

### Unit Tests
- Layer statistics calculation
- Visibility filtering logic
- Filter combination (visibility + filter mode)
- Validation functions (color format, name)

### Integration Tests
- GraphQL mutations
- Layer CRUD operations
- Database constraints (unique layer_id)

### E2E Tests
- Toggle layer visibility
- Edit layer properties
- Create and delete layers
- Select layer to highlight nodes
- Apply filters

### Manual Testing Checklist
- [ ] Panel displays correctly on all screen sizes
- [ ] Visibility toggle works for each layer
- [ ] Bulk show/hide works
- [ ] Statistics are accurate
- [ ] Layer selection highlights correct nodes
- [ ] Editing saves and displays immediately
- [ ] Color changes reflect in graph
- [ ] Cannot delete layer with nodes (error shown)
- [ ] Can delete empty layer
- [ ] Add layer creates with default colors
- [ ] Filter mode shows only selected layers
- [ ] Filters clear properly
- [ ] Panel collapse/expand works smoothly

---

## Security & Validation

### Input Validation
- **Layer Name:** Max 100 characters, required
- **Layer ID:** Valid identifier format, unique within graph
- **Colors:** Valid hex format (#RRGGBB or RRGGBB)
- **Properties:** Valid JSON structure

### Authorization
- Check user has edit permission on graph
- Validate graph ownership before mutations

### Error Handling
- Graceful degradation if panel fails to load
- Clear error messages for users
- Log errors for debugging

---

## Future Enhancements (Out of Scope)

- **Layer Reordering:** Drag-and-drop to reorder (z-index auto-calculated)
- **Layer Groups:** Organize layers into collapsible groups
- **Layer Templates:** Save and apply layer color schemes
- **Layer Search:** Search/filter layers by name
- **Bulk Operations:** Multi-select layers for bulk edit/delete
- **Layer Presets:** Predefined color palettes
- **Export/Import:** Export layer configuration as JSON
- **Layer History:** Undo/redo layer changes
- **Keyboard Shortcuts:** Quick actions for power users

---

## Success Metrics

- [ ] Panel loads in < 500ms
- [ ] Layer visibility toggle responds in < 100ms
- [ ] Editing layer saves in < 1s (network dependent)
- [ ] No noticeable lag when filtering 100+ nodes
- [ ] All tests passing
- [ ] Zero critical bugs in production after 1 week

---

## Implementation Timeline

| Phase | Estimated Time | Dependencies |
|-------|---------------|--------------|
| Phase 1: Basic Panel UI | 3-4 hours | None |
| Phase 2: Visibility Controls | 2-3 hours | Phase 1 |
| Phase 3: Statistics & Selection | 2 hours | Phase 1 |
| Phase 4: Property Editing | 4-5 hours | Phase 1, Backend |
| Phase 5: Filtering | 2-3 hours | Phase 2 |
| Phase 6: Add/Delete Layers | 4-5 hours | Phase 4, Backend |
| **Total** | **17-22 hours** | |

Plus testing, bug fixes, and polish: **+5-8 hours**

**Grand Total: 22-30 hours** (3-4 working days)

---

## Risk Assessment

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|------------|
| Performance issues with many layers | High | Medium | Implement virtualization, memoization |
| Complex state management | Medium | Medium | Use clear state structure, consider Zustand |
| GraphQL mutation errors | Medium | Low | Add proper error handling, optimistic updates |
| UI/UX not intuitive | Medium | Medium | User testing, iterative design |
| Backend layer deletion edge cases | High | Low | Thorough validation, clear error messages |

---

## Questions to Resolve

- [ ] Should layer visibility persist across sessions (save to backend)?
- [ ] Should we support layer reordering in future?
- [ ] What should happen to nodes/edges if their layer is deleted? (Orphaned or prevented?)
- [ ] Should we support multi-user real-time collaboration on layer changes?
- [ ] Max number of layers per graph?

---

## Dependencies

### Frontend
- `@mantine/core` - UI components
- `@tabler/icons-react` - Icons
- `reactflow` - Graph rendering
- `@apollo/client` - GraphQL client

### Backend
- `async-graphql` - GraphQL schema
- `sea-orm` - Database ORM
- `serde_json` - JSON handling

---

## Rollout Strategy

1. **Development:** Implement in feature branch
2. **Code Review:** Review all phases before merge
3. **Testing:** Full test suite + manual QA
4. **Deployment:** Deploy to staging first
5. **User Feedback:** Gather feedback from 2-3 users
6. **Refinement:** Address feedback and edge cases
7. **Production:** Deploy to production
8. **Monitoring:** Monitor for errors and performance

---

## Conclusion

This comprehensive layers panel will significantly enhance the graph editor's usability by providing essential layer management features. The phased approach allows for incremental development and testing, reducing risk and ensuring quality.

The implementation prioritizes user experience with real-time feedback, clear visual indicators, and robust error handling. By following this plan, we'll deliver a professional, polished feature that meets user needs.
