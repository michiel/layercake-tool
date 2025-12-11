import { useMemo, useState } from 'react'
import { useNavigate, useParams, useSearchParams } from 'react-router-dom'
import { gql, useApolloClient, useMutation, useQuery } from '@apollo/client'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group } from '@/components/layout-primitives'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { IconAffiliate, IconDatabase, IconExternalLink, IconPlayerPlay, IconUpload } from '@tabler/icons-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Input } from '@/components/ui/input'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { createApolloClientForEndpoint } from '@/graphql/client'

const LIST_PROJECTIONS = gql`
  query ListProjections($projectId: Int!) {
    projections(projectId: $projectId) {
      id
      name
      projectionType
      graphId
      updatedAt
    }
  }
`

const LIST_GRAPHS = gql`
  query ListGraphsForProjections($projectId: Int!) {
    graphs(projectId: $projectId) {
      id
      name
    }
  }
`

const CREATE_PROJECTION = gql`
  mutation CreateProjection($input: CreateProjectionInput!) {
    createProjection(input: $input) {
      id
      name
      projectionType
      graphId
    }
  }
`

const DELETE_PROJECTION = gql`
  mutation DeleteProjection($id: ID!) {
    deleteProjection(id: $id)
  }
`

const EXPORT_PROJECTION = gql`
  mutation ExportProjection($id: ID!) {
    exportProjection(id: $id) {
      filename
      contentBase64
    }
  }
`

