# Plan: Implement Floating Edges in Plan DAG Editor

## Overview

Convert the Plan DAG editor from using fixed four-directional handles (top/left/bottom/right) to floating edges that dynamically connect to the closest point on nodes. This will make edge rendering more flexible and visually cleaner.

## Goals

1. Remove fixed handle positions (top/left/bottom/right connectors)
2. Implement floating edges that connect dynamically to the closest point on node boundaries
3. Add a visual edge creation icon in the top-right of source nodes
4. Update data model to remove sourceHandle/targetHandle fields
5. Maintain all existing edge validation and connection logic

## Reference Implementation

ReactFlow Floating Edges example: https://reactflow.dev/examples/edges/floating-edges

## Current Implementation Analysis

### Frontend

**Current Handle System (BaseNode.tsx)**
- Input handles: `input-left` (Position.Left), `input-top` (Position.Top)
- Output handles: `output-right` (Position.Right), `output-bottom` (Position.Bottom)
- Handles are visible connectors at fixed positions on node edges

**Current Edge Data Model**
```typescript
// frontend/src/types/plan-dag.ts
export interface PlanDagEdge {
  id: string;
  source: string;
  target: string;
  sourceHandle?: string | null;  // TO REMOVE
  targetHandle?: string | null;  // TO REMOVE
  metadata: EdgeMetadata;
}

export interface ReactFlowEdge extends PlanDagEdge {
  sourceHandle?: string | null;  // TO REMOVE
  targetHandle?: string | null;  // TO REMOVE
  type?: string;
  animated?: boolean;
  style?: Record<string, any>;
}
```

**Current Connection Logic**
- `onConnect` callback in PlanVisualEditor.tsx creates edges with handle IDs
- `isValidConnection` validates based on node types and existing connections
- Edge reconnection (`onReconnect`) updates handle positions

### Backend

**Database Schema**
```rust
// layercake-core/src/database/entities/plan_dag_edges.rs
// Contains source_handle and target_handle columns
```

**GraphQL Types**
```rust
// layercake-core/src/graphql/types/plan_dag.rs
// PlanDagEdge type includes sourceHandle and targetHandle fields
```

## Implementation Plan

### Phase 1: Create Floating Edge Components ✅ **COMPLETED**

**Status**: Implemented floating edge components with bezier paths
- Created `edgeUtils.ts` with intersection calculation algorithms
- Created `FloatingEdge.tsx` with dynamic edge rendering
- Created `FloatingConnectionLine.tsx` for connection preview

**1.1 Create FloatingEdge Component**
- Location: `frontend/src/components/editors/PlanVisualEditor/edges/FloatingEdge.tsx`
- Implement helper functions:
  - `getNodeIntersection(intersectionNode, targetNode)` - Calculate intersection point between two nodes
  - `getEdgeParams(source, target)` - Get parameters for edge positioning
  - `getEdgePosition(nodeIntersection, targetPosition)` - Determine which side of node to connect to

```typescript
interface FloatingEdgeProps {
  id: string;
  source: string;
  target: string;
  markerEnd?: string;
  style?: React.CSSProperties;
  data?: any;
}

export function FloatingEdge({ id, source, target, markerEnd, style, data }: FloatingEdgeProps) {
  const sourceNode = useStore(selector(source), shallow);
  const targetNode = useStore(selector(target), shallow);

  if (!sourceNode || !targetNode) {
    return null;
  }

  const { sx, sy, tx, ty } = getEdgeParams(sourceNode, targetNode);

  const [edgePath] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    targetX: tx,
    targetY: ty,
  });

  return (
    <BaseEdge
      id={id}
      path={edgePath}
      markerEnd={markerEnd}
      style={style}
    />
  );
}
```

**1.2 Create FloatingConnectionLine Component**
- Location: `frontend/src/components/editors/PlanVisualEditor/edges/FloatingConnectionLine.tsx`
- Used during edge creation to show preview before connection completes

```typescript
export function FloatingConnectionLine({
  toX,
  toY,
  fromPosition,
  fromNode
}: ConnectionLineComponentProps) {
  if (!fromNode) {
    return null;
  }

  const targetNode = {
    id: 'connection-target',
    width: 1,
    height: 1,
    positionAbsolute: { x: toX, y: toY }
  };

  const { sx, sy } = getEdgeParams(fromNode, targetNode);

  const [edgePath] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    targetX: toX,
    targetY: toY,
  });

  return (
    <g>
      <path
        fill="none"
        stroke="#222"
        strokeWidth={1.5}
        className="animated"
        d={edgePath}
      />
      <circle
        cx={toX}
        cy={toY}
        fill="#fff"
        r={3}
        stroke="#222"
        strokeWidth={1.5}
      />
    </g>
  );
}
```

