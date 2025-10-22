import React, { useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Title,
  Group,
  Button,
  Stack,
  Card,
  Text,
  Modal,
  Alert,
  LoadingOverlay,
  TextInput,
  Badge,
  SimpleGrid
} from '@mantine/core'
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
import { gql } from '@apollo/client'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { Graph, GET_GRAPHS, CREATE_GRAPH, UPDATE_GRAPH, DELETE_GRAPH, EXECUTE_NODE, GET_GRAPH_DETAILS } from '../../graphql/graphs'
import { GET_PLAN_DAG, UPDATE_PLAN_DAG_NODE } from '../../graphql/plan-dag'
import PageContainer from '../layout/PageContainer'
import { getExecutionStateColor, getExecutionStateLabel, isExecutionInProgress } from '../../graphql/preview'
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
      backgroundColor: layer.properties?.background_color,
      borderColor: layer.properties?.border_color,
      textColor: layer.properties?.text_color
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
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
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

        <Group justify="space-between" mb="md">
          <div>
            <Title order={1}>Graphs</Title>
            <Text size="sm" c="dimmed" mt="xs">
              Manage graph entities for this project
            </Text>
          </div>
          <Group gap="xs">
            <Button
              leftSection={<IconPlus size={16} />}
              onClick={handleCreate}
              variant="light"
            >
              New Graph Node
            </Button>
          </Group>
        </Group>

        {error && (
          <Alert icon={<IconAlertCircle size={16} />} title="Error" color="red" mb="md">
            {error.message}
          </Alert>
        )}

        <Card withBorder p="lg" style={{ position: 'relative' }}>
          <LoadingOverlay visible={loading} />
          {graphNodes.length === 0 && !loading ? (
            <Stack align="center" py="xl" gap="md">
              <IconGraph size={48} color="gray" />
              <div style={{ textAlign: 'center' }}>
                <Title order={3}>No Graph Nodes</Title>
                <Text c="dimmed" mb="md">
                  Create a graph node or run your plan to materialize one.
                </Text>
                <Button
                  leftSection={<IconPlus size={16} />}
                  onClick={handleCreate}
                >
                  Create Graph Node
                </Button>
              </div>
            </Stack>
          ) : (
            <SimpleGrid cols={{ base: 1, sm: 1, md: 2, xl: 3 }} spacing="lg">
              {graphNodes.map((graph) => {
                const executionState = graph.executionState || 'NOT_STARTED'
                const isRunning =
                  executingGraphId === graph.id || isExecutionInProgress(executionState)
                const nodeType = nodeTypeMap.get(graph.nodeId) || 'GraphNode'
                const lastUpdated = graph.updatedAt ? formatDateTime(graph.updatedAt) : '—'
                const layerCount = graph.layers?.length ?? 0

                return (
                  <Card key={graph.id} withBorder shadow="xs" padding="lg" radius="md">
                    <Stack gap="sm">
                      <Group justify="space-between" align="flex-start">
                        <div>
                          <Text fw={600}>{graph.name}</Text>
                          <Group gap="xs" mt={4}>
                            <Badge variant="dot" color="cyan">
                              {nodeType}
                            </Badge>
                          </Group>
                        </div>
                        <Badge
                          variant="light"
                          color={getExecutionStateColor(executionState)}
                          leftSection={getExecutionStateIcon(executionState)}
                        >
                          {getExecutionStateLabel(executionState)}
                        </Badge>
                      </Group>

                      <Group gap="xs" wrap="wrap">
                        <Badge variant="light" color="blue">
                          Nodes: {graph.nodeCount}
                        </Badge>
                        <Badge variant="light" color="grape">
                          Edges: {graph.edgeCount}
                        </Badge>
                        <Badge variant="light" color="teal">
                          Layers: {layerCount}
                        </Badge>
                      </Group>
                      <Text size="sm" c="dimmed">
                        Last updated {lastUpdated}
                      </Text>

                      <Group gap="sm" wrap="wrap">
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconGraph size={14} />}
                          onClick={() =>
                            navigate(`/projects/${projectId}/plan-nodes/${graph.id}/edit`)
                          }
                        >
                          Open Graph Editor
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconChartDots size={14} />}
                          onClick={() => {
                            setPreviewGraphId(graph.id)
                            setPreviewTitle(`Graph Preview: ${graph.name}`)
                          }}
                        >
                          Preview
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconTable size={14} />}
                          onClick={() => setDataDialogGraphId(graph.id)}
                        >
                          View Data
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconEdit size={14} />}
                          onClick={() => handleEdit(graph)}
                        >
                          Edit Properties
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          leftSection={<IconRefresh size={14} />}
                          onClick={() => handleReprocess(graph)}
                          disabled={isRunning}
                          loading={executingGraphId === graph.id}
                        >
                          Reprocess
                        </Button>
                        <Button
                          size="xs"
                          variant="light"
                          color="red"
                          leftSection={<IconTrash size={14} />}
                          onClick={() => handleDelete(graph)}
                        >
                          Delete
                        </Button>
                      </Group>
                    </Stack>
                  </Card>
                )
              })}
            </SimpleGrid>
          )}
        </Card>
      </PageContainer>

      <Modal
        opened={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="Delete Graph Node"
      >
        <Text mb="md">
          Are you sure you want to delete "{selectedGraph?.name}"? This action cannot be undone.
        </Text>
        <Group justify="flex-end" gap="sm">
          <Button variant="light" onClick={() => setDeleteModalOpen(false)}>
            Cancel
          </Button>
          <Button
            color="red"
            loading={deleteLoading}
            onClick={confirmDelete}
          >
            Delete
          </Button>
        </Group>
      </Modal>

      <Modal
        opened={editModalOpen}
        onClose={() => setEditModalOpen(false)}
        title={selectedGraph ? 'Edit Graph Node' : 'Create Graph Node'}
      >
        <EditGraphForm
          graph={selectedGraph}
          onSave={handleSave}
          onCancel={() => setEditModalOpen(false)}
          loading={createLoading || updateLoading}
        />
      </Modal>

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
      <Stack>
        <TextInput
          label="Name"
          value={name}
          onChange={(e) => setName(e.currentTarget.value)}
          required
        />
        <Group justify="flex-end" mt="md">
          <Button variant="light" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" loading={loading}>
            Save
          </Button>
        </Group>
      </Stack>
    </form>
  )
}