export const ProjectionsPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const [params] = useSearchParams()
  const planId = params.get('planId')
  const projectIdNum = Number(projectId || 0)

  const [newName, setNewName] = useState('')
  const [newGraphId, setNewGraphId] = useState<string>('')
  const [newType, setNewType] = useState<'force3d' | 'layer3d'>('force3d')

  const projectionsClient = useMemo(
    () =>
      createApolloClientForEndpoint({
        httpPath: '/projections/graphql',
        wsPath: '/projections/graphql/ws',
      }),
    []
  )

  const { data: projectionsData, loading, refetch } = useQuery(LIST_PROJECTIONS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'cache-and-network',
    client: projectionsClient,
  })

  const { data: graphsData, loading: graphsLoading } = useQuery(LIST_GRAPHS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'cache-and-network',
  })

  const [createProjection, { loading: creating }] = useMutation(CREATE_PROJECTION, {
    client: projectionsClient,
  })
  const [deleteProjection, { loading: deleting }] = useMutation(DELETE_PROJECTION, {
    client: projectionsClient,
  })
  const [exportProjection, { loading: exporting }] = useMutation(EXPORT_PROJECTION, {
    client: projectionsClient,
  })

  const projections = projectionsData?.projections ?? []
  const graphs = graphsData?.graphs ?? []

  const projectName = ''

  const handleCreate = async () => {
    if (!newName.trim() || !newGraphId) {
      showErrorNotification('Missing fields', 'Choose a graph and name your projection.')
      return
    }
    try {
      await createProjection({
        variables: {
          input: {
            projectId: projectIdNum,
            graphId: Number(newGraphId),
            name: newName.trim(),
            projectionType: newType,
          },
        },
      })
      setNewName('')
      setNewGraphId('')
      await refetch()
      showSuccessNotification('Projection created', 'You can now open it.')
    } catch (err: any) {
      showErrorNotification('Create failed', err?.message || 'Unable to create projection')
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await deleteProjection({ variables: { id } })
      await refetch()
      showSuccessNotification('Projection deleted', '')
    } catch (err: any) {
      showErrorNotification('Delete failed', err?.message || 'Unable to delete projection')
    }
  }

  const handleOpen = (id: string) => {
    window.open(`/projections/${id}`, '_blank', 'noreferrer')
  }

  const handleExport = async (id: string, name: string) => {
    try {
      const { data } = await exportProjection({ variables: { id } })
      const payload = data?.exportProjection
      if (!payload?.contentBase64) {
        showErrorNotification('Export failed', 'No export payload returned')
        return
      }
      const binary = atob(payload.contentBase64)
      const len = binary.length
      const bytes = new Uint8Array(len)
      for (let i = 0; i < len; i += 1) {
        bytes[i] = binary.charCodeAt(i)
      }
      const blob = new Blob([bytes], { type: 'application/zip' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = payload.filename || `${name || 'projection'}-export.zip`
      document.body.appendChild(link)
      link.click()
      link.remove()
      URL.revokeObjectURL(url)
      showSuccessNotification('Export ready', 'Downloaded projection bundle.')
    } catch (err: any) {
      showErrorNotification('Export failed', err?.message || 'Unable to export projection')
    }
  }

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading projections...</span>
        </Group>
      </PageContainer>
    )
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={projectName}
        projectId={projectIdNum}
        currentPage="Projections"
        onNavigate={(route) => navigate(route)}
        sections={[
          {
            title: 'Workbench',
            href: `/projects/${projectIdNum}/workbench${planId ? `?planId=${planId}` : ''}`,
          },
          { title: 'Projections', href: `/projects/${projectIdNum}/workbench/projections${planId ? `?planId=${planId}` : ''}` },
        ]}
      />

      <Group justify="between" className="mb-4 flex-wrap">
        <div>
          <h1 className="text-3xl font-bold">Projections</h1>
          <p className="text-muted-foreground">Build interactive views for your graphs.</p>
        </div>
        <Group gap="sm">
          <Button variant="secondary" onClick={() => navigate(`/projects/${projectIdNum}/graphs${planId ? `?planId=${planId}` : ''}`)}>
            <IconDatabase className="mr-2 h-4 w-4" />
            Graphs
          </Button>
        </Group>
      </Group>

      <Card className="mb-6">
        <CardHeader>
          <CardTitle>Create projection</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Group gap="md" className="flex-wrap">
            <Input
              placeholder="Projection name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              className="w-[260px]"
            />
            <Select value={newGraphId} onValueChange={(value) => setNewGraphId(value)}>
              <SelectTrigger className="w-[240px]">
                <SelectValue placeholder="Select graph" />
              </SelectTrigger>
              <SelectContent>
                {graphs.map((g: any) => (
                  <SelectItem key={g.id} value={g.id.toString()}>
                    {g.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={newType} onValueChange={(value) => setNewType(value as 'force3d' | 'layer3d')}>
              <SelectTrigger className="w-[180px]">
                <SelectValue placeholder="Projection type" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="force3d">Force 3D</SelectItem>
                <SelectItem value="layer3d">Layer 3D</SelectItem>
              </SelectContent>
            </Select>
            <Button onClick={handleCreate} disabled={creating}>
              {creating && <Spinner className="mr-2 h-4 w-4" />}
              Create
            </Button>
          </Group>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Saved projections</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {projections.length === 0 ? (
            <p className="text-muted-foreground">No projections yet.</p>
          ) : (
            projections.map((p: any) => (
              <div key={p.id} className="flex items-center justify-between rounded-md border p-3">
                <div>
                  <Group gap="sm" align="center" className="flex-wrap">
                    <IconAffiliate className="h-4 w-4 text-muted-foreground" />
                    <div className="font-semibold">{p.name}</div>
                    <div className="text-xs uppercase text-muted-foreground">{p.projectionType}</div>
                  </Group>
                  <div className="text-xs text-muted-foreground">Graph #{p.graphId}</div>
                </div>
                <Group gap="sm" className="flex-wrap">
                  <Button size="sm" variant="outline" onClick={() => handleOpen(p.id)}>
                    <IconExternalLink className="mr-2 h-4 w-4" />
                    Open
                  </Button>
                  <Button size="sm" variant="outline" onClick={() => handleExport(p.id, p.name)} disabled={exporting}>
                    <IconUpload className="mr-2 h-4 w-4" />
                    Export
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    className="text-destructive hover:text-destructive"
                    onClick={() => handleDelete(p.id)}
                    disabled={deleting}
                  >
                    Delete
                  </Button>
                </Group>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </PageContainer>
  )
}

export default ProjectionsPage
