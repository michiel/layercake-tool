/**
 * Layer3D Data Validation and Sanitization
 *
 * Validates ProjectionGraph data before layout calculation and provides
 * sensible fallbacks for invalid/missing data.
 *
 * Handles:
 * - Cycle detection in hierarchy
 * - Missing/invalid layers
 * - Invalid node properties (NaN, Infinity, negative weights)
 * - Orphaned edges (source/target doesn't exist)
 */

export interface ValidationResult<T> {
  valid: boolean
  data: T
  warnings: ValidationWarning[]
  errors: ValidationError[]
}

export interface ValidationWarning {
  type: string
  message: string
  context?: any
}

export interface ValidationError {
  type: string
  message: string
  context?: any
}

export interface GraphNode {
  id: string
  label?: string
  layer?: string
  weight?: number
  attrs?: Record<string, any>
  [key: string]: any
}

export interface GraphEdge {
  id: string
  source: string
  target: string
  weight?: number
  attrs?: Record<string, any>
  [key: string]: any
}

export interface GraphLayer {
  layerId: string
  name: string
  backgroundColor?: string
  textColor?: string
  [key: string]: any
}

export interface ValidatedGraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
  layers: GraphLayer[]
}

/**
 * Validate and sanitize ProjectionGraph data
 */
export function validateGraphData(
  nodes: GraphNode[],
  edges: GraphEdge[],
  layers: GraphLayer[]
): ValidationResult<ValidatedGraphData> {
  const warnings: ValidationWarning[] = []
  const errors: ValidationError[] = []

  // 1. Validate and sanitize layers
  let validatedLayers = validateLayers(layers, warnings, errors)

  // 2. Validate and sanitize nodes
  const validatedNodes = validateNodes(nodes, validatedLayers, warnings, errors)

  // 3. Validate and sanitize edges
  const validatedEdges = validateEdges(edges, validatedNodes, warnings, errors)

  // 4. Detect cycles in hierarchy
  detectCycles(validatedNodes, validatedEdges, warnings)

  return {
    valid: errors.length === 0,
    data: {
      nodes: validatedNodes,
      edges: validatedEdges,
      layers: validatedLayers,
    },
    warnings,
    errors,
  }
}

/**
 * Validate layers array
 */
function validateLayers(
  layers: GraphLayer[],
  warnings: ValidationWarning[],
  _errors: ValidationError[]
): GraphLayer[] {
  // If no layers provided, create default layer
  if (!layers || layers.length === 0) {
    warnings.push({
      type: 'missing_layers',
      message: 'No layers provided, creating default "Unnamed Layer 0"',
    })

    return [
      {
        layerId: 'default',
        name: 'Unnamed Layer 0',
        backgroundColor: '#F0F0F0',
        textColor: '#000000',
      },
    ]
  }

  // Validate each layer
  const validated = layers.map((layer, index) => {
    const validatedLayer = { ...layer }

    // Ensure layerId exists
    if (!layer.layerId) {
      warnings.push({
        type: 'missing_layer_id',
        message: `Layer at index ${index} missing layerId, using "layer-${index}"`,
        context: { layer },
      })
      validatedLayer.layerId = `layer-${index}`
    }

    // Ensure name exists
    if (!layer.name) {
      validatedLayer.name = `Layer ${index}`
    }

    // Ensure colors exist
    if (!layer.backgroundColor) {
      validatedLayer.backgroundColor = '#F0F0F0'
    }
    if (!layer.textColor) {
      validatedLayer.textColor = '#000000'
    }

    return validatedLayer
  })

  return validated
}

/**
 * Validate nodes array
 */
