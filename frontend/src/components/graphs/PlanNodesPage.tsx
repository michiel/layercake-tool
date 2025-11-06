import React, { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  IconPlus,
  IconGraph,
  IconEdit,
  IconTrash,
  IconAlertCircle,
  IconRefresh,
  IconDots,
  IconCheck,
  IconClock,
  IconX
} from '@tabler/icons-react'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '../ui/dropdown-menu'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Spinner } from '../ui/spinner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table'
import { gql } from '@apollo/client'
import { Breadcrumbs } from '../common/Breadcrumbs'
import PageContainer from '../layout/PageContainer'
import { Graph, GET_GRAPHS, CREATE_GRAPH, UPDATE_GRAPH, DELETE_GRAPH, EXECUTE_NODE } from '../../graphql/graphs'
import { getExecutionStateLabel, isExecutionInProgress } from '../../graphql/preview'
import { GET_PLAN_DAG } from '../../graphql/plan-dag'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`

interface PlanNodesPageProps {}

interface PlanDagDatasourceExecution {
  dataSourceId?: number
  filename?: string
  status?: string
  processedAt?: string
  executionState?: string
}

interface PlanDagGraphExecution {
  graphId?: number
  nodeCount?: number
  edgeCount?: number
  executionState?: string
  computedDate?: string
}

interface PlanDagNode {
  id: string
  nodeType: string
  metadata?: {
    label?: string
    description?: string
  }
  graphExecution?: PlanDagGraphExecution
  datasourceExecution?: PlanDagDatasourceExecution
}

interface PlanDagResponse {
  getPlanDag: {
    nodes: PlanDagNode[]
  }
}

interface PlanNodeRow {
  nodeId: string
  nodeType: string
  label: string
  executionState: string
  nodeCount: number | null
  edgeCount: number | null
  layerCount: number | null
  updatedAt: string | null
  graph: Graph | undefined
}

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

export const PlanNodesPage: React.FC<PlanNodesPageProps> = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [selectedGraph, setSelectedGraph] = useState<Graph | null>(null)
  const [executingGraphId, setExecutingGraphId] = useState<number | null>(null)

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'))

  const { data, loading, error } = useQuery<{ graphs: Graph[] }>(GET_GRAPHS, {
    variables: { projectId: parseInt(projectId || '0') },
    fetchPolicy: 'cache-and-network'
  })

  const { data: planDagData } = useQuery<PlanDagResponse>(GET_PLAN_DAG, {
    variables: { projectId: parseInt(projectId || '0') },
    fetchPolicy: 'cache-and-network'
  })

  // Create a map of nodeId to nodeType from plan DAG
  const nodeTypeMap = React.useMemo(() => {
    const map = new Map<string, string>()
    const nodes = planDagData?.getPlanDag?.nodes || []
    nodes.forEach((node) => {
      map.set(node.id, node.nodeType)
    })
    return map
  }, [planDagData])

  const [createGraph, { loading: createLoading }] = useMutation(CREATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [updateGraph, { loading: updateLoading }] = useMutation(UPDATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [deleteGraph, { loading: deleteLoading }] = useMutation(DELETE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const [executeNode] = useMutation(EXECUTE_NODE, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  })

  const graphs: Graph[] = data?.graphs || []
  const graphsByNodeId = React.useMemo(() => {
    const map = new Map<string, Graph>()
    graphs.forEach((graph) => {
      map.set(graph.nodeId, graph)
    })
    return map
  }, [graphs])

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

  const planNodes: PlanNodeRow[] = React.useMemo(() => {
    const nodes = planDagData?.getPlanDag?.nodes || []
    return nodes.map<PlanNodeRow>((node) => {
      const graph = graphsByNodeId.get(node.id)
      const metadata = node.metadata || {}
      const label = metadata.label || graph?.name || node.id
      const graphExecution = node.graphExecution || {}
      const datasourceExecution = node.datasourceExecution || {}
      const executionState =
        graphExecution.executionState ||
        datasourceExecution.executionState ||
        graph?.executionState ||
        datasourceExecution.status ||
        'NOT_STARTED'
      const nodeCount =
        graphExecution.nodeCount !== undefined
          ? graphExecution.nodeCount ?? null
          : graph?.nodeCount ?? null
      const edgeCount =
        graphExecution.edgeCount !== undefined
          ? graphExecution.edgeCount ?? null
          : graph?.edgeCount ?? null
      const layerCount = graph?.layers?.length ?? null
      const updatedAt =
        graphExecution.computedDate ||
        graph?.updatedAt ||
        datasourceExecution.processedAt ||
        null

      return {
        nodeId: node.id,
        nodeType: node.nodeType,
        label,
        executionState,
        nodeCount,
        edgeCount,
        layerCount,
        updatedAt,
        graph,
      }
    })
  }, [graphsByNodeId, planDagData])

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
          currentPage="Plan Nodes"
          onNavigate={handleNavigate}
        />

        <Group justify="between" className="mb-4">
          <div>
            <h1 className="text-3xl font-bold">Plan Nodes</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Review every plan node and track execution progress across datasources and graphs
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
          {planNodes.length === 0 && !loading ? (
            <CardContent className="py-12">
              <Stack align="center" gap="md">
                <IconGraph size={48} className="text-muted-foreground" />
                <div className="text-center">
                  <h3 className="text-xl font-semibold mb-2">No Plan Nodes</h3>
                  <p className="text-muted-foreground mb-4">
                    Define plan nodes to see data source and graph execution details.
                  </p>
                  <Button onClick={handleCreate}>
                    <IconPlus className="mr-2 h-4 w-4" />
                    Create First Graph Node
                  </Button>
                </div>
              </Stack>
            </CardContent>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Node Type</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Nodes</TableHead>
                    <TableHead>Edges</TableHead>
                    <TableHead>Layers</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {planNodes.map((planNode) => {
                    const { graph, nodeType } = planNode
                    const executionState = planNode.executionState || 'NOT_STARTED'
                    const isRunning =
                      (graph && executingGraphId === graph.id) ||
                      isExecutionInProgress(executionState)
                    const nodeCount =
                      planNode.nodeCount !== null && planNode.nodeCount !== undefined
                        ? planNode.nodeCount
                        : '—'
                    const edgeCount =
                      planNode.edgeCount !== null && planNode.edgeCount !== undefined
                        ? planNode.edgeCount
                        : '—'
                    const layerCount =
                      planNode.layerCount !== null && planNode.layerCount !== undefined
                        ? planNode.layerCount
                        : '—'
                    const updatedDisplay = planNode.updatedAt
                      ? formatDateTime(planNode.updatedAt)
                      : '—'

                    return (
                      <TableRow key={planNode.nodeId}>
                        <TableCell>
                          <p className="font-medium">{planNode.label}</p>
                        </TableCell>
                        <TableCell>
                          <Badge variant="secondary" className="text-xs">
                            {nodeTypeMap.get(planNode.nodeId) || nodeType}
                          </Badge>
                        </TableCell>
                        <TableCell>
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
                        </TableCell>
                        <TableCell>
                          <Badge variant="secondary">
                            {nodeCount}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          <Badge variant="secondary">
                            {edgeCount}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          <Badge variant="secondary">
                            {layerCount}
                          </Badge>
                        </TableCell>
                        <TableCell>
                          <p className="text-sm text-muted-foreground">
                            {updatedDisplay}
                          </p>
                        </TableCell>
                        <TableCell>
                          {graph ? (
                            <DropdownMenu>
                              <DropdownMenuTrigger asChild>
                                <Button variant="ghost" size="icon">
                                  <IconDots className="h-4 w-4" />
                                </Button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent align="end" className="w-[220px]">
                                <DropdownMenuItem
                                  onClick={() =>
                                    navigate(`/projects/${projectId}/plan-nodes/${graph.id}/edit`)
                                  }
                                >
                                  <IconGraph className="mr-2 h-3.5 w-3.5" />
                                  Open Graph Editor
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                  onClick={() => handleEdit(graph)}
                                >
                                  <IconEdit className="mr-2 h-3.5 w-3.5" />
                                  Rename
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                  onClick={() => handleReprocess(graph)}
                                  disabled={isRunning}
                                >
                                  <IconRefresh className="mr-2 h-3.5 w-3.5" />
                                  Reprocess
                                </DropdownMenuItem>
                                <DropdownMenuSeparator />
                                <DropdownMenuItem
                                  onClick={() => handleDelete(graph)}
                                  className="text-red-600"
                                >
                                  <IconTrash className="mr-2 h-3.5 w-3.5" />
                                  Delete
                                </DropdownMenuItem>
                              </DropdownMenuContent>
                            </DropdownMenu>
                          ) : (
                            <p className="text-sm text-muted-foreground">
                              No actions
                            </p>
                          )}
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            </div>
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
