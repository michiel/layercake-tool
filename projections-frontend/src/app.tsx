import { useEffect, useMemo, useRef } from 'react'
import { gql } from '@apollo/client/core'
import { useMutation, useQuery, useSubscription } from '@apollo/client/react'
import ForceGraph3D from '3d-force-graph'
import { Leva, useControls, folder } from 'leva'
import {
  CanvasTexture,
  Group,
  Mesh,
  MeshBasicMaterial,
  SphereGeometry,
  Sprite,
  SpriteMaterial,
} from 'three'

const PROJECTION_QUERY = gql`
  query ProjectionView($id: ID!) {
    projection(id: $id) {
      id
      name
      projectionType
      graphId
    }
    projectionGraph(id: $id) {
      nodes { id label layer color labelColor }
      edges { id source target }
      layers { layerId name backgroundColor textColor borderColor }
    }
    projectionState(id: $id) {
      projectionId
      projectionType
      stateJson
    }
  }
`

const GRAPH_SUB = gql`
  subscription ProjectionGraphUpdated($id: ID!) {
    projectionGraphUpdated(id: $id) {
      nodes { id label layer color labelColor }
      edges { id source target }
      layers { layerId name backgroundColor textColor borderColor }
    }
  }
`

const STATE_SUB = gql`
  subscription ProjectionStateUpdated($id: ID!) {
    projectionStateUpdated(id: $id) {
      projectionId
      projectionType
      stateJson
    }
  }
`

const SAVE_STATE = gql`
  mutation SaveProjectionState($id: ID!, $state: JSON!) {
    saveProjectionState(id: $id, state: $state)
  }
`

const getProjectionId = () => {
  const url = new URL(window.location.href)
  // New canonical path: /projections/viewer/:id
  const match = url.pathname.match(/\/projections\/viewer\/(\d+)/)
  if (match) return match[1]
  // Legacy direct route: /projections/:id
  const legacy = url.pathname.match(/\/projections\/(\d+)/)
  if (legacy) return legacy[1]
  // Fallback to hash param
  const hash = url.hash.match(/projectionId=(\d+)/)
  if (hash) return hash[1]
  return null
}

