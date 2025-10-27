import React, { useState } from 'react'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  Title,
  Text,
  Group,
  Stack,
  Button,
  Card,
  Table,
  Badge,
  Menu,
  ActionIcon,
  Alert,
  Modal,
  TextInput,
  Textarea,
  LoadingOverlay,
  FileInput,
  Divider,
} from '@mantine/core'
import {
  IconPlus,
  IconDots,
  IconTrash,
  IconRefresh,
  IconEdit,
  IconFileDownload,
  IconAlertCircle,
  IconDatabaseImport,
} from '@tabler/icons-react'
import { useForm } from '@mantine/form'
import PageContainer from '../layout/PageContainer'
import { DataSourceUploader } from '../datasources/DataSourceUploader'
import {
  GET_LIBRARY_SOURCES,
  DELETE_LIBRARY_SOURCE,
  REPROCESS_LIBRARY_SOURCE,
  UPDATE_LIBRARY_SOURCE,
  LibrarySource,
  UpdateLibrarySourceInput,
  SEED_LIBRARY_SOURCES,
  SeedLibrarySourcesResult,
  formatFileSize,
  getDataTypeDisplayName,
  getFileFormatDisplayName,
  getStatusColor,
  detectFileFormat,
} from '../../graphql/librarySources'
import { showErrorNotification, showSuccessNotification } from '../../utils/notifications'

const fileToBase64 = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      const base64 = result.split(',')[1]
      resolve(base64)
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })

export const LibrarySourcesPage: React.FC = () => {
  const [uploaderOpen, setUploaderOpen] = useState(false)
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [selectedSource, setSelectedSource] = useState<LibrarySource | null>(null)
  const [replacementFile, setReplacementFile] = useState<File | null>(null)
  const [editError, setEditError] = useState<string | null>(null)

  const { data, loading, error, refetch } = useQuery(GET_LIBRARY_SOURCES, {
    fetchPolicy: 'cache-and-network'
  })

  const [deleteLibrarySource, { loading: deleteLoading, error: deleteError }] = useMutation(
    DELETE_LIBRARY_SOURCE
  )
  const [reprocessLibrarySource, { loading: reprocessLoading }] = useMutation(
    REPROCESS_LIBRARY_SOURCE
  )
  const [updateLibrarySource, { loading: updateLoading }] = useMutation(UPDATE_LIBRARY_SOURCE)
  const [seedLibrarySourcesMutation, { loading: seedLoading }] = useMutation<{
    seedLibrarySources: SeedLibrarySourcesResult
  }>(SEED_LIBRARY_SOURCES)

  const librarySources: LibrarySource[] = (data as any)?.librarySources || []

  const form = useForm({
    initialValues: {
      name: '',
      description: ''
    },
    validate: {
      name: (value) => (value.trim().length > 0 ? null : 'Name is required')
    }
  })

  const openEditModal = (source: LibrarySource) => {
    setSelectedSource(source)
    form.setValues({
      name: source.name,
      description: source.description || ''
    })
    setReplacementFile(null)
    setEditError(null)
    setEditModalOpen(true)
  }

  const openDeleteModal = (source: LibrarySource) => {
    setSelectedSource(source)
    setDeleteModalOpen(true)
  }

  const handleReprocess = async (source: LibrarySource) => {
    try {
      await reprocessLibrarySource({ variables: { id: source.id } })
      await refetch()
    } catch (err) {
      console.error('Failed to reprocess library source', err)
    }
  }

  const handleDelete = async () => {
    if (!selectedSource) return

    try {
      await deleteLibrarySource({ variables: { id: selectedSource.id } })
      setDeleteModalOpen(false)
      setSelectedSource(null)
      await refetch()
    } catch (err) {
      console.error('Failed to delete library source', err)
    }
  }

  const handleSeedLibrary = async () => {
    try {
      const { data } = await seedLibrarySourcesMutation()
      const result: SeedLibrarySourcesResult | undefined = data?.seedLibrarySources

      if (result) {
        const summary = `${result.createdCount} added, ${result.skippedCount} skipped out of ${result.totalRemoteFiles}`
        showSuccessNotification('Library seeded', summary)

        if (result.failedFiles && result.failedFiles.length > 0) {
          showErrorNotification(
            'Some files could not be imported',
            result.failedFiles.join('\n')
          )
        }
      } else {
        showSuccessNotification('Library seeded', 'No new files were added.')
      }

      await refetch()
    } catch (err: any) {
      console.error('Failed to seed library', err)
      showErrorNotification('Failed to seed library', err?.message || 'Unknown error')
    }
  }

  const handleEditSubmit = async (values: { name: string; description: string }) => {
    if (!selectedSource) return
    setEditError(null)

    try {
      let fileContent: string | undefined
      let filename: string | undefined

      if (replacementFile) {
        const detectedFormat = detectFileFormat(replacementFile.name)
        if (!detectedFormat) {
          setEditError('Unsupported file format. Please provide CSV, TSV, or JSON.')
          return
        }

        fileContent = await fileToBase64(replacementFile)
        filename = replacementFile.name
      }

      const input: UpdateLibrarySourceInput = {
        name: values.name,
        description: values.description || undefined,
        ...(fileContent ? { filename, fileContent } : {})
      }

      await updateLibrarySource({
        variables: {
          id: selectedSource.id,
          input
        }
      })

      setEditModalOpen(false)
      setSelectedSource(null)
      setReplacementFile(null)
      await refetch()
    } catch (err: any) {
      console.error('Failed to update library source', err)
      setEditError(err?.message || 'Failed to update library source')
    }
  }

  const handleDownloadJson = (source: LibrarySource) => {
    const blob = new Blob([source.graphJson], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = `${source.name}_graph.json`
    document.body.appendChild(anchor)
    anchor.click()
    document.body.removeChild(anchor)
    URL.revokeObjectURL(url)
  }

  const busy = loading || deleteLoading || reprocessLoading || updateLoading || seedLoading

  return (
    <PageContainer>
      <Stack gap="lg">
        <Group justify="space-between" align="flex-start">
          <div>
            <Title order={2}>Library Sources</Title>
            <Text size="sm" c="dimmed">
              Manage reusable data sources that can be imported into any project.
            </Text>
          </div>
          <Group gap="xs">
            <Button
              variant="light"
              leftSection={<IconDatabaseImport size={16} />}
              loading={seedLoading}
              onClick={handleSeedLibrary}
            >
              Seed library
            </Button>
            <Button
              leftSection={<IconPlus size={16} />}
              onClick={() => setUploaderOpen(true)}
            >
              Add Library Source
            </Button>
          </Group>
        </Group>

        <Card withBorder>
          <LoadingOverlay visible={busy} />
          <Stack gap="md">
            {(error || deleteError) && (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Unable to load library sources"
                color="red"
              >
                {error?.message || deleteError?.message}
              </Alert>
            )}

            {librarySources.length === 0 && !loading ? (
              <Stack align="center" py="xl" gap="xs">
                <Text fw={500}>No library sources yet</Text>
                <Text size="sm" c="dimmed" ta="center" style={{ maxWidth: 360 }}>
                  Add datasources here to share them across projects. They can be imported into any
                  project without re-uploading.
                </Text>
              </Stack>
            ) : (
              <Table highlightOnHover verticalSpacing="sm">
                <Table.Thead>
                  <Table.Tr>
                    <Table.Th>Name</Table.Th>
                    <Table.Th>Format</Table.Th>
                    <Table.Th>Data Type</Table.Th>
                    <Table.Th>Status</Table.Th>
                    <Table.Th>Processed</Table.Th>
                    <Table.Th>File Size</Table.Th>
                    <Table.Th></Table.Th>
                  </Table.Tr>
                </Table.Thead>
                <Table.Tbody>
                  {librarySources.map((source) => (
                    <Table.Tr key={source.id}>
                      <Table.Td>
                        <Stack gap={2}>
                          <Text fw={500}>{source.name}</Text>
                          {source.description && (
                            <Text size="sm" c="dimmed">
                              {source.description}
                            </Text>
                          )}
                        </Stack>
                      </Table.Td>
                      <Table.Td>{getFileFormatDisplayName(source.fileFormat)}</Table.Td>
                      <Table.Td>{getDataTypeDisplayName(source.dataType)}</Table.Td>
                      <Table.Td>
                        <Badge color={getStatusColor(source.status)} variant="light">
                          {source.status === 'processing'
                            ? 'Processing'
                            : source.status === 'error'
                              ? 'Error'
                              : 'Active'}
                        </Badge>
                        {source.status === 'error' && source.errorMessage && (
                          <Text size="xs" c="red" mt={4}>
                            {source.errorMessage}
                          </Text>
                        )}
                      </Table.Td>
                      <Table.Td>
                        {source.processedAt
                          ? new Date(source.processedAt).toLocaleString()
                          : '—'}
                      </Table.Td>
                      <Table.Td>{formatFileSize(source.fileSize)}</Table.Td>
                      <Table.Td width={60}>
                        <Menu withinPortal position="bottom-end" shadow="sm">
                          <Menu.Target>
                            <ActionIcon variant="subtle">
                              <IconDots size={16} />
                            </ActionIcon>
                          </Menu.Target>
                          <Menu.Dropdown>
                            <Menu.Item
                              leftSection={<IconEdit size={14} />}
                              onClick={() => openEditModal(source)}
                            >
                              Edit details
                            </Menu.Item>
                            <Menu.Item
                              leftSection={<IconRefresh size={14} />}
                              onClick={() => handleReprocess(source)}
                            >
                              Reprocess
                            </Menu.Item>
                            <Menu.Item
                              leftSection={<IconFileDownload size={14} />}
                              onClick={() => handleDownloadJson(source)}
                            >
                              Download JSON
                            </Menu.Item>
                            <Divider my={4} />
                            <Menu.Item
                              color="red"
                              leftSection={<IconTrash size={14} />}
                              onClick={() => openDeleteModal(source)}
                            >
                              Delete
                            </Menu.Item>
                          </Menu.Dropdown>
                        </Menu>
                      </Table.Td>
                    </Table.Tr>
                  ))}
                </Table.Tbody>
              </Table>
            )}
          </Stack>
        </Card>
      </Stack>

      <DataSourceUploader
        mode="library"
        opened={uploaderOpen}
        onClose={() => setUploaderOpen(false)}
        onSuccess={() => refetch()}
      />

      <Modal
        opened={editModalOpen}
        onClose={() => setEditModalOpen(false)}
        title="Edit Library Source"
        size="lg"
      >
        <form onSubmit={form.onSubmit(handleEditSubmit)}>
          <Stack gap="md">
            <TextInput
              label="Name"
              placeholder="Library source name"
              required
              {...form.getInputProps('name')}
            />
            <Textarea
              label="Description"
              placeholder="Optional description"
              rows={3}
              {...form.getInputProps('description')}
            />
            <FileInput
              label="Replace file"
              placeholder="Upload a new file (optional)"
              description="Upload a CSV, TSV, or JSON file to replace the existing data."
              value={replacementFile}
              onChange={setReplacementFile}
              accept=".csv,.tsv,.json"
              leftSection={<IconFileDownload size={16} />}
            />
            {editError && (
              <Alert icon={<IconAlertCircle size={16} />} color="red">
                {editError}
              </Alert>
            )}
            <Group justify="flex-end">
              <Button variant="light" onClick={() => setEditModalOpen(false)}>
                Cancel
              </Button>
              <Button type="submit" loading={updateLoading}>
                Save changes
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>

      <Modal
        opened={deleteModalOpen}
        onClose={() => setDeleteModalOpen(false)}
        title="Delete Library Source"
      >
        <Text mb="md">
          Are you sure you want to delete{' '}
          <Text span fw={600}>
            {selectedSource?.name}
          </Text>
          ? This cannot be undone.
        </Text>
        <Group justify="flex-end">
          <Button variant="light" onClick={() => setDeleteModalOpen(false)}>
            Cancel
          </Button>
          <Button color="red" onClick={handleDelete} loading={deleteLoading}>
            Delete
          </Button>
        </Group>
      </Modal>
    </PageContainer>
  )
}