function validateNodes(
  nodes: GraphNode[],
  layers: GraphLayer[],
  warnings: ValidationWarning[],
  errors: ValidationError[]
): GraphNode[] {
  if (!nodes || nodes.length === 0) {
    errors.push({
      type: 'empty_graph',
      message: 'Graph has no nodes',
    })
    return []
  }

  const layerIds = new Set(layers.map((l) => l.layerId))
  const firstLayerId = layers[0]?.layerId || 'default'

  return nodes.map((node) => {
    const validated = { ...node }
    const hasWeight = node.weight !== undefined && node.weight !== null
    ;(validated as any).__hasWeight = hasWeight

    // Validate node ID
    if (!node.id) {
      errors.push({
        type: 'missing_node_id',
        message: 'Node missing required id field',
        context: { node },
      })
      validated.id = `node-${Math.random().toString(36).substr(2, 9)}`
    }

    // Validate label
    if (!node.label) {
      validated.label = node.id
    }

    // Validate layer assignment
    if (!node.layer || !layerIds.has(node.layer)) {
      if (node.layer) {
        warnings.push({
          type: 'invalid_layer',
          message: `Node ${node.id} references non-existent layer "${node.layer}", assigning to first layer`,
          context: { nodeId: node.id, layer: node.layer },
        })
      }
      validated.layer = firstLayerId
    }

    // Validate weight
    if (hasWeight) {
      const weightVal = Number(node.weight)
      if (!Number.isFinite(weightVal) || weightVal <= 0) {
        warnings.push({
          type: 'invalid_weight',
          message: `Node ${node.id} has invalid weight ${node.weight}, using 1`,
          context: { nodeId: node.id, weight: node.weight },
        })
        validated.weight = 1
      } else {
        validated.weight = weightVal
      }
    } else {
      // Default weight if not specified
      validated.weight = 1
    }

    return validated
  })
}

/**
 * Validate edges array
 */
function validateEdges(
  edges: GraphEdge[],
  nodes: GraphNode[],
  warnings: ValidationWarning[],
  _errors: ValidationError[]
): GraphEdge[] {
  if (!edges) {
    return []
  }

  const nodeIds = new Set(nodes.map((n) => n.id))

  return edges.filter((edge) => {
    // Validate edge ID
    if (!edge.id) {
      warnings.push({
        type: 'missing_edge_id',
        message: 'Edge missing id, generating random id',
        context: { edge },
      })
      edge.id = `edge-${Math.random().toString(36).substr(2, 9)}`
    }

    // Validate source exists
    if (!nodeIds.has(edge.source)) {
      warnings.push({
        type: 'orphaned_edge',
        message: `Edge ${edge.id} references non-existent source node "${edge.source}", removing edge`,
        context: { edgeId: edge.id, source: edge.source },
      })
      return false
    }

    // Validate target exists
    if (!nodeIds.has(edge.target)) {
      warnings.push({
        type: 'orphaned_edge',
        message: `Edge ${edge.id} references non-existent target node "${edge.target}", removing edge`,
        context: { edgeId: edge.id, target: edge.target },
      })
      return false
    }

    // Validate weight
    if (edge.weight !== undefined) {
      if (!Number.isFinite(edge.weight) || edge.weight <= 0) {
        warnings.push({
          type: 'invalid_weight',
          message: `Edge ${edge.id} has invalid weight ${edge.weight}, using 1`,
          context: { edgeId: edge.id, weight: edge.weight },
        })
        edge.weight = 1
      }
    }

    return true
  })
}

/**
 * Detect cycles in hierarchy using DFS
 */
