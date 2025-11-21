import React, { useMemo, useState } from 'react'
import { useQuery, useMutation } from '@apollo/client/react'
import {
  IconFilter,
  IconDatabase,
  IconTrash,
  IconDots,
  IconDownload,
  IconSparkles,
  IconSearch,
  IconUpload,
  IconCircleCheck,
  IconEye,
} from '@tabler/icons-react'
import PageContainer from '../layout/PageContainer'
import { DataSetUploader } from '../datasets/DataSetUploader'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../ui/dialog'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu'
import { Input } from '../ui/input'
import { Label } from '../ui/label'
import { Spinner } from '../ui/spinner'
import { Textarea } from '../ui/textarea'
import { Separator } from '../ui/separator'
import {
  GET_LIBRARY_ITEMS,
  DELETE_LIBRARY_ITEM,
  CREATE_PROJECT_FROM_LIBRARY,
  SEED_LIBRARY_ITEMS,
  UPLOAD_LIBRARY_ITEM,
  UPDATE_LIBRARY_ITEM,
  REDETECT_LIBRARY_DATASET_TYPE,
  LibraryItem,
  LibraryItemType,
  UploadLibraryItemInput,
  formatFileSize,
  getFileFormatDisplayName,
  getDataTypeDisplayName,
} from '../../graphql/libraryItems'
import { showErrorNotification, showSuccessNotification } from '../../utils/notifications'

const typeFilters: { label: string; value: LibraryItemType | 'ALL' }[] = [
  { label: 'All', value: 'ALL' },
  { label: 'Datasets', value: LibraryItemType.DATASET },
  { label: 'Projects', value: LibraryItemType.PROJECT },
  { label: 'Templates', value: LibraryItemType.PROJECT_TEMPLATE },
  { label: 'Prompts', value: LibraryItemType.PROMPT },
]

const fileToBase64 = (file: File): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      const result = reader.result as string
      resolve(result.split(',')[1])
    }
    reader.onerror = reject
    reader.readAsDataURL(file)
  })