### Phase 2: Update Node Component

**2.1 Remove Fixed Handles from BaseNode.tsx**
- Remove all `<Handle>` components (lines 134-164 for inputs, 296-326 for outputs)
- Replace with single invisible handle covering entire node

```typescript
// BaseNode.tsx changes
return (
  <>
    {/* Single invisible handle for the entire node - targets can connect anywhere */}
    {requiredInputs > 0 && (
      <Handle
        type="target"
        position={Position.Top}  // Position doesn't matter for floating edges
        style={{ visibility: 'hidden' }}
        isConnectable={!readonly}
      />
    )}

    <Paper /* ... existing Paper component ... */>
      {/* Existing node content */}
    </Paper>

    {/* Single invisible source handle - sources can originate anywhere */}
    {canHaveOutputs && (
      <Handle
        type="source"
        position={Position.Bottom}  // Position doesn't matter for floating edges
        style={{ visibility: 'hidden' }}
        isConnectable={!readonly}
      />
    )}
  </>
);
```

**2.2 Add Edge Creation Icon**
- Add icon button in top-right corner of nodes that can be sources
- Use `IconArrowRight` or `IconLink` from `@tabler/icons-react`
- Position absolute in top-right corner

```typescript
// In BaseNode.tsx, add to Paper component:
{canHaveOutputs && !readonly && (
  <div
    style={{
      position: 'absolute',
      top: 8,
      right: 8,
      zIndex: 10,
      cursor: 'pointer',
      color: color,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      width: 24,
      height: 24,
      borderRadius: '50%',
      background: '#f8f9fa',
      border: `1px solid ${color}`,
    }}
    onMouseDown={(e) => {
      e.stopPropagation();
      // This will trigger ReactFlow's connection mode
    }}
    className="nodrag"
  >
    <IconArrowRight size={14} />
  </div>
)}
```

### Phase 3: Update ReactFlow Configuration

**3.1 Register Edge Types**
```typescript
// In PlanVisualEditor.tsx
import { FloatingEdge } from './edges/FloatingEdge';
import { FloatingConnectionLine } from './edges/FloatingConnectionLine';

const edgeTypes = {
  floating: FloatingEdge,
};
```

**3.2 Update ReactFlow Props**
```typescript
<ReactFlow
  nodes={nodes}
  edges={edges}
  edgeTypes={edgeTypes}
  connectionLineComponent={FloatingConnectionLine}
  onConnect={onConnect}
  isValidConnection={isValidConnection}
  // Remove connectionMode or keep as ConnectionMode.Loose
  // ... other props
/>
```

**3.3 Update onConnect Handler**
- Remove sourceHandle and targetHandle from edge creation
- Set edge type to 'floating'

```typescript
const onConnect = useCallback(
  (connection: Connection) => {
    // ... validation logic ...

    const newEdge: ReactFlowEdge = {
      id: `temp-edge-${Date.now()}`,
      source: connection.source!,
      target: connection.target!,
      // Remove: sourceHandle, targetHandle
      type: 'floating',  // Use floating edge type
      animated: false,
      style: { /* ... */ },
      data: { metadata: { /* ... */ } },
      metadata: { /* ... */ }
    };

    setEdges((eds) => addEdge(newEdge, eds));

    // Backend edge without handle fields
    const graphqlEdge: Partial<ReactFlowEdge> = {
      source: connection.source!,
      target: connection.target!,
      // Remove: sourceHandle, targetHandle
      metadata: { /* ... */ }
    };

    mutations.addEdge(graphqlEdge);
  },
  [/* deps */]
);
```

### Phase 4: Update Data Model

**4.1 Frontend TypeScript Types**
```typescript
// frontend/src/types/plan-dag.ts
export interface PlanDagEdge {
  id: string;
  source: string;
  target: string;
  // REMOVED: sourceHandle?: string | null;
  // REMOVED: targetHandle?: string | null;
  metadata: EdgeMetadata;
}

export interface ReactFlowEdge extends PlanDagEdge {
  // REMOVED: sourceHandle?: string | null;
  // REMOVED: targetHandle?: string | null;
  type?: string;
  animated?: boolean;
  style?: Record<string, any>;
}
```

**4.2 Backend Rust Types**
```rust
// layercake-core/src/graphql/types/plan_dag.rs
#[derive(async_graphql::SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    // REMOVED: pub source_handle: Option<String>,
    // REMOVED: pub target_handle: Option<String>,
    pub metadata: EdgeMetadata,
}
```

