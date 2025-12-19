import { useMemo } from 'react'
import { gql } from '@apollo/client/core'
import { useMutation, useQuery, useSubscription } from '@apollo/client/react'
import { Leva, useControls, folder } from 'leva'
import Layer3DScene from './projections/layer3d-projection/Layer3DScene'
import Force3DViewer from './components/Force3DViewer'

// --- GraphQL Queries and Mutations (Unchanged) ---
const PROJECTION_QUERY = gql`
  query ProjectionView($id: ID!) {
    projection(id: $id) {
      id
      name
      projectionType
      graphId
    }
    projectionGraph(id: $id) {
      nodes { id label layer color labelColor weight attributes }
      edges { id source target label weight attributes }
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
      nodes { id label layer color labelColor weight attributes }
      edges { id source target label weight attributes }
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
  const searchParam = url.searchParams.get('id')
  if (searchParam) return searchParam
  const match = url.pathname.match(/\/projections\/viewer\/(\d+)/)
  if (match) return match[1]
  const legacy = url.pathname.match(/\/projections\/(\d+)/)
  if (legacy) return legacy[1]
  const hash = url.hash.match(/projectionId=(\d+)/)
  if (hash) return hash[1]
  return null
}

export default function App() {
  const projectionId = getProjectionId()

  // --- Leva Controls (Unchanged) ---
  const controls = useControls(() => ({
    Forces: folder({
      linkDistance: { value: 60, min: 10, max: 300, step: 5 },
      chargeStrength: { value: -120, min: -2000, max: 0, step: 10 },
    }),
    Display: folder({
      showLinks: true,
      showLabels: true,
      nodeRelSize: { value: 4, min: 1, max: 12, step: 0.5 },
      linkColor: '#9ad8ff',
      defaultNodeColor: '#ffd166',
    }),
  }))

  const showLinks = (controls as any).showLinks ?? true
  const showLabels = (controls as any).showLabels ?? true
  const nodeRelSize = Number((controls as any).nodeRelSize ?? 4)
  const linkColor = (controls as any).linkColor ?? '#9ad8ff'
  const defaultNodeColor = (controls as any).defaultNodeColor ?? '#ffd166'
  const linkDistance = (controls as any).linkDistance ?? 60
  const chargeStrength = (controls as any).chargeStrength ?? -120

  // --- Data Fetching (Unchanged) ---
  const { data, loading, error } = useQuery(PROJECTION_QUERY, {
    variables: { id: projectionId },
    skip: !projectionId,
  })

  if (error) {
    console.error('[App] GraphQL Query Error:', error)
  }

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
  const isLayer3d = projection?.projectionType === 'layer3d'
  const layer3dState = (state?.stateJson as any)?.layer3d ?? {}

  // --- Layer Controls (Unchanged) ---
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
        schema[labelKey] = { value: layer.textColor || '#ffffff' }
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
        label: (layerControls as any)[labelKey] || layer.textColor || '#ffffff',
      })
    })
    return map
  }, [layers, layerControls, defaultNodeColor])

  // Memoized graph data structure
  const graphData = useMemo(() => {
    if (!graph) return { nodes: [], links: [] }

    return {
      nodes:
        graph.nodes?.map((n: any) => ({
          id: n.id,
          name: n.label || n.id,
          layer: n.layer,
          attrs: n.attributes || {},
          weight: n.weight,
          color:
            (n.layer && layerColors.get(n.layer)?.body) ||
            n.color ||
            defaultNodeColor,
          textColor:
            (n.layer && layerColors.get(n.layer)?.label) ||
            n.labelColor ||
            '#ffffff',
        })) ?? [],
      links:
        graph.edges?.map((e: any) => ({
          id: e.id,
          source: e.source,
          target: e.target,
          name: e.label,
          layer: e.layer,
          attrs: e.attributes || {},
          weight: e.weight,
        })) ?? [],
    }
  }, [graph, layerColors, defaultNodeColor])

  console.log('[App] Data state:', {
    projectionId,
    loading,
    hasData: !!data,
    projection: projection ? { id: projection.id, type: projection.projectionType } : null,
    hasGraph: !!graph,
    graphNodes: graph?.nodes?.length,
    graphEdges: graph?.edges?.length,
    isLayer3d,
  })

  // --- Save State Handler (Unchanged) ---
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

  // --- Render Logic (Unchanged) ---
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
        <div className="max-w-md text-center space-y-4">
          <div className="text-4xl">🔍</div>
          <div className="text-xl font-semibold">Projection Not Found</div>
          <div className="text-slate-400 text-sm">
            Projection #{projectionId} doesn't exist yet.
            {error && (
              <div className="mt-2 text-xs text-red-400">
                {error.message}
              </div>
            )}
          </div>
          <div className="mt-4 p-4 bg-slate-800 rounded-lg text-left text-sm space-y-2">
            <div className="font-semibold text-slate-300">To create this projection:</div>
            <ol className="list-decimal list-inside text-slate-400 space-y-1">
              <li>Open the Plan DAG editor</li>
              <li>Add a Projection node and configure it</li>
              <li>Connect it to a graph computation node</li>
              <li>Execute the DAG</li>
            </ol>
          </div>
        </div>
      </div>
    )
  }

  if (isLayer3d) {
    return (
      <div className="h-screen w-screen bg-slate-900 text-slate-100 flex flex-col">
        <Leva collapsed />
        <div className="flex items-center justify-between p-3 border-b border-slate-700">
          <div>
            <div className="font-semibold">{projection.name}</div>
            <div className="text-xs text-slate-400">
              Type: {projection.projectionType} · Graph {projection.graphId}
            </div>
          </div>
        </div>
        <div className="flex-1 relative">
          {graph?.nodes && graph?.edges && graph?.layers ? (
            <Layer3DScene
              nodes={graph.nodes}
              edges={graph.edges}
              layers={graph.layers}
              state={layer3dState}
              onSaveState={(nextState) => {
                if (!projectionId) return
                const merged = {
                  ...(state?.stateJson ?? {}),
                  layer3d: nextState,
                }
                saveState({ variables: { id: projectionId, state: merged } })
              }}
            />
          ) : (
            <div className="flex h-full items-center justify-center">
              <div className="text-slate-400">No graph data available</div>
            </div>
          )}
        </div>
      </div>
    )
  }

  return (
    <div className="h-screen w-screen bg-slate-900 text-slate-100 flex flex-col">
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
      <Force3DViewer
        graphData={graphData}
        showLinks={showLinks}
        showLabels={showLabels}
        nodeRelSize={nodeRelSize}
        linkColor={linkColor}
        defaultNodeColor={defaultNodeColor}
        linkDistance={linkDistance}
        chargeStrength={chargeStrength}
      />
    </div>
  )
}
