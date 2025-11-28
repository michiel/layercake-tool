import { useState, useRef } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import {
  IconScript,
  IconPlus,
  IconTrash,
  IconEdit,
  IconDownload,
  IconUpload,
  IconFileTypeCsv,
  IconBraces,
} from '@tabler/icons-react'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Spinner } from '@/components/ui/spinner'
import { LIST_STORIES, CREATE_STORY, DELETE_STORY, IMPORT_STORY, EXPORT_STORY, Story, StoryExport } from '@/graphql/stories'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

const GET_PROJECTS = gql`
  query GetProjectsForStories {
    projects {
      id
      name
      description
    }
  }
`

export const StoriesPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = Number(projectId || 0)

  const [createModalOpen, setCreateModalOpen] = useState(false)
  const [newStoryName, setNewStoryName] = useState('')
  const [newStoryDescription, setNewStoryDescription] = useState('')
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [storyToDelete, setStoryToDelete] = useState<Story | null>(null)
  const [importModalOpen, setImportModalOpen] = useState(false)
  const [importFormat, setImportFormat] = useState<'CSV' | 'JSON'>('JSON')
  const fileInputRef = useRef<HTMLInputElement>(null)

  const { data: projectsData, loading: projectsLoading } = useQuery(GET_PROJECTS)
  const projects = (projectsData as any)?.projects || []
  const project = projects.find((p: any) => p.id === projectIdNum)

  const { data: storiesData, loading: storiesLoading, refetch } = useQuery(LIST_STORIES, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })
  const stories: Story[] = (storiesData as any)?.stories || []

  const [createStory, { loading: createLoading }] = useMutation(CREATE_STORY, {
    onCompleted: (data) => {
      const newStory = (data as any)?.createStory
      if (newStory) {
        setCreateModalOpen(false)
        setNewStoryName('')
        setNewStoryDescription('')
        navigate(`/projects/${projectIdNum}/stories/${newStory.id}`)
      }
    },
    onError: (error) => {
      console.error('Failed to create story:', error)
      alert(`Failed to create story: ${error.message}`)
    },
  })

  const [deleteStory, { loading: deleteLoading }] = useMutation(DELETE_STORY, {
    onCompleted: () => {
      setDeleteModalOpen(false)
      setStoryToDelete(null)
      refetch()
    },
    onError: (error) => {
      console.error('Failed to delete story:', error)
      alert(`Failed to delete story: ${error.message}`)
    },
  })

  const [importStory, { loading: importLoading }] = useMutation(IMPORT_STORY, {
    onCompleted: (data) => {
      const result = (data as any)?.importStory
      if (result) {
        setImportModalOpen(false)
        if (fileInputRef.current) {
          fileInputRef.current.value = ''
        }
        refetch()

        let message = `Import completed:\n${result.createdCount} created, ${result.updatedCount} updated`
        if (result.errors && result.errors.length > 0) {
          message += `\n\nErrors:\n${result.errors.join('\n')}`
        }
        alert(message)
      }
    },
    onError: (error) => {
      console.error('Failed to import story:', error)
      alert(`Failed to import story: ${error.message}`)
    },
  })

  const [exportStory] = useMutation(EXPORT_STORY, {
    onCompleted: (data) => {
      const result = (data as any)?.exportStory as StoryExport
      if (result) {
        // Decode base64 and download
        const blob = new Blob([atob(result.content)], { type: result.mimeType })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = result.filename
        document.body.appendChild(a)
        a.click()
        document.body.removeChild(a)
        URL.revokeObjectURL(url)
      }
    },
    onError: (error) => {
      console.error('Failed to export story:', error)
      alert(`Failed to export story: ${error.message}`)
    },
  })

  const handleCreateStory = async () => {
    if (!newStoryName.trim()) {
      alert('Please enter a story name')
      return
    }

    await createStory({
      variables: {
        input: {
          projectId: projectIdNum,
          name: newStoryName.trim(),
          description: newStoryDescription.trim() || null,
        },
      },
    })
  }

  const handleDeleteStory = async () => {
    if (!storyToDelete) return

    await deleteStory({
      variables: { id: storyToDelete.id },
    })
  }

  const handleOpenCreate = () => {
    setNewStoryName('')
    setNewStoryDescription('')
    setCreateModalOpen(true)
  }

  const handleOpenDelete = (story: Story, e: React.MouseEvent) => {
    e.stopPropagation()
    setStoryToDelete(story)
    setDeleteModalOpen(true)
  }

  const handleOpenImport = () => {
    setImportModalOpen(true)
  }

  const handleFileSelect = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    try {
      const text = await file.text()
      await importStory({
        variables: {
          projectId: projectIdNum,
          format: importFormat,
          content: text,
        },
      })
    } catch (error) {
      console.error('Failed to read file:', error)
      alert('Failed to read file')
    }
  }

  const handleExportStory = async (storyId: number, format: 'CSV' | 'JSON', e: React.MouseEvent) => {
    e.stopPropagation()
    await exportStory({
      variables: {
        storyId,
        format,
      },
    })
  }

  const loading = projectsLoading || storiesLoading

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading stories...</span>
        </Group>
      </PageContainer>
    )
  }

  if (!project) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Project not found</h1>
        <Button className="mt-4" onClick={() => navigate('/projects')}>
          Back to projects
        </Button>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project.name}
        projectId={project.id}
        sections={[
          { title: 'Workbench', href: `/projects/${project.id}/workbench` },
        ]}
        currentPage="Stories"
        onNavigate={(route) => navigate(route)}
      />

      <Group justify="between" className="mb-6">
        <div>
          <h1 className="text-3xl font-bold">Stories</h1>
          <p className="text-muted-foreground">Create narrative sequences from your graph data.</p>
        </div>
        <Group gap="sm">
          <Button variant="outline" onClick={handleOpenImport}>
            <IconUpload className="mr-2 h-4 w-4" />
            Import Story
          </Button>
          <Button onClick={handleOpenCreate}>
            <IconPlus className="mr-2 h-4 w-4" />
            New Story
          </Button>
        </Group>
      </Group>

      {stories.length === 0 ? (
        <Card className="border p-6">
          <div className="flex flex-col items-center gap-4">
            <IconScript size={48} className="text-muted-foreground" />
            <h3 className="text-xl font-bold">No Stories Yet</h3>
            <p className="text-center text-muted-foreground">
              Create your first story to start building narrative sequences from your graph edges.
            </p>
            <Button onClick={handleOpenCreate}>
              <IconPlus className="mr-2 h-4 w-4" />
              Create First Story
            </Button>
          </div>
        </Card>
      ) : (
        <Stack gap="md">
          {stories.map((story) => (
            <Card
              key={story.id}
              className="border p-4 cursor-pointer hover:shadow-md transition-shadow"
              onClick={() => navigate(`/projects/${projectIdNum}/stories/${story.id}`)}
            >
              <Group justify="between" align="start">
                <div className="flex-1">
                  <Group gap="sm" className="mb-2">
                    <IconScript className="h-5 w-5 text-primary" />
                    <h4 className="text-lg font-semibold">{story.name}</h4>
                    <Badge variant="secondary">
                      {story.sequenceCount} sequence{story.sequenceCount !== 1 ? 's' : ''}
                    </Badge>
                  </Group>
                  {story.description && (
                    <p className="text-sm text-muted-foreground mb-2">
                      {story.description}
                    </p>
                  )}
                  <Group gap="sm">
                    {story.tags.map((tag) => (
                      <Badge key={tag} variant="outline">{tag}</Badge>
                    ))}
                  </Group>
                  <p className="text-xs text-muted-foreground mt-2">
                    Updated: {new Date(story.updatedAt).toLocaleDateString()}
                  </p>
                </div>
                <Group gap="xs">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation()
                      navigate(`/projects/${projectIdNum}/stories/${story.id}`)
                    }}
                  >
                    <IconEdit className="mr-2 h-3.5 w-3.5" />
                    Edit
                  </Button>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <IconDownload className="mr-2 h-3.5 w-3.5" />
                        Export
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent>
                      <DropdownMenuItem onClick={(e) => handleExportStory(story.id, 'JSON', e)}>
                        <IconBraces className="mr-2 h-4 w-4" />
                        Export as JSON
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={(e) => handleExportStory(story.id, 'CSV', e)}>
                        <IconFileTypeCsv className="mr-2 h-4 w-4" />
                        Export as CSV
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-destructive hover:text-destructive/80"
                    onClick={(e) => handleOpenDelete(story, e)}
                  >
                    <IconTrash className="mr-2 h-3.5 w-3.5" />
                    Delete
                  </Button>
                </Group>
              </Group>
            </Card>
          ))}
        </Stack>
      )}

      {/* Create Story Modal */}
      <Dialog open={createModalOpen} onOpenChange={setCreateModalOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Create Story</DialogTitle>
          </DialogHeader>
          <Stack gap="md" className="py-4">
            <div className="space-y-2">
              <Label htmlFor="story-name">Name</Label>
              <Input
                id="story-name"
                value={newStoryName}
                onChange={(e) => setNewStoryName(e.target.value)}
                placeholder="Enter story name"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="story-description">Description</Label>
              <Textarea
                id="story-description"
                value={newStoryDescription}
                onChange={(e) => setNewStoryDescription(e.target.value)}
                placeholder="Optional description"
                rows={3}
              />
            </div>
          </Stack>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setCreateModalOpen(false)} disabled={createLoading}>
              Cancel
            </Button>
            <Button onClick={handleCreateStory} disabled={createLoading || !newStoryName.trim()}>
              {createLoading && <Spinner className="mr-2 h-4 w-4" />}
              Create Story
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete Confirmation Modal */}
      <Dialog open={deleteModalOpen} onOpenChange={setDeleteModalOpen}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>Delete Story</DialogTitle>
          </DialogHeader>
          <p className="py-4">
            Are you sure you want to delete "{storyToDelete?.name}"? This will also delete all sequences in this story.
          </p>
          <DialogFooter>
            <Button variant="ghost" onClick={() => setDeleteModalOpen(false)} disabled={deleteLoading}>
              Cancel
            </Button>
            <Button variant="destructive" onClick={handleDeleteStory} disabled={deleteLoading}>
              {deleteLoading && <Spinner className="mr-2 h-4 w-4" />}
              Delete
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Import Story Modal */}
      <Dialog open={importModalOpen} onOpenChange={setImportModalOpen}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Import Story</DialogTitle>
          </DialogHeader>
          <Stack gap="md" className="py-4">
            <div className="space-y-2">
              <Label>File Format</Label>
              <Group gap="sm">
                <Button
                  variant={importFormat === 'JSON' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setImportFormat('JSON')}
                >
                  <IconBraces className="mr-2 h-4 w-4" />
                  JSON
                </Button>
                <Button
                  variant={importFormat === 'CSV' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setImportFormat('CSV')}
                >
                  <IconFileTypeCsv className="mr-2 h-4 w-4" />
                  CSV
                </Button>
              </Group>
            </div>
            <div className="space-y-2">
              <Label htmlFor="file-upload">Select File</Label>
              <Input
                id="file-upload"
                ref={fileInputRef}
                type="file"
                accept={importFormat === 'JSON' ? '.json' : '.csv'}
                onChange={handleFileSelect}
                disabled={importLoading}
              />
            </div>
            {importFormat === 'JSON' && (
              <p className="text-sm text-muted-foreground">
                Import stories from a JSON file. Story IDs of 0 will create new stories.
              </p>
            )}
            {importFormat === 'CSV' && (
              <p className="text-sm text-muted-foreground">
                Import stories from a CSV file. Required columns: story_name, sequence_name, dataset_id, edge_id
              </p>
            )}
          </Stack>
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => {
                setImportModalOpen(false)
                if (fileInputRef.current) {
                  fileInputRef.current.value = ''
                }
              }}
              disabled={importLoading}
            >
              Cancel
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}

export default StoriesPage