**4.3 Database Migration**
Create migration to drop columns:
```sql
-- migrations/YYYYMMDDHHMMSS_remove_edge_handles.sql
ALTER TABLE plan_dag_edges DROP COLUMN source_handle;
ALTER TABLE plan_dag_edges DROP COLUMN target_handle;
```

**4.4 Update Entity Definitions**
```rust
// layercake-core/src/database/entities/plan_dag_edges.rs
#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = plan_dag_edges)]
pub struct PlanDagEdge {
    pub id: String,
    pub project_id: i32,
    pub source: String,
    pub target: String,
    // REMOVED: pub source_handle: Option<String>,
    // REMOVED: pub target_handle: Option<String>,
    pub metadata: sqlx::types::Json<EdgeMetadata>,
    // ... other fields
}
```

### Phase 5: Update Edge Operations

**5.1 Update Edge Reconnection**
```typescript
// In PlanVisualEditor.tsx - handleReconnect
const handleReconnect = useCallback(
  (oldEdge: Edge, newConnection: Connection) => {
    // ... validation logic ...

    const newEdge: ReactFlowEdge = {
      ...oldEdge,
      id: `temp-edge-${Date.now()}`,
      source: newConnection.source!,
      target: newConnection.target!,
      // REMOVED: sourceHandle and targetHandle assignments
      type: 'floating',
      data: { /* ... */ }
    };

    // ... rest of reconnection logic
  },
  [/* deps */]
);
```

**5.2 Update Backend Services**
Remove all references to source_handle and target_handle:
- `layercake-core/src/services/plan_dag_service.rs`
- `layercake-core/src/graphql/mutations/mod.rs`
- `layercake-core/src/services/sample_project_service.rs`

### Phase 6: Visual Polish

**6.1 Edge Styling**
- Maintain current color coding (blue for GRAPH_REFERENCE, gray for GRAPH_DATA)
- Ensure smooth transitions when nodes move
- Add subtle animation on edge creation

**6.2 Connection Feedback**
- Visual feedback when hovering over nodes during edge creation
- Highlight valid connection targets
- Show invalid connection state clearly

**6.3 Edge Creation UX**
- Click the top-right icon to start edge creation
- Drag to target node
- Release to create connection
- ESC to cancel edge creation

## Testing Plan

### Unit Tests
1. Test getNodeIntersection calculation with various node positions
2. Test getEdgeParams with different node sizes
3. Test edge validation logic without handle constraints

### Integration Tests
1. Create edges between all node type combinations
2. Test edge reconnection
3. Test edge deletion
4. Test undo/redo with floating edges

### Visual Tests
1. Verify edges connect smoothly to node boundaries
2. Test with various node positions and layouts
3. Verify edge rendering during drag operations
4. Test connection preview line

### Migration Tests
1. Test data migration from old format (with handles) to new format (without handles)
2. Verify existing edges display correctly after migration
3. Test backward compatibility if needed

## Rollout Strategy

### Development
1. Create feature branch: `feature/floating-edges`
2. Implement frontend changes first (Phases 1-3)
3. Test thoroughly in isolation
4. Implement backend changes (Phase 4-5)
5. Run full integration tests

### Staging
1. Deploy to staging environment
2. Run migration on test data
3. Perform manual QA testing
4. Verify performance with large graphs

### Production
1. Create backup of database
2. Run migration during maintenance window
3. Monitor for errors
4. Have rollback plan ready

## Risks and Mitigations

### Risk: Edge positioning calculation performance
**Mitigation**: Use memoization and only recalculate when nodes move

### Risk: Breaking existing edge data
**Mitigation**: Write migration script carefully, test on copy of production data first

### Risk: UX confusion with new edge creation method
**Mitigation**: Add tooltip on edge creation icon, provide visual feedback during drag

### Risk: Edge routing looks worse than current fixed positions
**Mitigation**: Implement smart routing algorithm, allow manual edge styling if needed

## Success Criteria

- [ ] All existing edges display correctly with floating positions
- [ ] New edges can be created by clicking icon and dragging
- [ ] Edge validation logic works correctly
- [ ] No performance degradation with large graphs (100+ nodes)
- [ ] Database migration completes successfully
- [ ] All tests pass
- [ ] User testing shows improved or equal UX

## Timeline Estimate

- Phase 1 (Floating Edge Components): 4-6 hours
- Phase 2 (Update Node Component): 3-4 hours
- Phase 3 (ReactFlow Configuration): 2-3 hours
- Phase 4 (Data Model Updates): 4-6 hours
- Phase 5 (Edge Operations): 3-4 hours
- Phase 6 (Visual Polish): 2-3 hours
- Testing and QA: 4-6 hours
- **Total: 22-32 hours**

