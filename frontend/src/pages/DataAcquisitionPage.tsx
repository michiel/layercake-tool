import React, { useCallback, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'
import {
  IconClipboardList,
  IconDatabase,
  IconRefresh,
  IconSparkles,
  IconTrash,
} from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Label } from '../components/ui/label'
import { Textarea } from '../components/ui/textarea'
import { Spinner } from '../components/ui/spinner'
import { Separator } from '../components/ui/separator'
import { showSuccessNotification } from '../utils/notifications'
import { handleMutationErrors } from '../utils/graphqlHelpers'

const GET_DATA_ACQUISITION = gql`
  query DataAcquisitionOverview($projectId: Int!, $fileScope: String) {
    knowledgeBaseStatus(projectId: $projectId) {
      projectId
      fileCount
      chunkCount
      status
      lastIndexedAt
      embeddingProvider
      embeddingModel
    }
    dataAcquisitionTags(scope: $fileScope) {
      id
      name
      scope
      color
    }
  }
`

const RUN_KB_COMMAND = gql`
  mutation RunKbCommand($input: KnowledgeBaseCommandInput!) {
    runKnowledgeBaseCommand(input: $input)
  }
`

const GENERATE_DATASET = gql`
  mutation GenerateDataset($input: DatasetGenerationInput!) {
    generateDatasetFromPrompt(input: $input) {
      datasetYaml
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

type TagData = {
  id: string
  name: string
  scope: string
  color?: string | null
}

interface DataAcquisitionDashboardResponse {
  knowledgeBaseStatus?: KnowledgeBaseStatusData | null
  dataAcquisitionTags: TagData[]
}

interface DatasetGenerationResponse {
  generateDatasetFromPrompt?: {
    datasetYaml?: string | null
  } | null
}

const formatDate = (value?: string | null) => {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

export const DataAcquisitionPage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const numericProjectId = projectId ? parseInt(projectId, 10) : NaN
  const navigate = useNavigate()

  const [datasetPrompt, setDatasetPrompt] = useState('')
  const [selectedTags, setSelectedTags] = useState<string[]>([])
  const [generatedDataset, setGeneratedDataset] = useState<string | null>(null)
  const [mutationError, setMutationError] = useState<string | null>(null)

  const { data, refetch } = useQuery<DataAcquisitionDashboardResponse>(
    GET_DATA_ACQUISITION,
    {
      variables: {
        projectId: numericProjectId,
        fileScope: 'file',
      },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const [runKbCommand, { loading: kbMutating }] = useMutation(RUN_KB_COMMAND)
  const [generateDataset, { loading: datasetGenerating }] =
    useMutation<DatasetGenerationResponse>(GENERATE_DATASET)

  const tags = useMemo(() => data?.dataAcquisitionTags ?? [], [data])
  const kbStatus = data?.knowledgeBaseStatus

  const toggleSelectedTag = useCallback((name: string) => {
    setSelectedTags((prev) =>
      prev.includes(name)
        ? prev.filter((tag) => tag !== name)
        : [...prev, name],
    )
  }, [])

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
      })

      if (handleMutationErrors(result, 'Knowledge base operation failed')) {
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

  const handleDatasetGeneration = useCallback(async () => {
    if (!Number.isFinite(numericProjectId) || !datasetPrompt.trim()) return
    setMutationError(null)

    const response = await generateDataset({
      variables: {
        input: {
          projectId: numericProjectId,
          prompt: datasetPrompt,
          tagNames: selectedTags,
        },
      },
    })

    if (handleMutationErrors(response, 'Dataset generation failed')) {
      return
    }

    setGeneratedDataset(
      response.data?.generateDatasetFromPrompt?.datasetYaml ?? null,
    )
  }, [datasetPrompt, generateDataset, numericProjectId, selectedTags])

  if (!Number.isFinite(numericProjectId)) {
    return (
      <PageContainer>
        <Alert variant="destructive">
          <AlertDescription>
            No project context found. Please open a project before accessing
            data acquisition tools.
          </AlertDescription>
        </Alert>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Stack gap="lg">
        <Stack gap="xs">
          <h1 className="text-2xl font-bold">Data Acquisition</h1>
          <p className="text-muted-foreground">
            Monitor the knowledge base and launch Retrieval Augmented Generation
            workflows for new datasets.
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
              <IconClipboardList className="h-5 w-5 text-primary" />
              Source Management
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-4">
            <p className="text-sm text-muted-foreground">
              Upload files, edit metadata, and manage tags from the dedicated
              Source Management workspace.
            </p>
            <Button
              variant="secondary"
              className="w-full sm:w-fit"
              onClick={() =>
                navigate(`/projects/${projectId}/data-acquisition/source-management`)
              }
            >
              Open Source Management
            </Button>
          </CardContent>
        </Card>

        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <IconDatabase className="h-5 w-5 text-primary" />
                Knowledge Base
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
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

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <IconSparkles className="h-5 w-5 text-primary" />
                Data Set Creation
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid gap-4">
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
                    {tags.map((tag) => (
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
                <Separator className="my-2" />
                <div className="space-y-2">
                  <Label>Generated Dataset</Label>
                  <Textarea
                    readOnly
                    value={generatedDataset ?? ''}
                    placeholder="Generated YAML will appear here."
                    rows={8}
                  />
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </Stack>
    </PageContainer>
  )
}

export default DataAcquisitionPage
