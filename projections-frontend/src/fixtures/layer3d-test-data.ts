/**
 * Test Data Fixtures for Layer3D Projection
 *
 * Provides various test cases for development and testing:
 * - Minimal: Simple graph for basic functionality
 * - Hierarchical: Parent-child relationships via attrs
 * - Flat: No hierarchy metadata (fallback test)
 * - Large: Performance testing with many nodes
 * - EdgeCases: Cycles, missing layers, invalid data
 */

export interface TestNode {
  id: string
  label: string
  layer: string
  color?: string
  labelColor?: string
  weight?: number
  attrs?: Record<string, any>
}

export interface TestEdge {
  id: string
  source: string
  target: string
  label?: string
  weight?: number
  attrs?: Record<string, any>
}

export interface TestLayer {
  layerId: string
  name: string
  backgroundColor: string
  textColor: string
  borderColor: string
}

export interface TestGraphData {
  nodes: TestNode[]
  edges: TestEdge[]
  layers: TestLayer[]
}

/**
 * Minimal Test Case: 3 nodes, 2 edges, 2 layers
 * Use for: Basic functionality, quick iteration
 */
export const minimalGraph: TestGraphData = {
  nodes: [
    { id: '1', label: 'Node A', layer: 'layer1', color: '#FF6B6B' },
    { id: '2', label: 'Node B', layer: 'layer1', color: '#4ECDC4' },
    { id: '3', label: 'Node C', layer: 'layer2', color: '#45B7D1' },
  ],
  edges: [
    { id: 'e1', source: '1', target: '2', label: 'connects' },
    { id: 'e2', source: '2', target: '3', label: 'flows to' },
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'Application Layer',
      backgroundColor: '#FFD166',
      textColor: '#000000',
      borderColor: '#FF6B6B',
    },
    {
      layerId: 'layer2',
      name: 'Data Layer',
      backgroundColor: '#06FFA5',
      textColor: '#000000',
      borderColor: '#4ECDC4',
    },
  ],
}

/**
 * Hierarchical Test Case: Parent-child relationships via attrs
 * Use for: Testing treemap layout, container nodes
 */
export const hierarchicalGraph: TestGraphData = {
  nodes: [
    // Root container
    { id: 'root', label: 'System', layer: 'layer1', attrs: { parent_id: null }, weight: 100 },

    // Level 1: Subsystems
    { id: 'auth', label: 'Auth Service', layer: 'layer2', attrs: { parent_id: 'root' }, weight: 40 },
    { id: 'api', label: 'API Gateway', layer: 'layer2', attrs: { parent_id: 'root' }, weight: 60 },

    // Level 2: Components
    { id: 'auth-login', label: 'Login', layer: 'layer3', attrs: { parent_id: 'auth' }, weight: 20 },
    { id: 'auth-session', label: 'Session', layer: 'layer3', attrs: { parent_id: 'auth' }, weight: 20 },
    { id: 'api-rest', label: 'REST', layer: 'layer3', attrs: { parent_id: 'api' }, weight: 30 },
    { id: 'api-graphql', label: 'GraphQL', layer: 'layer3', attrs: { parent_id: 'api' }, weight: 30 },
  ],
  edges: [
    { id: 'e1', source: 'api-rest', target: 'auth-login', label: 'authenticates' },
    { id: 'e2', source: 'api-graphql', target: 'auth-session', label: 'validates' },
    { id: 'e3', source: 'auth-session', target: 'auth-login', label: 'creates' },
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'System',
      backgroundColor: '#E3F2FD',
      textColor: '#1565C0',
      borderColor: '#2196F3',
    },
    {
      layerId: 'layer2',
      name: 'Services',
      backgroundColor: '#FFF3E0',
      textColor: '#E65100',
      borderColor: '#FF9800',
    },
    {
      layerId: 'layer3',
      name: 'Components',
      backgroundColor: '#F3E5F5',
      textColor: '#6A1B9A',
      borderColor: '#9C27B0',
    },
  ],
}

/**
 * Edge-based Hierarchy Test Case: Hierarchy inferred from edge semantics
 * Use for: Testing secondary hierarchy detection
 */
