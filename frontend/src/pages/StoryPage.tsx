import { useState, useEffect } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import {
  IconScript,
  IconDeviceFloppy,
  IconStack2,
  IconListDetails,
  IconDatabase,
} from '@tabler/icons-react'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group, Stack } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Spinner } from '@/components/ui/spinner'
import { GET_STORY, UPDATE_STORY, Story, StoryLayerConfig, UpdateStoryInput } from '@/graphql/stories'
import { GET_DATASOURCES, DataSet } from '@/graphql/datasets'
import { StorySequencesTab } from '@/components/stories/StorySequencesTab'
import { StoryLayersTab } from '@/components/stories/StoryLayersTab'

const GET_PROJECTS = gql`
  query GetProjectsForStory {
    projects {
      id
      name
      description
    }
  }
`

const VALID_TABS = ['sequences', 'datasets', 'layers'] as const
type TabValue = (typeof VALID_TABS)[number]

export const StoryPage = () => {
  const navigate = useNavigate()
  const { projectId, storyId } = useParams<{ projectId: string; storyId: string }>()
  const [searchParams, setSearchParams] = useSearchParams()
  const projectIdNum = Number(projectId || 0)
  const storyIdNum = Number(storyId || 0)

  // Get active tab from URL, default to 'sequences'
  const tabParam = searchParams.get('tab')
  const activeTab: TabValue = VALID_TABS.includes(tabParam as TabValue) ? (tabParam as TabValue) : 'sequences'

  const setActiveTab = (tab: string) => {
    setSearchParams((prev) => {
      const next = new URLSearchParams(prev)
      if (tab === 'sequences') {
        next.delete('tab')
      } else {
        next.set('tab', tab)
      }
      return next
    }, { replace: true })
  }
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [tagsInput, setTagsInput] = useState('')
  const [enabledDatasetIds, setEnabledDatasetIds] = useState<number[]>([])
  const [layerConfig, setLayerConfig] = useState<StoryLayerConfig[]>([])
  const [hasChanges, setHasChanges] = useState(false)

  const { data: projectsData, loading: projectsLoading } = useQuery(GET_PROJECTS)
  const projects = (projectsData as any)?.projects || []
  const project = projects.find((p: any) => p.id === projectIdNum)

  const { data: storyData, loading: storyLoading, refetch: refetchStory } = useQuery(GET_STORY, {
    variables: { id: storyIdNum },
    skip: !storyIdNum,
  })
  const story: Story | null = (storyData as any)?.story || null

  const { data: datasetsData, loading: datasetsLoading } = useQuery(GET_DATASOURCES, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })
  const datasets: DataSet[] = (datasetsData as any)?.dataSets || []

  const [updateStory, { loading: updateLoading }] = useMutation(UPDATE_STORY, {
    onCompleted: () => {
      setHasChanges(false)
      refetchStory()
    },
    onError: (error) => {
      console.error('Failed to update story:', error)
      alert(`Failed to update story: ${error.message}`)
    },
  })

  // Initialize form when story loads
  useEffect(() => {
    if (story) {
      setName(story.name)
      setDescription(story.description || '')
      setTagsInput(story.tags.join(', '))
      setEnabledDatasetIds(story.enabledDatasetIds)
      setLayerConfig(story.layerConfig || [])
      setHasChanges(false)
    }
  }, [story])

  const handleSave = async () => {
    const tags = tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t.length > 0)

    // Strip __typename from layerConfig items
    const cleanLayerConfig = layerConfig.map(({ sourceDatasetId, mode }) => ({
      sourceDatasetId,
      mode,
    }))

    const input: UpdateStoryInput = {
      name,
      description: description || undefined,
      tags,
      enabledDatasetIds,
      layerConfig: cleanLayerConfig,
    }

    try {
      await updateStory({
        variables: { id: storyIdNum, input },
      })
    } catch (error) {
      console.error('Save error:', error)
    }
  }

  const handleFieldChange = () => {
    setHasChanges(true)
  }

  const toggleDataset = (datasetId: number) => {
    setEnabledDatasetIds((prev) =>
      prev.includes(datasetId)
        ? prev.filter((id) => id !== datasetId)
        : [...prev, datasetId]
    )
    setHasChanges(true)
  }

  const handleLayerConfigChange = (config: StoryLayerConfig[]) => {
    setLayerConfig(config)
    setHasChanges(true)
  }

  const loading = projectsLoading || storyLoading || datasetsLoading

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading story...</span>
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

  if (!story) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Story not found</h1>
        <Button className="mt-4" onClick={() => navigate(`/projects/${projectIdNum}/stories`)}>
          Back to stories
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
          { title: 'Stories', href: `/projects/${project.id}/stories` },
        ]}
        currentPage={story.name}
        onNavigate={(route) => navigate(route)}
      />

      <Group justify="between" className="mb-6">
        <div>
          <Group gap="sm" align="center">
            <IconScript className="h-6 w-6 text-primary" />
            <h1 className="text-3xl font-bold">{story.name}</h1>
          </Group>
          <Group gap="sm" className="mt-2">
            <Badge variant="secondary">
              {story.sequenceCount} sequence{story.sequenceCount !== 1 ? 's' : ''}
            </Badge>
            <Badge variant="secondary">
              {enabledDatasetIds.length} dataset{enabledDatasetIds.length !== 1 ? 's' : ''} enabled
            </Badge>
          </Group>
        </div>
        {hasChanges && (
          <Button onClick={handleSave} disabled={updateLoading}>
            {updateLoading && <Spinner className="mr-2 h-4 w-4" />}
            <IconDeviceFloppy className="mr-2 h-4 w-4" />
            Save Changes
          </Button>
        )}
      </Group>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="sequences">
            <IconListDetails className="mr-2 h-4 w-4" />
            Sequences
          </TabsTrigger>
          <TabsTrigger value="datasets">
            <IconDatabase className="mr-2 h-4 w-4" />
            Datasets
          </TabsTrigger>
          <TabsTrigger value="layers">
            <IconStack2 className="mr-2 h-4 w-4" />
            Layers
          </TabsTrigger>
        </TabsList>

        <TabsContent value="sequences">
          {/* Story Details Row */}
          <Card className="border mt-4 mb-4">
            <CardContent className="py-4">
              <div className="grid gap-4 md:grid-cols-3">
                <div className="space-y-1">
                  <Label htmlFor="story-name" className="text-xs">Name</Label>
                  <Input
                    id="story-name"
                    value={name}
                    onChange={(e) => {
                      setName(e.target.value)
                      handleFieldChange()
                    }}
                    placeholder="Story name"
                    className="h-8"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="story-description" className="text-xs">Description</Label>
                  <Input
                    id="story-description"
                    value={description}
                    onChange={(e) => {
                      setDescription(e.target.value)
                      handleFieldChange()
                    }}
                    placeholder="Optional description"
                    className="h-8"
                  />
                </div>
                <div className="space-y-1">
                  <Label htmlFor="story-tags" className="text-xs">Tags</Label>
                  <Input
                    id="story-tags"
                    value={tagsInput}
                    onChange={(e) => {
                      setTagsInput(e.target.value)
                      handleFieldChange()
                    }}
                    placeholder="tag1, tag2, tag3"
                    className="h-8"
                  />
                </div>
              </div>
            </CardContent>
          </Card>
          <StorySequencesTab storyId={storyIdNum} projectId={projectIdNum} />
        </TabsContent>

        <TabsContent value="datasets">
          <Card className="border mt-4">
            <CardHeader>
              <CardTitle className="text-base">Dataset Selection</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-muted-foreground mb-4">
                Select which datasets are available for use in this story's sequences.
              </p>
              {datasets.length === 0 ? (
                <p className="text-sm text-muted-foreground italic">
                  No datasets available. Create datasets first.
                </p>
              ) : (
                <Stack gap="sm">
                  {datasets.map((dataset) => (
                    <div key={dataset.id} className="flex items-center space-x-3">
                      <Checkbox
                        id={`dataset-${dataset.id}`}
                        checked={enabledDatasetIds.includes(dataset.id)}
                        onCheckedChange={() => toggleDataset(dataset.id)}
                      />
                      <label
                        htmlFor={`dataset-${dataset.id}`}
                        className="flex-1 text-sm cursor-pointer"
                      >
                        <div className="font-medium">{dataset.name}</div>
                        <div className="text-xs text-muted-foreground">
                          {dataset.nodeCount || 0} nodes, {dataset.edgeCount || 0} edges
                        </div>
                      </label>
                    </div>
                  ))}
                </Stack>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="layers">
          <StoryLayersTab
            projectId={projectIdNum}
            layerConfig={layerConfig}
            onLayerConfigChange={handleLayerConfigChange}
          />
        </TabsContent>
      </Tabs>
    </PageContainer>
  )
}

export default StoryPage
