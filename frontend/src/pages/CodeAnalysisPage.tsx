import React, { useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { IconPlayerPlay, IconPlus, IconTrash } from '@tabler/icons-react'

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

type Profile = {
  id: string
  filePath: string
  datasetId?: string
  lastRun?: string
  status?: 'idle' | 'running' | 'complete'
  report?: string
}

const mockDatasets = [
  { id: 'ds-1', name: 'Code Analysis - Default' },
  { id: 'ds-2', name: 'Graphs - Sandbox' },
]

export const CodeAnalysisPage: React.FC = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()
  const [profiles, setProfiles] = useState<Profile[]>([
    {
      id: 'p-1',
      filePath: '/home/user/project',
      datasetId: 'ds-1',
      lastRun: 'Never',
      status: 'idle',
      report: 'No report generated yet.',
    },
  ])
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<Profile | null>(null)
  const [filePath, setFilePath] = useState('')
  const [datasetId, setDatasetId] = useState<string | undefined>()

  const selectedProjectName = useMemo(() => `Project ${projectId ?? ''}`, [projectId])

  const handlePlay = (id: string) => {
    setProfiles(prev =>
      prev.map(p =>
        p.id === id
          ? {
              ...p,
              status: 'running',
              report: 'Running analysis...',
            }
          : p,
      ),
    )

    setTimeout(() => {
      setProfiles(prev =>
        prev.map(p =>
          p.id === id
            ? {
                ...p,
                status: 'complete',
                lastRun: new Date().toLocaleString(),
                report: `Code analysis complete for ${p.filePath}\n\nData flow and function metrics have been written to dataset ${p.datasetId ?? 'unlinked'}.`,
              }
            : p,
        ),
      )
    }, 800)
  }

  const handleDelete = (id: string) => {
    setProfiles(prev => prev.filter(p => p.id !== id))
  }

  const handleOpenModal = (profile?: Profile) => {
    setEditing(profile ?? null)
    setFilePath(profile?.filePath ?? '')
    setDatasetId(profile?.datasetId)
    setModalOpen(true)
  }

  const handleSave = () => {
    if (!filePath) return
    if (editing) {
      setProfiles(prev =>
        prev.map(p =>
          p.id === editing.id
            ? { ...p, filePath, datasetId }
            : p,
        ),
      )
    } else {
      const newProfile: Profile = {
        id: `p-${Date.now()}`,
        filePath,
        datasetId,
        lastRun: 'Never',
        status: 'idle',
        report: 'No report generated yet.',
      }
      setProfiles(prev => [...prev, newProfile])
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
          {profiles.length === 0 ? (
            <Stack align="center" gap="md" className="py-10">
              <Spinner className="h-6 w-6" />
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
                      {mockDatasets.find(ds => ds.id === profile.datasetId)?.name ?? 'Not linked'}
                    </TableCell>
                    <TableCell>{profile.lastRun ?? 'Never'}</TableCell>
                    <TableCell className="capitalize">{profile.status ?? 'idle'}</TableCell>
                    <TableCell className="text-right">
                      <Group gap="xs" justify="end">
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
                    {mockDatasets.find(ds => ds.id === profile.datasetId)?.name ?? 'Not linked'}
                  </div>
                </div>
                <Separator orientation="vertical" className="h-6" />
                <div>
                  <div className="text-xs text-muted-foreground">Last run</div>
                  <div className="text-sm">{profile.lastRun ?? 'Never'}</div>
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
              <Select value={datasetId} onValueChange={setDatasetId}>
                <SelectTrigger>
                  <SelectValue placeholder="Select dataset" />
                </SelectTrigger>
                <SelectContent>
                  {mockDatasets.map(ds => (
                    <SelectItem key={ds.id} value={ds.id}>{ds.name}</SelectItem>
                  ))}
                </SelectContent>
              </Select>
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