## Dependencies

- ReactFlow 11.x (current version supports floating edges)
- No new npm packages required
- Database migration tool (existing)

## Follow-up Work

After initial implementation:
1. Consider adding edge labels at midpoint
2. Implement custom edge paths (curved vs straight)
3. Add edge splitting/merging capabilities
4. Consider edge bundling for dense graphs

---

## Technical Implementation Details

### Mathematical Algorithm for Edge Intersection

The core of floating edges is calculating where an edge should connect to a node's boundary. This involves finding the intersection point between a line (from source center to target center) and the node's bounding box.

**Algorithm: getNodeIntersection**
```typescript
import { Node, internalsSymbol } from 'reactflow';

interface Point {
  x: number;
  y: number;
}

function getNodeIntersection(
  intersectionNode: Node,
  targetNode: Node
): Point {
  // Get the dimensions from node internals
  const {
    width: intersectionNodeWidth,
    height: intersectionNodeHeight,
    positionAbsolute: intersectionNodePosition,
  } = intersectionNode[internalsSymbol] || {};

  const targetPosition = targetNode[internalsSymbol]?.positionAbsolute || { x: 0, y: 0 };

  // Calculate centers of both nodes
  const w = (intersectionNodeWidth ?? 0) / 2;
  const h = (intersectionNodeHeight ?? 0) / 2;

  const x2 = (intersectionNodePosition?.x ?? 0) + w;
  const y2 = (intersectionNodePosition?.y ?? 0) + h;
  const x1 = targetPosition.x + (targetNode[internalsSymbol]?.width ?? 0) / 2;
  const y1 = targetPosition.y + (targetNode[internalsSymbol]?.height ?? 0) / 2;

  // Calculate the slope of the line between node centers
  const xx1 = (x1 - x2) / (2 * w) - (y1 - y2) / (2 * h);
  const yy1 = (x1 - x2) / (2 * w) + (y1 - y2) / (2 * h);
  const a = 1 / (Math.abs(xx1) + Math.abs(yy1));
  const xx3 = a * xx1;
  const yy3 = a * yy1;
  const x = w * (xx3 + yy3) + x2;
  const y = h * (-xx3 + yy3) + y2;

  return { x, y };
}
```

**Explanation:**
1. Get absolute positions and dimensions of both nodes from ReactFlow internals
2. Calculate center points of both nodes
3. Use vector math to find the intersection point on the bounding box
4. The algorithm normalizes the vector between centers and projects it onto the node boundary

**Algorithm: getEdgeParams**
```typescript
import { Position } from 'reactflow';

interface EdgeParams {
  sx: number;  // source x
  sy: number;  // source y
  tx: number;  // target x
  ty: number;  // target y
  sourcePos: Position;
  targetPos: Position;
}

function getEdgeParams(source: Node, target: Node): EdgeParams {
  const sourceIntersectionPoint = getNodeIntersection(source, target);
  const targetIntersectionPoint = getNodeIntersection(target, source);

  const sourcePos = getEdgePosition(source, sourceIntersectionPoint);
  const targetPos = getEdgePosition(target, targetIntersectionPoint);

  return {
    sx: sourceIntersectionPoint.x,
    sy: sourceIntersectionPoint.y,
    tx: targetIntersectionPoint.x,
    ty: targetIntersectionPoint.y,
    sourcePos,
    targetPos,
  };
}
```

**Algorithm: getEdgePosition**
```typescript
function getEdgePosition(node: Node, intersectionPoint: Point): Position {
  const nodePosition = node[internalsSymbol]?.positionAbsolute || { x: 0, y: 0 };
  const nodeWidth = node[internalsSymbol]?.width ?? 0;
  const nodeHeight = node[internalsSymbol]?.height ?? 0;

  const n = { ...nodePosition, width: nodeWidth, height: nodeHeight };
  const nx = Math.round(n.x);
  const ny = Math.round(n.y);
  const px = Math.round(intersectionPoint.x);
  const py = Math.round(intersectionPoint.y);

  // Determine which edge of the node the intersection is closest to
  if (px <= nx + 1) {
    return Position.Left;
  }
  if (px >= nx + n.width - 1) {
    return Position.Right;
  }
  if (py <= ny + 1) {
    return Position.Top;
  }
  if (py >= ny + n.height - 1) {
    return Position.Bottom;
  }

  return Position.Top; // Default fallback
}
```

### Complete FloatingEdge Implementation

> **Note:** This implementation uses `getBezierPath` from ReactFlow to match the official floating edges example at https://reactflow.dev/examples/edges/floating-edges. This creates smooth bezier curves instead of the previous step-based edges.

