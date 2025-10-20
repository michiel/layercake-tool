import React, { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Title,
  Group,
  Button,
  Stack,
  Card,
  Text,
  ActionIcon,
  Modal,
  Alert,
  Table,
  LoadingOverlay,
  TextInput,
  Badge,
  Menu
} from '@mantine/core'
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
import { gql } from '@apollo/client'
import { Breadcrumbs } from '../common/Breadcrumbs'
import PageContainer from '../layout/PageContainer'
import { Graph, GET_GRAPHS, CREATE_GRAPH, UPDATE_GRAPH, DELETE_GRAPH, EXECUTE_NODE } from '../../graphql/graphs'
import { getExecutionStateColor, getExecutionStateLabel, isExecutionInProgress } from '../../graphql/preview'
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
          currentPage="Plan Nodes"
          onNavigate={handleNavigate}
        />

        <Group justify="space-between" mb="md">
          <div>
            <Title order={1}>Plan Nodes</Title>
            <Text size="sm" c="dimmed" mt="xs">
              Review every plan node and track execution progress across datasources and graphs
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

        <Card withBorder>
          <LoadingOverlay visible={loading} />
          {planNodes.length === 0 && !loading ? (
            <Stack align="center" py="xl" gap="md">
              <IconGraph size={48} color="gray" />
              <div style={{ textAlign: 'center' }}>
                <Title order={3}>No Plan Nodes</Title>
                <Text c="dimmed" mb="md">
                  Define plan nodes to see data source and graph execution details.
                </Text>
                <Button
                  leftSection={<IconPlus size={16} />}
                  onClick={handleCreate}
                >
                  Create First Graph Node
                </Button>
              </div>
            </Stack>
          ) : (
            <Table.ScrollContainer minWidth={900}>
              <Table striped highlightOnHover>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Name</Table.Th>
                    <Table.Th>Node Type</Table.Th>
                    <Table.Th>Status</Table.Th>
                    <Table.Th>Nodes</Table.Th>
                    <Table.Th>Edges</Table.Th>
                    <Table.Th>Layers</Table.Th>
                    <Table.Th>Updated</Table.Th>
                    <Table.Th>Actions</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
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
                      <Table.Tr key={planNode.nodeId}>
                        <Table.Td>
                          <Text fw={500}>{planNode.label}</Text>
                        </Table.Td>
                        <Table.Td>
                          <Badge variant="dot" color="cyan">
                            {nodeTypeMap.get(planNode.nodeId) || nodeType}
                          </Badge>
                        </Table.Td>
                        <Table.Td>
                          <Badge
                            variant="light"
                            color={getExecutionStateColor(executionState)}
                            leftSection={getExecutionStateIcon(executionState)}
                          >
                            {getExecutionStateLabel(executionState)}
                          </Badge>
                        </Table.Td>
                        <Table.Td>
                          <Badge variant="light" color="blue">
                            {nodeCount}
                          </Badge>
                        </Table.Td>
                        <Table.Td>
                          <Badge variant="light" color="grape">
                            {edgeCount}
                          </Badge>
                        </Table.Td>
                        <Table.Td>
                          <Badge variant="light" color="teal">
                            {layerCount}
                          </Badge>
                        </Table.Td>
                        <Table.Td>
                          <Text size="sm" c="dimmed">
                            {updatedDisplay}
                          </Text>
                        </Table.Td>
                        <Table.Td>
                          {graph ? (
                            <Menu shadow="md" width={220}>
                              <Menu.Target>
                                <ActionIcon variant="subtle">
                                  <IconDots size={16} />
                                </ActionIcon>
                              </Menu.Target>
                              <Menu.Dropdown>
                                <Menu.Item
                                  leftSection={<IconGraph size={14} />}
                                  onClick={() =>
                                    navigate(`/projects/${projectId}/plan-nodes/${graph.id}/edit`)
                                  }
                                >
                                  Open Graph Editor
                                </Menu.Item>
                                <Menu.Item
                                  leftSection={<IconEdit size={14} />}
                                  onClick={() => handleEdit(graph)}
                                >
                                  Rename
                                </Menu.Item>
                                <Menu.Item
                                  leftSection={<IconRefresh size={14} />}
                                  onClick={() => handleReprocess(graph)}
                                  disabled={isRunning}
                                >
                                  Reprocess
                                </Menu.Item>
                                <Menu.Divider />
                                <Menu.Item
                                  leftSection={<IconTrash size={14} />}
                                  color="red"
                                  onClick={() => handleDelete(graph)}
                                >
                                  Delete
                                </Menu.Item>
                              </Menu.Dropdown>
                            </Menu>
                          ) : (
                            <Text size="sm" c="dimmed">
                              No actions
                            </Text>
                          )}
                        </Table.Td>
                      </Table.Tr>
                    )
                  })}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
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
