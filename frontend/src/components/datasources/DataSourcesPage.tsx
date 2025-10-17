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
  LoadingOverlay,
  Checkbox
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
  IconX,
  IconFileExport,
  IconFileUpload
} from '@tabler/icons-react'
import { useQuery as useProjectsQuery } from '@apollo/client/react'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { DataSourceUploader } from './DataSourceUploader'
import { BulkDataSourceUploader } from './BulkDataSourceUploader'
import {
  GET_DATASOURCES,
  DELETE_DATASOURCE,
  REPROCESS_DATASOURCE,
  EXPORT_DATASOURCES,
  IMPORT_DATASOURCES,
  DataSource,
  formatFileSize,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
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
  const [bulkUploaderOpen, setBulkUploaderOpen] = useState(false)
  const [selectedRows, setSelectedRows] = useState<Set<number>>(new Set())
  const [exportFormatModalOpen, setExportFormatModalOpen] = useState(false)
  const [importModalOpen, setImportModalOpen] = useState(false)

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
  const [exportDataSources] = useMutation(EXPORT_DATASOURCES)
  const [importDataSources] = useMutation(IMPORT_DATASOURCES)

  const dataSources: DataSource[] = (dataSourcesData as any)?.dataSources || []

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleCreateNew = () => {
    setUploaderOpen(true)
  }

  const handleBulkUpload = () => {
    setBulkUploaderOpen(true)
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

  const toggleRowSelection = (id: number) => {
    setSelectedRows((prev) => {
      const newSet = new Set(prev)
      if (newSet.has(id)) {
        newSet.delete(id)
      } else {
        newSet.add(id)
      }
      return newSet
    })
  }

  const toggleSelectAll = () => {
    if (selectedRows.size === dataSources.length) {
      setSelectedRows(new Set())
    } else {
      setSelectedRows(new Set(dataSources.map(ds => ds.id)))
    }
  }

  const handleExportClick = () => {
    setExportFormatModalOpen(true)
  }

  const handleExport = async (format: 'xlsx' | 'ods') => {
    const selectedDataSources = dataSources.filter(ds => selectedRows.has(ds.id))
    console.log('Exporting datasources:', selectedDataSources.map(ds => ds.id), 'as', format)

    try {
      const result = await exportDataSources({
        variables: {
          input: {
            projectId: parseInt(projectId || '0'),
            dataSourceIds: Array.from(selectedRows),
            format: format.toUpperCase()
          }
        }
      })

      const data = (result.data as any)?.exportDataSources
      if (data) {
        // Decode base64 to binary
        const binaryString = atob(data.fileContent)
        const bytes = new Uint8Array(binaryString.length)
        for (let i = 0; i < binaryString.length; i++) {
          bytes[i] = binaryString.charCodeAt(i)
        }

        // Create blob and download
        const blob = new Blob([bytes], {
          type: format === 'xlsx'
            ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
            : 'application/vnd.oasis.opendocument.spreadsheet'
        })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = data.filename
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)
        URL.revokeObjectURL(url)
      }

      setExportFormatModalOpen(false)
      setSelectedRows(new Set()) // Clear selection after successful export
      alert(`Successfully exported ${selectedRows.size} datasource${selectedRows.size !== 1 ? 's' : ''} to ${format.toUpperCase()}`)
    } catch (error) {
      console.error('Failed to export datasources:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      alert(`Failed to export datasources: ${errorMessage}`)
    }
  }

  const handleImportClick = () => {
    setImportModalOpen(true)
  }

  const handleImport = async (file: File) => {
    console.log('Importing file:', file.name)

    try {
      // Read file as ArrayBuffer then convert to base64
      const reader = new FileReader()
      reader.onload = async (e) => {
        const arrayBuffer = e.target?.result as ArrayBuffer

        // Convert ArrayBuffer to base64
        const bytes = new Uint8Array(arrayBuffer)
        let binary = ''
        for (let i = 0; i < bytes.byteLength; i++) {
          binary += String.fromCharCode(bytes[i])
        }
        const base64 = btoa(binary)

        console.log('File read successfully, size:', arrayBuffer.byteLength, 'bytes')

        const result = await importDataSources({
          variables: {
            input: {
              projectId: parseInt(projectId || '0'),
              fileContent: base64,
              filename: file.name
            }
          }
        })

        const data = (result.data as any)?.importDataSources
        if (data) {
          console.log(`Imported: ${data.createdCount} created, ${data.updatedCount} updated`)
          await refetchDataSources()
          const message = `Successfully imported datasources:\n` +
            `• ${data.createdCount} created\n` +
            `• ${data.updatedCount} updated`
          alert(message)
          setImportModalOpen(false)
        }
      }
      reader.readAsArrayBuffer(file)
    } catch (error) {
      console.error('Failed to import datasources:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      alert(`Failed to import datasources: ${errorMessage}`)
    }
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
              Manage CSV, TSV, and JSON files that serve as input data for your Plan DAGs
            </Text>
          </div>
          <Group gap="xs">
            <Button
              leftSection={<IconFileExport size={16} />}
              onClick={handleExportClick}
              disabled={selectedRows.size === 0}
              variant="light"
            >
              Export ({selectedRows.size})
            </Button>
            <Button
              leftSection={<IconFileUpload size={16} />}
              onClick={handleImportClick}
              variant="light"
            >
              Import
            </Button>
            <Button
              leftSection={<IconPlus size={16} />}
              onClick={handleCreateNew}
              variant="light"
            >
              Upload Single File
            </Button>
            <Button
              leftSection={<IconPlus size={16} />}
              onClick={handleBulkUpload}
            >
              Bulk Upload
            </Button>
          </Group>
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
                  Upload CSV, TSV, or JSON files to create your first data source.
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
                    <Table.Th style={{ width: 40 }}>
                      <Checkbox
                        checked={selectedRows.size === dataSources.length && dataSources.length > 0}
                        indeterminate={selectedRows.size > 0 && selectedRows.size < dataSources.length}
                        onChange={toggleSelectAll}
                      />
                    </Table.Th>
                    <Table.Th>Name</Table.Th>
                    <Table.Th>Format</Table.Th>
                    <Table.Th>Data Type</Table.Th>
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
                        <Checkbox
                          checked={selectedRows.has(dataSource.id)}
                          onChange={() => toggleRowSelection(dataSource.id)}
                        />
                      </Table.Td>
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
                        <Badge variant="light" color="blue" size="sm">
                          {getFileFormatDisplayName(dataSource.fileFormat)}
                        </Badge>
                      </Table.Td>
                      <Table.Td>
                        <Badge variant="light" color="green" size="sm">
                          {getDataTypeDisplayName(dataSource.dataType)}
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

      {/* Bulk DataSource Uploader Modal */}
      <BulkDataSourceUploader
        projectId={parseInt(projectId || '0')}
        opened={bulkUploaderOpen}
        onClose={() => setBulkUploaderOpen(false)}
        onSuccess={() => {
          console.log('Bulk upload completed')
          refetchDataSources()
        }}
      />

      {/* Export Format Selection Modal */}
      <Modal
        opened={exportFormatModalOpen}
        onClose={() => setExportFormatModalOpen(false)}
        title="Export Data Sources"
      >
        <Text mb="md">
          Select the format for exporting {selectedRows.size} data source{selectedRows.size !== 1 ? 's' : ''}:
        </Text>

        <Stack gap="sm">
          <Button
            fullWidth
            leftSection={<IconFileExport size={16} />}
            onClick={() => handleExport('xlsx')}
          >
            Export as XLSX (Excel)
          </Button>
          <Button
            fullWidth
            leftSection={<IconFileExport size={16} />}
            onClick={() => handleExport('ods')}
            variant="light"
          >
            Export as ODS (OpenDocument)
          </Button>
        </Stack>
      </Modal>

      {/* Import Data Sources Modal */}
      <Modal
        opened={importModalOpen}
        onClose={() => setImportModalOpen(false)}
        title="Import Data Sources"
      >
        <Text mb="md">
          Upload an XLSX or ODS file containing data sources. Each sheet will be imported as a data source.
          If a sheet name matches an existing data source name in this project, it will update that data source.
          Otherwise, a new data source will be created.
        </Text>

        <input
          type="file"
          accept=".xlsx,.ods"
          onChange={(e) => {
            const file = e.target.files?.[0]
            if (file) {
              handleImport(file)
            }
          }}
          style={{ marginBottom: '1rem' }}
        />

        <Group justify="flex-end" gap="sm">
          <Button variant="light" onClick={() => setImportModalOpen(false)}>
            Cancel
          </Button>
        </Group>
      </Modal>
    </>
  )
}