**File: `frontend/src/components/editors/PlanVisualEditor/edges/FloatingEdge.tsx`**

```typescript
import { useCallback } from 'react';
import { useStore, getBezierPath, EdgeProps, Node, internalsSymbol } from 'reactflow';
import { BaseEdge, EdgeLabelRenderer } from 'reactflow';

// Selector for accessing nodes from ReactFlow store
const selector = (s: any) => ({
  nodeInternals: s.nodeInternals,
  edges: s.edges,
});

interface FloatingEdgeProps extends EdgeProps {
  // All standard EdgeProps from ReactFlow
}

export function FloatingEdge({
  id,
  source,
  target,
  markerEnd,
  markerStart,
  style,
  data,
  selected,
}: FloatingEdgeProps) {
  // Access ReactFlow store to get current node positions
  const { nodeInternals } = useStore(selector);

  const sourceNode = nodeInternals.get(source) as Node;
  const targetNode = nodeInternals.get(target) as Node;

  // If nodes don't exist yet, don't render the edge
  if (!sourceNode || !targetNode) {
    return null;
  }

  // Calculate dynamic edge parameters
  const { sx, sy, tx, ty, sourcePos, targetPos } = getEdgeParams(
    sourceNode,
    targetNode
  );

  // Generate bezier path (matches ReactFlow floating edges example)
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    sourcePosition: sourcePos,
    targetX: tx,
    targetY: ty,
    targetPosition: targetPos,
  });

  // Apply styling based on edge data type (maintaining current behavior)
  const edgeColor = data?.metadata?.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96';
  const strokeWidth = selected ? 3 : 2;

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        markerEnd={markerEnd}
        markerStart={markerStart}
        style={{
          ...style,
          stroke: edgeColor,
          strokeWidth,
        }}
      />
      {/* Optional: Add edge label rendering */}
      {data?.metadata?.label && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              fontSize: 10,
              fontWeight: 500,
              background: '#fff',
              padding: '2px 4px',
              borderRadius: 3,
              border: '1px solid #ccc',
              pointerEvents: 'all',
            }}
            className="nodrag nopan"
          >
            {data.metadata.label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}

// Helper functions (as defined above)
function getNodeIntersection(/* ... */) { /* ... */ }
function getEdgeParams(/* ... */) { /* ... */ }
function getEdgePosition(/* ... */) { /* ... */ }
```

### Complete FloatingConnectionLine Implementation

> **Note:** This implementation uses `getBezierPath` from ReactFlow to match the official floating edges example at https://reactflow.dev/examples/edges/floating-edges. Bezier curves provide smooth, natural-looking connections that dynamically adjust based on node positions.

**File: `frontend/src/components/editors/PlanVisualEditor/edges/FloatingConnectionLine.tsx`**

```typescript
import { ConnectionLineComponentProps, getBezierPath, Node } from 'reactflow';

export function FloatingConnectionLine({
  toX,
  toY,
  fromPosition,
  fromNode,
}: ConnectionLineComponentProps) {
  if (!fromNode) {
    return null;
  }

  // Create a virtual target node at the cursor position
  const targetNode = {
    id: 'connection-target',
    [Symbol.for('rf_internals')]: {
      width: 1,
      height: 1,
      positionAbsolute: { x: toX, y: toY },
    },
  } as unknown as Node;

  // Calculate where to start the connection from
  const { sx, sy, tx, ty, sourcePos, targetPos } = getEdgeParams(fromNode, targetNode);

  // Create the bezier path to the cursor (matches ReactFlow floating edges example)
  const [edgePath] = getBezierPath({
    sourceX: sx,
    sourceY: sy,
    sourcePosition: sourcePos,
    targetX: tx || toX,
    targetY: ty || toY,
    targetPosition: targetPos,
  });

  return (
    <g>
      <path
        fill="none"
        stroke="#222"
        strokeWidth={2}
        strokeDasharray="5,5"
        d={edgePath}
      />
      <circle
        cx={toX}
        cy={toY}
        fill="#fff"
        r={4}
        stroke="#222"
        strokeWidth={2}
      />
    </g>
  );
}

// Import helper functions or define them here
function getEdgeParams(/* ... */) { /* ... */ }
```

### Node Component Update - Complete Implementation

**File: `frontend/src/components/editors/PlanVisualEditor/nodes/BaseNode.tsx`**

