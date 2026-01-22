import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { RESET_PROJECT } from '@/graphql/libraryItems'
import {
  IconGraph,
  IconDatabase,
  IconArrowRight,
  IconAdjustments,
  IconBooks,
  IconHierarchy2,
  IconAffiliate,
} from '@tabler/icons-react'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useProjectPlanSelection } from '@/hooks/useProjectPlanSelection'
import { LIST_STORIES, type Story } from '@/graphql/stories'

const GET_PROJECTS = gql`
  query GetProjectsForWorkbench {
    projects {
      id
      name
      description
      updatedAt
    }
  }
`

export const WorkbenchPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = Number(projectId || 0)

  const { data: projectsData, loading: projectsLoading } = useQuery(GET_PROJECTS)
  const projects = (projectsData as any)?.projects || []
  const project = useMemo(
    () => projects.find((p: any) => p.id === projectIdNum),
    [projects, projectIdNum]
  )

  const {
    plans,
    selectedPlanId,
    loading: plansLoading,
    selectPlan,
  } = useProjectPlanSelection(projectIdNum)
  const { data: storiesData, loading: storiesLoading } = useQuery<{ stories: Story[] }>(LIST_STORIES, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'cache-and-network',
  })

  const planQuerySuffix = selectedPlanId ? `?planId=${selectedPlanId}` : ''
  const stories = storiesData?.stories ?? []

  const [resetProjectMutation, { loading: resetProjectLoading }] = useMutation(RESET_PROJECT)

  const handleResetProject = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    if (!window.confirm('Are you sure you want to reset this project? This will re-initialise the project with fresh IDs while preserving all data.')) {
      return
    }
    try {
      const { data } = await resetProjectMutation({
        variables: { projectId: projectIdNum },
      })
      const result = (data as any)?.resetProject

      if (result) {
        showSuccessNotification(
          'Project reset successfully',
          `The project "${result.name}" has been reset with fresh IDs. Please refresh the page to see the changes.`
        )
        // Redirect to the new project
        window.location.href = `/projects/${result.id}/workbench`
      }
    } catch (error: any) {
      console.error('Failed to reset project', error)
      showErrorNotification('Project reset failed', error?.message || 'Unable to reset the project.')
    }
  }

  const loading = projectsLoading || plansLoading

  const formatUpdatedAt = (value?: string | null) => {
    if (!value) {
      return '—'
    }
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) {
      return '—'
    }
    return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
  }

  if (loading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading workbench...</span>
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

  const handleOpenPlanEditor = () => {
    if (selectedPlanId) {
      navigate(`/projects/${project.id}/plans/${selectedPlanId}`)
    } else {
      navigate(`/projects/${project.id}/plans`)
    }
  }

  const handleOpenGraphs = () => {
    navigate(`/projects/${project.id}/graphs${planQuerySuffix}`)
  }

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project.name}
        projectId={project.id}
        currentPage="Workbench"
        onNavigate={(route) => navigate(route)}
        sections={[{ title: 'Workbench', href: `/projects/${project.id}/workbench${planQuerySuffix}` }]}
      />

      <Group justify="between" className="mb-6">
        <div>
          <h1 className="text-3xl font-bold">Workbench</h1>
          <p className="text-muted-foreground">Overview of your plan and graph build tools.</p>
        </div>
        <Group gap="sm" className="flex-wrap justify-end">
          <Select
            value={selectedPlanId ? selectedPlanId.toString() : ''}
            onValueChange={(value) => selectPlan(Number(value))}
            disabled={plans.length === 0}
          >
            <SelectTrigger className="w-[220px]">
              <SelectValue placeholder={plans.length ? 'Select a plan' : 'No plans available'} />
            </SelectTrigger>
            <SelectContent>
              {plans.map((plan) => (
                <SelectItem key={plan.id} value={plan.id.toString()}>
                  {plan.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/plans`)}>
            Manage plans
          </Button>
          <Button
            variant="secondary"
            onClick={handleResetProject}
            disabled={resetProjectLoading}
          >
            {resetProjectLoading && <Spinner className="mr-2 h-4 w-4" />}
            <IconAdjustments className="mr-2 h-4 w-4" />
            Reset project
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/workbench/layers`)}>
            Layers
          </Button>
          <Button variant="secondary" onClick={handleOpenPlanEditor}>
            <IconGraph className="mr-2 h-4 w-4" />
            Open plan editor
          </Button>
          <Button variant="secondary" onClick={handleOpenGraphs}>
            <IconDatabase className="mr-2 h-4 w-4" />
            Graphs
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/workbench/projections${planQuerySuffix}`)}>
            <IconAffiliate className="mr-2 h-4 w-4" />
            Projections
          </Button>
        </Group>
      </Group>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconGraph className="h-4 w-4 text-primary" />
              Plan summary
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">Plans</p>
              {plansLoading ? (
                <p className="text-sm text-muted-foreground">Loading plans…</p>
              ) : plans.length ? (
                <div className="space-y-1 max-h-48 overflow-y-auto pr-1">
                  {plans.map((plan) => (
                    <button
                      key={plan.id}
                      type="button"
                      className="w-full rounded-md border px-3 py-2 text-left text-sm hover:bg-muted transition flex flex-col gap-1"
                      onClick={() => navigate(`/projects/${project.id}/plans/${plan.id}`)}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-medium truncate">{plan.name}</span>
                        {plan.status && (
                          <Badge variant="outline" className="text-[11px] uppercase">
                            {plan.status}
                          </Badge>
                        )}
                      </div>
                      <p className="text-xs text-muted-foreground">
                        Updated {formatUpdatedAt(plan.updatedAt)}
                      </p>
                    </button>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">No plans yet.</p>
              )}
            </div>
            <Button variant="secondary" className="w-full" onClick={handleOpenPlanEditor}>
              <IconArrowRight className="mr-2 h-4 w-4" />
              Manage plans
            </Button>
          </CardContent>
        </Card>

        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconBooks className="h-4 w-4 text-primary" />
              Stories
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Craft walkthroughs for stakeholders and capture graph sequences.
            </p>
            {storiesLoading ? (
              <p className="text-sm text-muted-foreground">Loading stories…</p>
            ) : stories.length ? (
              <div className="space-y-1 max-h-48 overflow-y-auto pr-1">
                {stories.map((story) => (
                  <button
                    key={story.id}
                    type="button"
                    className="w-full rounded-md border px-3 py-2 text-left text-sm hover:bg-muted transition flex flex-col gap-1"
                    onClick={() => navigate(`/projects/${project.id}/stories/${story.id}`)}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-medium truncate">{story.name}</span>
                      <Badge variant="outline">{story.sequenceCount} seq</Badge>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      Updated {formatUpdatedAt(story.updatedAt)}
                    </p>
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-sm text-muted-foreground">No stories yet.</p>
            )}
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => navigate(`/projects/${project.id}/stories`)}
            >
              <IconArrowRight className="mr-2 h-4 w-4" />
              Manage stories
            </Button>
          </CardContent>
        </Card>

        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconHierarchy2 className="h-4 w-4 text-primary" />
              Build views
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Jump straight into the editors for layers, generated graphs, and artefacts.
            </p>
            <div className="space-y-2">
              <Button
                variant="secondary"
                className="w-full justify-start gap-2"
                onClick={() => navigate(`/projects/${project.id}/workbench/layers`)}
              >
                <IconHierarchy2 className="h-4 w-4" />
                Layer palette
              </Button>
              <Button
                variant="secondary"
                className="w-full justify-start gap-2"
                onClick={handleOpenGraphs}
              >
                <IconGraph className="h-4 w-4" />
                Generated graphs
              </Button>
              <Button
                variant="secondary"
                className="w-full justify-start gap-2"
                onClick={() => navigate(`/projects/${project.id}/artefacts`)}
              >
                <IconDatabase className="h-4 w-4" />
                Artefacts
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  )
}

export default WorkbenchPage
