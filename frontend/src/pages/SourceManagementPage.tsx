import React, { useCallback, useMemo, useState } from 'react'
import { useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconDatabase,
  IconEdit,
  IconUpload,
} from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Stack } from '../components/layout-primitives'
import { Alert, AlertDescription } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog'
import { Input } from '../components/ui/input'
import { Label } from '../components/ui/label'
import { Switch } from '../components/ui/switch'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table'
import { Spinner } from '../components/ui/spinner'
import { Textarea } from '../components/ui/textarea'
import { Separator } from '../components/ui/separator'
import { showSuccessNotification } from '../utils/notifications'
import { handleMutationErrors } from '../utils/graphqlHelpers'

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
    }
  }
`

type ProjectFile = {
  id: string
  filename: string
  mediaType: string
  sizeBytes: number
  checksum: string
  createdAt: string
  tags: string[]
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

export const SourceManagementPage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN

  const [tagInput, setTagInput] = useState('')
  const [indexImmediately, setIndexImmediately] = useState(true)
  const [mutationError, setMutationError] = useState<string | null>(null)
  const [editingFile, setEditingFile] = useState<ProjectFile | null>(null)
  const [editFilename, setEditFilename] = useState('')
  const [editTags, setEditTags] = useState('')

  const { data, loading, refetch } = useQuery<SourceManagementResponse>(
    SOURCE_MANAGEMENT_QUERY,
    {
      variables: {
        projectId: numericProjectId,
        fileScope: 'file',
      },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const [ingestFile, { loading: ingesting }] = useMutation(INGEST_FILE)
  const [updateFile, { loading: updating }] = useMutation(UPDATE_FILE)

  const files = useMemo(() => data?.dataAcquisitionFiles ?? [], [data])
  const tags = useMemo(() => data?.dataAcquisitionTags ?? [], [data])

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
      await refetch()
      event.target.value = ''
    },
    [ingestFile, indexImmediately, numericProjectId, refetch, tagInput],
  )

  const openEditDialog = useCallback((file: ProjectFile) => {
    setEditingFile(file)
    setEditFilename(file.filename)
    setEditTags(file.tags.join(', '))
  }, [])

  const handleEditSave = useCallback(async () => {
    if (!editingFile || !Number.isFinite(numericProjectId)) return
    setMutationError(null)

    const tagsList = editTags
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)
    const result = await updateFile({
      variables: {
        input: {
          projectId: numericProjectId,
          fileId: editingFile.id,
          filename: editFilename.trim() || editingFile.filename,
          tags: tagsList,
        },
      },
    })

    if (handleMutationErrors(result, 'Failed to update file metadata')) {
      return
    }

    setEditingFile(null)
    showSuccessNotification('File metadata updated', 'Changes saved successfully')
    await refetch()
  }, [editFilename, editTags, editingFile, numericProjectId, refetch, updateFile])

  if (!Number.isFinite(numericProjectId)) {
    return (
      <PageContainer>
        <Alert variant="destructive">
          <AlertDescription>
            No project context found. Please open a project before accessing data acquisition tools.
          </AlertDescription>
        </Alert>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Stack gap="lg">
        <Stack gap="xs">
          <h1 className="text-2xl font-bold">Source Management</h1>
          <p className="text-muted-foreground">
            Upload files, assign metadata and tags, and control which assets feed the project knowledge base.
          </p>
        </Stack>

        {mutationError && (
          <Alert variant="destructive">
            <AlertDescription>{mutationError}</AlertDescription>
          </Alert>
        )}

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <IconUpload className="h-5 w-5 text-primary" />
              Upload Files
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
              disabled={ingesting || loading}
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
              Uploaded Files
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-muted-foreground">Recent uploads</h3>
              {(loading || ingesting || updating) && <Spinner className="h-4 w-4" />}
            </div>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Type</TableHead>
                    <TableHead>Size</TableHead>
                    <TableHead>Tags</TableHead>
                    <TableHead>Uploaded</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {files.length === 0 && (
                    <TableRow>
                      <TableCell colSpan={6} className="text-center text-sm text-muted-foreground">
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
                      <TableCell>{formatDate(file.createdAt)}</TableCell>
                      <TableCell className="text-right">
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => openEditDialog(file)}
                          className="inline-flex items-center gap-1"
                        >
                          <IconEdit className="h-4 w-4" />
                          Edit
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </CardContent>
        </Card>
      </Stack>

      <Dialog open={!!editingFile} onOpenChange={() => setEditingFile(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Edit file metadata</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="edit-name">Filename</Label>
              <Input
                id="edit-name"
                value={editFilename}
                onChange={(event) => setEditFilename(event.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="edit-tags">Tags (comma separated)</Label>
              <Textarea
                id="edit-tags"
                rows={3}
                value={editTags}
                onChange={(event) => setEditTags(event.target.value)}
              />
              {tags.length > 0 && (
                <>
                  <Separator className="my-2" />
                  <p className="text-xs text-muted-foreground">Existing tags:</p>
                  <div className="flex flex-wrap gap-1">
                    {tags.map((tag) => (
                      <Badge
                        key={tag.id}
                        variant="outline"
                        className="cursor-pointer"
                        onClick={() => {
                          const current = editTags.split(',').map((t) => t.trim()).filter(Boolean)
                          if (!current.includes(tag.name)) {
                            setEditTags(
                              current.concat(tag.name).join(', '),
                            )
                          }
                        }}
                      >
                        {tag.name}
                      </Badge>
                    ))}
                  </div>
                </>
              )}
            </div>
          </div>
          <DialogFooter className="gap-2">
            <Button variant="outline" onClick={() => setEditingFile(null)}>
              Cancel
            </Button>
            <Button onClick={handleEditSave} disabled={updating}>
              {updating && <Spinner className="mr-2 h-4 w-4" />}
              Save Changes
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}

export default SourceManagementPage