```typescript
// Add new imports
import { IconLink } from '@tabler/icons-react';

// In the BaseNode component, modify the return statement:

return (
  <>
    {/* Invisible target handle - allows connections from any angle */}
    {requiredInputs > 0 && (
      <Handle
        type="target"
        position={Position.Top}
        style={{
          visibility: 'hidden',
          width: '100%',
          height: '100%',
          border: 'none',
          background: 'transparent',
        }}
        isConnectable={!readonly}
      />
    )}

    <Paper
      shadow={selected ? "md" : "sm"}
      p={0}
      style={{
        position: 'relative',  // Important for absolute positioning of connection icon
        border: `2px solid ${color}`,
        borderRadius: 8,
        minWidth: 200,
        maxWidth: 280,
        background: '#fff',
        cursor: 'default',
        pointerEvents: 'all',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Connection Icon - positioned absolutely in top right */}
      {canHaveOutputs && !readonly && (
        <Tooltip label="Drag to create connection" position="left">
          <div
            data-connection-handle="true"
            style={{
              position: 'absolute',
              top: 8,
              right: 8,
              zIndex: 100,
              cursor: 'crosshair',
              color: color,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              width: 28,
              height: 28,
              borderRadius: '50%',
              background: 'white',
              border: `2px solid ${color}`,
              transition: 'all 0.2s ease',
              '&:hover': {
                transform: 'scale(1.1)',
                boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
              }
            }}
            className="nodrag"  // Prevents node dragging when clicking icon
            onMouseEnter={(e) => {
              e.currentTarget.style.transform = 'scale(1.1)';
              e.currentTarget.style.boxShadow = '0 2px 8px rgba(0,0,0,0.15)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = 'scale(1)';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            <IconLink size={16} stroke={2.5} />
          </div>
        </Tooltip>
      )}

      {/* Rest of node content */}
      {/* ... existing node structure ... */}
    </Paper>

    {/* Invisible source handle - exits from calculated position */}
    {canHaveOutputs && (
      <Handle
        type="source"
        position={Position.Bottom}
        style={{
          visibility: 'hidden',
          width: '100%',
          height: '100%',
          border: 'none',
          background: 'transparent',
        }}
        isConnectable={!readonly}
      />
    )}
  </>
);
```

### ReactFlow Configuration Details

**File: `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`**

```typescript
import { FloatingEdge } from './edges/FloatingEdge';
import { FloatingConnectionLine } from './edges/FloatingConnectionLine';

// Define edge types object
const edgeTypes = useMemo(
  () => ({
    floating: FloatingEdge,
  }),
  []
);

// Inside component:
<ReactFlow
  nodes={nodes}
  edges={edges}
  nodeTypes={nodeTypes}
  edgeTypes={edgeTypes}

  // Connection configuration
  onConnect={onConnect}
  onConnectStart={handleConnectStart}
  onConnectEnd={handleConnectEnd}

  // Use custom connection line
  connectionLineComponent={FloatingConnectionLine}

  // Connection mode
  connectionMode={ConnectionMode.Loose}

  // Validation
  isValidConnection={isValidConnection}

  // Edge interactions
  onEdgeDoubleClick={handleEdgeDoubleClick}
  onReconnect={handleReconnect}
  onReconnectStart={handleReconnectStart}
  onReconnectEnd={handleReconnectEnd}

  // Other props
  {...otherProps}
/>
```

### Updated Connection Handlers

**onConnect Handler - Complete Implementation:**

```typescript
const onConnect = useCallback(
  (connection: Connection) => {
    console.log('[PlanVisualEditor] onConnect called:', connection);

    if (readonly) {
      console.log('[PlanVisualEditor] Connection blocked: readonly mode');
      return;
    }

    // Validate connection
    const sourceNode = nodes.find((n) => n.id === connection.source);
    const targetNode = nodes.find((n) => n.id === connection.target);

    if (!sourceNode || !targetNode) {
      console.warn('[PlanVisualEditor] Invalid connection: missing nodes');
      return;
    }

    // Validate connection using existing logic
    const isValid = validateConnectionWithCycleDetection(
      connection.source!,
      connection.target!,
      nodes,
      edges,
      (sourceNode.data as PlanDagNodeData).nodeType,
      (targetNode.data as PlanDagNodeData).nodeType
    );

    if (!isValid.valid) {
      console.warn('[PlanVisualEditor] Invalid connection:', isValid.message);
      alert(isValid.message);
      return;
    }

    // Create new edge with floating type
    const tempEdgeId = `temp-edge-${Date.now()}-${Math.random()}`;
    const newEdge: ReactFlowEdge = {
      id: tempEdgeId,
      source: connection.source!,
      target: connection.target!,
      // NO sourceHandle or targetHandle - removed!
      type: 'floating',
      animated: false,
      style: {
        stroke: isValid.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96',
        strokeWidth: 2,
      },
      data: {
        metadata: {
          label: '',
          dataType: isValid.dataType
        }
      },
      metadata: {
        label: '',
        dataType: isValid.dataType
      }
    };

    // Optimistic update
    setEdges((eds) => addEdge(newEdge, eds));

    // Persist to backend
    const graphqlEdge: Partial<ReactFlowEdge> = {
      source: connection.source!,
      target: connection.target!,
      // NO sourceHandle or targetHandle!
      metadata: {
        label: '',
        dataType: isValid.dataType
      }
    };

    mutations.addEdge(graphqlEdge).catch(err => {
      console.error('[PlanVisualEditor] Failed to create edge:', err);
      // Remove optimistic edge on failure
      setEdges((eds) => eds.filter(e => e.id !== tempEdgeId));
      alert(`Failed to create connection: ${err.message}`);
    });
  },
  [readonly, nodes, edges, setEdges, mutations]
);
```

