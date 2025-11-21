import React, { useEffect, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
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
  IconBooks
} from '@tabler/icons-react'
import { useQuery as useProjectsQuery } from '@apollo/client/react'
import { Breadcrumbs } from '../common/Breadcrumbs'
import { DataSetUploader } from './DataSetUploader'
import PageContainer from '../layout/PageContainer'
import { Stack, Group } from '../layout-primitives'
import { Alert, AlertDescription, AlertTitle } from '../ui/alert'
import { Badge } from '../ui/badge'
import { Button } from '../ui/button'
import { Card, CardContent } from '../ui/card'
import { Checkbox } from '../ui/checkbox'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '../ui/dialog'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, DropdownMenuSeparator } from '../ui/dropdown-menu'
import { Spinner } from '../ui/spinner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table'
import {
  GET_DATASOURCES,
  DELETE_DATASOURCE,
  REPROCESS_DATASOURCE,
  EXPORT_DATASOURCES,
  DataSet,
  formatFileSize,
  getFileFormatDisplayName,
  getOriginDisplayName
} from '../../graphql/datasets'

import {
  GET_LIBRARY_ITEMS,
  IMPORT_LIBRARY_DATASETS,
  LibraryItem,
  LibraryItemType
} from '../../graphql/libraryItems'

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

interface DataSetsPageProps {}