function detectCycles(
  nodes: GraphNode[],
  edges: GraphEdge[],
  warnings: ValidationWarning[]
): void {
  // Build parent-child relationships
  const parentMap = new Map<string, string | null>()

  // From node attributes
  nodes.forEach((node) => {
    if (node.attrs?.parent_id) {
      parentMap.set(node.id, node.attrs.parent_id)
    } else if (node.attrs?.belongs_to) {
      parentMap.set(node.id, node.attrs.belongs_to)
    }
  })

  // From edges with semantic relations
  edges.forEach((edge) => {
    const relation = edge.attrs?.relation
    if (relation && ['contains', 'parent_of', 'has', 'includes'].includes(relation)) {
      // In containment edges, source contains target, so target's parent is source
      parentMap.set(edge.target, edge.source)
    }
  })

  // Detect cycles using DFS
  const visited = new Set<string>()
  const recursionStack = new Set<string>()
  const cycleNodes: string[] = []

  function dfs(nodeId: string, path: string[]): boolean {
    if (recursionStack.has(nodeId)) {
      // Cycle detected
      const cycleStart = path.indexOf(nodeId)
      const cycle = path.slice(cycleStart)
      cycleNodes.push(...cycle, nodeId)
      return true
    }

    if (visited.has(nodeId)) {
      return false
    }

    visited.add(nodeId)
    recursionStack.add(nodeId)
    path.push(nodeId)

    const parentId = parentMap.get(nodeId)
    if (parentId) {
      if (dfs(parentId, path)) {
        return true
      }
    }

    recursionStack.delete(nodeId)
    path.pop()
    return false
  }

  // Check all nodes for cycles
  nodes.forEach((node) => {
    if (!visited.has(node.id)) {
      dfs(node.id, [])
    }
  })

  // Report cycles
  if (cycleNodes.length > 0) {
    const cycle = [...new Set(cycleNodes)].join(' → ')
    warnings.push({
      type: 'hierarchy_cycle',
      message: `Detected cycle in hierarchy: ${cycle}. Layout will break cycle at arbitrary edge.`,
      context: { cycleNodes },
    })
  }
}

/**
 * Break cycles in hierarchy by removing edges
 */
export function breakCycles(
  nodes: GraphNode[],
  edges: GraphEdge[]
): { nodes: GraphNode[]; edges: GraphEdge[] } {
  // Build adjacency list from parent relationships
  const parentMap = new Map<string, string>()

  nodes.forEach((node) => {
    if (node.attrs?.parent_id) {
      parentMap.set(node.id, node.attrs.parent_id)
    } else if (node.attrs?.belongs_to) {
      parentMap.set(node.id, node.attrs.belongs_to)
    }
  })

  // Find cycles using DFS
  const visited = new Set<string>()
  const recursionStack = new Set<string>()
  const cyclicEdges = new Set<string>()

  function dfs(nodeId: string): boolean {
    if (recursionStack.has(nodeId)) {
      return true // Cycle detected
    }

    if (visited.has(nodeId)) {
      return false
    }

    visited.add(nodeId)
    recursionStack.add(nodeId)

    const parentId = parentMap.get(nodeId)
    if (parentId) {
      if (dfs(parentId)) {
        // This edge is part of a cycle, mark it for removal
        cyclicEdges.add(`${nodeId}-${parentId}`)
        return true
      }
    }

    recursionStack.delete(nodeId)
    return false
  }

  // Detect cycles
  nodes.forEach((node) => {
    if (!visited.has(node.id)) {
      dfs(node.id)
    }
  })

  // Remove parent_id from nodes involved in cycles
  const cleanedNodes = nodes.map((node) => {
    const parentId = node.attrs?.parent_id || node.attrs?.belongs_to
    if (parentId && cyclicEdges.has(`${node.id}-${parentId}`)) {
      const cleaned = { ...node, attrs: { ...node.attrs } }
      delete cleaned.attrs.parent_id
      delete cleaned.attrs.belongs_to
      return cleaned
    }
    return node
  })

  return { nodes: cleanedNodes, edges }
}

/**
 * Log validation results to console
 */
export function logValidationResults(result: ValidationResult<any>): void {
  if (result.errors.length > 0) {
    console.error('[Layer3D Validation] Errors:', result.errors)
  }

  if (result.warnings.length > 0) {
    console.warn('[Layer3D Validation] Warnings:', result.warnings)
  }

  if (result.valid && result.warnings.length === 0) {
    console.log('[Layer3D Validation] Data validated successfully, no issues found')
  }
}
