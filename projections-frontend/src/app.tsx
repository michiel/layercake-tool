import { useEffect, useRef } from 'react'
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
        nodes: graph.nodes?.map((n: any) => ({ id: n.id, name: n.label || n.id })) ?? [],
        links: graph.edges?.map((e: any) => ({ id: e.id, source: e.source, target: e.target, name: e.label })) ?? [],
      })
      .nodeLabel('name')
      .linkDirectionalParticles(0)
      .linkColor(() => '#6ddcff')
      .nodeColor(() => '#ffd166')
      .backgroundColor('#0b1021')
    return () => {
      fg.graphData({ nodes: [], links: [] })
    }
  }, [graph])

  const handleSaveState = () => {
    if (!projectionId) return
    saveState({ variables: { id: projectionId, state: state?.stateJson ?? {} } })
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
