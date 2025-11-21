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
import { GET_PLAN_DAG, VALIDATE_AND_MIGRATE_PLAN_DAG } from '@/graphql/plan-dag'
import { IconGraph, IconLayout2, IconDatabase, IconArrowRight, IconNetwork, IconAdjustments } from '@tabler/icons-react'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useProjectPlanSelection } from '@/hooks/useProjectPlanSelection'

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
    selectedPlan,
    loading: plansLoading,
    selectPlan,
  } = useProjectPlanSelection(projectIdNum)

  const { data: planDagData, loading: planDagLoading } = useQuery(GET_PLAN_DAG, {
    variables: { projectId: projectIdNum, planId: selectedPlanId },
    skip: !projectIdNum || !selectedPlanId,
  })

  const planDag = (planDagData as any)?.getPlanDag
  const planNodeCount = planDag?.nodes?.length || 0
  const planEdgeCount = planDag?.edges?.length || 0
  const datasetNodeCount = planDag?.nodes?.filter((n: any) => n.nodeType === 'DataSetNode').length || 0
  const planQuerySuffix = selectedPlanId ? `?planId=${selectedPlanId}` : ''

  const [validatePlanDagMutation, { loading: validatePlanDagLoading }] = useMutation(
    VALIDATE_AND_MIGRATE_PLAN_DAG
  )

  const handleValidateAndMigratePlan = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    if (!selectedPlanId) {
      showErrorNotification('Select a plan', 'Choose a plan to validate before running checks.')
      return
    }
    try {
      const { data } = await validatePlanDagMutation({
        variables: { projectId: projectIdNum, planId: selectedPlanId },
      })
      const result = (data as any)?.validateAndMigratePlanDag
      const migratedCount = result?.updatedNodes?.length || 0
      const warningCount = result?.warnings?.length || 0
      const errors: string[] = result?.errors || []

      if (errors.length > 0) {
        showErrorNotification(
          'Plan DAG validation failed',
          `Found ${errors.length} error(s). First: ${errors[0]}`
        )
        console.error('Plan DAG validation errors', errors)
        return
      }

      showSuccessNotification(
        'Plan DAG validated',
        `Migrated ${migratedCount} legacy node(s). Warnings: ${warningCount}.`
      )
    } catch (error: any) {
      console.error('Failed to validate/migrate plan DAG', error)
      showErrorNotification(
        'Plan validation failed',
        error?.message || 'Unable to validate or migrate the plan DAG.'
      )
    }
  }

  const loading = projectsLoading || planDagLoading || plansLoading

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

  const handleOpenPlanNodes = () => {
    navigate(`/projects/${project.id}/plan-nodes${planQuerySuffix}`)
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
            onClick={handleValidateAndMigratePlan}
            disabled={validatePlanDagLoading || !selectedPlanId}
          >
            {validatePlanDagLoading && <Spinner className="mr-2 h-4 w-4" />}
            <IconAdjustments className="mr-2 h-4 w-4" />
            Validate &amp; migrate plan
          </Button>
          <Button variant="secondary" onClick={() => navigate(`/projects/${project.id}/workbench/layers`)}>
            Layers
          </Button>
          <Button variant="secondary" onClick={handleOpenPlanEditor}>
            <IconGraph className="mr-2 h-4 w-4" />
            Open plan editor
          </Button>
          <Button variant="secondary" onClick={handleOpenPlanNodes}>
            <IconLayout2 className="mr-2 h-4 w-4" />
            Plan nodes
          </Button>
          <Button variant="secondary" onClick={handleOpenGraphs}>
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
              {selectedPlan ? `Plan summary · ${selectedPlan.name}` : 'Plan summary'}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2">
            <p className="text-xs text-muted-foreground">
              {selectedPlan?.description || 'Track node counts and execution state for the active plan.'}
            </p>
            <Group gap="sm">
              <Badge variant="secondary">Nodes: {planNodeCount}</Badge>
              <Badge variant="secondary">Edges: {planEdgeCount}</Badge>
            </Group>
            <p className="text-sm text-muted-foreground">
              Version: {planDag?.version ?? 'n/a'}
            </p>
            <Button
              variant="secondary"
              className="w-full"
              onClick={handleOpenPlanEditor}
              disabled={!selectedPlanId}
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
              <Button className="flex-1" variant="secondary" onClick={handleOpenPlanNodes}>
                Plan nodes
              </Button>
              <Button className="flex-1" variant="secondary" onClick={handleOpenGraphs}>
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