export const edgeHierarchyGraph: TestGraphData = {
  nodes: [
    { id: 'container1', label: 'Container A', layer: 'layer1', weight: 50 },
    { id: 'item1', label: 'Item 1', layer: 'layer2', weight: 20 },
    { id: 'item2', label: 'Item 2', layer: 'layer2', weight: 30 },
  ],
  edges: [
    // Semantic edges that indicate containment
    { id: 'e1', source: 'container1', target: 'item1', label: 'contains', attrs: { relation: 'contains' } },
    { id: 'e2', source: 'container1', target: 'item2', label: 'contains', attrs: { relation: 'contains' } },
    { id: 'e3', source: 'item1', target: 'item2', label: 'connects' },
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'Containers',
      backgroundColor: '#E8EAF6',
      textColor: '#1A237E',
      borderColor: '#3F51B5',
    },
    {
      layerId: 'layer2',
      name: 'Items',
      backgroundColor: '#FCE4EC',
      textColor: '#880E4F',
      borderColor: '#E91E63',
    },
  ],
}

/**
 * Flat Test Case: No hierarchy metadata (fallback test)
 * Use for: Testing flat layout fallback, layer-based grouping
 */
export const flatGraph: TestGraphData = {
  nodes: [
    { id: '1', label: 'Service A', layer: 'services', color: '#FF6B6B' },
    { id: '2', label: 'Service B', layer: 'services', color: '#4ECDC4' },
    { id: '3', label: 'Service C', layer: 'services', color: '#45B7D1' },
    { id: '4', label: 'DB Primary', layer: 'data', color: '#96CEB4' },
    { id: '5', label: 'DB Replica', layer: 'data', color: '#FFEAA7' },
  ],
  edges: [
    { id: 'e1', source: '1', target: '4', label: 'reads/writes' },
    { id: 'e2', source: '2', target: '4', label: 'reads/writes' },
    { id: 'e3', source: '3', target: '5', label: 'reads' },
    { id: 'e4', source: '4', target: '5', label: 'replicates', weight: 5 },
  ],
  layers: [
    {
      layerId: 'services',
      name: 'Services',
      backgroundColor: '#FFD166',
      textColor: '#000000',
      borderColor: '#FF6B6B',
    },
    {
      layerId: 'data',
      name: 'Data Layer',
      backgroundColor: '#06FFA5',
      textColor: '#000000',
      borderColor: '#4ECDC4',
    },
  ],
}

/**
 * Large Test Case: 100 nodes, 200 edges, 5 layers
 * Use for: Performance testing, stress testing layout algorithms
 */
export function generateLargeGraph(nodeCount: number = 100, edgeCount: number = 200): TestGraphData {
  const layers: TestLayer[] = [
    { layerId: 'layer1', name: 'Presentation', backgroundColor: '#E3F2FD', textColor: '#1565C0', borderColor: '#2196F3' },
    { layerId: 'layer2', name: 'Application', backgroundColor: '#FFF3E0', textColor: '#E65100', borderColor: '#FF9800' },
    { layerId: 'layer3', name: 'Business Logic', backgroundColor: '#F3E5F5', textColor: '#6A1B9A', borderColor: '#9C27B0' },
    { layerId: 'layer4', name: 'Data Access', backgroundColor: '#E0F2F1', textColor: '#00695C', borderColor: '#009688' },
    { layerId: 'layer5', name: 'Infrastructure', backgroundColor: '#FBE9E7', textColor: '#BF360C', borderColor: '#FF5722' },
  ]

  const nodes: TestNode[] = []
  for (let i = 0; i < nodeCount; i++) {
    const layerIndex = Math.floor(i / (nodeCount / layers.length))
    const layer = layers[Math.min(layerIndex, layers.length - 1)]

    nodes.push({
      id: `node-${i}`,
      label: `Node ${i}`,
      layer: layer.layerId,
      weight: Math.random() * 10 + 1,
      color: layer.backgroundColor,
    })
  }

  const edges: TestEdge[] = []
  for (let i = 0; i < edgeCount; i++) {
    const source = nodes[Math.floor(Math.random() * nodeCount)]
    const target = nodes[Math.floor(Math.random() * nodeCount)]

    if (source.id !== target.id) {
      edges.push({
        id: `edge-${i}`,
        source: source.id,
        target: target.id,
        label: `Edge ${i}`,
        weight: Math.random() * 5 + 1,
      })
    }
  }

  return { nodes, edges, layers }
}

/**
 * Cyclic Test Case: Graph with cycles (should be detected and broken)
 * Use for: Testing cycle detection algorithm
 */