export default function App() {
  const projectionId = getProjectionId()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const fgRef = useRef<any>(null)

  const controls = useControls(() => ({
    Forces: folder({
      linkDistance: { value: 60, min: 10, max: 300, step: 5 },
      chargeStrength: { value: -120, min: -2000, max: 0, step: 10 },
    }),
    Display: folder({
      showLinks: true,
      showLabels: true,
      nodeRelSize: { value: 4, min: 1, max: 12, step: 0.5 },
      linkColor: '#6ddcff',
      defaultNodeColor: '#ffd166',
    }),
  }))

  const {
    showLinks,
    showLabels,
    nodeRelSize,
    linkColor,
    defaultNodeColor,
    linkDistance,
    chargeStrength,
  } = controls as any

  const safeNodeSize = useMemo(() => Number(nodeRelSize) || 4, [nodeRelSize])
  const safeLinkDistance = useMemo(() => Number(linkDistance) || 60, [linkDistance])
  const safeChargeStrength = useMemo(() => Number(chargeStrength) || -120, [chargeStrength])

  const { data, loading } = useQuery(PROJECTION_QUERY, {
    variables: { id: projectionId },
    skip: !projectionId,
  })

  const { data: graphUpdates } = useSubscription(GRAPH_SUB, {
    variables: { id: projectionId },
    skip: !projectionId,
  })

  const { data: stateUpdates } = useSubscription(STATE_SUB, {
    variables: { id: projectionId },
    skip: !projectionId,
  })

  const [saveState] = useMutation(SAVE_STATE)

  const projection = (data as any)?.projection
  const graph = (graphUpdates as any)?.projectionGraphUpdated ?? (data as any)?.projectionGraph
  const state =
    (stateUpdates as any)?.projectionStateUpdated ?? (data as any)?.projectionState

  console.log('[App] Data state:', {
    loading,
    hasData: !!data,
    hasProjection: !!projection,
    hasGraph: !!graph,
    graphNodes: graph?.nodes?.length,
    graphEdges: graph?.edges?.length,
    graphLayers: graph?.layers?.length,
  })

  const layers = graph?.layers ?? []
  const layersKey = useMemo(
    () => JSON.stringify(layers.map((l: any) => [l.layerId, l.backgroundColor, l.textColor])),
    [layers]
  )

  const layerControls = useControls(
    'Layers',
    () => {
      const schema: Record<string, any> = {}
      layers.forEach((layer: any) => {
        const bodyKey = `${layer.layerId || layer.name || 'layer'} body`
        const labelKey = `${layer.layerId || layer.name || 'layer'} label`
        schema[bodyKey] = { value: layer.backgroundColor || defaultNodeColor }
        schema[labelKey] = { value: layer.textColor || '#0f172a' }
      })
      return schema
    },
    [layersKey, defaultNodeColor]
  )

  const layerColors = useMemo(() => {
    const map = new Map<string, { body: string; label: string }>()
    layers.forEach((layer: any) => {
      const bodyKey = `${layer.layerId || layer.name || 'layer'} body`
      const labelKey = `${layer.layerId || layer.name || 'layer'} label`
      map.set(layer.layerId, {
        body: (layerControls as any)[bodyKey] || layer.backgroundColor || defaultNodeColor,
        label: (layerControls as any)[labelKey] || layer.textColor || '#0f172a',
      })
    })
    return map
  }, [layers, layerControls, defaultNodeColor])

  const graphData = useMemo(() => {
    if (!graph) return { nodes: [], links: [] }

    return {
      nodes:
        graph.nodes?.map((n: any) => ({
          id: n.id,
          name: n.label || n.id,
          layer: n.layer,
          color:
            (n.layer && layerColors.get(n.layer)?.body) ||
            n.color ||
            defaultNodeColor,
          textColor:
            (n.layer && layerColors.get(n.layer)?.label) ||
            n.labelColor ||
            '#0f172a',
        })) ?? [],
      links:
        graph.edges?.map((e: any) => ({
          id: e.id,
          source: e.source,
          target: e.target,
          name: e.label,
          layer: e.layer,
        })) ?? [],
    }
  }, [graph, layerColors, defaultNodeColor])

  const isLayer3d = projection?.projectionType === 'layer3d'

  // Effect 1: Initialize the ForceGraph3D instance (runs once on mount)
  useEffect(() => {
    console.log('[ForceGraph] Initializing graph instance', {
      isLayer3d,
      hasContainer: !!containerRef.current,
      hasExistingInstance: !!fgRef.current,
    })

    if (isLayer3d) {
      console.log('[ForceGraph] Skipping initialization: isLayer3d')
      return
    }

    if (!containerRef.current) {
      console.log('[ForceGraph] Skipping initialization: no container')
      return
    }

    if (fgRef.current) {
      console.log('[ForceGraph] Instance already exists, skipping initialization')
      return
    }

    const elem = containerRef.current

    console.log('[ForceGraph] Creating ForceGraph3D instance')
    const fg = (ForceGraph3D as any)()(elem)
      .forceEngine('d3')
      .backgroundColor('#0b1021')
      .showNavInfo(false)
      .nodeThreeObject(() => {
        // Return a placeholder group - will be updated in effect 2
        return new Group()
      })
      .graphData({ nodes: [], links: [] })

    fgRef.current = fg

    console.log('[ForceGraph] Instance created and stored')

    return () => {
      console.log('[ForceGraph] Cleanup: Destroying instance')
      const instance = fgRef.current
      if (instance) {
        if (typeof instance.pauseAnimation === 'function') {
          instance.pauseAnimation()
        }
        if (typeof instance.graphData === 'function') {
          instance.graphData({ nodes: [], links: [] })
        }
      }
      fgRef.current = null
      elem.innerHTML = ''
      console.log('[ForceGraph] Cleanup completed')
    }
  }, [isLayer3d, containerRef.current])

  // Effect 2: Update graph data and configuration (runs on prop changes)
  useEffect(() => {
    const fg = fgRef.current
    if (isLayer3d || !fg) {
      console.log('[ForceGraph] Update skipped (layer3d or not initialized)')
      return
    }

    console.log('[ForceGraph] Updating graph properties and data', {
      nodes: graphData.nodes.length,
      links: graphData.links.length,
    })

    // Update graph data
    fg.graphData(graphData)

    // Update link properties
    fg.linkColor(() => (showLinks ? linkColor : 'rgba(0,0,0,0)'))

    // Update node properties and rendering
    fg.nodeLabel((n: any) => n.name || n.id)
      .nodeRelSize(safeNodeSize)
      .nodeColor((n: any) => n.color || defaultNodeColor)
      .nodeThreeObject((n: any) => {
        const group = new Group()

        const sphereGeom = new SphereGeometry(safeNodeSize * 0.8, 12, 12)
        const sphereMat = new MeshBasicMaterial({
          color: n.color || defaultNodeColor,
        })
        const sphere = new Mesh(sphereGeom, sphereMat)
        group.add(sphere)

        if (showLabels) {
          const label = n.name || n.id
          const canvas = document.createElement('canvas')
          const width = 256
          const height = 64
          canvas.width = width
          canvas.height = height
          const ctx = canvas.getContext('2d')
          if (ctx) {
            ctx.fillStyle = 'rgba(0,0,0,0)'
            ctx.fillRect(0, 0, width, height)
            ctx.fillStyle = n.textColor || '#0f172a'
            ctx.font = '24px sans-serif'
            ctx.textAlign = 'center'
            ctx.textBaseline = 'middle'
            ctx.fillText(label, width / 2, height / 2, width - 16)
          }
          const texture = new CanvasTexture(canvas)
          const material = new SpriteMaterial({
            map: texture,
            transparent: true,
          })
          const sprite = new Sprite(material)
          const scale = Math.max(6, safeNodeSize * 2)
          sprite.scale.set(scale * 0.8, scale * 0.4, 1)
          sprite.position.set(0, safeNodeSize * 1.2, 0)
          group.add(sprite)
        }

        return group
      })

    // Update forces
    const linkForce = fg.d3Force('link')
    if (linkForce && typeof linkForce.distance === 'function' && Number.isFinite(safeLinkDistance)) {
      linkForce.distance(safeLinkDistance)
      console.log('[ForceGraph] Link force distance configured:', safeLinkDistance)
    }

    const chargeForce = fg.d3Force('charge')
    if (chargeForce && typeof chargeForce.strength === 'function' && Number.isFinite(safeChargeStrength)) {
      chargeForce.strength(safeChargeStrength)
      console.log('[ForceGraph] Charge force strength configured:', safeChargeStrength)
    }

    // Reheat simulation after changes
    if (typeof fg.d3ReheatSimulation === 'function') {
      fg.d3ReheatSimulation()
    }

    console.log('[ForceGraph] Properties and data updated, simulation reheated')
  }, [
    graphData,
    showLinks,
    linkColor,
    defaultNodeColor,
    safeNodeSize,
    showLabels,
    isLayer3d,
    safeLinkDistance,
    safeChargeStrength,
  ])

  const handleSaveState = () => {
    if (!projectionId) return
    const nextState = {
      ...(state?.stateJson ?? {}),
      ui: {
        showLinks,
        showLabels,
        linkColor,
        defaultNodeColor,
        nodeRelSize,
        linkDistance,
        chargeStrength,
        layers: Object.fromEntries(
          Array.from(layerColors.entries()).map(([layerId, colors]) => [
            layerId,
            { body: colors.body, label: colors.label },
          ])
        ),
      },
    }
    saveState({ variables: { id: projectionId, state: nextState } })
  }

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center text-slate-100 bg-slate-900">
        Loading projection...
      </div>
    )
  }

  if (!projection) {
    return (
      <div className="flex h-full items-center justify-center text-slate-100 bg-slate-900">
        Projection not found
      </div>
    )
  }

  if (isLayer3d) {
    return (
      <div className="h-screen w-screen bg-slate-900 text-slate-100">
        <div className="flex items-center justify-between p-3 border-b border-slate-700">
          <div>
            <div className="font-semibold">{projection.name}</div>
            <div className="text-xs text-slate-400">
              Type: {projection.projectionType} · Graph {projection.graphId}
            </div>
          </div>
        </div>
        <div className="flex h-full items-center justify-center flex-col gap-4 pb-20">
          <div className="text-6xl">🏗️</div>
          <div className="text-2xl font-bold">Layer 3D Coming Soon</div>
          <div className="text-slate-400 max-w-md text-center">
            The Layer 3D visualization type is currently under development.
            Please use Force 3D for now, or check back later for updates.
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="h-screen w-screen bg-slate-900 text-slate-100">
      <Leva collapsed />
      <div className="flex items-center justify-between p-3 border-b border-slate-700">
        <div>
          <div className="font-semibold">{projection.name}</div>
          <div className="text-xs text-slate-400">
            Type: {projection.projectionType} · Graph {projection.graphId}
          </div>
        </div>
        <div className="flex gap-2">
          <button
            className="rounded bg-slate-100 px-3 py-1 text-slate-900 text-sm"
            onClick={handleSaveState}
          >
            Save state
          </button>
        </div>
      </div>
      <div ref={containerRef} className="h-full w-full" />
    </div>
  )
}
