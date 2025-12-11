import { useEffect, useRef } from 'react'
import { useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery, useSubscription } from '@apollo/client/react'
import PageContainer from '@/components/layout/PageContainer'
import { Group } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { createApolloClientForEndpoint } from '@/graphql/client'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import ForceGraph from 'force-graph'

const PROJECTION_QUERY = gql`
  query ProjectionView($id: ID!) {
    projection(id: $id) {
      id
      name
      projectionType: projection_type
      graphId: graph_id
    }
    projectionGraph(id: $id) {
      nodes { id label layer }
      edges { id source target }
    }
    projectionState(id: $id) {
      projectionId: projection_id
      projectionType: projection_type
      stateJson: state_json
    }
  }
`

const SAVE_STATE = gql`
  mutation SaveProjectionState($id: ID!, $state: JSON!) {
    saveProjectionState(id: $id, state: $state)
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
      projection_id
      projection_type
      state_json
    }
  }
`

const projectionsClient = createApolloClientForEndpoint({
  httpPath: '/projections/graphql',
  wsPath: '/projections/graphql/ws',
})

export const ProjectionViewerPage = () => {
  const { projectionId } = useParams<{ projectionId: string }>()
  const id = projectionId ?? ''
  const containerRef = useRef<HTMLDivElement | null>(null)

  const { data, loading, refetch } = useQuery(PROJECTION_QUERY, {
    variables: { id },
    skip: !id,
    client: projectionsClient,
    fetchPolicy: 'cache-and-network',
  })

  const { data: graphUpdates } = useSubscription(GRAPH_SUB, {
    variables: { id },
    skip: !id,
    client: projectionsClient,
  })
  const { data: stateUpdates } = useSubscription(STATE_SUB, {
    variables: { id },
    skip: !id,
    client: projectionsClient,
  })

  const [saveState, { loading: saving }] = useMutation(SAVE_STATE, {
    client: projectionsClient,
  })

  const projection = (data as any)?.projection
  const graph = (graphUpdates as any)?.projectionGraphUpdated ?? (data as any)?.projectionGraph
  const state = (stateUpdates as any)?.projectionStateUpdated ?? (data as any)?.projectionState

  useEffect(() => {
    if (graphUpdates) {
      showSuccessNotification('Projection updated', 'Graph changes received')
    }
  }, [graphUpdates])

  useEffect(() => {
    if (!graph || !containerRef.current) return
    const elem = containerRef.current
    elem.innerHTML = ''
    const fgFactory = (ForceGraph as any)()
    const fg = fgFactory(elem)
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

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading projection...</span>
        </Group>
      </PageContainer>
    )
  }

  if (!projection) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Projection not found</h1>
      </PageContainer>
    )
  }

  const handleSaveState = async () => {
    try {
      await saveState({
        variables: { id, state: state?.stateJson ?? {} },
      })
      showSuccessNotification('State saved', '')
    } catch (err: any) {
      showErrorNotification('Save failed', err?.message || 'Unable to save state')
    }
  }

  return (
    <PageContainer>
      <Group justify="between" className="mb-4">
        <div>
        <h1 className="text-3xl font-bold">{projection.name}</h1>
        <p className="text-muted-foreground">
            Projection type: {projection.projectionType} · Graph {projection.graphId}
        </p>
        </div>
        <Group gap="sm">
          <Button onClick={handleSaveState} disabled={saving}>
            Save state
          </Button>
          <Button variant="secondary" onClick={() => refetch()}>
            Refresh
          </Button>
        </Group>
      </Group>

      <Card className="mb-4">
        <CardHeader>
          <CardTitle>Graph</CardTitle>
        </CardHeader>
        <CardContent>
          <div ref={containerRef} className="h-[380px] w-full rounded border bg-black" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>State</CardTitle>
        </CardHeader>
        <CardContent>
          <pre className="max-h-[240px] overflow-auto rounded bg-muted p-3 text-xs">
            {JSON.stringify(state, null, 2)}
          </pre>
        </CardContent>
      </Card>
    </PageContainer>
  )
}

export default ProjectionViewerPage
