import React, { useCallback, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'
import { IconDatabase, IconRefresh, IconTrash, IconUpload, IconDownload, IconEdit } from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog'
import { Separator } from '../components/ui/separator'
import { Input } from '../components/ui/input'
import { Label } from '../components/ui/label'
import { Switch } from '../components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table'
import { Spinner } from '../components/ui/spinner'
import { Textarea } from '../components/ui/textarea'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { showSuccessNotification } from '../utils/notifications'
import { handleMutationErrors } from '../utils/graphqlHelpers'

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

const GET_KNOWLEDGE_BASE = gql`
  query KnowledgeBaseStatus($projectId: Int!) {
    knowledgeBaseStatus(projectId: $projectId) {
      projectId
      fileCount
      chunkCount
      status
      lastIndexedAt
      embeddingProvider
      embeddingModel
    }
  }
`

const RUN_KB_COMMAND = gql`
  mutation RunKbCommand($input: KnowledgeBaseCommandInput!) {
    runKnowledgeBaseCommand(input: $input)
  }
`

const SOURCE_MANAGEMENT_QUERY = gql`
  query SourceManagement($projectId: Int!, $fileScope: String) {
    dataAcquisitionFiles(projectId: $projectId) {
      id
      filename
      mediaType
      sizeBytes
      checksum
      createdAt
      tags
      indexed
    }
    dataAcquisitionTags(scope: $fileScope) {
      id
      name
      scope
      color
    }
  }
`

const INGEST_FILE = gql`
  mutation IngestFile($input: IngestFileInput!) {
    ingestFile(input: $input) {
      fileId
      checksum
      chunkCount
      indexed
    }
  }
`

const UPDATE_FILE = gql`
  mutation UpdateIngestedFile($input: UpdateIngestedFileInput!) {
    updateIngestedFile(input: $input) {
      id
      filename
      mediaType
      sizeBytes
      checksum
      createdAt
      tags
      indexed
    }
  }
`

const DELETE_FILE = gql`
  mutation DeleteFile($input: DeleteFileInput!) {
    deleteFile(input: $input)
  }
`

const TOGGLE_FILE_INDEX = gql`
  mutation ToggleFileIndex($input: ToggleFileIndexInput!) {
    toggleFileIndex(input: $input)
  }
`

const GET_FILE_CONTENT = gql`
  mutation GetFileContent($input: GetFileContentInput!) {
    getFileContent(input: $input) {
      filename
      mediaType
      content
    }
  }
`

type KnowledgeBaseStatusData = {
  projectId: number
  fileCount: number
  chunkCount: number
  status: string
  lastIndexedAt?: string | null
  embeddingProvider?: string | null
  embeddingModel?: string | null
}

type ProjectFile = {
  id: string
  filename: string
  mediaType: string
  sizeBytes: number
  checksum: string
  createdAt: string
  tags: string[]
  indexed: boolean
}

type TagOption = {
  id: string
  name: string
  scope: string
  color?: string | null
}

interface SourceManagementResponse {
  dataAcquisitionFiles: ProjectFile[]
  dataAcquisitionTags: TagOption[]
}

const formatBytes = (value: number) => {
  if (!Number.isFinite(value)) return '0 B'
  if (value === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  const index = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1)
  return `${(value / Math.pow(1024, index)).toFixed(1)} ${units[index]}`
}

const formatDate = (value?: string | null) => {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

export const KnowledgeBasePage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{ id: number; name: string }>
  }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find(
    (p: any) => p.id === numericProjectId,
  )

  const [tagInput, setTagInput] = useState('')
  const [indexImmediately, setIndexImmediately] = useState(true)
  const [mutationError, setMutationError] = useState<string | null>(null)
  const [editingFile, setEditingFile] = useState<ProjectFile | null>(null)
  const [editFilename, setEditFilename] = useState('')
  const [editTags, setEditTags] = useState('')

  const { data, refetch } = useQuery<{ knowledgeBaseStatus?: KnowledgeBaseStatusData | null }>(
    GET_KNOWLEDGE_BASE,
    {
      variables: { projectId: numericProjectId },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const { data: sourceData, loading: sourceLoading, refetch: refetchSource } = useQuery<SourceManagementResponse>(
    SOURCE_MANAGEMENT_QUERY,
    {
      variables: { projectId: numericProjectId, fileScope: 'file' },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const [runKbCommand, { loading: kbMutating }] = useMutation(RUN_KB_COMMAND)
  const [ingestFile, { loading: ingesting }] = useMutation(INGEST_FILE)
  const [updateFile, { loading: updating }] = useMutation(UPDATE_FILE)
  const [deleteFile] = useMutation(DELETE_FILE)
  const [toggleFileIndex] = useMutation(TOGGLE_FILE_INDEX)
  const [getFileContent] = useMutation(GET_FILE_CONTENT)

  const kbStatus = data?.knowledgeBaseStatus
  const files = useMemo(() => sourceData?.dataAcquisitionFiles ?? [], [sourceData])

  const triggerKbCommand = useCallback(
    async (action: 'REBUILD' | 'CLEAR') => {
      if (!Number.isFinite(numericProjectId)) return
      setMutationError(null)
      const result = await runKbCommand({
        variables: {
          input: {
            projectId: numericProjectId,
            action,
          },
        },
        errorPolicy: 'all',
      })

      if (handleMutationErrors(result, 'Knowledge base operation failed')) {
        setMutationError('Knowledge base operation failed.')
        return
      }

      showSuccessNotification(
        action === 'REBUILD'
          ? 'Knowledge base rebuild started'
          : 'Knowledge base cleared',
      )
      await refetch()
    },
    [numericProjectId, refetch, runKbCommand],
  )

  const handleUpload = useCallback(
    async (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0]
      if (!file || !Number.isFinite(numericProjectId)) {
        return
      }
      setMutationError(null)

      const tagsList = tagInput
        .split(',')
        .map((tag) => tag.trim())
        .filter(Boolean)
      const result = await ingestFile({
        variables: {
          input: {
            projectId: numericProjectId,
            filename: file.name,
            mediaType: file.type || 'application/octet-stream',
            file,
            tags: tagsList,
            indexImmediately,
          },
        },
      })

      if (handleMutationErrors(result, 'Failed to upload file')) {
        event.target.value = ''
        return
      }

      showSuccessNotification('File uploaded', `${file.name} queued for ingestion`)
      event.target.value = ''
      await Promise.all([refetchSource(), refetch()])
    },
    [ingestFile, numericProjectId, tagInput, indexImmediately, refetchSource, refetch],
  )

  const handleDownload = useCallback(
    async (file: ProjectFile) => {
      const result = await getFileContent({
        variables: {
          input: {
            projectId: numericProjectId,
            fileId: file.id,
          },
        },
        errorPolicy: 'all',
      })

      if (handleMutationErrors(result, 'Failed to fetch file')) {
        return
      }

      const payload = (result.data as any)?.getFileContent
      if (!payload) return

      const link = document.createElement('a')
      link.href = `data:${payload.mediaType || 'application/octet-stream'};base64,${payload.content}`
      link.download = payload.filename || file.filename
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
    },
    [getFileContent, numericProjectId],
  )

  const openEditDialog = (file: ProjectFile) => {
    setEditingFile(file)
    setEditFilename(file.filename)
    setEditTags(file.tags.join(', '))
  }

  const handleToggleIndex = async (file: ProjectFile, checked: boolean) => {
    const result = await toggleFileIndex({
      variables: {
        input: {
          projectId: numericProjectId,
          fileId: file.id,
          indexed: checked,
        },
      },
      errorPolicy: 'all',
    })
    if (handleMutationErrors(result, 'Failed to toggle indexing')) {
      return
    }
    await refetchSource()
  }

  const handleDelete = async (file: ProjectFile) => {
    if (!window.confirm(`Delete ${file.filename}?`)) return
    const result = await deleteFile({
      variables: {
        input: {
          projectId: numericProjectId,
          fileId: file.id,
        },
      },
      errorPolicy: 'all',
    })
    if (handleMutationErrors(result, 'Failed to delete file')) {
      return
    }
    await Promise.all([refetchSource(), refetch()])
  }

  const handleSaveEdit = async () => {
    if (!editingFile) return
    const tagsList = editTags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)

    const result = await updateFile({
      variables: {
        input: {
          projectId: numericProjectId,
          fileId: editingFile.id,
          filename: editFilename,
          tags: tagsList,
        },
      },
      errorPolicy: 'all',
    })

    if (handleMutationErrors(result, 'Failed to update file')) {
      return
    }

    showSuccessNotification('File updated', editFilename)
    setEditingFile(null)
    await refetchSource()
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
            No project context found. Please open a project before accessing knowledge base tools.
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
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to Projects
        </Button>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        sections={breadcrumbSections}
        currentPage="Knowledge Base"
        onNavigate={(path) => navigate(path)}
      />

      <Stack gap="lg">
        <Stack gap="xs">
          <h1 className="text-2xl font-bold">Knowledge Base</h1>
          <p className="text-muted-foreground">
            Monitor embeddings, rebuild the knowledge base, and ensure provider/model consistency.
          </p>
        </Stack>

        {mutationError && (
          <Alert variant="destructive">
            <AlertDescription>{mutationError}</AlertDescription>
        </Alert>
      )}

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconUpload className="h-5 w-5 text-primary" />
              Upload documents
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 md:grid-cols-2">
              <div className="flex flex-col gap-2">
                <Label htmlFor="upload-tags">Default tags (comma separated)</Label>
                <Input
                  id="upload-tags"
                  placeholder="compliance, storage"
                  value={tagInput}
                  onChange={(event) => setTagInput(event.target.value)}
                />
              </div>
              <div className="flex items-center gap-3">
                <Switch
                  id="upload-index-immediately"
                  checked={indexImmediately}
                  onCheckedChange={setIndexImmediately}
                />
                <Label htmlFor="upload-index-immediately">
                  Index immediately after upload
                </Label>
              </div>
            </div>

            <Input
              type="file"
              accept=".txt,.md,.csv,.pdf,.docx,.xlsx,.ods,.odt"
              onChange={handleUpload}
              disabled={ingesting || sourceLoading}
            />
            <p className="text-xs text-muted-foreground">
              Supported formats: txt, markdown, csv, pdf, odf/ods, xlsx, docx.
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconDatabase className="h-5 w-5 text-primary" />
              Knowledge Base Status
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <p className="text-sm text-muted-foreground">Files</p>
                <p className="text-2xl font-semibold">
                  {kbStatus?.fileCount ?? 0}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Chunks</p>
                <p className="text-2xl font-semibold">
                  {kbStatus?.chunkCount ?? 0}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Status</p>
                <Badge variant="secondary" className="mt-1">
                  {kbStatus?.status ?? 'uninitialized'}
                </Badge>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Last Indexed</p>
                <p className="text-sm">
                  {formatDate(kbStatus?.lastIndexedAt)}
                </p>
              </div>
            </div>

            <Separator className="my-2" />

            <div className="grid gap-4 sm:grid-cols-2">
              <div>
                <p className="text-sm text-muted-foreground">Embedding Provider</p>
                <p className="text-sm font-medium">
                  {kbStatus?.embeddingProvider ?? '—'}
                </p>
              </div>
              <div>
                <p className="text-sm text-muted-foreground">Embedding Model</p>
                <p className="text-sm font-medium">
                  {kbStatus?.embeddingModel ?? '—'}
                </p>
              </div>
            </div>

            <Group gap="md">
              <Button
                variant="default"
                onClick={() => triggerKbCommand('REBUILD')}
                disabled={kbMutating}
              >
                <IconRefresh className="mr-2 h-4 w-4" />
                Rebuild
              </Button>
              <Button
                variant="outline"
                onClick={() => triggerKbCommand('CLEAR')}
                disabled={kbMutating}
              >
                <IconTrash className="mr-2 h-4 w-4" />
                Clear
              </Button>
            </Group>
          </CardContent>
        </Card>

      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <IconDatabase className="h-5 w-5 text-primary" />
            Uploaded files
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-muted-foreground">Recent uploads</h3>
            {(sourceLoading || ingesting || updating) && <Spinner className="h-4 w-4" />}
          </div>
          <div className="rounded-md border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Name</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Size</TableHead>
                  <TableHead>Tags</TableHead>
                  <TableHead>Index</TableHead>
                  <TableHead>Uploaded</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {files.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center text-sm text-muted-foreground">
                      No files uploaded yet.
                    </TableCell>
                  </TableRow>
                )}
                {files.map((file) => (
                  <TableRow key={file.id}>
                    <TableCell className="font-medium">{file.filename}</TableCell>
                    <TableCell className="text-muted-foreground">{file.mediaType}</TableCell>
                    <TableCell>{formatBytes(file.sizeBytes)}</TableCell>
                    <TableCell>
                      <div className="flex flex-wrap gap-1">
                        {file.tags.length === 0 && (
                          <span className="text-xs text-muted-foreground">—</span>
                        )}
                        {file.tags.map((tag) => (
                          <Badge key={`${file.id}-${tag}`} variant="secondary">
                            {tag}
                          </Badge>
                        ))}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Switch
                        checked={file.indexed}
                        onCheckedChange={(checked) => handleToggleIndex(file, checked)}
                        aria-label="Toggle file indexing"
                      />
                    </TableCell>
                    <TableCell>{formatDate(file.createdAt)}</TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => handleDownload(file)}
                          title="Download file"
                          aria-label="Download file"
                        >
                          <IconDownload className="h-4 w-4" />
                        </Button>
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => openEditDialog(file)}
                          title="Edit metadata"
                          aria-label="Edit metadata"
                        >
                          <IconEdit className="h-4 w-4" />
                        </Button>
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => handleDelete(file)}
                          title="Delete file"
                          aria-label="Delete file"
                        >
                          <IconTrash className="h-4 w-4 text-red-600" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      </Stack>

      <Dialog open={!!editingFile} onOpenChange={(open) => !open && setEditingFile(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit file metadata</DialogTitle>
          </DialogHeader>
          <Stack gap="md">
            <div>
              <Label htmlFor="editFilename">Filename</Label>
              <Input
                id="editFilename"
                value={editFilename}
                onChange={(e) => setEditFilename(e.target.value)}
              />
            </div>
            <div>
              <Label htmlFor="editTags">Tags</Label>
              <Textarea
                id="editTags"
                value={editTags}
                onChange={(e) => setEditTags(e.target.value)}
                placeholder="Comma separated tags"
              />
            </div>
          </Stack>
          <DialogFooter>
            <Button variant="secondary" onClick={() => setEditingFile(null)}>
              Cancel
            </Button>
            <Button onClick={handleSaveEdit} disabled={updating}>
              {updating && <Spinner className="mr-2 h-4 w-4" />}
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}

export default KnowledgeBasePage
