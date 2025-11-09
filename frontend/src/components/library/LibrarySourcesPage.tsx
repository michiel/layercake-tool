import React, { useState } from 'react'
import { useQuery, useMutation } from '@apollo/client/react'
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
import { useForm } from 'react-hook-form'
import PageContainer from '../layout/PageContainer'
import { DataSetUploader } from '../datasets/DataSetUploader'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, DropdownMenuSeparator } from '../ui/dropdown-menu'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Spinner } from '../ui/spinner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table'
import { Textarea } from '../ui/textarea'
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

  const form = useForm<{name: string; description: string}>({
    defaultValues: {
      name: '',
      description: ''
    }
  })

  const openEditModal = (source: LibrarySource) => {
    setSelectedSource(source)
    form.reset({
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
        <Group justify="between" align="start">
          <div>
            <h2 className="text-2xl font-bold">Library Sources</h2>
            <p className="text-sm text-muted-foreground">
              Manage reusable data sources that can be imported into any project.
            </p>
          </div>
          <Group gap="xs">
            <Button
              variant="secondary"
              onClick={handleSeedLibrary}
              disabled={seedLoading}
            >
              {seedLoading && <Spinner className="mr-2 h-4 w-4" />}
              <IconDatabaseImport className="mr-2 h-4 w-4" />
              Seed library
            </Button>
            <Button
              onClick={() => setUploaderOpen(true)}
            >
              <IconPlus className="mr-2 h-4 w-4" />
              Add Library Source
            </Button>
          </Group>
        </Group>

        <Card className="border relative">
          {busy && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
              <Spinner className="h-8 w-8" />
            </div>
          )}
          <CardContent className="pt-6">
            <Stack gap="md">
              {(error || deleteError) && (
                <Alert variant="destructive">
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertTitle>Unable to load library sources</AlertTitle>
                  <AlertDescription>
                    {error?.message || deleteError?.message}
                  </AlertDescription>
                </Alert>
              )}

              {librarySources.length === 0 && !loading ? (
                <Stack align="center" className="py-12" gap="xs">
                  <p className="font-medium">No library sources yet</p>
                  <p className="text-sm text-muted-foreground text-center max-w-sm">
                    Add datasets here to share them across projects. They can be imported into any
                    project without re-uploading.
                  </p>
                </Stack>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Name</TableHead>
                      <TableHead>Format</TableHead>
                      <TableHead>Data Type</TableHead>
                      <TableHead>Status</TableHead>
                      <TableHead>Processed</TableHead>
                      <TableHead>File Size</TableHead>
                      <TableHead className="w-[60px]"></TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                  {librarySources.map((source) => (
                    <TableRow key={source.id}>
                      <TableCell>
                        <Stack gap="xs">
                          <p className="font-medium">{source.name}</p>
                          {source.description && (
                            <p className="text-sm text-muted-foreground">
                              {source.description}
                            </p>
                          )}
                        </Stack>
                      </TableCell>
                      <TableCell>{getFileFormatDisplayName(source.fileFormat)}</TableCell>
                      <TableCell>{getDataTypeDisplayName(source.dataType)}</TableCell>
                      <TableCell>
                        <div>
                          <Badge
                            variant="secondary"
                            className={
                              source.status === 'processing'
                                ? 'bg-blue-100 text-blue-900'
                                : source.status === 'error'
                                  ? 'bg-red-100 text-red-900'
                                  : 'bg-green-100 text-green-900'
                            }
                          >
                            {source.status === 'processing'
                              ? 'Processing'
                              : source.status === 'error'
                                ? 'Error'
                                : 'Active'}
                          </Badge>
                          {source.status === 'error' && source.errorMessage && (
                            <p className="text-xs text-red-600 mt-1">
                              {source.errorMessage}
                            </p>
                          )}
                        </div>
                      </TableCell>
                      <TableCell>
                        {source.processedAt
                          ? new Date(source.processedAt).toLocaleString()
                          : '—'}
                      </TableCell>
                      <TableCell>{formatFileSize(source.fileSize)}</TableCell>
                      <TableCell>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon">
                              <IconDots className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => openEditModal(source)}>
                              <IconEdit className="mr-2 h-3.5 w-3.5" />
                              Edit details
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleReprocess(source)}>
                              <IconRefresh className="mr-2 h-3.5 w-3.5" />
                              Reprocess
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={() => handleDownloadJson(source)}>
                              <IconFileDownload className="mr-2 h-3.5 w-3.5" />
                              Download JSON
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              onClick={() => openDeleteModal(source)}
                              className="text-red-600"
                            >
                              <IconTrash className="mr-2 h-3.5 w-3.5" />
                              Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </TableCell>
                    </TableRow>
                  ))}
                  </TableBody>
                </Table>
              )}
            </Stack>
          </CardContent>
        </Card>
      </Stack>

      <DataSetUploader
        mode="library"
        opened={uploaderOpen}
        onClose={() => setUploaderOpen(false)}
        onSuccess={() => refetch()}
      />

      <Dialog open={editModalOpen} onOpenChange={setEditModalOpen}>
        <DialogContent className="sm:max-w-[600px]">
          <DialogHeader>
            <DialogTitle>Edit Library Source</DialogTitle>
          </DialogHeader>
          <form onSubmit={form.handleSubmit(handleEditSubmit)}>
            <Stack gap="md" className="py-4">
              <div className="space-y-2">
                <Label htmlFor="edit-name">Name *</Label>
                <Input
                  id="edit-name"
                  placeholder="Library source name"
                  {...form.register('name', { required: true })}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-description">Description</Label>
                <Textarea
                  id="edit-description"
                  placeholder="Optional description"
                  rows={3}
                  {...form.register('description')}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="edit-file">Replace file</Label>
                <Input
                  id="edit-file"
                  type="file"
                  accept=".csv,.tsv,.json"
                  onChange={(e) => setReplacementFile(e.target.files?.[0] || null)}
                />
                <p className="text-xs text-muted-foreground">
                  Upload a CSV, TSV, or JSON file to replace the existing data.
                </p>
              </div>
              {editError && (
                <Alert variant="destructive">
                  <IconAlertCircle className="h-4 w-4" />
                  <AlertDescription>{editError}</AlertDescription>
                </Alert>
              )}
            </Stack>
            <DialogFooter>
              <Button
                type="button"
                variant="secondary"
                onClick={() => setEditModalOpen(false)}
              >
                Cancel
              </Button>
              <Button type="submit" disabled={updateLoading}>
                {updateLoading && <Spinner className="mr-2 h-4 w-4" />}
                Save changes
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog open={deleteModalOpen} onOpenChange={setDeleteModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Library Source</DialogTitle>
          </DialogHeader>
          <p className="py-4">
            Are you sure you want to delete{' '}
            <span className="font-semibold">{selectedSource?.name}</span>
            ? This cannot be undone.
          </p>
          <DialogFooter>
            <Button
              variant="secondary"
              onClick={() => setDeleteModalOpen(false)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={deleteLoading}
            >
              {deleteLoading && <Spinner className="mr-2 h-4 w-4" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}