export const DataSetsPage: React.FC<DataSetsPageProps> = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectNumericId = parseInt(projectId || '0')
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [selectedDataSet, setSelectedDataSet] = useState<DataSet | null>(null)
  const [uploaderOpen, setUploaderOpen] = useState(false)
  const [selectedRows, setSelectedRows] = useState<Set<number>>(new Set())
  const [exportFormatModalOpen, setExportFormatModalOpen] = useState(false)
  const [libraryImportModalOpen, setLibraryImportModalOpen] = useState(false)
  const [selectedLibraryRows, setSelectedLibraryRows] = useState<Set<number>>(new Set())
  const [librarySelectionError, setLibrarySelectionError] = useState<string | null>(null)

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
  const selectedProject = projects.find(p => p.id === projectNumericId)

  // Query for DataSets
  const {
    data: dataSourcesData,
    loading: dataSourcesLoading,
    error: dataSourcesError,
    refetch: refetchDataSets
  } = useQuery(GET_DATASOURCES, {
    variables: { projectId: projectNumericId },
    errorPolicy: 'all',
    fetchPolicy: 'cache-and-network'
  })

  const {
    data: libraryItemsData,
    loading: libraryItemsLoading,
    error: libraryItemsError,
    refetch: refetchLibraryItems
  } = useQuery(GET_LIBRARY_ITEMS, {
    skip: !libraryImportModalOpen,
    variables: { filter: { types: [LibraryItemType.DATASET] } },
    fetchPolicy: 'cache-and-network'
  })

  // Mutations
  const [deleteDataSet, { loading: deleteLoading }] = useMutation(DELETE_DATASOURCE)
  const [reprocessDataSet, { loading: reprocessLoading }] = useMutation(REPROCESS_DATASOURCE)
  const [exportDataSets] = useMutation(EXPORT_DATASOURCES)
  const [importLibraryDatasets, { loading: libraryImportLoading, error: libraryImportError }] =
    useMutation(IMPORT_LIBRARY_DATASETS)

  const dataSources: DataSet[] = (dataSourcesData as any)?.dataSets || []
  const libraryItems: LibraryItem[] = (libraryItemsData as any)?.libraryItems || []

  useEffect(() => {
    if (!libraryImportModalOpen) {
      setSelectedLibraryRows(new Set())
      setLibrarySelectionError(null)
      return
    }

    refetchLibraryItems()
  }, [libraryImportModalOpen, refetchLibraryItems])

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleCreateNew = () => {
    setUploaderOpen(true)
  }

  const handleEdit = (dataSource: DataSet) => {
    navigate(`/projects/${projectId}/datasets/${dataSource.id}/edit`)
  }

  const handleDelete = (dataSource: DataSet) => {
    setSelectedDataSet(dataSource)
    setDeleteModalOpen(true)
  }

  const confirmDelete = async () => {
    if (selectedDataSet) {
      try {
        await deleteDataSet({
          variables: { id: selectedDataSet.id }
        })
        await refetchDataSets()
        setDeleteModalOpen(false)
        setSelectedDataSet(null)
      } catch (error) {
        console.error('Failed to delete DataSet:', error)
        // TODO: Show error notification
      }
    }
  }

  const renderGraphContents = (dataSource: DataSet) => {
    const nodeCount = dataSource.nodeCount ?? 0
    const edgeCount = dataSource.edgeCount ?? 0
    const layerCount = dataSource.layerCount ?? 0
    const parts: { key: string; label: string }[] = []

    if (nodeCount > 0) {
      parts.push({ key: 'nodes', label: `${nodeCount.toLocaleString()} nodes` })
    }
    if (edgeCount > 0) {
      parts.push({ key: 'edges', label: `${edgeCount.toLocaleString()} edges` })
    }
    if (layerCount > 0 || dataSource.hasLayers) {
      parts.push({
        key: 'layers',
        label:
          layerCount > 0
            ? `${layerCount.toLocaleString()} layers`
            : 'Layers available'
      })
    }

    if (parts.length === 0) {
      return [
        <Badge key="empty" variant="outline" className="bg-transparent">
          Empty
        </Badge>
      ]
    }

    return parts.map((part) => (
      <Badge key={part.key} variant="outline" className="bg-transparent">
        {part.label}
      </Badge>
    ))
  }

  const handleReprocess = async (dataSource: DataSet) => {
    try {
      await reprocessDataSet({
        variables: { id: dataSource.id }
      })
      await refetchDataSets()
      // TODO: Show success notification
    } catch (error) {
      console.error('Failed to reprocess DataSet:', error)
      // TODO: Show error notification
    }
  }

  const handleDownloadRaw = (dataSource: DataSet) => {
    // TODO: Implement file download via GraphQL endpoint
    console.log('Download raw file for:', dataSource.filename)
  }

  const handleDownloadJson = (dataSource: DataSet) => {
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

  const toggleLibraryRowSelection = (id: number) => {
    setSelectedLibraryRows((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const toggleLibrarySelectAll = () => {
    if (selectedLibraryRows.size === libraryItems.length) {
      setSelectedLibraryRows(new Set())
    } else {
      setSelectedLibraryRows(new Set(libraryItems.map(item => item.id)))
    }
  }

  const handleOpenLibraryImport = () => {
    setLibrarySelectionError(null)
    setSelectedLibraryRows(new Set())
    setLibraryImportModalOpen(true)
  }

  const handleImportFromLibrary = async () => {
    if (selectedLibraryRows.size === 0) {
      setLibrarySelectionError('Select at least one library item to import')
      return
    }

    if (!Number.isFinite(projectNumericId)) {
      setLibrarySelectionError('Project context is missing or invalid')
      return
    }

    try {
      await importLibraryDatasets({
        variables: {
          input: {
            projectId: projectNumericId,
            libraryItemIds: Array.from(selectedLibraryRows)
          }
        }
      })

      await refetchDataSets()
      setLibraryImportModalOpen(false)
      setSelectedLibraryRows(new Set())
      setLibrarySelectionError(null)
    } catch (err) {
      console.error('Failed to import library items', err)
    }
  }

  const handleExportClick = () => {
    setExportFormatModalOpen(true)
  }

  const handleExport = async (format: 'xlsx' | 'ods') => {
    const selectedDataSets = dataSources.filter(ds => selectedRows.has(ds.id))
    console.log('Exporting datasets:', selectedDataSets.map(ds => ds.id), 'as', format)

    try {
      const result = await exportDataSets({
        variables: {
          input: {
            projectId: projectNumericId,
            dataSetIds: Array.from(selectedRows),
            format: format.toUpperCase()
          }
        }
      })

      const data = (result.data as any)?.exportDataSets
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
      alert(`Successfully exported ${selectedRows.size} dataset${selectedRows.size !== 1 ? 's' : ''} to ${format.toUpperCase()}`)
    } catch (error) {
      console.error('Failed to export datasets:', error)
      const errorMessage = error instanceof Error ? error.message : 'Unknown error'
      alert(`Failed to export datasets: ${errorMessage}`)
    }
  }

  const getStatusIcon = (status: DataSet['status']) => {
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
          sections={[{ title: 'Data management', href: `/projects/${selectedProject.id}/datasets` }]}
          currentPage="Data sets"
          onNavigate={handleNavigate}
        />

        <Group justify="between" className="mb-4">
          <div>
            <h1 className="text-3xl font-bold">Data sets</h1>
            <p className="text-sm text-muted-foreground mt-1">
              Manage CSV, TSV, and JSON files that serve as input data for your Plan DAGs
            </p>
          </div>
          <Group gap="xs">
            <Button
              onClick={handleExportClick}
              disabled={selectedRows.size === 0}
              variant="secondary"
            >
              <IconFileExport className="mr-2 h-4 w-4" />
              Export ({selectedRows.size})
            </Button>
            <Button
              onClick={handleOpenLibraryImport}
              variant="secondary"
            >
              <IconBooks className="mr-2 h-4 w-4" />
              Import from Library
            </Button>
            <Button onClick={handleCreateNew}>
              <IconPlus className="mr-2 h-4 w-4" />
              New
            </Button>
          </Group>
        </Group>

        {dataSourcesError && (
          <Alert variant="destructive" className="mb-4">
            <IconAlertCircle className="h-4 w-4" />
            <AlertTitle>Error Loading Data Sets</AlertTitle>
            <AlertDescription>
              {dataSourcesError.message}
            </AlertDescription>
          </Alert>
        )}

        <Card className="border relative">
          {dataSourcesLoading && (
            <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
              <Spinner className="h-8 w-8" />
            </div>
          )}

          {dataSources.length === 0 && !dataSourcesLoading ? (
            <CardContent className="py-12">
              <Stack align="center" gap="md">
                <IconFile size={48} className="text-muted-foreground" />
                <div className="text-center">
                  <h3 className="text-xl font-semibold mb-2">No Data Sets</h3>
                  <p className="text-muted-foreground mb-4">
                    Upload CSV, TSV, or JSON files to create your first data set.
                  </p>
                  <Button onClick={handleCreateNew}>
                    <IconPlus className="mr-2 h-4 w-4" />
                    Create First Data Set
                  </Button>
                </div>
              </Stack>
            </CardContent>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-[40px]">
                      <Checkbox
                        checked={selectedRows.size === dataSources.length && dataSources.length > 0}
                        onCheckedChange={toggleSelectAll}
                      />
                    </TableHead>
                    <TableHead>Name</TableHead>
                    <TableHead>Origin</TableHead>
                    <TableHead>Contents</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Size</TableHead>
                    <TableHead>Updated</TableHead>
                    <TableHead>Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {dataSources.map((dataSource) => (
                    <TableRow key={dataSource.id}>
                      <TableCell>
                        <Checkbox
                          checked={selectedRows.has(dataSource.id)}
                          onCheckedChange={() => toggleRowSelection(dataSource.id)}
                        />
                      </TableCell>
                      <TableCell>
                        <div>
                          <p className="font-medium">{dataSource.name}</p>
                          {dataSource.description && (
                            <p className="text-xs text-muted-foreground mt-0.5">
                              {dataSource.description}
                            </p>
                          )}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge variant="secondary">
                          {getOriginDisplayName(dataSource.origin)}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
                          {renderGraphContents(dataSource)}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Group gap="xs">
                          <Badge
                            variant="secondary"
                            className={
                              dataSource.status === 'active'
                                ? 'bg-green-100 text-green-900'
                                : dataSource.status === 'processing'
                                  ? 'bg-blue-100 text-blue-900'
                                  : 'bg-red-100 text-red-900'
                            }
                          >
                            {getStatusIcon(dataSource.status)}
                            <span className="ml-1">{dataSource.status}</span>
                          </Badge>
                          {dataSource.status === 'error' && dataSource.errorMessage && (
                            <Button
                              size="icon"
                              variant="ghost"
                              className="h-6 w-6 text-red-600"
                              title={dataSource.errorMessage}
                            >
                              <IconAlertCircle className="h-3 w-3" />
                            </Button>
                          )}
                        </Group>
                      </TableCell>
                      <TableCell>
                        <p className="text-sm">
                          {formatFileSize(dataSource.fileSize)}
                        </p>
                      </TableCell>
                      <TableCell>
                        <p className="text-sm text-muted-foreground">
                          {new Date(dataSource.updatedAt).toLocaleString()}
                        </p>
                      </TableCell>
                      <TableCell>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon">
                              <IconDots className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>

                          <DropdownMenuContent align="end" className="w-[200px]">
                            <DropdownMenuItem onClick={() => handleEdit(dataSource)}>
                              <IconEdit className="mr-2 h-3.5 w-3.5" />
                              Edit
                            </DropdownMenuItem>

                            <DropdownMenuItem
                              onClick={() => handleReprocess(dataSource)}
                              disabled={dataSource.status === 'processing' || reprocessLoading}
                            >
                              <IconRefresh className="mr-2 h-3.5 w-3.5" />
                              Reprocess
                            </DropdownMenuItem>

                            <DropdownMenuSeparator />

                            <DropdownMenuItem onClick={() => handleDownloadRaw(dataSource)}>
                              <IconDownload className="mr-2 h-3.5 w-3.5" />
                              Download Original
                            </DropdownMenuItem>

                            <DropdownMenuItem
                              onClick={() => handleDownloadJson(dataSource)}
                              disabled={dataSource.status !== 'active'}
                            >
                              <IconDownload className="mr-2 h-3.5 w-3.5" />
                              Download JSON
                            </DropdownMenuItem>

                            <DropdownMenuSeparator />

                            <DropdownMenuItem
                              onClick={() => handleDelete(dataSource)}
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
            </div>
          )}
        </Card>
      </PageContainer>

      {/* Delete Confirmation Modal */}
      <Dialog open={deleteModalOpen} onOpenChange={setDeleteModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Data Set</DialogTitle>
          </DialogHeader>
          <p className="mb-4">
            Are you sure you want to delete "{selectedDataSet?.name}"?
            This action cannot be undone.
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

      {/* Import from Library Modal */}
      <Dialog open={libraryImportModalOpen} onOpenChange={setLibraryImportModalOpen}>
        <DialogContent className="sm:max-w-[800px]">
          <DialogHeader>
            <DialogTitle>Import from Library</DialogTitle>
            <DialogDescription>
              Select one or more library sources to copy into this project. Imported items appear in the project list and can be edited independently.
            </DialogDescription>
          </DialogHeader>

          <Stack gap="md" className="py-4">
            {libraryItemsError && (
              <Alert variant="destructive">
                <IconAlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {libraryItemsError.message}
                </AlertDescription>
              </Alert>
            )}

            {libraryImportError && (
              <Alert variant="destructive">
                <IconAlertCircle className="h-4 w-4" />
                <AlertDescription>
                  {libraryImportError.message}
                </AlertDescription>
              </Alert>
            )}

            {librarySelectionError && (
              <Alert className="border-orange-200 bg-orange-50 text-orange-900">
                <IconAlertCircle className="h-4 w-4 text-orange-600" />
                <AlertDescription>
                  {librarySelectionError}
                </AlertDescription>
              </Alert>
            )}
            <div className="relative">
              {(libraryItemsLoading || libraryImportLoading) && (
                <div className="absolute inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50 rounded-lg">
                  <Spinner className="h-8 w-8" />
                </div>
              )}
              {libraryItems.length === 0 && !libraryItemsLoading ? (
                <Stack align="center" className="py-8" gap="xs">
                  <p className="font-medium">No library datasets available</p>
                  <p className="text-sm text-muted-foreground text-center max-w-[360px]">
                    Add datasets from the Library page before importing them into this project.
                  </p>
                </Stack>
              ) : (
                <div className="rounded-md border overflow-hidden">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-[40px]">
                          <Checkbox
                            checked={
                              selectedLibraryRows.size === libraryItems.length &&
                              libraryItems.length > 0
                            }
                            onCheckedChange={toggleLibrarySelectAll}
                          />
                        </TableHead>
                        <TableHead>Name</TableHead>
                        <TableHead>Format</TableHead>
                        <TableHead>Updated</TableHead>
                        <TableHead>Size</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {libraryItems.map((item) => {
                        const metadata = item.metadata || {}
                        const format = metadata.format || metadata.file_format || 'csv'
                        return (
                          <TableRow key={item.id}>
                            <TableCell>
                              <Checkbox
                                checked={selectedLibraryRows.has(item.id)}
                                onCheckedChange={() => toggleLibraryRowSelection(item.id)}
                                aria-label={`Select ${item.name}`}
                              />
                            </TableCell>
                            <TableCell>
                              <Stack gap="xs">
                                <p className="font-medium">{item.name}</p>
                                {item.description && (
                                  <p className="text-sm text-muted-foreground">{item.description}</p>
                                )}
                              </Stack>
                            </TableCell>
                            <TableCell>{getFileFormatDisplayName(format)}</TableCell>
                            <TableCell>{new Date(item.updatedAt).toLocaleString()}</TableCell>
                            <TableCell>
                              {item.contentSize ? formatFileSize(item.contentSize) : '—'}
                            </TableCell>
                          </TableRow>
                        )
                      })}
                    </TableBody>
                  </Table>
                </div>
              )}
            </div>

          </Stack>

          <DialogFooter>
            <Button variant="secondary" onClick={() => setLibraryImportModalOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleImportFromLibrary}
              disabled={libraryItems.length === 0 || libraryImportLoading}
            >
              {libraryImportLoading && <Spinner className="mr-2 h-4 w-4" />}
              Import Selected ({selectedLibraryRows.size})
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* DataSet Uploader Modal */}
      <DataSetUploader
        projectId={projectNumericId}
        opened={uploaderOpen}
        onClose={() => setUploaderOpen(false)}
        onSuccess={() => {
          console.log('DataSet created')
          refetchDataSets()
        }}
      />

      {/* Export Format Selection Modal */}
      <Dialog open={exportFormatModalOpen} onOpenChange={setExportFormatModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Export Data Sets</DialogTitle>
            <DialogDescription>
              Select the format for exporting {selectedRows.size} data set{selectedRows.size !== 1 ? 's' : ''}:
            </DialogDescription>
          </DialogHeader>

          <Stack gap="sm" className="py-4">
            <Button
              className="w-full"
              onClick={() => handleExport('xlsx')}
            >
              <IconFileExport className="mr-2 h-4 w-4" />
              Export as XLSX (Excel)
            </Button>
            <Button
              className="w-full"
              onClick={() => handleExport('ods')}
              variant="secondary"
            >
              <IconFileExport className="mr-2 h-4 w-4" />
              Export as ODS (OpenDocument)
            </Button>
          </Stack>
        </DialogContent>
      </Dialog>
    </>
  )
}
