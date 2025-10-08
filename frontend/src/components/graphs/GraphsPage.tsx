import React, { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Container,
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
  TextInput
} from '@mantine/core'
import {
  IconPlus,
  IconGraph,
  IconEdit,
  IconTrash,
  IconAlertCircle
} from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { Graph, GET_GRAPHS, CREATE_GRAPH, UPDATE_GRAPH, DELETE_GRAPH } from '../../graphql/graphs'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`

interface GraphsPageProps {}

export const GraphsPage: React.FC<GraphsPageProps> = () => {
  const navigate = useNavigate();
  const { projectId } = useParams<{ projectId: string }>();
  const [deleteModalOpen, setDeleteModalOpen] = useState(false);
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [selectedGraph, setSelectedGraph] = useState<Graph | null>(null);

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS);
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'));

  const { data, loading, error } = useQuery<{ graphs: Graph[] }>(GET_GRAPHS, {
    variables: { projectId: parseInt(projectId || '0') },
    fetchPolicy: 'cache-and-network'
  });

  const [createGraph, { loading: createLoading }] = useMutation(CREATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  });

  const [updateGraph, { loading: updateLoading }] = useMutation(UPDATE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  });

  const [deleteGraph, { loading: deleteLoading }] = useMutation(DELETE_GRAPH, {
    refetchQueries: [{ query: GET_GRAPHS, variables: { projectId: parseInt(projectId || '0') } }]
  });

  const graphs: Graph[] = data?.graphs || [];

  const handleNavigate = (route: string) => {
    navigate(route);
  };

  const handleCreate = () => {
    setSelectedGraph(null);
    setEditModalOpen(true);
  };

  const handleEdit = (graph: Graph) => {
    setSelectedGraph(graph);
    setEditModalOpen(true);
  };

  const handleDelete = (graph: Graph) => {
    setSelectedGraph(graph);
    setDeleteModalOpen(true);
  };

  const confirmDelete = async () => {
    if (selectedGraph) {
      await deleteGraph({ variables: { id: selectedGraph.id } });
      setDeleteModalOpen(false);
      setSelectedGraph(null);
    }
  };

  const handleSave = async (values: { name: string }) => {
    if (selectedGraph) {
      await updateGraph({ variables: { id: selectedGraph.id, input: { name: values.name } } });
    } else {
      // For creation, we need to generate a nodeId internally or derive it.
      // For now, we'll use a placeholder. This will be handled by the backend.
      await createGraph({ variables: { input: { name: values.name, projectId: parseInt(projectId || '0'), nodeId: 'generated-node-id' } } });
    }
    setEditModalOpen(false);
    setSelectedGraph(null);
  };

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    )
  }

  return (
    <>
      <Container size="xl">
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
          <Button
            leftSection={<IconPlus size={16} />}
            onClick={handleCreate}
          >
            New Graph
          </Button>
        </Group>

        {error && (
          <Alert icon={<IconAlertCircle size={16} />} title="Error" color="red" mb="md">
            {error.message}
          </Alert>
        )}

        <Card withBorder>
          <LoadingOverlay visible={loading} />
          {graphs.length === 0 && !loading ? (
            <Stack align="center" py="xl" gap="md">
              <IconGraph size={48} color="gray" />
              <div style={{ textAlign: 'center' }}>
                <Title order={3}>No Graphs</Title>
                <Text c="dimmed" mb="md">
                  Create your first graph.
                </Text>
                <Button
                  leftSection={<IconPlus size={16} />}
                  onClick={handleCreate}
                >
                  Create First Graph
                </Button>
              </div>
            </Stack>
          ) : (
            <Table.ScrollContainer minWidth={800}>
              <Table striped highlightOnHover>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Name</Table.Th>
                    <Table.Th>Node ID</Table.Th>
                    <Table.Th>Execution State</Table.Th>
                    <Table.Th>Nodes</Table.Th>
                    <Table.Th>Edges</Table.Th>
                    <Table.Th>Layers</Table.Th>
                    <Table.Th>Created</Table.Th>
                    <Table.Th>Updated</Table.Th>
                    <Table.Th>Actions</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {graphs.map((graph) => (
                    <Table.Tr key={graph.id}>
                      <Table.Td>{graph.name}</Table.Td>
                      <Table.Td>{graph.nodeId}</Table.Td>
                      <Table.Td>{graph.executionState}</Table.Td>
                      <Table.Td>{graph.nodeCount}</Table.Td>
                      <Table.Td>{graph.edgeCount}</Table.Td>
                      <Table.Td>{graph.layers.length}</Table.Td>
                      <Table.Td>{new Date(graph.createdAt).toLocaleDateString()}</Table.Td>
                      <Table.Td>{new Date(graph.updatedAt).toLocaleDateString()}</Table.Td>
                      <Table.Td>
                        <Group gap="xs">
                          <ActionIcon onClick={() => navigate(`/projects/${projectId}/graphs/${graph.id}/edit`)}><IconGraph size={16} /></ActionIcon>
                          <ActionIcon onClick={() => handleEdit(graph)}><IconEdit size={16} /></ActionIcon>
                          <ActionIcon onClick={() => handleDelete(graph)} color="red"><IconTrash size={16} /></ActionIcon>
                        </Group>
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            </Table.ScrollContainer>
          )}
        </Card>
      </Container>

      <Modal
        opened={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="Delete Graph"
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
        title={selectedGraph ? 'Edit Graph' : 'Create Graph'}
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