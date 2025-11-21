import React, { useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'
import { IconSparkles } from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Stack } from '../components/layout-primitives'
import { Alert, AlertDescription } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { Textarea } from '../components/ui/textarea'
import { Spinner } from '../components/ui/spinner'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { showSuccessNotification } from '../utils/notifications'
import { handleMutationErrors } from '../utils/graphqlHelpers'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../components/ui/dialog'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '../components/ui/table'
import {
  GET_LIBRARY_ITEMS,
  IMPORT_LIBRARY_DATASETS,
  LibraryItem,
  LibraryItemType,
  formatFileSize,
  getFileFormatDisplayName,
} from '../graphql/libraryItems'

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

const GET_TAGS = gql`
  query DatasetCreationTags($projectId: Int!, $fileScope: String) {
    dataAcquisitionTags(scope: $fileScope) {
      id
      name
      scope
      color
    }
  }
`

const GENERATE_DATASET = gql`
  mutation GenerateDataset($input: DatasetGenerationInput!) {
    generateDatasetFromPrompt(input: $input) {
      datasetYaml
    }
  }
`

type TagData = {
  id: string
  name: string
  scope: string
  color?: string | null
}

interface DatasetGenerationResponse {
  generateDatasetFromPrompt?: {
    datasetYaml?: string | null
  } | null
}