### Performance Optimization

**Memoization Strategy:**

```typescript
// In FloatingEdge.tsx
export const FloatingEdge = memo(function FloatingEdge(props: FloatingEdgeProps) {
  // ... implementation
}, (prevProps, nextProps) => {
  // Custom comparison function
  return (
    prevProps.id === nextProps.id &&
    prevProps.source === nextProps.source &&
    prevProps.target === nextProps.target &&
    prevProps.selected === nextProps.selected &&
    prevProps.data?.metadata?.dataType === nextProps.data?.metadata?.dataType
  );
});
```

**Store Selector Optimization:**

```typescript
// Create stable selector
const createNodeSelector = (id: string) => (s: any) => {
  const node = s.nodeInternals.get(id);
  return node ? {
    id: node.id,
    width: node[internalsSymbol]?.width,
    height: node[internalsSymbol]?.height,
    positionAbsolute: node[internalsSymbol]?.positionAbsolute,
  } : null;
};

// Use in component
const sourceNodeData = useStore(
  useCallback(createNodeSelector(source), [source])
);
```

### Database Migration Script

**File: `migrations/YYYYMMDDHHMMSS_remove_edge_handles.sql`**

```sql
-- Remove handle columns from plan_dag_edges table
-- Run this migration carefully with backup

BEGIN;

-- Step 1: Check if columns exist before dropping
DO $$
BEGIN
    -- Drop source_handle column if it exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'plan_dag_edges'
        AND column_name = 'source_handle'
    ) THEN
        ALTER TABLE plan_dag_edges DROP COLUMN source_handle;
        RAISE NOTICE 'Dropped column source_handle';
    END IF;

    -- Drop target_handle column if it exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'plan_dag_edges'
        AND column_name = 'target_handle'
    ) THEN
        ALTER TABLE plan_dag_edges DROP COLUMN target_handle;
        RAISE NOTICE 'Dropped column target_handle';
    END IF;
END$$;

-- Step 2: Verify migration
DO $$
DECLARE
    col_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO col_count
    FROM information_schema.columns
    WHERE table_name = 'plan_dag_edges'
    AND column_name IN ('source_handle', 'target_handle');

    IF col_count > 0 THEN
        RAISE EXCEPTION 'Migration failed: handle columns still exist';
    END IF;

    RAISE NOTICE 'Migration verified successfully';
END$$;

COMMIT;
```

### Rust Backend Updates

**File: `layercake-core/src/database/entities/plan_dag_edges.rs`**

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PlanDagEdge {
    pub id: String,
    pub project_id: i32,
    pub source: String,
    pub target: String,
    // REMOVED: pub source_handle: Option<String>,
    // REMOVED: pub target_handle: Option<String>,
    pub metadata: sqlx::types::Json<EdgeMetadata>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub label: Option<String>,
    #[serde(rename = "dataType")]
    pub data_type: String, // "GRAPH_DATA" or "GRAPH_REFERENCE"
}

// Insert query (update to remove handle columns)
impl PlanDagEdge {
    pub fn insert_query() -> &'static str {
        r#"
        INSERT INTO plan_dag_edges (
            id, project_id, source, target, metadata, created_at, updated_at
        )
        VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
        RETURNING *
        "#
    }
}
```

**File: `layercake-core/src/graphql/types/plan_dag.rs`**

```rust
use async_graphql::*;
use serde::{Deserialize, Serialize};

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
#[graphql(name = "PlanDagEdge")]
pub struct PlanDagEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    // REMOVED: pub source_handle: Option<String>,
    // REMOVED: pub target_handle: Option<String>,
    pub metadata: EdgeMetadata,
}

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    pub label: Option<String>,
    #[serde(rename = "dataType")]
    pub data_type: String,
}

