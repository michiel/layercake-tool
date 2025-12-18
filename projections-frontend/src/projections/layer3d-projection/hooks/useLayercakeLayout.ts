/**
 * Layercake Layout Hook
 *
 * Phase 1: Simple grid layout on XZ plane
 * Phase 2: Will upgrade to D3 treemap with hierarchy
 *
 * Calculates 3D positions for nodes in Layer3D projection:
 * - X/Z: Grid layout based on sqrt(nodeCount) per layer
 * - Y: Layer stratification (layerIndex * layerSpacing)
 */

import { useMemo } from 'react'
import {
  validateGraphData,
  breakCycles,
  logValidationResults,
  type GraphNode,
  type GraphEdge,
  type GraphLayer,
} from '../lib/layer3d-validation'

export interface LayoutConfiguration {
  layerSpacing: number // Vertical distance between layers (default: 10)
  nodeSize: number // Base size for nodes (default: 2)
  gridSpacing: number // Spacing between nodes in grid (default: 3)
}

export interface PositionedNode {
  id: string
  label: string
  x: number
  y: number
  z: number
  width: number
  height: number
  depth: number
  color: string
  labelColor: string
  layerId: string
  isPartition: boolean // Will be true for container nodes in Phase 2
  weight: number
}

export interface BoundingBox {
  minX: number
  maxX: number
  minY: number
  maxY: number
  minZ: number
  maxZ: number
  centerX: number
  centerY: number
  centerZ: number
  sizeX: number
  sizeY: number
  sizeZ: number
}

export interface LayoutResult {
  nodes: PositionedNode[]
  boundingBox: BoundingBox
  warnings: string[]
  errors: string[]
}

const DEFAULT_CONFIG: LayoutConfiguration = {
  layerSpacing: 10,
  nodeSize: 2,
  gridSpacing: 3,
}

/**
 * Calculate layout for Layer3D projection
 *
 * Phase 1: Simple grid layout per layer
 */
export function useLayercakeLayout(
  nodes: GraphNode[],
  edges: GraphEdge[],
  layers: GraphLayer[],
  config: Partial<LayoutConfiguration> = {}
): LayoutResult {
  const fullConfig = { ...DEFAULT_CONFIG, ...config }

  return useMemo(() => {
    // 1. Validate and sanitize input data
    const validationResult = validateGraphData(nodes, edges, layers)
    logValidationResults(validationResult)

    if (!validationResult.valid) {
      return {
        nodes: [],
        boundingBox: createEmptyBoundingBox(),
        warnings: validationResult.warnings.map((w) => w.message),
        errors: validationResult.errors.map((e) => e.message),
      }
    }

    let { nodes: validatedNodes, edges: validatedEdges, layers: validatedLayers } = validationResult.data

    // 2. Break cycles if detected
    if (validationResult.warnings.some((w) => w.type === 'hierarchy_cycle')) {
      const cycleBreakResult = breakCycles(validatedNodes, validatedEdges)
      validatedNodes = cycleBreakResult.nodes
      validatedEdges = cycleBreakResult.edges
    }

    // 3. Calculate positions using Phase 1 grid layout
    const positionedNodes = calculateGridLayout(validatedNodes, validatedLayers, fullConfig)

    // 4. Calculate bounding box for camera positioning
    const boundingBox = calculateBoundingBox(positionedNodes)

    return {
      nodes: positionedNodes,
      boundingBox,
      warnings: validationResult.warnings.map((w) => w.message),
      errors: [],
    }
  }, [nodes, edges, layers, fullConfig.layerSpacing, fullConfig.nodeSize, fullConfig.gridSpacing])
}

/**
 * Phase 1: Simple grid layout on XZ plane
 *
 * Each layer gets its own grid. Grid size determined by sqrt(nodeCount).
 * Nodes are evenly spaced in rows and columns.
 */