export const LibraryPage: React.FC = () => {
  const [activeType, setActiveType] = useState<LibraryItemType | 'ALL'>('ALL')
  const [searchQuery, setSearchQuery] = useState('')
  const [tagQuery, setTagQuery] = useState('')
  const [datasetUploaderOpen, setDatasetUploaderOpen] = useState(false)
  const [templateUploading, setTemplateUploading] = useState(false)
  const [projectModalItem, setProjectModalItem] = useState<LibraryItem | null>(null)
  const [newProjectName, setNewProjectName] = useState('')
  const [editItem, setEditItem] = useState<LibraryItem | null>(null)
  const [editName, setEditName] = useState('')
  const [editDescription, setEditDescription] = useState('')
  const [editTags, setEditTags] = useState('')
  const [previewItem, setPreviewItem] = useState<LibraryItem | null>(null)
  const [previewContent, setPreviewContent] = useState<string>('')
  const [previewLoading, setPreviewLoading] = useState(false)

  const filterVariables = useMemo(() => {
    const tags = tagQuery
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)

    return {
      filter: {
        types: activeType === 'ALL' ? undefined : [activeType],
        tags: tags.length ? tags : undefined,
        searchQuery: searchQuery.trim() || undefined,
      },
    }
  }, [activeType, searchQuery, tagQuery])

  const { data, loading, error, refetch } = useQuery(GET_LIBRARY_ITEMS, {
    variables: filterVariables,
    fetchPolicy: 'cache-and-network',
  })

  const items: LibraryItem[] = (data as any)?.libraryItems || []

  const [deleteLibraryItem] = useMutation(DELETE_LIBRARY_ITEM)
  const [createProjectFromLibrary, { loading: createProjectLoading }] = useMutation(
    CREATE_PROJECT_FROM_LIBRARY,
  )
  const [seedLibraryItems, { loading: seedLoading }] = useMutation(SEED_LIBRARY_ITEMS)
  const [uploadLibraryItem, { loading: uploadMutationLoading }] = useMutation(UPLOAD_LIBRARY_ITEM)
  const [updateLibraryItem, { loading: updateLibraryItemLoading }] = useMutation(UPDATE_LIBRARY_ITEM)
  const [redetectDatasetType, { loading: redetecting }] = useMutation(REDETECT_LIBRARY_DATASET_TYPE)

  const handleDownload = (item: LibraryItem) => {
    window.open(`/api/library/${item.id}/download`, '_blank')
  }

  const handleDelete = async (item: LibraryItem) => {
    if (!window.confirm(`Delete "${item.name}" from the library?`)) {
      return
    }

    try {
      await deleteLibraryItem({ variables: { id: item.id } })
      showSuccessNotification('Item removed', `"${item.name}" was deleted.`)
      await refetch()
    } catch (err: any) {
      console.error(err)
      showErrorNotification('Failed to delete item', err?.message || 'Unknown error')
    }
  }

  const handleSeed = async () => {
    try {
      const { data } = await seedLibraryItems()
      const result = (data as any)?.seedLibraryItems
      if (result) {
        const summary = `${result.createdCount} added, ${result.skippedCount} skipped`
        showSuccessNotification('Library seeded', summary)
      }
      await refetch()
    } catch (err: any) {
      console.error(err)
      showErrorNotification('Failed to seed library', err?.message || 'Unknown error')
    }
  }

  const handleTemplateUpload = async (file: File) => {
    setTemplateUploading(true)
    try {
      const base64 = await fileToBase64(file)
      const input: UploadLibraryItemInput = {
        type: LibraryItemType.PROJECT_TEMPLATE,
        name: file.name.replace(/\.zip$/i, ''),
        fileName: file.name,
        fileContent: base64,
        contentType: 'application/zip',
      }
      await uploadLibraryItem({ variables: { input } })
      showSuccessNotification('Template uploaded', file.name)
      await refetch()
    } catch (err: any) {
      console.error(err)
      showErrorNotification('Template upload failed', err?.message || 'Unknown error')
    } finally {
      setTemplateUploading(false)
    }
  }

  const openProjectModal = (item: LibraryItem) => {
    setProjectModalItem(item)
    setNewProjectName(`${item.name} Copy`)
  }

  const openEditModal = (item: LibraryItem) => {
    setEditItem(item)
    setEditName(item.name)
    setEditDescription(item.description || '')
    setEditTags(item.tags.join(', '))
  }

  const isPreviewable = (item: LibraryItem): boolean => {
    const contentType = item.metadata?.contentType
    const format = item.metadata?.format
    if (contentType === 'text/markdown' || contentType === 'text/plain') return true
    if (format === 'markdown' || format === 'text') return true
    if (item.type === LibraryItemType.PROMPT) return true
    return false
  }

  const handlePreview = async (item: LibraryItem) => {
    setPreviewItem(item)
    setPreviewContent('')
    setPreviewLoading(true)
    try {
      const response = await fetch(`/api/library/${item.id}/download`)
      if (!response.ok) throw new Error('Failed to fetch content')
      const text = await response.text()
      setPreviewContent(text)
    } catch (err: any) {
      showErrorNotification('Preview failed', err?.message || 'Unknown error')
      setPreviewItem(null)
    } finally {
      setPreviewLoading(false)
    }
  }

  const handleCreateProject = async () => {
    if (!projectModalItem) return
    try {
      await createProjectFromLibrary({
        variables: {
          libraryItemId: projectModalItem.id,
          name: newProjectName.trim() || undefined,
        },
      })
      showSuccessNotification('Project created', `"${newProjectName}" is ready.`)
      setProjectModalItem(null)
    } catch (err: any) {
      console.error(err)
      showErrorNotification('Failed to create project', err?.message || 'Unknown error')
    }
  }

  const handleRedetectType = async () => {
    if (!editItem) return
    try {
      const result = await redetectDatasetType({
        variables: { id: editItem.id },
      })
      // Update the editItem with new metadata
      const updatedItem = (result.data as any)?.redetectLibraryDatasetType
      if (updatedItem) {
        setEditItem(updatedItem)
      }
      showSuccessNotification('Type re-detected', 'Dataset type has been updated based on file content.')
      await refetch()
    } catch (err: any) {
      console.error(err)
      showErrorNotification('Failed to re-detect type', err?.message || 'Unknown error')
    }
  }

  const renderMetadata = (item: LibraryItem) => {
    const metadata = item.metadata || {}
    const format = metadata.format || metadata.file_format
    const dataType = metadata.dataType || metadata.data_type
    const details = []
    if (format) {
      details.push(getFileFormatDisplayName(format))
    }
    if (dataType) {
      details.push(getDataTypeDisplayName(dataType))
    }
    if (item.contentSize) {
      details.push(formatFileSize(item.contentSize))
    }
    return details.join(' • ')
  }

  return (
    <PageContainer>
      <Stack gap="xl">
        <Stack gap="xs">
          <h2 className="text-2xl font-semibold">Library</h2>
          <p className="text-muted-foreground">
            Browse shared datasets, example projects, and reusable templates.
          </p>
        </Stack>

        <div className="flex flex-col gap-4 lg:flex-row">
          <Card className="flex-1">
            <CardHeader>
              <CardTitle>Filters</CardTitle>
              <CardDescription>Search and narrow down library items.</CardDescription>
            </CardHeader>
            <CardContent>
              <Stack gap="md">
                <Group gap="sm" wrap>
                  {typeFilters.map((filter) => (
                    <Button
                      key={filter.label}
                      variant={activeType === filter.value ? 'default' : 'outline'}
                      onClick={() => setActiveType(filter.value)}
                    >
                      {filter.label}
                    </Button>
                  ))}
                </Group>
                <Group gap="sm" wrap>
                  <div className="flex-1 min-w-[240px]">
                    <Label className="text-xs uppercase text-muted-foreground flex items-center gap-1">
                      <IconSearch className="h-4 w-4" />
                      Search
                    </Label>
                    <Input
                      placeholder="Name or description…"
                      value={searchQuery}
                      onChange={(e) => setSearchQuery(e.target.value)}
                    />
                  </div>
                  <div className="flex-1 min-w-[240px]">
                    <Label className="text-xs uppercase text-muted-foreground flex items-center gap-1">
                      <IconFilter className="h-4 w-4" />
                      Tags
                    </Label>
                    <Input
                      placeholder="Comma separated tags"
                      value={tagQuery}
                      onChange={(e) => setTagQuery(e.target.value)}
                    />
                  </div>
                </Group>
              </Stack>
            </CardContent>
          </Card>

          <Card className="flex-1 lg:w-[360px] lg:flex-none">
            <CardHeader>
              <CardTitle>Manage</CardTitle>
              <CardDescription>Upload new assets and sync bundled samples.</CardDescription>
            </CardHeader>
            <CardContent>
              <Stack gap="md">
                <Group gap="sm" wrap>
                  <Button onClick={() => setDatasetUploaderOpen(true)}>
                    <IconDatabase className="mr-2 h-4 w-4" />
                    Upload Dataset
                  </Button>
                  <label className="inline-flex items-center gap-2">
                    <input
                      type="file"
                      accept=".zip"
                      className="hidden"
                      onChange={(event) => {
                        const file = event.target.files?.[0]
                        if (file) {
                          handleTemplateUpload(file)
                          event.target.value = ''
                        }
                      }}
                    />
                    <Button variant="outline" disabled={templateUploading || uploadMutationLoading}>
                      {(templateUploading || uploadMutationLoading) && (
                        <Spinner className="mr-2 h-4 w-4" />
                      )}
                      <IconUpload className="mr-2 h-4 w-4" />
                      Upload Template (ZIP)
                    </Button>
                  </label>
                </Group>
                <Button variant="secondary" onClick={handleSeed} disabled={seedLoading}>
                  {seedLoading && <Spinner className="mr-2 h-4 w-4" />}
                  <IconSparkles className="mr-2 h-4 w-4" />
                  Seed Samples
                </Button>
              </Stack>
            </CardContent>
          </Card>
        </div>

        {error && (
          <Alert variant="destructive">
            <AlertDescription>{error.message}</AlertDescription>
          </Alert>
        )}

        <div>
          {loading ? (
            <div className="flex items-center gap-2 text-muted-foreground">
              <Spinner className="h-5 w-5" /> Loading library…
            </div>
          ) : items.length === 0 ? (
            <Card>
              <CardContent className="py-12 text-center text-muted-foreground">
                No items match the current filters.
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
              {items.map((item) => (
                <Card key={item.id}>
                  <CardHeader className="flex flex-row items-start justify-between space-y-0">
                    <div>
                      <CardTitle className="text-lg">{item.name}</CardTitle>
                      {item.description && (
                        <CardDescription>{item.description}</CardDescription>
                      )}
                    </div>
                    <Badge variant="secondary">
                      {item.type === LibraryItemType.DATASET
                        ? 'Dataset'
                        : item.type === LibraryItemType.PROJECT
                          ? 'Project'
                          : item.type === LibraryItemType.PROMPT
                            ? 'Prompt'
                            : 'Template'}
                    </Badge>
                  </CardHeader>
                  <CardContent>
                    <Stack gap="sm">
                      <p className="text-sm text-muted-foreground">{renderMetadata(item)}</p>
                      {item.tags.length > 0 && (
                        <Group gap="xs" wrap>
                          {item.tags.map((tag) => (
                            <Badge key={tag} variant="outline">
                              {tag}
                            </Badge>
                          ))}
                        </Group>
                      )}
                      <Group gap="xs">
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleDownload(item)}
                        >
                          <IconDownload className="mr-2 h-4 w-4" />
                          Download
                        </Button>
                        {isPreviewable(item) && (
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() => handlePreview(item)}
                          >
                            <IconEye className="mr-2 h-4 w-4" />
                            Preview
                          </Button>
                        )}
                        {item.type !== LibraryItemType.DATASET && item.type !== LibraryItemType.PROMPT && (
                          <Button size="sm" onClick={() => openProjectModal(item)}>
                            <IconCircleCheck className="mr-2 h-4 w-4" />
                            Create Project
                          </Button>
                        )}
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button size="sm" variant="ghost">
                              <IconDots className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => handleDownload(item)}>
                              <IconDownload className="mr-2 h-4 w-4" /> Download
                            </DropdownMenuItem>
                            {isPreviewable(item) && (
                              <DropdownMenuItem onClick={() => handlePreview(item)}>
                                <IconEye className="mr-2 h-4 w-4" /> Preview
                              </DropdownMenuItem>
                            )}
                            <DropdownMenuItem onClick={() => openEditModal(item)}>
                              <IconSparkles className="mr-2 h-4 w-4" /> Edit
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem
                              className="text-red-600"
                              onClick={() => handleDelete(item)}
                            >
                              <IconTrash className="mr-2 h-4 w-4" /> Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </Group>
                      <p className="text-xs text-muted-foreground">
                        Updated {new Date(item.updatedAt).toLocaleString()}
                      </p>
                    </Stack>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </Stack>

      <DataSetUploader
        mode="library"
        opened={datasetUploaderOpen}
        onClose={() => setDatasetUploaderOpen(false)}
        onSuccess={() => refetch()}
      />

      <Dialog open={!!projectModalItem} onOpenChange={(open) => !open && setProjectModalItem(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New Project from Library</DialogTitle>
          </DialogHeader>
          <Stack gap="sm">
            <Label htmlFor="projectName">Project name</Label>
            <Input
              id="projectName"
              value={newProjectName}
              onChange={(e) => setNewProjectName(e.target.value)}
              placeholder="Name your project"
            />
          </Stack>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setProjectModalItem(null)}>
              Cancel
            </Button>
            <Button onClick={handleCreateProject} disabled={createProjectLoading}>
              {createProjectLoading && <Spinner className="mr-2 h-4 w-4" />}
              Create Project
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!editItem} onOpenChange={(open) => !open && setEditItem(null)}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Edit Library Item</DialogTitle>
          </DialogHeader>
          <Stack gap="md">
            <div>
              <Label htmlFor="editName">Name</Label>
              <Input
                id="editName"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
                placeholder="Item name"
              />
            </div>

            <div>
              <Label htmlFor="editDescription">Description</Label>
              <Textarea
                id="editDescription"
                value={editDescription}
                onChange={(e) => setEditDescription(e.target.value)}
                placeholder="Describe this item..."
                rows={3}
              />
            </div>

            <div>
              <Label htmlFor="editTags">Tags</Label>
              <Input
                id="editTags"
                value={editTags}
                onChange={(e) => setEditTags(e.target.value)}
                placeholder="Comma separated tags"
              />
              <p className="text-xs text-muted-foreground mt-1">
                Separate tags with commas
              </p>
            </div>

            {editItem && editItem.type === LibraryItemType.DATASET && (
              <>
                <Separator />
                <div>
                  <div className="flex items-center justify-between mb-3">
                    <h4 className="text-sm font-semibold">Dataset Metadata</h4>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={handleRedetectType}
                      disabled={redetecting}
                    >
                      {redetecting && <Spinner className="mr-2 h-3 w-3" />}
                      <IconSparkles className="mr-2 h-4 w-4" />
                      Re-detect Type
                    </Button>
                  </div>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <span className="text-muted-foreground">Format:</span>
                      <div className="font-medium">
                        {editItem.metadata?.format
                          ? getFileFormatDisplayName(editItem.metadata.format)
                          : 'Unknown'}
                      </div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Data Type:</span>
                      <div className="font-medium">
                        {editItem.metadata?.dataType
                          ? getDataTypeDisplayName(editItem.metadata.dataType)
                          : 'Unknown'}
                      </div>
                    </div>
                    {editItem.metadata?.rowCount && (
                      <div>
                        <span className="text-muted-foreground">Rows:</span>
                        <div className="font-medium">
                          {editItem.metadata.rowCount.toLocaleString()}
                        </div>
                      </div>
                    )}
                    {editItem.metadata?.columnCount && (
                      <div>
                        <span className="text-muted-foreground">Columns:</span>
                        <div className="font-medium">
                          {editItem.metadata.columnCount}
                        </div>
                      </div>
                    )}
                    {editItem.contentSize && (
                      <div>
                        <span className="text-muted-foreground">File Size:</span>
                        <div className="font-medium">
                          {formatFileSize(editItem.contentSize)}
                        </div>
                      </div>
                    )}
                    {editItem.metadata?.filename && (
                      <div className="col-span-2">
                        <span className="text-muted-foreground">Filename:</span>
                        <div className="font-medium font-mono text-xs">
                          {editItem.metadata.filename}
                        </div>
                      </div>
                    )}
                  </div>
                  {editItem.metadata?.headers && editItem.metadata.headers.length > 0 && (
                    <div className="mt-3">
                      <span className="text-muted-foreground text-sm">Headers:</span>
                      <div className="flex flex-wrap gap-1 mt-1">
                        {editItem.metadata.headers.map((header: string, idx: number) => (
                          <Badge key={idx} variant="outline" className="font-mono text-xs">
                            {header}
                          </Badge>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </>
            )}
          </Stack>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setEditItem(null)}>
              Cancel
            </Button>
            <Button
              onClick={async () => {
                if (!editItem) return
                try {
                  const tags = editTags
                    .split(',')
                    .map((t) => t.trim())
                    .filter(Boolean)
                  await updateLibraryItem({
                    variables: {
                      id: editItem.id,
                      input: {
                        name: editName.trim(),
                        description: editDescription.trim() || null,
                        tags,
                      },
                    },
                  })
                  showSuccessNotification('Item updated', 'Library item saved.')
                  setEditItem(null)
                  await refetch()
                } catch (err: any) {
                  console.error(err)
                  showErrorNotification('Failed to update item', err?.message || 'Unknown error')
                }
              }}
              disabled={updateLibraryItemLoading}
            >
              {updateLibraryItemLoading && <Spinner className="mr-2 h-4 w-4" />}
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog open={!!previewItem} onOpenChange={(open) => !open && setPreviewItem(null)}>
        <DialogContent className="max-w-3xl max-h-[90vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>Preview: {previewItem?.name}</DialogTitle>
          </DialogHeader>
          <div className="flex-1 overflow-auto min-h-0">
            {previewLoading ? (
              <div className="flex items-center justify-center py-12">
                <Spinner className="h-6 w-6" />
                <span className="ml-2 text-muted-foreground">Loading preview...</span>
              </div>
            ) : (
              <pre className="whitespace-pre-wrap text-sm font-mono bg-muted p-4 rounded-md overflow-auto max-h-[60vh]">
                {previewContent}
              </pre>
            )}
          </div>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setPreviewItem(null)}>
              Close
            </Button>
            {previewItem && (
              <Button variant="outline" onClick={() => handleDownload(previewItem)}>
                <IconDownload className="mr-2 h-4 w-4" />
                Download
              </Button>
            )}
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}
