import React, { useState, useEffect } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Container,
  Title,
  Group,
  Button,
  Stack,
  Card,
  TextInput,
  Textarea,
  Badge,
  Text,
  Alert,
  LoadingOverlay,
  Tabs,
  Code,
  ScrollArea,
  ActionIcon
} from '@mantine/core'
import {
  IconArrowLeft,
  IconDeviceFloppy,
  IconFile,
  IconCode,
  IconRefresh,
  IconDownload,
  IconAlertCircle,
  IconCheck,
  IconClock,
  IconX
} from '@tabler/icons-react'
import { useForm } from '@mantine/form'
import { Breadcrumbs } from '../common/Breadcrumbs'
import {
  GET_DATASOURCE,
  UPDATE_DATASOURCE,
  REPROCESS_DATASOURCE,
  DataSource,
  UpdateDataSourceInput,
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

interface DataSourceEditorProps {}

export const DataSourceEditor: React.FC<DataSourceEditorProps> = () => {
  const navigate = useNavigate()
  const { projectId, dataSourceId } = useParams<{ projectId: string; dataSourceId: string }>()
  const [activeTab, setActiveTab] = useState<string | null>('details')
  const [fileUploadMode, setFileUploadMode] = useState(false)
  const [selectedFile, setSelectedFile] = useState<File | null>(null)

  // Query for project info
  const { data: projectsData } = useQuery<{
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

  // Query for DataSource
  const {
    data: dataSourceData,
    loading: dataSourceLoading,
    error: dataSourceError,
    refetch: refetchDataSource
  } = useQuery(GET_DATASOURCE, {
    variables: { id: parseInt(dataSourceId || '0') },
    errorPolicy: 'all'
  })

  // Mutations
  const [updateDataSource, { loading: updateLoading }] = useMutation(UPDATE_DATASOURCE)
  const [reprocessDataSource, { loading: reprocessLoading }] = useMutation(REPROCESS_DATASOURCE)

  const dataSource: DataSource | null = (dataSourceData as any)?.dataSource || null

  // Form for editing DataSource metadata
  const form = useForm({
    initialValues: {
      name: '',
      description: ''
    }
  })

  // Update form when dataSource loads
  useEffect(() => {
    if (dataSource) {
      form.setValues({
        name: dataSource.name,
        description: dataSource.description || ''
      })
    }
  }, [dataSource])

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleBack = () => {
    navigate(`/projects/${projectId}/datasources`)
  }

  const handleSave = async (values: { name: string; description: string }) => {
    if (!dataSource) return

    try {
      const input: UpdateDataSourceInput = {
        name: values.name,
        description: values.description || undefined,
        file: selectedFile || undefined
      }

      await updateDataSource({
        variables: {
          id: dataSource.id,
          input
        }
      })

      await refetchDataSource()
      setSelectedFile(null)
      setFileUploadMode(false)
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to update DataSource:', error)
      // TODO: Show error notification
    }
  }

  const handleReprocess = async () => {
    if (!dataSource) return

    try {
      await reprocessDataSource({
        variables: { id: dataSource.id }
      })
      await refetchDataSource()
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to reprocess DataSource:', error)
      // TODO: Show error notification
    }
  }

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) {
      setSelectedFile(file)
    }
  }

  const handleDownloadRaw = () => {
    if (!dataSource) return
    // TODO: Implement file download via GraphQL endpoint
    console.log('Download raw file for:', dataSource.filename)
  }

  const handleDownloadJson = () => {
    if (!dataSource) return

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
        return <IconCheck size={16} />
      case 'processing':
        return <IconClock size={16} />
      case 'error':
        return <IconX size={16} />
      default:
        return null
    }
  }

  if (dataSourceLoading) {
    return (
      <Container size="xl">
        <LoadingOverlay visible />
        <div style={{ height: '400px' }} />
      </Container>
    )
  }

  if (dataSourceError || !dataSource) {
    return (
      <Container size="xl">
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Error Loading Data Source"
          color="red"
          mb="md"
        >
          {dataSourceError?.message || 'Data Source not found'}
        </Alert>
        <Button onClick={handleBack} leftSection={<IconArrowLeft size={16} />}>
          Back to Data Sources
        </Button>
      </Container>
    )
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
    <Container size="xl">
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        currentPage={`Data Sources > ${dataSource.name}`}
        onNavigate={handleNavigate}
      />

      <Group justify="space-between" mb="md">
        <Group>
          <ActionIcon onClick={handleBack} size="lg" variant="subtle">
            <IconArrowLeft size={18} />
          </ActionIcon>
          <div>
            <Title order={1}>{dataSource.name}</Title>
            <Group gap="xs" mt="xs">
              <Badge variant="light" size="sm">
                {getDataSourceTypeDisplayName(dataSource.sourceType)}
              </Badge>
              <Badge
                variant="light"
                color={getStatusColor(dataSource.status)}
                leftSection={getStatusIcon(dataSource.status)}
              >
                {dataSource.status}
              </Badge>
            </Group>
          </div>
        </Group>

        <Group gap="sm">
          <Button
            variant="light"
            leftSection={<IconRefresh size={16} />}
            onClick={handleReprocess}
            loading={reprocessLoading}
            disabled={dataSource.status === 'processing'}
          >
            Reprocess
          </Button>
          <Button
            variant="light"
            leftSection={<IconDownload size={16} />}
            onClick={handleDownloadRaw}
          >
            Download Original
          </Button>
          <Button
            variant="light"
            leftSection={<IconDownload size={16} />}
            onClick={handleDownloadJson}
            disabled={dataSource.status !== 'active'}
          >
            Download JSON
          </Button>
        </Group>
      </Group>

      {dataSource.status === 'error' && dataSource.errorMessage && (
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Processing Error"
          color="red"
          mb="md"
        >
          {dataSource.errorMessage}
        </Alert>
      )}

      <Tabs value={activeTab} onChange={setActiveTab}>
        <Tabs.List>
          <Tabs.Tab value="details" leftSection={<IconFile size={16} />}>
            Details
          </Tabs.Tab>
          <Tabs.Tab
            value="data"
            leftSection={<IconCode size={16} />}
            disabled={dataSource.status !== 'active'}
          >
            Graph Data
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="details">
          <Card withBorder mt="md">
            <form onSubmit={form.onSubmit(handleSave)}>
              <Stack gap="md">
                <TextInput
                  label="Name"
                  placeholder="Enter data source name"
                  required
                  {...form.getInputProps('name')}
                />

                <Textarea
                  label="Description"
                  placeholder="Optional description"
                  rows={3}
                  {...form.getInputProps('description')}
                />

                <div>
                  <Text size="sm" fw={500} mb="xs">File Information</Text>
                  <Group gap="md">
                    <div>
                      <Text size="xs" c="dimmed">Filename</Text>
                      <Text size="sm" ff="monospace">{dataSource.filename}</Text>
                    </div>
                    <div>
                      <Text size="xs" c="dimmed">Size</Text>
                      <Text size="sm">{formatFileSize(dataSource.fileSize)}</Text>
                    </div>
                    <div>
                      <Text size="xs" c="dimmed">Processed</Text>
                      <Text size="sm">
                        {dataSource.processedAt
                          ? new Date(dataSource.processedAt).toLocaleString()
                          : 'Not processed'
                        }
                      </Text>
                    </div>
                  </Group>
                </div>

                <div>
                  <Group justify="space-between" align="center" mb="xs">
                    <Text size="sm" fw={500}>Replace File</Text>
                    <Button
                      variant="light"
                      size="xs"
                      onClick={() => setFileUploadMode(!fileUploadMode)}
                    >
                      {fileUploadMode ? 'Cancel' : 'Upload New File'}
                    </Button>
                  </Group>

                  {fileUploadMode && (
                    <Stack gap="sm">
                      <input
                        type="file"
                        accept=".csv,.json"
                        onChange={handleFileChange}
                        style={{ width: '100%' }}
                      />
                      {selectedFile && (
                        <Text size="sm" c="dimmed">
                          Selected: {selectedFile.name} ({formatFileSize(selectedFile.size)})
                        </Text>
                      )}
                      <Text size="xs" c="dimmed">
                        Supported formats: CSV (nodes, edges, layers) and JSON (graph format)
                      </Text>
                    </Stack>
                  )}
                </div>

                <Group justify="flex-end">
                  <Button
                    type="submit"
                    loading={updateLoading}
                    leftSection={<IconDeviceFloppy size={16} />}
                  >
                    Save Changes
                  </Button>
                </Group>
              </Stack>
            </form>
          </Card>
        </Tabs.Panel>

        <Tabs.Panel value="data">
          <Card withBorder mt="md">
            <Stack gap="md">
              <Group justify="space-between">
                <Text fw={500}>Processed Graph Data</Text>
                <Text size="sm" c="dimmed">
                  {dataSource.graphJson.length} characters
                </Text>
              </Group>

              <ScrollArea h={400}>
                <Code block>
                  {JSON.stringify(JSON.parse(dataSource.graphJson), null, 2)}
                </Code>
              </ScrollArea>

              <Text size="xs" c="dimmed">
                This is the processed graph data that will be available to Plan DAG nodes.
                The format includes nodes, edges, and layers arrays as defined by the graph schema.
              </Text>
            </Stack>
          </Card>
        </Tabs.Panel>
      </Tabs>
    </Container>
  )
}