/**
 * Layercake Layout Hook
 *
 * Phase 2: D3 treemap layout with hierarchy detection
 *
 * Calculates 3D positions for nodes in Layer3D projection:
 * - X/Z: Treemap layout with hierarchy or layer-based grouping
 * - Y: Layer stratification (layerIndex * layerSpacing)
 */

import { useMemo } from 'react'
import { hierarchy, treemap } from 'd3-hierarchy'
import {
  validateGraphData,
  breakCycles,
  logValidationResults,
  type GraphNode,
  type GraphEdge,
  type GraphLayer,
} from '../lib/layer3d-validation'

export interface LayoutConfiguration {
  canvasSize: number // Size of the treemap canvas (default: 100)
  layerSpacing: number // Vertical distance between layers (default: 10)
  partitionPadding: number // Padding between treemap partitions (default: 2)
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
  isPartition: boolean // True for container nodes with children
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
  canvasSize: 200, // Larger canvas for better spread
  layerSpacing: 20, // More vertical separation between layers
  partitionPadding: 3,
}

/**
 * Calculate layout for Layer3D projection
 *
 * Phase 2: D3 treemap layout with hierarchy detection
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

    // 3. Build hierarchy and get parent relationships
    const { hierarchyData, parentMap } = buildHierarchy(validatedNodes, validatedEdges, validatedLayers)

    // 4. Calculate positions using D3 treemap
    const positionedNodes = calculateTreemapLayout(hierarchyData, validatedLayers, fullConfig, parentMap, validatedNodes)

    // 5. Calculate bounding box for camera positioning
    const boundingBox = calculateBoundingBox(positionedNodes)

    return {
      nodes: positionedNodes,
      boundingBox,
      warnings: validationResult.warnings.map((w) => w.message),
      errors: [],
    }
  }, [
    nodes,
    edges,
    layers,
    fullConfig.canvasSize,
    fullConfig.layerSpacing,
    fullConfig.partitionPadding,
  ])
}

/**
 * Build hierarchy from nodes and edges
 *
 * Strategy:
 * 1. Check for attrs.parent_id or attrs.belongs_to
 * 2. Check for edges with semantic relations (contains, parent_of, has, includes)
 * 3. Fallback: Group by layer with virtual root per layer
 */
interface HierarchyData {
  id: string
  label: string
  layer: string
  weight: number
  color: string
  labelColor: string
  children?: HierarchyData[]
  isVirtual?: boolean // True for synthetic parent nodes
}

