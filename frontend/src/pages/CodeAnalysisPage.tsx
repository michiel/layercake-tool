import React, { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { IconPlayerPlay, IconPlus, IconTrash } from '@tabler/icons-react'
import { gql } from '@apollo/client'
import { useMutation, useQuery } from '@apollo/client/react'

import PageContainer from '../components/layout/PageContainer'
import { Breadcrumbs } from '../components/common/Breadcrumbs'
import { Button } from '../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card'
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from '../components/ui/dialog'
import { Input } from '../components/ui/input'
import { Label } from '../components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../components/ui/select'
import { Separator } from '../components/ui/separator'
import { Spinner } from '../components/ui/spinner'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../components/ui/table'
import { Textarea } from '../components/ui/textarea'
import { Group, Stack } from '../components/layout-primitives'
import { showSuccessNotification, showErrorNotification } from '../utils/notifications'
import { Link } from 'react-router-dom'

type Profile = {
  id: string
  filePath: string
  datasetId?: number | null
  lastRun?: string | null
  report?: string | null
  noInfra?: boolean
  options?: string | null
}

type DataSetOption = { id: number; name: string }

const GET_PROFILES = gql`
  query CodeAnalysisProfiles($projectId: Int!) {
    codeAnalysisProfiles(projectId: $projectId) {
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

const CREATE_PROFILE = gql`
  mutation CreateCodeAnalysisProfile($input: CreateCodeAnalysisProfileInput!) {
    createCodeAnalysisProfile(input: $input) {
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

const UPDATE_PROFILE = gql`
  mutation UpdateCodeAnalysisProfile($input: UpdateCodeAnalysisProfileInput!) {
    updateCodeAnalysisProfile(input: $input) {
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

const DELETE_PROFILE = gql`
  mutation DeleteCodeAnalysisProfile($id: String!) {
    deleteCodeAnalysisProfile(id: $id)
  }
`

const RUN_PROFILE = gql`
  mutation RunCodeAnalysisProfile($id: String!) {
    runCodeAnalysisProfile(id: $id) {
      profile {
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

export const CodeAnalysisPage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()
  const [profiles, setProfiles] = useState<Profile[]>([])
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<Profile | null>(null)
  const [filePath, setFilePath] = useState('')
  const [datasetId, setDatasetId] = useState<number | undefined>()
  const [noInfra, setNoInfra] = useState(false)
  const [includeDataFlow, setIncludeDataFlow] = useState(true)
  const [includeControlFlow, setIncludeControlFlow] = useState(true)
  const [includeImports, setIncludeImports] = useState(true)
  const [coalesceFunctions, setCoalesceFunctions] = useState(false)

  const selectedProjectName = useMemo(() => `Project ${projectId ?? ''}`, [projectId])

  const {
    data: profilesData,
    loading: profilesLoading,
    error: profilesError,
    refetch: refetchProfiles,
  } = useQuery<any>(GET_PROFILES, {
    skip: !projectId,
    variables: { projectId: projectId ? parseInt(projectId, 10) : undefined },
    fetchPolicy: 'cache-and-network',
  })

  React.useEffect(() => {
    const mapped: Profile[] =
      profilesData?.codeAnalysisProfiles?.map((p: any) => ({
        id: p.id,
        filePath: p.filePath,
        datasetId: p.datasetId,
        lastRun: p.lastRun,
        report: p.report,
        noInfra: p.noInfra,
        options: p.options,
      })) || []
    setProfiles(mapped)
  }, [profilesData])

  const { data: datasetsData } = useQuery<any>(GET_DATASETS, {
    skip: !projectId,
    variables: { projectId: projectId ? parseInt(projectId, 10) : undefined },
    fetchPolicy: 'cache-and-network',
  })
  const datasetOptions: DataSetOption[] = datasetsData?.dataSets ?? []

  const [createProfile] = useMutation<any>(CREATE_PROFILE, {
    onCompleted: () => {
      showSuccessNotification('Profile created')
      refetchProfiles()
    },
    onError: err => showErrorNotification(err.message),
  })
  const [updateProfile] = useMutation<any>(UPDATE_PROFILE, {
    onCompleted: () => {
      showSuccessNotification('Profile updated')
      refetchProfiles()
    },
    onError: err => showErrorNotification(err.message),
  })
  const [deleteProfileMutation] = useMutation<any>(DELETE_PROFILE, {
    onCompleted: () => {
      showSuccessNotification('Profile deleted')
      refetchProfiles()
    },
    onError: err => showErrorNotification(err.message),
  })
  const [runProfile] = useMutation<any>(RUN_PROFILE, {
    onCompleted: data => {
      const updated = data?.runCodeAnalysisProfile?.profile
      if (updated) {
        setProfiles(prev =>
          prev.map(p => (p.id === updated.id ? { ...p, ...updated } : p)),
        )
      }
      showSuccessNotification('Code analysis completed')
    },
    onError: err => showErrorNotification(err.message),
  })

  const handlePlay = (id: string) => {
    runProfile({ variables: { id } })
  }

  const handleDelete = (id: string) => {
    deleteProfileMutation({ variables: { id } })
  }

  const handleOpenModal = (profile?: Profile) => {
    setEditing(profile ?? null)
    setFilePath(profile?.filePath ?? '')
    setDatasetId(profile?.datasetId ?? undefined)
    setNoInfra(profile?.noInfra ?? false)
    const opts = profile?.options ? JSON.parse(profile.options) : {}
    setIncludeDataFlow(opts.includeDataFlow ?? true)
    setIncludeControlFlow(opts.includeControlFlow ?? true)
    setIncludeImports(opts.includeImports ?? true)
    setCoalesceFunctions(opts.coalesceFunctions ?? false)
    setModalOpen(true)
  }

  const handleSave = () => {
    const trimmedPath = filePath.trim()
    if (!trimmedPath) return
    const options = JSON.stringify({
      includeDataFlow,
      includeControlFlow,
      includeImports,
      coalesceFunctions,
    })
    if (editing) {
      updateProfile({
        variables: { input: { id: editing.id, filePath: trimmedPath, datasetId, noInfra, options } },
      })
    } else {
      createProfile({
        variables: {
          input: {
            projectId: projectId ? parseInt(projectId, 10) : 0,
            filePath: trimmedPath,
            datasetId,
            noInfra,
            options,
          },
        },
      })
    }
    setModalOpen(false)
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProjectName}
        projectId={projectId ? parseInt(projectId, 10) : undefined}
        sections={[{ title: 'Data management', href: `/projects/${projectId}/datasets` }]}
        currentPage="Code analysis"
        onNavigate={(href) => navigate(href)}
      />

      <Group justify="between" className="mb-4">
        <Stack gap="xs">
          <h1 className="text-3xl font-bold">Code analysis</h1>
          <p className="text-muted-foreground text-sm">
            Manage code analysis profiles. Run analysis to populate datasets with function and data-flow graphs.
          </p>
        </Stack>
        <Button onClick={() => handleOpenModal()}>
          <IconPlus className="mr-2 h-4 w-4" />
          New profile
        </Button>
      </Group>

      <Card className="border">
        <CardHeader className="pb-2">
          <CardTitle>Profiles</CardTitle>
        </CardHeader>
        <CardContent>
          {profilesError && (
            <div className="mb-3 text-sm text-red-600">
              Failed to load profiles: {profilesError.message}
            </div>
          )}
          {profilesLoading ? (
            <div className="py-8 flex justify-center">
              <Spinner className="h-6 w-6" />
            </div>
          ) : profiles.length === 0 ? (
              <Stack align="center" gap="md" className="py-10">
                <p className="text-muted-foreground">No profiles yet. Create one to get started.</p>
              </Stack>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-1/4">File path</TableHead>
                  <TableHead className="w-1/5">Dataset</TableHead>
                  <TableHead className="w-1/5">Last run</TableHead>
                  <TableHead className="w-1/5">Status</TableHead>
                  <TableHead className="text-right w-1/5">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {profiles.map(profile => (
                  <TableRow key={profile.id}>
                    <TableCell className="font-medium">{profile.filePath}</TableCell>
                    <TableCell>
                      {datasetOptions.find(ds => ds.id === profile.datasetId)?.name ?? 'Not linked'}
                    </TableCell>
                    <TableCell>{profile.lastRun ? new Date(profile.lastRun).toLocaleString() : 'Never'}</TableCell>
                    <TableCell className="capitalize">{profile.report ? 'complete' : 'idle'}</TableCell>
                    <TableCell className="text-right">
                      <Group gap="xs" justify="end">
                        <Button size="sm" variant="outline" asChild>
                          <Link to={`/projects/${projectId}/data-acquisition/code-analysis/${profile.id}`}>
                            View
                          </Link>
                        </Button>
                        <Button size="sm" variant="secondary" onClick={() => handleOpenModal(profile)}>
                          Edit
                        </Button>
                        <Button size="sm" onClick={() => handlePlay(profile.id)}>
                          <IconPlayerPlay className="h-4 w-4 mr-1" />
                          Run
                        </Button>
                        <Button size="sm" variant="destructive" onClick={() => handleDelete(profile.id)}>
                          <IconTrash className="h-4 w-4 mr-1" />
                          Delete
                        </Button>
                      </Group>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {profiles.map(profile => (
        <Card className="mt-4" key={`report-${profile.id}`}>
          <CardHeader>
            <CardTitle>Last report — {profile.filePath}</CardTitle>
          </CardHeader>
          <CardContent>
            <Stack gap="sm">
              <Group gap="md">
                <div>
                  <div className="text-xs text-muted-foreground">Dataset</div>
                  <div className="text-sm">
                    {datasetOptions.find(ds => ds.id === profile.datasetId)?.name ?? 'Not linked'}
                  </div>
                </div>
                <Separator orientation="vertical" className="h-6" />
                <div>
                  <div className="text-xs text-muted-foreground">Last run</div>
                  <div className="text-sm">{profile.lastRun ?? 'Never'}</div>
                </div>
                <Separator orientation="vertical" className="h-6" />
                <div>
                  <div className="text-xs text-muted-foreground">Infra correlation</div>
                  <div className="text-sm">{profile.noInfra ? 'Disabled' : 'Enabled'}</div>
                </div>
              </Group>
              <Textarea readOnly value={profile.report ?? 'No report'} className="min-h-[140px]" />
            </Stack>
          </CardContent>
        </Card>
      ))}

      <Dialog open={modalOpen} onOpenChange={setModalOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editing ? 'Edit profile' : 'New profile'}</DialogTitle>
          </DialogHeader>
          <Stack gap="md">
            <div className="space-y-2">
              <Label htmlFor="filePath">Project path</Label>
              <Input
                id="filePath"
                placeholder="/path/to/repo"
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label>Linked dataset</Label>
              <Select
                value={datasetId ? datasetId.toString() : undefined}
                onValueChange={(value) => setDatasetId(value ? parseInt(value, 10) : undefined)}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select dataset" />
                </SelectTrigger>
                <SelectContent>
                  {datasetOptions.map(ds => (
                    <SelectItem key={ds.id} value={ds.id.toString()}>{ds.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="space-y-1">
              <Label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={noInfra}
                  onChange={(e) => setNoInfra(e.target.checked)}
                />
                Disable infra correlation
              </Label>
              <Label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={includeDataFlow}
                  onChange={(e) => setIncludeDataFlow(e.target.checked)}
                />
                Include data flow
              </Label>
              <Label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={includeControlFlow}
                  onChange={(e) => setIncludeControlFlow(e.target.checked)}
                />
                Include control flow
              </Label>
              <Label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={includeImports}
                  onChange={(e) => setIncludeImports(e.target.checked)}
                />
                Include imports
              </Label>
              <Label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={coalesceFunctions}
                  onChange={(e) => setCoalesceFunctions(e.target.checked)}
                />
                Coalesce functions into files (aggregate edges)
              </Label>
            </div>
          </Stack>
          <DialogFooter className="mt-4">
            <Button variant="secondary" onClick={() => setModalOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleSave}>
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}
