import { Node, Edge } from 'reactflow'
import { PlanDag, PlanDagNode, ReactFlowEdge } from '../types/plan-dag'

/**
 * ReactFlow Adapter Layer - Isolates ReactFlow concerns from business logic
 * Provides pure transformation functions with no side effects
 * Handles stable conversion between Plan DAG and ReactFlow formats
 */
export class ReactFlowAdapter {
  private static readonly CONVERSION_CACHE = new Map<string, any>()

  /**
   * Convert Plan DAG to ReactFlow format
   * Pure transformation with memoization for performance
   */
  static planDagToReactFlow(planDag: PlanDag): { nodes: Node[], edges: Edge[] } {
    // Create a more reliable cache key that includes node positions
    const positionHash = planDag.nodes
      .map(n => `${n.id}:${n.position.x},${n.position.y}`)
      .join('|')
      .substring(0, 50) // Limit length
    const cacheKey = `plandag-${planDag.version}-${planDag.nodes.length}-${planDag.edges.length}-${positionHash}`

    if (this.CONVERSION_CACHE.has(cacheKey)) {
      console.log('[ReactFlowAdapter] Using cached ReactFlow conversion for version', planDag.version)
      return this.CONVERSION_CACHE.get(cacheKey)
    }

    console.log('[ReactFlowAdapter] Converting Plan DAG to ReactFlow format, version:', planDag.version)

    const result = {
      nodes: planDag.nodes.map(node => this.convertPlanDagNodeToReactFlow(node)),
      edges: planDag.edges.map(edge => this.convertPlanDagEdgeToReactFlow(edge))
    }

    // Cache the result with a reasonable limit
    if (this.CONVERSION_CACHE.size > 10) {
      const firstKey = this.CONVERSION_CACHE.keys().next().value
      if (firstKey !== undefined) {
        this.CONVERSION_CACHE.delete(firstKey)
      }
    }
    this.CONVERSION_CACHE.set(cacheKey, result)

    return result
  }

  /**
   * Convert ReactFlow format back to Plan DAG
   * Pure reverse transformation
   */
  static reactFlowToPlanDag(
    nodes: Node[],
    edges: Edge[],
    metadata?: any
  ): PlanDag {
    console.log('[ReactFlowAdapter] Converting ReactFlow to Plan DAG format')

    return {
      version: metadata?.version || Date.now().toString(),
      nodes: nodes.map(node => this.convertReactFlowNodeToPlanDag(node)),
      edges: edges.map(edge => this.convertReactFlowEdgeToPlanDag(edge)),
      metadata: {
        version: metadata?.version || Date.now().toString(),
        name: metadata?.name || 'Untitled Plan',
        description: metadata?.description || '',
        created: metadata?.created || new Date().toISOString(),
        lastModified: new Date().toISOString(),
        author: metadata?.author || 'Unknown'
      }
    }
  }

  /**
   * Convert individual Plan DAG node to ReactFlow node
   * Stable conversion with consistent positioning
   */
  private static convertPlanDagNodeToReactFlow(node: PlanDagNode): Node {
    // Normalize field names: GraphQL may return snake_case from backend
    const normalizedNode = {
      ...node,
      nodeType: node.nodeType || (node as any).node_type
    }

    // Debug undefined nodeType
    if (!normalizedNode.nodeType) {
      console.error('[ReactFlowAdapter] Node has undefined nodeType:', JSON.stringify(node, null, 2))
    }

    // Parse config if it's a string
    const parsedConfig = typeof normalizedNode.config === 'string' ? (() => {
      try {
        return JSON.parse(normalizedNode.config)
      } catch (e) {
        return {}
      }
    })() : (normalizedNode.config || {})

    // Check if config is valid
    const hasValidConfig = normalizedNode.config &&
      (typeof normalizedNode.config === 'object' ||
       (typeof normalizedNode.config === 'string' && normalizedNode.config.trim() !== '{}' && normalizedNode.config.trim() !== ''))

    return {
      id: normalizedNode.id,
      type: this.mapNodeTypeToReactFlow(normalizedNode.nodeType),
      position: {
        x: normalizedNode.position?.x ?? 0,
        y: normalizedNode.position?.y ?? 0
      },
      data: {
        // ReactFlow-specific data
        label: normalizedNode.metadata?.label || normalizedNode.id,
        description: normalizedNode.metadata?.description || '',
        nodeType: normalizedNode.nodeType,
        config: parsedConfig,
        metadata: normalizedNode.metadata,
        hasValidConfig,

        // Original Plan DAG data for round-trip consistency
        originalNode: {
          id: normalizedNode.id,
          nodeType: normalizedNode.nodeType,
          metadata: normalizedNode.metadata,
          config: normalizedNode.config
        }
      },
      // ReactFlow styling
      style: this.getNodeStyle(normalizedNode.nodeType),
      draggable: true,
      selectable: true,
      deletable: true
    }
  }

  /**
   * Convert ReactFlow node back to Plan DAG node
   * Preserves original data when available
   */
  private static convertReactFlowNodeToPlanDag(node: Node): PlanDagNode {
    const originalNode = node.data?.originalNode

    return {
      id: node.id,
      nodeType: originalNode?.nodeType || this.mapReactFlowTypeToNodeType(node.type),
      position: {
        x: Math.round(node.position.x),
        y: Math.round(node.position.y)
      },
      metadata: originalNode?.metadata || {
        label: node.data?.label || node.id,
        description: node.data?.description || ''
      },
      config: originalNode?.config || {}
    }
  }

