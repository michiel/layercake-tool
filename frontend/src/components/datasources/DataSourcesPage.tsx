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
  Badge,
  Text,
  ActionIcon,
  Modal,
  Alert,
  Table,
  Menu,
  LoadingOverlay
} from '@mantine/core'
import {
  IconPlus,
  IconFile,
  IconDownload,
  IconEdit,
  IconTrash,
  IconRefresh,
  IconDots,
  IconAlertCircle,
  IconCheck,
  IconClock,
  IconX
} from '@tabler/icons-react'
import { useQuery as useProjectsQuery } from '@apollo/client/react'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { DataSourceUploader } from './DataSourceUploader'
import {
  GET_DATASOURCES,
  DELETE_DATASOURCE,
  REPROCESS_DATASOURCE,
  DataSource,
  formatFileSize,
  getDataSourceTypeDisplayName,
  getStatusColor
} from '../../graphql/datasources'

import { gql } from '@apollo/client'

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

interface DataSourcesPageProps {}

export const DataSourcesPage: React.FC<DataSourcesPageProps> = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [selectedDataSource, setSelectedDataSource] = useState<DataSource | null>(null)
  const [uploaderOpen, setUploaderOpen] = useState(false)

  // Query for project info
  const { data: projectsData } = useProjectsQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)
  const projects = projectsData?.projects || []
  const selectedProject = projects.find(p => p.id === parseInt(projectId || '0'))

  // Query for DataSources
  const {
    data: dataSourcesData,
    loading: dataSourcesLoading,
    error: dataSourcesError,
    refetch: refetchDataSources
  } = useQuery(GET_DATASOURCES, {
    variables: { projectId: parseInt(projectId || '0') },
    errorPolicy: 'all',
    fetchPolicy: 'cache-and-network'
  })

  // Mutations
  const [deleteDataSource, { loading: deleteLoading }] = useMutation(DELETE_DATASOURCE)
  const [reprocessDataSource, { loading: reprocessLoading }] = useMutation(REPROCESS_DATASOURCE)

  const dataSources: DataSource[] = (dataSourcesData as any)?.dataSources || []

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleCreateNew = () => {
    setUploaderOpen(true)
  }

  const handleEdit = (dataSource: DataSource) => {
    navigate(`/projects/${projectId}/datasources/${dataSource.id}/edit`)
  }

  const handleDelete = (dataSource: DataSource) => {
    setSelectedDataSource(dataSource)
    setDeleteModalOpen(true)
  }

  const confirmDelete = async () => {
    if (selectedDataSource) {
      try {
        await deleteDataSource({
          variables: { id: selectedDataSource.id }
        })
        await refetchDataSources()
        setDeleteModalOpen(false)
        setSelectedDataSource(null)
      } catch (error) {
        console.error('Failed to delete DataSource:', error)
        // TODO: Show error notification
      }
    }
  }

  const handleReprocess = async (dataSource: DataSource) => {
    try {
      await reprocessDataSource({
        variables: { id: dataSource.id }
      })
      await refetchDataSources()
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to reprocess DataSource:', error)
      // TODO: Show error notification
    }
  }

  const handleDownloadRaw = (dataSource: DataSource) => {
    // TODO: Implement file download via GraphQL endpoint
    console.log('Download raw file for:', dataSource.filename)
  }

  const handleDownloadJson = (dataSource: DataSource) => {
    // Create downloadable JSON file from graphJson
    const blob = new Blob([dataSource.graphJson], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${dataSource.name}_graph.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const getStatusIcon = (status: DataSource['status']) => {
    switch (status) {
      case 'active':
        return <IconCheck size={14} />
      case 'processing':
        return <IconClock size={14} />
      case 'error':
        return <IconX size={14} />
      default:
        return null
    }
  }

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
          currentPage="Data Sources"
          onNavigate={handleNavigate}
        />

        <Group justify="space-between" mb="md">
          <div>
            <Title order={1}>Data Sources</Title>
            <Text size="sm" c="dimmed" mt="xs">
              Manage CSV and JSON files that serve as input data for your Plan DAGs
            </Text>
          </div>
          <Button
            leftSection={<IconPlus size={16} />}
            onClick={handleCreateNew}
          >
            Upload Data Source
          </Button>
        </Group>

        {dataSourcesError && (
          <Alert
            icon={<IconAlertCircle size={16} />}
            title="Error Loading Data Sources"
            color="red"
            mb="md"
          >
            {dataSourcesError.message}
          </Alert>
        )}

        <Card withBorder>
          <LoadingOverlay visible={dataSourcesLoading} />

          {dataSources.length === 0 && !dataSourcesLoading ? (
            <Stack align="center" py="xl" gap="md">
              <IconFile size={48} color="gray" />
              <div style={{ textAlign: 'center' }}>
                <Title order={3}>No Data Sources</Title>
                <Text c="dimmed" mb="md">
                  Upload CSV or JSON files to create your first data source.
                </Text>
                <Button
                  leftSection={<IconPlus size={16} />}
                  onClick={handleCreateNew}
                >
                  Upload First Data Source
                </Button>
              </div>
            </Stack>
          ) : (
            <Table.ScrollContainer minWidth={800}>
              <Table striped highlightOnHover>
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Name</Table.Th>
                    <Table.Th>Type</Table.Th>
                    <Table.Th>Status</Table.Th>
                    <Table.Th>File</Table.Th>
                    <Table.Th>Size</Table.Th>
                    <Table.Th>Updated</Table.Th>
                    <Table.Th>Actions</Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {dataSources.map((dataSource) => (
                    <Table.Tr key={dataSource.id}>
                      <Table.Td>
                        <div>
                          <Text fw={500}>{dataSource.name}</Text>
                          {dataSource.description && (
                            <Text size="xs" c="dimmed" mt={2}>
                              {dataSource.description}
                            </Text>
                          )}
                        </div>
                      </Table.Td>
                      <Table.Td>
                        <Badge variant="light" size="sm">
                          {getDataSourceTypeDisplayName(dataSource.sourceType)}
                        </Badge>
                      </Table.Td>
                      <Table.Td>
                        <Group gap="xs">
                          <Badge
                            variant="light"
                            color={getStatusColor(dataSource.status)}
                            leftSection={getStatusIcon(dataSource.status)}
                          >
                            {dataSource.status}
                          </Badge>
                          {dataSource.status === 'error' && dataSource.errorMessage && (
                            <ActionIcon
                              size="sm"
                              variant="subtle"
                              color="red"
                              title={dataSource.errorMessage}
                            >
                              <IconAlertCircle size={12} />
                            </ActionIcon>
                          )}
                        </Group>
                      </Table.Td>
                      <Table.Td>
                        <Text size="sm" ff="monospace">
                          {dataSource.filename}
                        </Text>
                      </Table.Td>
                      <Table.Td>
                        <Text size="sm">
                          {formatFileSize(dataSource.fileSize)}
                        </Text>
                      </Table.Td>
                      <Table.Td>
                        <Text size="sm" c="dimmed">
                          {new Date(dataSource.updatedAt).toLocaleDateString()}
                        </Text>
                      </Table.Td>
                      <Table.Td>
                        <Group gap="xs">
                          <Menu shadow="md" width={200}>
                            <Menu.Target>
                              <ActionIcon variant="subtle">
                                <IconDots size={16} />
                              </ActionIcon>
                            </Menu.Target>

                            <Menu.Dropdown>
                              <Menu.Item
                                leftSection={<IconEdit size={14} />}
                                onClick={() => handleEdit(dataSource)}
                              >
                                Edit
                              </Menu.Item>

                              <Menu.Item
                                leftSection={<IconRefresh size={14} />}
                                onClick={() => handleReprocess(dataSource)}
                                disabled={dataSource.status === 'processing' || reprocessLoading}
                              >
                                Reprocess
                              </Menu.Item>

                              <Menu.Divider />

                              <Menu.Item
                                leftSection={<IconDownload size={14} />}
                                onClick={() => handleDownloadRaw(dataSource)}
                              >
                                Download Original
                              </Menu.Item>

                              <Menu.Item
                                leftSection={<IconDownload size={14} />}
                                onClick={() => handleDownloadJson(dataSource)}
                                disabled={dataSource.status !== 'active'}
                              >
                                Download JSON
                              </Menu.Item>

                              <Menu.Divider />

                              <Menu.Item
                                leftSection={<IconTrash size={14} />}
                                color="red"
                                onClick={() => handleDelete(dataSource)}
                              >
                                Delete
                              </Menu.Item>
                            </Menu.Dropdown>
                          </Menu>
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

      {/* Delete Confirmation Modal */}
      <Modal
        opened={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="Delete Data Source"
      >
        <Text mb="md">
          Are you sure you want to delete "{selectedDataSource?.name}"?
          This action cannot be undone.
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

      {/* DataSource Uploader Modal */}
      <DataSourceUploader
        projectId={parseInt(projectId || '0')}
        opened={uploaderOpen}
        onClose={() => setUploaderOpen(false)}
        onSuccess={(dataSource) => {
          console.log('DataSource created:', dataSource)
          refetchDataSources()
        }}
      />
    </>
  )
}