function buildHierarchy(
  nodes: GraphNode[],
  edges: GraphEdge[],
  layers: GraphLayer[]
): { hierarchyData: HierarchyData; parentMap: Map<string, string> } {
  // Build parent map from belongs_to attribute
  const parentMap = new Map<string, string>()

  // Log sample of attributes to debug
  console.log('[Hierarchy] Sample node attrs:', nodes.slice(0, 3).map(n => ({ id: n.id, attrs: n.attrs })))

  nodes.forEach((node) => {
    if (node.attrs?.belongs_to) {
      parentMap.set(node.id, node.attrs.belongs_to)
      console.log(`[Hierarchy] Node "${node.id}" belongs_to "${node.attrs.belongs_to}"`)
    }
  })

  // Augment with edge-based hierarchy (fallback)
  edges.forEach((edge) => {
    const relation = edge.attrs?.relation
    if (relation && ['contains', 'parent_of', 'has', 'includes'].includes(relation)) {
      // Only set if not already defined (attrs take precedence)
      if (!parentMap.has(edge.target)) {
        parentMap.set(edge.target, edge.source)
      }
    }
  })

  console.log('[Hierarchy] Built parentMap, size:', parentMap.size)
  console.log('[Hierarchy] Parent relationships:', Array.from(parentMap.entries()).slice(0, 5))

  // Check if we have any hierarchy
  const hasHierarchy = parentMap.size > 0

  if (hasHierarchy) {
    // Build tree structure
    const nodeMap = new Map(
      nodes.map((n) => [
        n.id,
        {
          id: n.id,
          label: n.label || n.id,
          layer: n.layer!,
          weight: n.weight || 1,
          color: n.color || '#CCCCCC',
          labelColor: n.labelColor || '#000000',
          children: [],
        } as HierarchyData,
      ])
    )

    // Find root nodes (no parent)
    const rootNodes: HierarchyData[] = []
    nodes.forEach((node) => {
      const parent = parentMap.get(node.id)
      const nodeData = nodeMap.get(node.id)!

      if (!parent || !nodeMap.has(parent)) {
        // Root node
        rootNodes.push(nodeData)
      } else {
        // Add to parent's children
        const parentData = nodeMap.get(parent)!
        if (!parentData.children) {
          parentData.children = []
        }
        parentData.children.push(nodeData)
      }
    })

    // Create virtual root if multiple roots
    const hierarchyData =
      rootNodes.length === 1
        ? rootNodes[0]
        : {
            id: '__virtual_root__',
            label: 'Root',
            layer: layers[0]?.layerId || 'default',
            weight: 0,
            color: '#000000',
            labelColor: '#FFFFFF',
            children: rootNodes,
            isVirtual: true,
          }

    return { hierarchyData, parentMap }
  } else {
    // Fallback: Group by layer
    const layerGroups = new Map<string, HierarchyData[]>()

    nodes.forEach((node) => {
      const layerData: HierarchyData = {
        id: node.id,
        label: node.label || node.id,
        layer: node.layer!,
        weight: node.weight || 1,
        color: node.color || '#CCCCCC',
        labelColor: node.labelColor || '#000000',
      }

      if (!layerGroups.has(node.layer!)) {
        layerGroups.set(node.layer!, [])
      }
      layerGroups.get(node.layer!)!.push(layerData)
    })

    // Create virtual layer parents
    const layerParents: HierarchyData[] = layers.map((layer) => ({
      id: `__layer_${layer.layerId}__`,
      label: layer.name,
      layer: layer.layerId,
      weight: 0,
      color: layer.backgroundColor || '#CCCCCC',
      labelColor: layer.textColor || '#000000',
      children: layerGroups.get(layer.layerId) || [],
      isVirtual: true,
    }))

    const hierarchyData = {
      id: '__virtual_root__',
      label: 'Root',
      layer: layers[0]?.layerId || 'default',
      weight: 0,
      color: '#000000',
      labelColor: '#FFFFFF',
      children: layerParents,
      isVirtual: true,
    }

    return { hierarchyData, parentMap }
  }
}

/**
 * Calculate treemap layout and map to 3D coordinates
 */
