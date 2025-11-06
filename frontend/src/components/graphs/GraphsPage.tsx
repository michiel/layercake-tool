import React, { useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  IconPlus,
  IconGraph,
  IconEdit,
  IconTrash,
  IconAlertCircle,
  IconRefresh,
  IconCheck,
  IconClock,
  IconX,
  IconChartDots,
  IconTable
} from '@tabler/icons-react'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Spinner } from '../ui/spinner'
import { gql } from '@apollo/client'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { Graph, GET_GRAPHS, CREATE_GRAPH, UPDATE_GRAPH, DELETE_GRAPH, EXECUTE_NODE, GET_GRAPH_DETAILS } from '../../graphql/graphs'
import { GET_PLAN_DAG, UPDATE_PLAN_DAG_NODE } from '../../graphql/plan-dag'
import PageContainer from '../layout/PageContainer'
import { getExecutionStateLabel, isExecutionInProgress } from '../../graphql/preview'
import { GraphDataDialog } from '../editors/PlanVisualEditor/dialogs/GraphDataDialog'
import { GraphPreviewDialog } from '../visualization/GraphPreviewDialog'
import { GraphData } from '../visualization/GraphPreview'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`

interface GraphsPageProps {}

const formatDateTime = (value: string) => {
  const date = new Date(value)
  return date.toLocaleString()
}

const getExecutionStateIcon = (state: string) => {
  switch (state) {
    case 'COMPLETED':
      return <IconCheck size={14} />
    case 'PENDING':
    case 'PROCESSING':
      return <IconClock size={14} />
    case 'ERROR':
      return <IconX size={14} />
    default:
      return null
  }
}

const toGraphPreviewData = (graph?: Graph | null): GraphData | null => {
  if (!graph) return null

  const normalizeAttrs = (attrs: any): Record<string, string> => {
    if (!attrs) return {}
    const result: Record<string, string> = {}
    Object.entries(attrs).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        result[key] = String(value)
      }
    })
    return result
  }

  return {
    nodes: (graph.graphNodes || []).map((node) => ({
      id: node.id,
      name: node.label || node.id,
      layer: node.layer || 'default',
      attrs: {
        ...normalizeAttrs(node.attrs),
        is_partition: node.isPartition ? 'true' : 'false',
        belongs_to: node.belongsTo ? String(node.belongsTo) : ''
      }
    })),
    links: (graph.graphEdges || []).map((edge) => ({
      id: edge.id,
      source: edge.source,
      target: edge.target,
      name: edge.label || '',
      layer: edge.layer || 'default',
      attrs: normalizeAttrs(edge.attrs)
    })),
    layers: (graph.layers || []).map((layer) => ({
      layerId: layer.layerId,
      name: layer.name,
      backgroundColor: layer.backgroundColor,
      borderColor: layer.borderColor,
      textColor: layer.textColor
    }))
  }
}

export const GraphsPage: React.FC<GraphsPageProps> = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [selectedGraph, setSelectedGraph] = useState<Graph | null>(null)
  const [executingGraphId, setExecutingGraphId] = useState<number | null>(null)
  const [previewGraphId, setPreviewGraphId] = useState<number | null>(null)
  const [previewTitle, setPreviewTitle] = useState<string>('Graph Preview')
  const [dataDialogGraphId, setDataDialogGraphId] = useState<number | null>(null)

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'))

  const { data, loading, error } = useQuery<{ graphs: Graph[] }>(GET_GRAPHS, {
    variables: { projectId: parseInt(projectId || '0') },
    fetchPolicy: 'cache-and-network'
  })

  interface PlanDagNode {
    id: string
    nodeType: string
  }

  interface PlanDagResponse {
    getPlanDag: {
      nodes: PlanDagNode[]
    }
  }

  const { data: planDagData } = useQuery<PlanDagResponse>(GET_PLAN_DAG, {
    variables: { projectId: parseInt(projectId || '0') },
    fetchPolicy: 'cache-and-network'
  })

  const [createGraph, { loading: createLoading }] = useMutation(CREATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [updateGraph, { loading: updateLoading }] = useMutation(UPDATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [updatePlanDagNode] = useMutation(UPDATE_PLAN_DAG_NODE)

  const [deleteGraph, { loading: deleteLoading }] = useMutation(DELETE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [executeNode] = useMutation(EXECUTE_NODE, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const { data: previewDetails, loading: previewLoading, error: previewError } = useQuery<{ graph: Graph }>(
    GET_GRAPH_DETAILS,
    {
      variables: { id: previewGraphId ?? 0 },
      skip: previewGraphId === null,
      fetchPolicy: 'network-only'
    }
  )

  const previewData = useMemo(() => toGraphPreviewData(previewDetails?.graph), [previewDetails])

  const graphs: Graph[] = data?.graphs || []
  const nodeTypeMap = useMemo(() => {
    const map = new Map<string, string>()
    const nodes = planDagData?.getPlanDag?.nodes || []
    nodes.forEach((node) => {
      map.set(node.id, node.nodeType)
    })
    return map
  }, [planDagData])

  const graphNodes = useMemo(
    () => graphs.filter((graph) => nodeTypeMap.get(graph.nodeId) === 'GraphNode'),
    [graphs, nodeTypeMap]
  )

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleCreate = () => {
    setSelectedGraph(null)
    setEditModalOpen(true)
  }

  const handleEdit = (graph: Graph) => {
    setSelectedGraph(graph)
    setEditModalOpen(true)
  }

  const handleDelete = (graph: Graph) => {
    setSelectedGraph(graph)
    setDeleteModalOpen(true)
  }

  const confirmDelete = async () => {
    if (selectedGraph) {
      await deleteGraph({ variables: { id: selectedGraph.id } })
      setDeleteModalOpen(false)
      setSelectedGraph(null)
    }
  }

  const handleSave = async (values: { name: string }) => {
    if (selectedGraph) {
      await updateGraph({ variables: { id: selectedGraph.id, input: { name: values.name } } })
      await updatePlanDagNode({
        variables: {
          projectId: parseInt(projectId || '0'),
          nodeId: selectedGraph.nodeId,
          updates: {
            metadata: {
              label: values.name
            }
          }
        }
      })
    } else {
      // For creation, we need to generate a nodeId internally or derive it.
      // For now, we'll use a placeholder. This will be handled by the backend.
      await createGraph({ variables: { input: { name: values.name, projectId: parseInt(projectId || '0'), nodeId: 'generated-node-id' } } })
    }
    setEditModalOpen(false)
    setSelectedGraph(null)
  }

  const handleReprocess = async (graph: Graph) => {
    try {
      setExecutingGraphId(graph.id)
      await executeNode({
        variables: {
          projectId: parseInt(projectId || '0'),
          nodeId: graph.nodeId
        }
      })
    } catch (err) {
      console.error('Failed to reprocess graph:', err)
    } finally {
      setExecutingGraphId(null)
    }
  }

  if (!selectedProject) {
    return (
      <PageContainer>
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to Projects
        </Button>
      </PageContainer>
    )
  }

  return (
    <>
      <PageContainer>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          currentPage="Graphs"
          onNavigate={handleNavigate}
        />

        <Group justify="between" className="mb-4">
          <div>
            <h1 className="text-3xl font-bold">Graphs</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Manage graph entities for this project
            </p>
          </div>
          <Group gap="xs">
            <Button
              onClick={handleCreate}
              variant="secondary"
            >
              <IconPlus className="mr-2 h-4 w-4" />
              New Graph Node
            </Button>
          </Group>
        </Group>

        {error && (
          <Alert variant="destructive" className="mb-4">
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>
              {error.message}
            </AlertDescription>
          </Alert>
        )}

        <Card className="border relative">
          {loading && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
              <Spinner className="h-8 w-8" />
            </div>
          )}
          {graphNodes.length === 0 && !loading ? (
            <CardContent className="py-12">
              <Stack align="center" gap="md">
                <IconGraph size={48} className="text-muted-foreground" />
                <div className="text-center">
                  <h3 className="text-xl font-semibold mb-2">No Graph Nodes</h3>
                  <p className="text-muted-foreground mb-4">
                    Create a graph node or run your plan to materialize one.
                  </p>
                  <Button onClick={handleCreate}>
                    <IconPlus className="mr-2 h-4 w-4" />
                    Create Graph Node
                  </Button>
                </div>
              </Stack>
            </CardContent>
          ) : (
            <CardContent className="p-6">
              <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
              {graphNodes.map((graph) => {
                const executionState = graph.executionState || 'NOT_STARTED'
                const isRunning =
                  executingGraphId === graph.id || isExecutionInProgress(executionState)
                const nodeType = nodeTypeMap.get(graph.nodeId) || 'GraphNode'
                const lastUpdated = graph.updatedAt ? formatDateTime(graph.updatedAt) : '—'
                const layerCount = graph.layers?.length ?? 0

                return (
                  <Card key={graph.id} className="border shadow-sm">
                    <CardContent className="p-4">
                      <Stack gap="sm">
                        <Group justify="between" align="start">
                          <div>
                            <p className="font-semibold">{graph.name}</p>
                            <Group gap="xs" className="mt-1">
                              <Badge variant="secondary" className="text-xs">
                                {nodeType}
                              </Badge>
                            </Group>
                          </div>
                          <Badge
                            variant="secondary"
                            className={
                              executionState === 'COMPLETED'
                                ? 'bg-green-100 text-green-900'
                                : executionState === 'PROCESSING' || executionState === 'PENDING'
                                  ? 'bg-blue-100 text-blue-900'
                                  : executionState === 'ERROR'
                                    ? 'bg-red-100 text-red-900'
                                    : ''
                            }
                          >
                            {getExecutionStateIcon(executionState)}
                            <span className="ml-1">{getExecutionStateLabel(executionState)}</span>
                          </Badge>
                        </Group>

                        <Group gap="xs" className="flex-wrap">
                          <Badge variant="secondary">
                            Nodes: {graph.nodeCount}
                          </Badge>
                          <Badge variant="secondary">
                            Edges: {graph.edgeCount}
                          </Badge>
                          <Badge variant="secondary">
                            Layers: {layerCount}
                          </Badge>
                        </Group>
                        <p className="text-sm text-muted-foreground">
                          Last updated {lastUpdated}
                        </p>

                        <Group gap="sm" className="flex-wrap">
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() =>
                              navigate(`/projects/${projectId}/plan-nodes/${graph.id}/edit`)
                            }
                          >
                            <IconGraph className="mr-1.5 h-3.5 w-3.5" />
                            Open Graph Editor
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => {
                              setPreviewGraphId(graph.id)
                              setPreviewTitle(`Graph Preview: ${graph.name}`)
                            }}
                          >
                            <IconChartDots className="mr-1.5 h-3.5 w-3.5" />
                            Preview
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => setDataDialogGraphId(graph.id)}
                          >
                            <IconTable className="mr-1.5 h-3.5 w-3.5" />
                            View Data
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => handleEdit(graph)}
                          >
                            <IconEdit className="mr-1.5 h-3.5 w-3.5" />
                            Edit Properties
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => handleReprocess(graph)}
                            disabled={isRunning}
                          >
                            {executingGraphId === graph.id && <Spinner className="mr-1.5 h-3.5 w-3.5" />}
                            {executingGraphId !== graph.id && <IconRefresh className="mr-1.5 h-3.5 w-3.5" />}
                            Reprocess
                          </Button>
                          <Button
                            size="sm"
                            variant="secondary"
                            onClick={() => handleDelete(graph)}
                            className="text-red-600"
                          >
                            <IconTrash className="mr-1.5 h-3.5 w-3.5" />
                            Delete
                          </Button>
                        </Group>
                      </Stack>
                    </CardContent>
                  </Card>
                )
              })}
              </div>
            </CardContent>
          )}
        </Card>
      </PageContainer>

      <Dialog open={deleteModalOpen} onOpenChange={setDeleteModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Graph Node</DialogTitle>
          </DialogHeader>
          <p className="mb-4">
            Are you sure you want to delete "{selectedGraph?.name}"? This action cannot be undone.
          </p>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setDeleteModalOpen(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              disabled={deleteLoading}
              onClick={confirmDelete}
            >
              {deleteLoading && <Spinner className="mr-2 h-4 w-4" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={editModalOpen} onOpenChange={setEditModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{selectedGraph ? 'Edit Graph Node' : 'Create Graph Node'}</DialogTitle>
          </DialogHeader>
          <EditGraphForm
            graph={selectedGraph}
            onSave={handleSave}
            onCancel={() => setEditModalOpen(false)}
            loading={createLoading || updateLoading}
          />
        </DialogContent>
      </Dialog>

      <GraphDataDialog
        opened={dataDialogGraphId !== null}
        onClose={() => setDataDialogGraphId(null)}
        graphId={dataDialogGraphId}
        title={dataDialogGraphId ? `Graph Data: ${graphs.find((g) => g.id === dataDialogGraphId)?.name ?? ''}` : 'Graph Data'}
      />

      <GraphPreviewDialog
        opened={previewGraphId !== null}
        onClose={() => {
          setPreviewGraphId(null)
          setPreviewTitle('Graph Preview')
        }}
        data={previewData}
        title={previewTitle}
        loading={previewLoading}
        error={previewError?.message ?? null}
      />
    </>
  )
}

interface EditGraphFormProps {
  graph: Graph | null
  onSave: (values: { name: string }) => void
  onCancel: () => void
  loading: boolean
}

const EditGraphForm: React.FC<EditGraphFormProps> = ({ graph, onSave, onCancel, loading }) => {
  const [name, setName] = useState(graph?.name || '')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSave({ name })
  }

  return (
    <form onSubmit={handleSubmit}>
      <Stack gap="md" className="py-4">
        <div className="space-y-2">
          <Label htmlFor="name">Name *</Label>
          <Input
            id="name"
            value={name}
            onChange={(e) => setName(e.currentTarget.value)}
            required
          />
        </div>
      </Stack>
      <DialogFooter>
        <Button variant="secondary" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" disabled={loading}>
          {loading && <Spinner className="mr-2 h-4 w-4" />}
          Save
        </Button>
      </DialogFooter>
    </form>
  )
}
