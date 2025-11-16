import { useMemo } from 'react'
import { useNavigate, useParams } from 'react-router-dom'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Group } from '@/components/layout-primitives'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { GET_PLAN_DAG } from '@/graphql/plan-dag'
import { IconGraph, IconLayout2, IconDatabase, IconArrowRight, IconNetwork } from '@tabler/icons-react'

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

  const { data: planDagData, loading: planDagLoading } = useQuery(GET_PLAN_DAG, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
  })

  const planDag = (planDagData as any)?.getPlanDag
  const planNodeCount = planDag?.nodes?.length || 0
  const planEdgeCount = planDag?.edges?.length || 0
  const datasetNodeCount = planDag?.nodes?.filter((n: any) => n.nodeType === 'DataSetNode').length || 0

  const loading = projectsLoading || planDagLoading

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

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project.name}
        projectId={project.id}
        currentPage="Workbench"
        onNavigate={(route) => navigate(route)}
        sections={[{ title: 'Workbench', href: `/projects/${project.id}/workbench` }]}
      />

      <Group justify="between" className="mb-6">
        <div>
          <h1 className="text-3xl font-bold">Workbench</h1>
          <p className="text-muted-foreground">Overview of your plan and graph build tools.</p>
        </div>
        <Group gap="sm">
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/plan`)}>
            <IconGraph className="mr-2 h-4 w-4" />
            Open plan editor
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/plan-nodes`)}>
            <IconLayout2 className="mr-2 h-4 w-4" />
            Plan nodes
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/graphs`)}>
            <IconDatabase className="mr-2 h-4 w-4" />
            Graphs
          </Button>
        </Group>
      </Group>

      <div className="grid gap-4 md:grid-cols-3">
        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconGraph className="h-4 w-4 text-primary" />
              Plan summary
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <Group gap="sm">
              <Badge variant="secondary">Nodes: {planNodeCount}</Badge>
              <Badge variant="secondary">Edges: {planEdgeCount}</Badge>
            </Group>
            <p className="text-sm text-muted-foreground">
              Version: {planDag?.version || 'n/a'}
            </p>
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => navigate(`/projects/${project.id}/plan`)}
            >
              <IconArrowRight className="mr-2 h-4 w-4" />
              Edit plan
            </Button>
          </CardContent>
        </Card>

        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconDatabase className="h-4 w-4 text-primary" />
              Datasets in plan
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <Badge variant="secondary">{datasetNodeCount}</Badge>
            <p className="text-sm text-muted-foreground">
              Count of dataset nodes referenced in the plan DAG.
            </p>
            <Button
              variant="secondary"
              className="w-full"
              onClick={() => navigate(`/projects/${project.id}/datasets`)}
            >
              <IconArrowRight className="mr-2 h-4 w-4" />
              Manage datasets
            </Button>
          </CardContent>
        </Card>

        <Card className="border">
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base font-semibold">
              <IconNetwork className="h-4 w-4 text-primary" />
              Outputs & graphs
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-sm text-muted-foreground">
              Review plan nodes and generated graphs, or jump to graph listings.
            </p>
            <Group gap="xs">
              <Button className="flex-1" variant="secondary" onClick={() => navigate(`/projects/${project.id}/plan-nodes`)}>
                Plan nodes
              </Button>
              <Button className="flex-1" variant="secondary" onClick={() => navigate(`/projects/${project.id}/graphs`)}>
                Graphs
              </Button>
            </Group>
          </CardContent>
        </Card>
      </div>
    </PageContainer>
  )
}

export default WorkbenchPage