function calculateTreemapLayout(
  hierarchyData: HierarchyData,
  layers: GraphLayer[],
  config: LayoutConfiguration,
  parentMap: Map<string, string>,
  nodes: GraphNode[]
): PositionedNode[] {
  const { canvasSize, layerSpacing, partitionPadding } = config

  // Create D3 hierarchy
  const root = hierarchy(hierarchyData)
    .sum((d: any) => {
      // Leaf nodes contribute their weight
      // Parent nodes DON'T add their own weight - treemap sums children automatically
      if (d.children && d.children.length > 0) {
        return 0 // Treemap will sum children's values
      }
      return d.weight || 10 // Give each leaf node more weight for visibility
    })
    .sort((a: any, b: any) => (b.value || 0) - (a.value || 0))

  // Apply treemap layout
  const layoutFn = (treemap as any)()
    .size([canvasSize, canvasSize])
    .paddingOuter(1) // Minimal outer padding
    .paddingInner(partitionPadding * 2) // More padding between partitions for clear separation
    .paddingTop(20) // Extra top padding for labels

  // IMPORTANT: Mark partitions BEFORE layout, because treemap flattens the tree
  // A node is a partition if:
  // 1. It has is_partition=true in attributes, OR
  // 2. It appears as a parent in the parentMap (has children)
  const partitionIds = new Set<string>()

  // Add nodes marked as partitions
  nodes.forEach((node) => {
    if (node.attrs?.is_partition === true || node.attrs?.is_partition === 'true') {
      partitionIds.add(node.id)
      console.log(`[Layout] Node "${node.id}" marked as partition from is_partition attribute`)
    }
  })

  // Add parent nodes
  parentMap.forEach((parentId) => {
    partitionIds.add(parentId)
  })

  console.log('[Layout] Partition IDs:', Array.from(partitionIds))
  console.log('[Layout] Sample parent relationships:', Array.from(parentMap.entries()).slice(0, 5))

  layoutFn(root)

  // Create layer index map
  const layerMap = new Map(layers.map((l, i) => [l.layerId, i]))

  // First pass: Calculate layer ranges for each partition
  // Partitions start at their own layer and extend down to deepest child
  const partitionLayerRanges = new Map<string, { minLayer: number; maxLayer: number; ownLayer: number }>()

  root.each((node: any) => {
    const data = node.data
    if (node.children && node.children.length > 0) {
      const ownLayerIdx = layerMap.get(data.layer) || 0
      let minLayer = ownLayerIdx // Start from partition's own layer
      let maxLayer = ownLayerIdx

      // Find the deepest descendant layer
      node.each((descendant: any) => {
        if (!descendant.data.isVirtual) {
          const layerIdx = layerMap.get(descendant.data.layer) || 0
          minLayer = Math.min(minLayer, layerIdx)
          maxLayer = Math.max(maxLayer, layerIdx)
        }
      })

      partitionLayerRanges.set(data.id, { minLayer, maxLayer, ownLayer: ownLayerIdx })
      console.log(`[Layout] Partition "${data.label}" spans layers ${minLayer}-${maxLayer}, own layer: ${ownLayerIdx}`)
    }
  })

  // Convert to PositionedNode array
  const positionedNodes: PositionedNode[] = []

  root.each((node: any) => {
    const data = node.data

    // Skip virtual root
    if (data.isVirtual && data.id === '__virtual_root__') {
      return
    }

    // Skip virtual layer parents
    if (data.isVirtual && data.id.startsWith('__layer_')) {
      return
    }

    const layerIndex = layerMap.get(data.layer) || 0
    const isPartition = partitionIds.has(data.id)

    console.log(`[Layout] Node "${data.label}" id="${data.id}" isPartition: ${isPartition}, inSet: ${partitionIds.has(data.id)}`)

    // Calculate 3D position from treemap coordinates
    const x0 = node.x0 || 0
    const x1 = node.x1 || 0
    const y0 = node.y0 || 0
    const y1 = node.y1 || 0

    const x = (x0 + x1) / 2 - canvasSize / 2
    const z = (y0 + y1) / 2 - canvasSize / 2

    // Calculate dimensions based on node type
    const cellWidth = x1 - x0
    const cellDepth = y1 - y0

    // Partition nodes fill their entire cell to show containment
    // Leaf nodes use slightly less to show separation from partition borders
    const width = isPartition ? cellWidth : cellWidth * 0.85
    const depth = isPartition ? cellDepth : cellDepth * 0.85

    // Calculate Y position and height based on node type
    let y: number
    let height: number

    if (isPartition) {
      // Partition nodes span vertically from min to max child layer with extra height
      const layerRange = partitionLayerRanges.get(data.id)
      if (layerRange) {
        const minY = layerRange.minLayer * layerSpacing
        const maxY = layerRange.maxLayer * layerSpacing
        y = (minY + maxY) / 2
        height = maxY - minY + layerSpacing * 0.8 // Span entire range plus extra
      } else {
        y = layerIndex * layerSpacing
        height = layerSpacing * 0.8
      }
    } else {
      // Flow nodes are at their specific layer - make them taller for visibility
      y = layerIndex * layerSpacing
      height = Math.max(4, layerSpacing * 0.25) // Taller minimum height
    }

    positionedNodes.push({
      id: data.id,
      label: data.label,
      x,
      y,
      z,
      width,
      height,
      depth,
      color: data.color,
      labelColor: data.labelColor,
      layerId: data.layer,
      isPartition,
      weight: data.weight,
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
