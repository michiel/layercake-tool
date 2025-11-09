import React, { useCallback, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'
import { IconDatabase, IconRefresh, IconTrash } from '@tabler/icons-react'

import PageContainer from '../components/layout/PageContainer'
import { Stack, Group } from '../components/layout-primitives'
import { Alert, AlertDescription } from '../components/ui/alert'
import { Badge } from '../components/ui/badge'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Separator } from '../components/ui/separator'
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

type KnowledgeBaseStatusData = {
  projectId: number
  fileCount: number
  chunkCount: number
  status: string
  lastIndexedAt?: string | null
  embeddingProvider?: string | null
  embeddingModel?: string | null
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

  const [mutationError, setMutationError] = useState<string | null>(null)

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{ id: number; name: string }>
  }>(GET_PROJECTS)
  const selectedProject = projectsData?.projects.find(
    (p: any) => p.id === numericProjectId,
  )

  const { data, refetch } = useQuery<{ knowledgeBaseStatus?: KnowledgeBaseStatusData | null }>(
    GET_KNOWLEDGE_BASE,
    {
      variables: { projectId: numericProjectId },
      skip: !Number.isFinite(numericProjectId),
      fetchPolicy: 'cache-and-network',
    },
  )

  const [runKbCommand, { loading: kbMutating }] = useMutation(RUN_KB_COMMAND)

  const kbStatus = data?.knowledgeBaseStatus

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
      </Stack>
    </PageContainer>
  )
}

export default KnowledgeBasePage