function calculateGridLayout(
  nodes: GraphNode[],
  layers: GraphLayer[],
  config: LayoutConfiguration
): PositionedNode[] {
  const { layerSpacing, nodeSize, gridSpacing } = config

  // Group nodes by layer
  const nodesByLayer = new Map<string, GraphNode[]>()
  nodes.forEach((node) => {
    const layerNodes = nodesByLayer.get(node.layer!) || []
    layerNodes.push(node)
    nodesByLayer.set(node.layer!, layerNodes)
  })

  // Get layer metadata for colors
  const layerMetadata = new Map(layers.map((layer) => [layer.layerId, layer]))

  const positionedNodes: PositionedNode[] = []

  // Position each layer
  layers.forEach((layer, layerIndex) => {
    const layerNodes = nodesByLayer.get(layer.layerId) || []
    if (layerNodes.length === 0) return

    // Calculate grid dimensions
    const gridSize = Math.ceil(Math.sqrt(layerNodes.length))
    const cellSize = nodeSize + gridSpacing

    // Calculate offset to center the grid around origin
    const gridWidth = gridSize * cellSize
    const gridDepth = gridSize * cellSize
    const offsetX = -gridWidth / 2 + cellSize / 2
    const offsetZ = -gridDepth / 2 + cellSize / 2

    // Y position for this layer
    const y = layerIndex * layerSpacing

    // Position nodes in grid
    layerNodes.forEach((node, index) => {
      const row = Math.floor(index / gridSize)
      const col = index % gridSize

      const x = offsetX + col * cellSize
      const z = offsetZ + row * cellSize

      const layerMeta = layerMetadata.get(layer.layerId)
      const color = node.color || layerMeta?.backgroundColor || '#CCCCCC'
      const labelColor = node.labelColor || layerMeta?.textColor || '#000000'

      positionedNodes.push({
        id: node.id,
        label: node.label || node.id,
        x,
        y,
        z,
        width: nodeSize,
        height: nodeSize,
        depth: nodeSize,
        color,
        labelColor,
        layerId: node.layer!,
        isPartition: false, // Phase 1: no hierarchy, all leaf nodes
        weight: node.weight || 1,
      })
    })
  })

  return positionedNodes
}

/**
 * Calculate bounding box for camera positioning
 */
function calculateBoundingBox(nodes: PositionedNode[]): BoundingBox {
  if (nodes.length === 0) {
    return createEmptyBoundingBox()
  }

  let minX = Infinity
  let maxX = -Infinity
  let minY = Infinity
  let maxY = -Infinity
  let minZ = Infinity
  let maxZ = -Infinity

  nodes.forEach((node) => {
    const halfWidth = node.width / 2
    const halfHeight = node.height / 2
    const halfDepth = node.depth / 2

    minX = Math.min(minX, node.x - halfWidth)
    maxX = Math.max(maxX, node.x + halfWidth)
    minY = Math.min(minY, node.y - halfHeight)
    maxY = Math.max(maxY, node.y + halfHeight)
    minZ = Math.min(minZ, node.z - halfDepth)
    maxZ = Math.max(maxZ, node.z + halfDepth)
  })

  const centerX = (minX + maxX) / 2
  const centerY = (minY + maxY) / 2
  const centerZ = (minZ + maxZ) / 2

  const sizeX = maxX - minX
  const sizeY = maxY - minY
  const sizeZ = maxZ - minZ

  return {
    minX,
    maxX,
    minY,
    maxY,
    minZ,
    maxZ,
    centerX,
    centerY,
    centerZ,
    sizeX,
    sizeY,
    sizeZ,
  }
}

/**
 * Create empty bounding box (fallback for error cases)
 */
function createEmptyBoundingBox(): BoundingBox {
  return {
    minX: 0,
    maxX: 0,
    minY: 0,
    maxY: 0,
    minZ: 0,
    maxZ: 0,
    centerX: 0,
    centerY: 0,
    centerZ: 0,
    sizeX: 0,
    sizeY: 0,
    sizeZ: 0,
  }
}
