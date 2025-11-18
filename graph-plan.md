# Graph Layout Improvement Plan

## Current Issues Analysis

Based on the screenshot showing the graph editor with excessive spacing and diagonal layout patterns, I've identified the following core problems:

### 1. Excessive Spacing Between Nodes
**Problem**: Default spacing values are too large, creating vast empty spaces between nodes.

**Root Cause**:
- `nodeSpacing: 75` and `rankSpacing: 75` in LayercakeGraphEditor.tsx (line 71-72)
- Even larger defaults in autoLayout.ts: 150/350 (horizontal) and 120/200 (vertical)

### 2. Diagonal "Staircase" Layout Pattern
**Problem**: Groups/containers are arranged in a diagonal line, wasting space.

**Root Cause** (graphUtils.ts lines 77-144):
- Each subgraph is layouted independently via `layoutSubgraph()`
- Groups at the same level are not positioned optimally relative to each other
- Dagre doesn't handle compound/hierarchical graphs well
- Children are positioned within their parent's local coordinate system, but parent sizing is done before children are layouted

### 3. Insufficient Zoom Range
**Problem**: Cannot zoom out far enough to see the entire graph.

**Root Cause**:
- ReactFlow component doesn't have explicit `minZoom` prop
- The `fitView({ minZoom: 0.02 })` only affects the fit action, not the zoom controls

### 4. Groups Not Sized to Content
**Problem**: Container nodes use fixed minimum sizes (320x240) regardless of actual content.

**Root Cause** (graphUtils.ts lines 95-101):
- Groups are given hardcoded minimum dimensions
- No calculation of actual required space based on children

---

## Improvement Plan

### Stage 1: Quick Wins - Spacing and Zoom
**Goal**: Immediately reduce wasted space and allow viewing full graph
**Success Criteria**: 50% reduction in empty space, zoom out to see 100+ nodes
**Tests**: Visual comparison, verify fitView works correctly

**Changes**:
1. Reduce default spacing in LayercakeGraphEditor.tsx:
   ```typescript
   nodeSpacing = 40,  // was 75
   rankSpacing = 50,  // was 75
   ```

2. Add explicit zoom limits to ReactFlow component:
   ```typescript
   <ReactFlow
     minZoom={0.01}
     maxZoom={4}
     ...
   />
   ```

3. Update autoLayout.ts defaults:
   ```typescript
   const DEFAULT_HORIZONTAL_NODE_SPACING = 80;   // was 150
   const DEFAULT_HORIZONTAL_RANK_SPACING = 150;  // was 350
   const DEFAULT_VERTICAL_NODE_SPACING = 60;     // was 120
   const DEFAULT_VERTICAL_RANK_SPACING = 100;    // was 200
   ```

**Status**: Complete (commit e0c17a99)

---

### Stage 2: Smart Container Sizing
**Goal**: Size containers based on their actual content
**Success Criteria**: Containers only as large as needed, plus padding
**Tests**: Verify containers shrink/grow appropriately

**Changes**:
1. Layout children first, then calculate parent size
2. Modify `layoutSubgraph()` to handle compound graphs:
   ```typescript
   // First pass: layout children to get their bounding box
   const childBounds = calculateChildBounds(children);

   // Second pass: set parent size based on children
   const parentWidth = childBounds.width + GROUP_PADDING * 2;
   const parentHeight = childBounds.height + GROUP_PADDING * 2;
   ```

3. Reduce GROUP_MIN_WIDTH/HEIGHT:
   ```typescript
   const GROUP_MIN_WIDTH = 200;   // was 320
   const GROUP_MIN_HEIGHT = 120;  // was 240
   ```

**Status**: Complete (commit b28a9223)

---

### Stage 3: Improved Hierarchical Layout
**Goal**: Eliminate diagonal staircase pattern for sibling groups
**Success Criteria**: Groups at same level arranged horizontally or vertically
**Tests**: Layout multiple sibling containers without diagonal pattern

**Changes**:
1. Implement two-phase layout for compound graphs:
   - Phase 1: Layout all children within each group
   - Phase 2: Layout groups themselves considering their final sizes

2. Use compound graph mode in Dagre:
   ```typescript
   const g = new dagre.graphlib.Graph({ compound: true });

   // Set parent relationships
   children.forEach(child => {
     g.setParent(child.id, parentId);
   });
   ```

3. Add group-to-group spacing parameter separate from node spacing

**Status**: Complete (commit b5a7b07f)

---

### Stage 4: Layout Algorithm Migration to ELK
**Goal**: Use ELK.js for superior hierarchical layout support
**Success Criteria**: Better automatic layouts for complex nested graphs
**Tests**: Compare ELK vs Dagre layouts on same graphs

**Changes**:
1. Add elkjs dependency:
   ```bash
   npm install elkjs
   ```

2. Create `elkLayout.ts` utility similar to `autoLayout.ts`

3. Implement ELK layout options:
   ```typescript
   const elkOptions = {
     'elk.algorithm': 'layered',
     'elk.direction': 'DOWN',
     'elk.spacing.nodeNode': 40,
     'elk.hierarchyHandling': 'INCLUDE_CHILDREN',
     'elk.layered.spacing.nodeNodeBetweenLayers': 50,
   };
   ```

4. Update `getLayoutedElements()` to use ELK when groups are present

**Status**: Complete (commit 75e77f07)

---

### Stage 5: User Controls for Layout
**Goal**: Allow users to adjust layout parameters
**Success Criteria**: UI controls for spacing, algorithm selection
**Tests**: Verify controls update layout in real-time

**Changes**:
1. Add layout preset selector:
   - Compact
   - Comfortable (default)
   - Spacious

2. Add advanced layout panel with sliders:
   - Node spacing
   - Rank spacing
   - Minimum edge length
   - Algorithm selection (Dagre/ELK)

3. Store preferences per graph in user settings

**Status**: Complete (commit 0002300d) - UI controls already existed, updated defaults

---

## Implementation Priority

1. **Stage 1** (High priority): Immediate improvement with minimal risk
2. **Stage 2** (High priority): Significant improvement to container sizing
3. **Stage 3** (Medium priority): Fixes the diagonal pattern issue
4. **Stage 4** (Medium priority): Best long-term solution for hierarchical layouts
5. **Stage 5** (Low priority): Polish and user experience

## Technical Notes

### Dagre Limitations
- Poor support for compound graphs (nested groups)
- Cannot set parent-child relationships with proper edge routing
- Fixed node sizes during layout (can't adapt to content)

### ELK Advantages
- Native hierarchical graph support
- Better edge routing around groups
- More layout algorithm options (layered, force-directed, etc.)
- Active development and better documentation

### Migration Risk
- ELK.js is larger (~500KB gzipped)
- Different API requires new utility functions
- Testing needed across all graph types

## Files to Modify

- `frontend/src/utils/graphUtils.ts` - Main layout logic
- `frontend/src/components/editors/PlanVisualEditor/utils/autoLayout.ts` - DAG editor layout
- `frontend/src/components/graphs/LayercakeGraphEditor.tsx` - Graph editor component
- `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx` - Plan editor

## Related Documentation

- [Dagre Documentation](https://github.com/dagrejs/dagre/wiki)
- [ELK.js Documentation](https://www.eclipse.org/elk/documentation.html)
- [React Flow Layouting Guide](https://reactflow.dev/learn/layouting/layouting)