// Input type for creating edges
#[derive(InputObject, Debug, Clone)]
pub struct CreateEdgeInput {
    pub source: String,
    pub target: String,
    // REMOVED: pub source_handle: Option<String>,
    // REMOVED: pub target_handle: Option<String>,
    pub metadata: EdgeMetadataInput,
}
```

### Error Handling and Edge Cases

**1. Nodes Not Yet Loaded:**
```typescript
// In FloatingEdge component
if (!sourceNode || !targetNode) {
  console.warn(`Edge ${id}: nodes not found`, { source, target });
  return null; // Gracefully handle missing nodes
}
```

**2. Node Dimensions Not Available:**
```typescript
const width = node[internalsSymbol]?.width ?? 200; // Default fallback
const height = node[internalsSymbol]?.height ?? 100; // Default fallback
```

**3. Division by Zero in Intersection Calculation:**
```typescript
const denominator = Math.abs(xx1) + Math.abs(yy1);
if (denominator === 0) {
  // Nodes are at same position - return center point
  return { x: x2, y: y2 };
}
const a = 1 / denominator;
```

**4. Invalid Connection During Drag:**
```typescript
const isValidConnection = useCallback(
  (connection: Connection) => {
    try {
      // Validation logic
      return isValid;
    } catch (error) {
      console.error('Validation error:', error);
      return false; // Fail safe
    }
  },
  [nodes, edges]
);
```

### Testing Implementation

**Unit Test: Edge Intersection Calculation**

```typescript
// __tests__/edges/floating-edge-calculations.test.ts
import { getNodeIntersection, getEdgeParams } from '../FloatingEdge';

describe('getNodeIntersection', () => {
  it('calculates correct intersection for horizontal alignment', () => {
    const sourceNode = createMockNode(0, 0, 100, 50);
    const targetNode = createMockNode(200, 0, 100, 50);

    const result = getNodeIntersection(sourceNode, targetNode);

    expect(result.x).toBeCloseTo(50);
    expect(result.y).toBeCloseTo(25);
  });

  it('calculates correct intersection for vertical alignment', () => {
    const sourceNode = createMockNode(0, 0, 100, 50);
    const targetNode = createMockNode(0, 100, 100, 50);

    const result = getNodeIntersection(sourceNode, targetNode);

    expect(result.x).toBeCloseTo(50);
    expect(result.y).toBeCloseTo(50);
  });

  it('handles diagonal connections', () => {
    const sourceNode = createMockNode(0, 0, 100, 100);
    const targetNode = createMockNode(200, 200, 100, 100);

    const result = getNodeIntersection(sourceNode, targetNode);

    expect(result.x).toBeGreaterThan(50);
    expect(result.y).toBeGreaterThan(50);
  });
});

function createMockNode(x: number, y: number, width: number, height: number) {
  return {
    id: 'test',
    [Symbol.for('rf_internals')]: {
      width,
      height,
      positionAbsolute: { x, y },
    },
  };
}
```

### State Management Considerations

**ReactFlow Store Access:**
- FloatingEdge components need real-time access to node positions
- Use `useStore` hook for reactive updates
- Implement proper memoization to prevent unnecessary re-renders

**Edge Re-rendering Strategy:**
```typescript
// Only re-render when:
// 1. Source or target node moves
// 2. Edge selection state changes
// 3. Edge data/style changes

const EdgeComponent = memo(FloatingEdge, (prev, next) => {
  return (
    prev.source === next.source &&
    prev.target === next.target &&
    prev.selected === next.selected &&
    JSON.stringify(prev.data) === JSON.stringify(next.data)
  );
});
```

### Backward Compatibility Strategy

If backward compatibility with existing edges is required during migration:

```typescript
// In ReactFlowAdapter.tsx - when loading edges from backend
const edges = backendEdges.map(edge => ({
  ...edge,
  // Legacy edges with handle data use 'smoothstep', new edges use 'floating' with bezier paths
  type: edge.sourceHandle || edge.targetHandle ? 'smoothstep' : 'floating',
  // Remove handle fields for floating edges
  sourceHandle: edge.type === 'floating' ? undefined : edge.sourceHandle,
  targetHandle: edge.type === 'floating' ? undefined : edge.targetHandle,
}));
```

> **Note:** Once all edges are migrated to floating edges (no sourceHandle/targetHandle data), all edges will use the 'floating' type with bezier paths as implemented in the FloatingEdge component above.

This comprehensive technical implementation guide provides all the details needed to successfully implement floating edges in the Plan DAG editor.