export const DatasetCreationPage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [datasetPrompt, setDatasetPrompt] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [generatedDataset, setGeneratedDataset] = useState<string | null>(null)
  const [mutationError, setMutationError] = useState<string | null>(null)
  const [libraryPickerOpen, setLibraryPickerOpen] = useState(false)
  const [selectedLibraryId, setSelectedLibraryId] = useState<number | null>(null)

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{ id: number; name: string }>
  }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find(
    (p: any) => p.id === numericProjectId,
  )

  const { data, refetch } = useQuery<{
    dataAcquisitionTags: TagData[]
  }>(
    GET_TAGS,
    {
      variables: {
        projectId: numericProjectId,
        fileScope: 'file',
      },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const [generateDataset, { loading: datasetGenerating }] =
    useMutation<DatasetGenerationResponse>(GENERATE_DATASET, {
      errorPolicy: 'all',
    })

  const { data: libraryData, loading: libraryLoading, error: libraryError } = useQuery(
    GET_LIBRARY_ITEMS,
    {
      skip: !libraryPickerOpen,
      variables: { filter: { types: [LibraryItemType.DATASET] } },
      fetchPolicy: 'cache-and-network',
    },
  )
  const libraryItems: LibraryItem[] = (libraryData as any)?.libraryItems || []

  const [importLibraryDatasets, { loading: importLibraryLoading }] = useMutation(
    IMPORT_LIBRARY_DATASETS,
  )

  const tags = useMemo(() => data?.dataAcquisitionTags ?? [], [data])

  const toggleSelectedTag = (name: string) => {
    setSelectedTags((prev) =>
      prev.includes(name)
        ? prev.filter((tag) => tag !== name)
        : [...prev, name],
    )
  }

  const handleDatasetGeneration = async () => {
    if (!Number.isFinite(numericProjectId) || !datasetPrompt.trim()) return
    setMutationError(null)

    const result = await generateDataset({
      variables: {
        input: {
          projectId: numericProjectId,
          prompt: datasetPrompt,
          tagNames: selectedTags,
        },
      },
    })

    if (handleMutationErrors(result, 'Dataset generation failed')) {
      setMutationError('Dataset generation failed.')
      return
    }

    const yaml = result.data?.generateDatasetFromPrompt?.datasetYaml ?? null
    setGeneratedDataset(yaml)

    if (yaml) {
      showSuccessNotification('Dataset generated', 'Review the YAML output below.')
    }
    await refetch()
  }

  const handleImportDatasetFromLibrary = async () => {
    if (!selectedLibraryId || !Number.isFinite(numericProjectId)) {
      return
    }

    try {
      await importLibraryDatasets({
        variables: {
          input: {
            projectId: numericProjectId,
            libraryItemIds: [selectedLibraryId],
          },
        },
      })
      showSuccessNotification('Dataset imported', 'The library dataset was added to this project.')
      setLibraryPickerOpen(false)
      setSelectedLibraryId(null)
    } catch (err) {
      console.error('Failed to import dataset from library', err)
    }
  }

  const breadcrumbSections = useMemo(() => {
    if (!selectedProject) {
      return undefined
    }
    return [
      {
        title: 'Data acquisition',
        href: `/projects/${selectedProject.id}/datasets`,
      },
    ]
  }, [selectedProject])

  if (!Number.isFinite(numericProjectId)) {
    return (
      <PageContainer>
        <Alert variant="destructive">
          <AlertDescription>
            No project context found. Please open a project before accessing dataset tools.
          </AlertDescription>
        </Alert>
      </PageContainer>
    )
  }

  if (projectsLoading && !selectedProject) {
    return (
      <PageContainer>
        <p>Loading project…</p>
      </PageContainer>
    )
  }

  if (!selectedProject) {
    return (
      <PageContainer>
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <p className="text-muted-foreground mt-2">The requested project could not be found.</p>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        sections={breadcrumbSections}
        currentPage="Data Set Creation"
      />

      <Stack gap="lg">
        <Stack gap="xs">
          <h1 className="text-2xl font-bold">Data Set Creation</h1>
          <p className="text-muted-foreground">
            Use Retrieval-Augmented Generation to create structured datasets from the knowledge base.
          </p>
        </Stack>
        <Button variant="outline" className="w-fit" onClick={() => setLibraryPickerOpen(true)}>
          Import from Library
        </Button>

        {mutationError && (
          <Alert variant="destructive">
            <AlertDescription>{mutationError}</AlertDescription>
          </Alert>
        )}

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconSparkles className="h-5 w-5 text-primary" />
              Dataset Generator
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="dataset-prompt">Prompt</Label>
              <Textarea
                id="dataset-prompt"
                rows={6}
                value={datasetPrompt}
                onChange={(event) => setDatasetPrompt(event.target.value)}
                placeholder="Describe the desired dataset..."
              />
            </div>
            <div className="space-y-2">
              <Label>Tag Filters</Label>
              <div className="flex flex-wrap gap-2">
                {tags.length === 0 && (
                  <p className="text-sm text-muted-foreground">
                    No tags defined yet.
                  </p>
                )}
                {tags.map((tag: TagData) => (
                  <Badge
                    key={tag.id}
                    variant={
                      selectedTags.includes(tag.name) ? 'default' : 'outline'
                    }
                    className="cursor-pointer"
                    onClick={() => toggleSelectedTag(tag.name)}
                  >
                    {tag.name}
                  </Badge>
                ))}
              </div>
            </div>
            <Button
              onClick={handleDatasetGeneration}
              disabled={datasetGenerating || !datasetPrompt.trim()}
            >
              {datasetGenerating && <Spinner className="mr-2 h-4 w-4" />}
              Generate Dataset
            </Button>
            <div className="space-y-2">
              <Label>Generated Dataset</Label>
              <Textarea
                readOnly
                value={generatedDataset ?? ''}
                placeholder="Generated YAML will appear here."
                rows={8}
              />
            </div>
          </CardContent>
        </Card>
      </Stack>

      <Dialog
        open={libraryPickerOpen}
        onOpenChange={(open) => {
          setLibraryPickerOpen(open)
          if (!open) {
            setSelectedLibraryId(null)
          }
        }}
      >
        <DialogContent className="sm:max-w-[720px]">
          <DialogHeader>
            <DialogTitle>Select Library Dataset</DialogTitle>
          </DialogHeader>
          {libraryError && (
            <Alert variant="destructive">
              <AlertDescription>{libraryError.message}</AlertDescription>
            </Alert>
          )}
          <div className="max-h-[360px] overflow-y-auto border rounded-md">
            {libraryLoading ? (
              <div className="flex items-center justify-center py-8 text-muted-foreground">
                <Spinner className="mr-2 h-4 w-4" /> Loading library datasets…
              </div>
            ) : libraryItems.length === 0 ? (
              <div className="py-8 text-center text-muted-foreground">
                No datasets available in the shared library.
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-12" />
                    <TableHead>Name</TableHead>
                    <TableHead>Format</TableHead>
                    <TableHead>Size</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {libraryItems.map((item) => {
                    const metadata = item.metadata || {}
                    const format = metadata.format || metadata.file_format || 'csv'
                    return (
                      <TableRow
                        key={item.id}
                        className="cursor-pointer"
                        onClick={() => setSelectedLibraryId(item.id)}
                      >
                        <TableCell>
                          <input
                            type="radio"
                            checked={selectedLibraryId === item.id}
                            onChange={() => setSelectedLibraryId(item.id)}
                          />
                        </TableCell>
                        <TableCell>
                          <div className="font-medium">{item.name}</div>
                          <div className="text-sm text-muted-foreground">{item.description}</div>
                        </TableCell>
                        <TableCell>{getFileFormatDisplayName(format)}</TableCell>
                        <TableCell>
                          {item.contentSize ? formatFileSize(item.contentSize) : '—'}
                        </TableCell>
                      </TableRow>
                    )
                  })}
                </TableBody>
              </Table>
            )}
          </div>
          <DialogFooter>
            <Button variant='secondary' onClick={() => setLibraryPickerOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleImportDatasetFromLibrary}
              disabled={!selectedLibraryId || importLibraryLoading}
            >
              {importLibraryLoading && <Spinner className="mr-2 h-4 w-4" />}
              Import Selected
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}

export default DatasetCreationPage