export const cyclicGraph: TestGraphData = {
  nodes: [
    { id: 'a', label: 'Node A', layer: 'layer1', attrs: { parent_id: 'c' } }, // Creates cycle!
    { id: 'b', label: 'Node B', layer: 'layer2', attrs: { parent_id: 'a' } },
    { id: 'c', label: 'Node C', layer: 'layer3', attrs: { parent_id: 'b' } },
  ],
  edges: [
    { id: 'e1', source: 'a', target: 'b', label: 'to B' },
    { id: 'e2', source: 'b', target: 'c', label: 'to C' },
    { id: 'e3', source: 'c', target: 'a', label: 'to A (cycle!)' },
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'Layer 1',
      backgroundColor: '#FFD166',
      textColor: '#000000',
      borderColor: '#FF6B6B',
    },
    {
      layerId: 'layer2',
      name: 'Layer 2',
      backgroundColor: '#06FFA5',
      textColor: '#000000',
      borderColor: '#4ECDC4',
    },
    {
      layerId: 'layer3',
      name: 'Layer 3',
      backgroundColor: '#4ECDC4',
      textColor: '#FFFFFF',
      borderColor: '#45B7D1',
    },
  ],
}

/**
 * Missing Layers Test Case: Nodes reference non-existent layers
 * Use for: Testing layer validation and fallback
 */
export const missingLayersGraph: TestGraphData = {
  nodes: [
    { id: '1', label: 'Node A', layer: 'layer1' },
    { id: '2', label: 'Node B', layer: 'nonexistent-layer' }, // Layer doesn't exist!
    { id: '3', label: 'Node C', layer: '' }, // Empty layer!
  ],
  edges: [
    { id: 'e1', source: '1', target: '2' },
    { id: 'e2', source: '2', target: '3' },
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'Layer 1',
      backgroundColor: '#FFD166',
      textColor: '#000000',
      borderColor: '#FF6B6B',
    },
    // Note: layer2 and layer3 don't exist!
  ],
}

/**
 * Empty Graph Test Case: No nodes or edges
 * Use for: Testing empty state handling
 */
export const emptyGraph: TestGraphData = {
  nodes: [],
  edges: [],
  layers: [
    {
      layerId: 'default',
      name: 'Default Layer',
      backgroundColor: '#F0F0F0',
      textColor: '#000000',
      borderColor: '#CCCCCC',
    },
  ],
}

/**
 * Invalid Data Test Case: Various invalid/edge case data
 * Use for: Testing validation and error handling
 */
export const invalidDataGraph: TestGraphData = {
  nodes: [
    { id: '1', label: 'Valid Node', layer: 'layer1', weight: 5 },
    { id: '2', label: 'Negative Weight', layer: 'layer1', weight: -10 }, // Invalid!
    { id: '3', label: 'NaN Weight', layer: 'layer1', weight: NaN }, // Invalid!
    { id: '4', label: 'Infinity Weight', layer: 'layer1', weight: Infinity }, // Invalid!
    // @ts-expect-error - Testing missing required field
    { id: '5', layer: 'layer1' }, // Missing label!
  ],
  edges: [
    { id: 'e1', source: '1', target: '2', weight: 3 },
    { id: 'e2', source: '2', target: 'nonexistent', weight: 2 }, // Target doesn't exist!
    { id: 'e3', source: '1', target: '1', weight: 1 }, // Self-loop!
  ],
  layers: [
    {
      layerId: 'layer1',
      name: 'Layer 1',
      backgroundColor: '#FFD166',
      textColor: '#000000',
      borderColor: '#FF6B6B',
    },
  ],
}

/**
 * Get test data by name
 */
export function getTestData(name: string): TestGraphData {
  switch (name) {
    case 'minimal':
      return minimalGraph
    case 'hierarchical':
      return hierarchicalGraph
    case 'edge-hierarchy':
      return edgeHierarchyGraph
    case 'flat':
      return flatGraph
    case 'large':
      return generateLargeGraph()
    case 'cyclic':
      return cyclicGraph
    case 'missing-layers':
      return missingLayersGraph
    case 'empty':
      return emptyGraph
    case 'invalid':
      return invalidDataGraph
    default:
      return minimalGraph
  }
}

/**
 * All available test cases
 */
export const ALL_TEST_CASES = [
  { name: 'minimal', label: 'Minimal (3 nodes)', data: minimalGraph },
  { name: 'hierarchical', label: 'Hierarchical (7 nodes)', data: hierarchicalGraph },
  { name: 'edge-hierarchy', label: 'Edge Hierarchy (3 nodes)', data: edgeHierarchyGraph },
  { name: 'flat', label: 'Flat (5 nodes)', data: flatGraph },
  { name: 'large', label: 'Large (100 nodes)', data: generateLargeGraph() },
  { name: 'cyclic', label: 'Cyclic (3 nodes)', data: cyclicGraph },
  { name: 'missing-layers', label: 'Missing Layers (3 nodes)', data: missingLayersGraph },
  { name: 'empty', label: 'Empty Graph', data: emptyGraph },
  { name: 'invalid', label: 'Invalid Data', data: invalidDataGraph },
] as const