  /**
   * Convert Plan DAG edge to ReactFlow edge
   */
  private static convertPlanDagEdgeToReactFlow(edge: ReactFlowEdge): Edge {
    // FIX: Ensure metadata exists with defaults to prevent invisible edges
    const metadata = edge.metadata || { label: 'Data', dataType: 'GRAPH_DATA' }

    return {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: edge.sourceHandle || null,
      targetHandle: edge.targetHandle || null,
      type: 'smoothstep',
      animated: false,
      label: metadata.label || 'Data',
      style: {
        stroke: metadata.dataType === 'GRAPH_REFERENCE' ? '#228be6' : '#868e96',
        strokeWidth: 2,
      },
      labelStyle: {
        fontSize: 12,
        fontWeight: 500,
      },
      data: {
        // Original edge data for round-trip consistency
        originalEdge: edge,
        metadata: metadata
      }
    }
  }

  /**
   * Convert ReactFlow edge back to Plan DAG edge
   */
  private static convertReactFlowEdgeToPlanDag(edge: Edge): ReactFlowEdge {
    const originalEdge = edge.data?.originalEdge

    return {
      id: edge.id,
      source: edge.source,
      target: edge.target,
      sourceHandle: edge.sourceHandle,
      targetHandle: edge.targetHandle,
      metadata: originalEdge?.metadata || {
        label: edge.label as string || '',
        dataType: 'unknown'
      }
    }
  }

  /**
   * Map Plan DAG node types to ReactFlow node types
   */
  private static mapNodeTypeToReactFlow(nodeType: string): string {
    const typeMap: Record<string, string> = {
      // Database format (snake_case)
      'data_source': 'DataSourceNode',
      'transform': 'TransformNode',
      'merge': 'MergeNode',
      'output': 'OutputNode',
      'copy': 'CopyNode',
      'graph': 'GraphNode',
      // Backend may return capitalized variants
      'DataSource': 'DataSourceNode',
      'Transform': 'TransformNode',
      'Merge': 'MergeNode',
      'Output': 'OutputNode',
      'Copy': 'CopyNode',
      'Graph': 'GraphNode',
      // TypeScript enum format (PascalCase) - pass through
      'DataSourceNode': 'DataSourceNode',
      'TransformNode': 'TransformNode',
      'MergeNode': 'MergeNode',
      'OutputNode': 'OutputNode',
      'CopyNode': 'CopyNode',
      'GraphNode': 'GraphNode'
    }

    const mapped = typeMap[nodeType]
    if (!mapped) {
      console.error(`[ReactFlowAdapter] Unknown node type: ${nodeType}, falling back to DataSourceNode`)
      return 'DataSourceNode' // Better than 'default' which won't match NODE_TYPES
    }
    return mapped
  }

  /**
   * Map ReactFlow node types back to Plan DAG node types
   */
  private static mapReactFlowTypeToNodeType(reactFlowType: string | undefined): string {
    const typeMap: Record<string, string> = {
      'DataSourceNode': 'data_source',
      'TransformNode': 'transform',
      'MergeNode': 'merge',
      'OutputNode': 'output',
      'CopyNode': 'copy',
      'GraphNode': 'graph'
    }

    return typeMap[reactFlowType || 'default'] || 'unknown'
  }

  /**
   * Get ReactFlow node styling based on node type
   */
  private static getNodeStyle(nodeType: string): Record<string, any> {
    const baseStyle = {
      padding: '10px',
      borderRadius: '8px',
      border: '2px solid',
      background: 'white',
      fontSize: '12px',
      minWidth: '120px',
      minHeight: '60px'
    }

    const typeStyles: Record<string, any> = {
      'data_source': {
        ...baseStyle,
        borderColor: '#3b82f6',
        background: '#eff6ff'
      },
      'transform': {
        ...baseStyle,
        borderColor: '#10b981',
        background: '#f0fdf4'
      },
      'merge': {
        ...baseStyle,
        borderColor: '#f59e0b',
        background: '#fffbeb'
      },
      'output': {
        ...baseStyle,
        borderColor: '#ef4444',
        background: '#fef2f2'
      },
      'copy': {
        ...baseStyle,
        borderColor: '#8b5cf6',
        background: '#f5f3ff'
      },
      'graph': {
        ...baseStyle,
        borderColor: '#06b6d4',
        background: '#f0fdfa'
      }
    }

    return typeStyles[nodeType] || baseStyle
  }


  /**
   * Validate ReactFlow data integrity
   */
  static validateReactFlowData(nodes: Node[], edges: Edge[]): ValidationResult {
    const errors: string[] = []
    const warnings: string[] = []

    // Check for orphaned edges
    const nodeIds = new Set(nodes.map(n => n.id))
    edges.forEach(edge => {
      if (!nodeIds.has(edge.source)) {
        errors.push(`Edge ${edge.id} has invalid source: ${edge.source}`)
      }
      if (!nodeIds.has(edge.target)) {
        errors.push(`Edge ${edge.id} has invalid target: ${edge.target}`)
      }
    })

    // Check for duplicate IDs
    const seenNodeIds = new Set<string>()
    nodes.forEach(node => {
      if (seenNodeIds.has(node.id)) {
        errors.push(`Duplicate node ID: ${node.id}`)
      }
      seenNodeIds.add(node.id)
    })

    // Check for nodes without positions
    nodes.forEach(node => {
      if (node.position.x === undefined || node.position.y === undefined) {
        warnings.push(`Node ${node.id} has undefined position`)
      }
    })

    return {
      isValid: errors.length === 0,
      errors,
      warnings
    }
  }

  /**
   * Clear conversion cache (useful for testing or memory management)
   */
  static clearCache(): void {
    this.CONVERSION_CACHE.clear()
  }
}

interface ValidationResult {
  isValid: boolean
  errors: string[]
  warnings: string[]
}