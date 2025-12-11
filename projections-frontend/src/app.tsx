import { useEffect, useRef, useState } from 'react'
import { gql } from '@apollo/client/core'
import { useMutation, useQuery, useSubscription } from '@apollo/client/react'
import ForceGraph from 'force-graph'

const PROJECTION_QUERY = gql`
  query ProjectionView($id: ID!) {
    projection(id: $id) {
      id
      name
      projectionType
      graphId
    }
    projectionGraph(id: $id) {
      nodes { id label layer }
      edges { id source target }
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
      nodes { id label layer }
      edges { id source target }
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
  const match = url.pathname.match(/\/projections\/(\d+)/)
  if (match) return match[1]
  const hash = url.hash.match(/projectionId=(\d+)/)
  if (hash) return hash[1]
  return null
}

export default function App() {
  const projectionId = getProjectionId()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [showLinks, setShowLinks] = useState(true)
  const [nodeColor, setNodeColor] = useState('#ffd166')
  const [linkColor, setLinkColor] = useState('#6ddcff')
  const [nodeRelSize, setNodeRelSize] = useState(4)

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

  useEffect(() => {
    if (!graph || !containerRef.current) return
    const elem = containerRef.current
    elem.innerHTML = ''
    const fg = (ForceGraph as any)(elem)
      .graphData({
        nodes: graph.nodes?.map((n: any) => ({ id: n.id, name: n.label || n.id, layer: n.layer })) ?? [],
        links: graph.edges?.map((e: any) => ({ id: e.id, source: e.source, target: e.target, name: e.label, layer: e.layer })) ?? [],
      })
      .nodeLabel('name')
      .linkDirectionalParticles(0)
      .linkColor(() => (showLinks ? linkColor : 'rgba(0,0,0,0)'))
      .nodeColor((n: any) => (n.layer ? nodeColor : '#9ae6b4'))
      .nodeRelSize(nodeRelSize)
      .backgroundColor('#0b1021')
    return () => {
      fg.graphData({ nodes: [], links: [] })
    }
  }, [graph, linkColor, nodeColor, nodeRelSize, showLinks])

  const handleSaveState = () => {
    if (!projectionId) return
    const nextState = {
      ...(state?.stateJson ?? {}),
      ui: {
        showLinks,
        nodeColor,
        linkColor,
        nodeRelSize,
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

  return (
    <div className="h-screen w-screen bg-slate-900 text-slate-100">
      <div className="flex items-center justify-between p-3 border-b border-slate-700">
        <div>
          <div className="font-semibold">{projection.name}</div>
          <div className="text-xs text-slate-400">
            Type: {projection.projectionType} · Graph {projection.graphId}
          </div>
        </div>
        <div className="flex gap-2">
          <label className="flex items-center gap-2 text-xs">
            <span>Nodes</span>
            <input
              type="color"
              value={nodeColor}
              onChange={(e) => setNodeColor(e.target.value)}
              className="h-6 w-10 bg-transparent"
            />
          </label>
          <label className="flex items-center gap-2 text-xs">
            <span>Links</span>
            <input
              type="color"
              value={linkColor}
              onChange={(e) => setLinkColor(e.target.value)}
              className="h-6 w-10 bg-transparent"
            />
          </label>
          <label className="flex items-center gap-2 text-xs">
            <span>Size</span>
            <input
              type="range"
              min={2}
              max={10}
              value={nodeRelSize}
              onChange={(e) => setNodeRelSize(Number(e.target.value))}
            />
          </label>
          <label className="flex items-center gap-2 text-xs">
            <input
              type="checkbox"
              checked={showLinks}
              onChange={(e) => setShowLinks(e.target.checked)}
            />
            <span>Links</span>
          </label>
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
