import React, { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'

import PageContainer from '../components/layout/PageContainer'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Separator } from '../components/ui/separator'
import { Spinner } from '../components/ui/spinner'
import { Textarea } from '../components/ui/textarea'
import { Group, Stack } from '../components/layout-primitives'
import { showErrorNotification, showSuccessNotification } from '../utils/notifications'

const GET_PROFILE = gql`
  query CodeAnalysisProfile($id: String!) {
    codeAnalysisProfile(id: $id) {
      id
      projectId
      filePath
      datasetId
      lastRun
      report
      noInfra
      options
    }
  }
`

const GET_DATASETS = gql`
  query DataSets($projectId: Int!) {
    dataSets(projectId: $projectId) {
      id
      name
    }
  }
`

const RUN_PROFILE = gql`
  mutation RunCodeAnalysisProfile($id: String!) {
    runCodeAnalysisProfile(id: $id) {
      profile {
        id
        lastRun
        report
        datasetId
      }
    }
  }
`

export const CodeAnalysisDetailPage: React.FC = () => {
  const { projectId, profileId } = useParams<{ projectId: string; profileId: string }>()
  const navigate = useNavigate()

  const { data, loading, error, refetch } = useQuery<any>(GET_PROFILE, {
    variables: { id: profileId },
    skip: !profileId,
    fetchPolicy: 'cache-and-network',
  })

  const { data: datasetsData } = useQuery<any>(GET_DATASETS, {
    skip: !projectId,
    variables: { projectId: projectId ? parseInt(projectId, 10) : undefined },
    fetchPolicy: 'cache-and-network',
  })

  const [runProfile, { loading: running }] = useMutation(RUN_PROFILE, {
    onCompleted: () => {
      showSuccessNotification('Code analysis run complete')
      refetch()
    },
    onError: (err) => showErrorNotification(err.message),
  })

  const profile = data?.codeAnalysisProfile
  const datasetName = useMemo(() => {
    if (!profile?.datasetId) return 'Not linked'
    return datasetsData?.dataSets?.find((ds: any) => ds.id === profile.datasetId)?.name ?? 'Not linked'
  }, [datasetsData, profile])

  const selectedProjectName = useMemo(() => `Project ${projectId ?? ''}`, [projectId])
  const options = useMemo(() => {
    try {
      return profile?.options ? JSON.parse(profile.options) : {}
    } catch (_) {
      return {}
    }
  }, [profile])

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProjectName}
        projectId={projectId ? parseInt(projectId, 10) : undefined}
        sections={[
          { title: 'Data management', href: `/projects/${projectId}/datasets` },
          { title: 'Code analysis', href: `/projects/${projectId}/data-acquisition/code-analysis` },
        ]}
        currentPage="Profile"
        onNavigate={(href) => navigate(href)}
      />

      {loading ? (
        <div className="py-10 flex justify-center">
          <Spinner className="h-6 w-6" />
        </div>
      ) : error ? (
        <div className="text-red-600">Failed to load profile: {error.message}</div>
      ) : !profile ? (
        <div className="text-muted-foreground">Profile not found</div>
      ) : (
        <Stack gap="lg">
          <Group justify="between">
            <Stack gap="xs">
              <h1 className="text-3xl font-bold">{profile.filePath}</h1>
              <p className="text-muted-foreground text-sm">Dataset: {datasetName}</p>
            </Stack>
            <Group gap="sm">
              <Button variant="secondary" onClick={() => navigate(-1)}>
                Back
              </Button>
              <Button onClick={() => runProfile({ variables: { id: profileId } })} disabled={running}>
                {running ? 'Running…' : 'Run analysis'}
              </Button>
            </Group>
          </Group>

          <Card className="border">
            <CardHeader className="pb-2">
              <CardTitle>Details</CardTitle>
            </CardHeader>
            <CardContent>
              <Group gap="lg">
                <div>
                  <div className="text-xs text-muted-foreground">Last run</div>
                  <div className="text-sm">{profile.lastRun ?? 'Never'}</div>
                </div>
                <Separator orientation="vertical" className="h-6" />
                <div>
                  <div className="text-xs text-muted-foreground">Infra correlation</div>
                  <div className="text-sm">{profile.noInfra ? 'Disabled' : 'Enabled'}</div>
                </div>
                <Separator orientation="vertical" className="h-6" />
                <div>
                  <div className="text-xs text-muted-foreground">Options</div>
                  <div className="text-sm space-y-1">
                    <div>Data flow: {options.includeDataFlow === false ? 'Off' : 'On'}</div>
                    <div>Control flow: {options.includeControlFlow === false ? 'Off' : 'On'}</div>
                    <div>Imports: {options.includeImports === false ? 'Off' : 'On'}</div>
                  </div>
                </div>
              </Group>
            </CardContent>
          </Card>

          <Card className="border">
            <CardHeader className="pb-2">
              <CardTitle>Last report</CardTitle>
            </CardHeader>
            <CardContent>
              <Textarea readOnly value={profile.report ?? 'No report'} className="min-h-[240px]" />
            </CardContent>
          </Card>
        </Stack>
      )}
    </PageContainer>
  )
}
