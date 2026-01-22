import { useMemo, useState } from 'react'
import { useMutation, useQuery } from '@apollo/client/react'
import { useNavigate, useParams } from 'react-router-dom'
import { gql } from '@apollo/client'
import { LIST_PLANS, DELETE_PLAN, DUPLICATE_PLAN } from '@/graphql/plans'
import { Plan } from '@/types/plan'
import { Breadcrumbs } from '@/components/common/Breadcrumbs'
import PageContainer from '@/components/layout/PageContainer'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Group, Stack } from '@/components/layout-primitives'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'
import { IconDots, IconPlus, IconArrowsMaximize, IconCopy, IconTrash } from '@tabler/icons-react'
import { Spinner } from '@/components/ui/spinner'
import { CreatePlanModal } from './CreatePlanModal'
import { EditPlanModal } from './EditPlanModal'
import { showErrorNotification, showSuccessNotification } from '@/utils/notifications'

const GET_PROJECT = gql`
  query GetProject($id: Int!) {
    project(id: $id) {
      id
      name
      description
    }
  }
`

const formatDateTime = (value: string) => {
  try {
    return new Date(value).toLocaleString()
  } catch {
    return value
  }
}

export const PlansPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = Number(projectId || 0)
  const [createModalOpen, setCreateModalOpen] = useState(false)
  const [editModalOpen, setEditModalOpen] = useState(false)
  const [planToEdit, setPlanToEdit] = useState<Plan | null>(null)

  const { data: projectData } = useQuery<{ project: { id: number; name: string; description?: string | null } }>(GET_PROJECT, {
    variables: { id: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'cache-and-network',
  })

  const {
    data,
    loading,
    error,
    refetch,
  } = useQuery<{ plans: Plan[] }>(LIST_PLANS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'cache-and-network',
  })

  const [deletePlan] = useMutation(DELETE_PLAN, {
    onCompleted: () => {
      showSuccessNotification('Plan deleted', 'The plan was deleted successfully.')
      refetch()
    },
    onError: (err: Error) => {
      console.error('Failed to delete plan', err)
      showErrorNotification('Delete failed', err.message)
    },
  })

  const [duplicatePlan] = useMutation(DUPLICATE_PLAN, {
    onCompleted: () => {
      showSuccessNotification('Plan duplicated', 'Copied plan was created successfully.')
      refetch()
    },
    onError: (err: Error) => {
      console.error('Failed to duplicate plan', err)
      showErrorNotification('Duplicate failed', err.message)
    },
  })

  const plans = data?.plans ?? []
  const project = projectData?.project

  const handleOpenPlan = (plan: Plan) => {
    navigate(`/projects/${plan.projectId}/plans/${plan.id}`)
  }

  const handleEditPlan = (plan: Plan) => {
    setPlanToEdit(plan)
    setEditModalOpen(true)
  }

  const handleDuplicatePlan = async (plan: Plan) => {
    const duplicateName = `${plan.name} Copy`
    await duplicatePlan({ variables: { id: plan.id, name: duplicateName } })
  }

  const handleDeletePlan = async (plan: Plan) => {
    const confirmed = window.confirm(
      `Delete plan "${plan.name}"? This will remove all plan nodes and edges.`
    )
    if (!confirmed) return
    await deletePlan({ variables: { id: plan.id } })
  }

  const copyTextToClipboard = async (value: string, label: string) => {
    if (!value) return
    try {
      if (navigator?.clipboard) {
        await navigator.clipboard.writeText(value)
      } else {
        const textArea = document.createElement('textarea')
        textArea.value = value
        document.body.appendChild(textArea)
        textArea.select()
        document.execCommand('copy')
        document.body.removeChild(textArea)
      }
      showSuccessNotification('Copied ID', label)
    } catch (error) {
      console.error('Failed to copy plan ID', error)
      showErrorNotification('Copy failed', 'Unable to copy the plan identifier.')
    }
  }

  const handleCopyPlanId = (plan: Plan) => {
    const canonicalId = `plan:${plan.projectId}:${plan.id}`
    copyTextToClipboard(canonicalId, canonicalId)
  }

  const pageTitle = useMemo(() => {
    if (project) {
      return `${project.name} · Plans`
    }
    return 'Plans'
  }, [project])

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={project?.name}
        projectId={project?.id}
        sections={[
          { title: 'Plans', href: `/projects/${projectIdNum}/plans` },
        ]}
        currentPage="Plans"
        onNavigate={(route) => navigate(route)}
      />

      <Group justify="between" className="mt-4 mb-6">
        <div>
          <h1 className="text-3xl font-bold">{pageTitle}</h1>
          <p className="text-muted-foreground">
            Create, duplicate, and manage multiple DAGs within this project.
          </p>
        </div>
        <Button onClick={() => setCreateModalOpen(true)}>
          <IconPlus className="mr-2 h-4 w-4" />
          New plan
        </Button>
      </Group>

      {loading && (
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading plans…</span>
        </Group>
      )}

      {error && (
        <p className="text-sm text-destructive">Failed to load plans: {error.message}</p>
      )}

      {!loading && plans.length === 0 && (
        <Card className="border-dashed">
          <CardContent className="py-10">
            <Stack gap="sm" className="text-center">
              <p className="text-muted-foreground">No plans yet.</p>
              <Button variant="secondary" onClick={() => setCreateModalOpen(true)}>
                <IconPlus className="mr-2 h-4 w-4" />
                Create your first plan
              </Button>
            </Stack>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
        {plans.map((plan: Plan) => {
          const canonicalPlanId = `plan:${plan.projectId}:${plan.id}`
          return (
            <Card key={plan.id} className="border">
            <CardHeader className="flex flex-row items-start justify-between space-y-0">
              <div>
                <CardTitle className="text-lg font-semibold">{plan.name}</CardTitle>
                <p className="text-xs text-muted-foreground">
                  Updated {formatDateTime(plan.updatedAt)}
                </p>
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon">
                    <IconDots className="h-4 w-4" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48">
                  <DropdownMenuItem onClick={() => handleEditPlan(plan)}>
                    <IconArrowsMaximize className="mr-2 h-4 w-4" />
                    Edit details
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => handleDuplicatePlan(plan)}>
                    <IconCopy className="mr-2 h-4 w-4" />
                    Duplicate
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem className="text-destructive" onClick={() => handleDeletePlan(plan)}>
                    <IconTrash className="mr-2 h-4 w-4" />
                    Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </CardHeader>
            <CardContent className="space-y-4">
              <p className="text-sm text-muted-foreground line-clamp-3">
                {plan.description || 'No description provided.'}
              </p>
              <Group gap="xs" wrap>
                {plan.tags.length ? (
                  plan.tags.map((tag: string) => (
                    <Badge key={tag} variant="outline">
                      {tag}
                    </Badge>
                  ))
                ) : (
                  <Badge variant="secondary">untagged</Badge>
                )}
              </Group>
              <Stack gap="xs">
                <Button variant="secondary" onClick={() => handleOpenPlan(plan)}>
                  <IconArrowsMaximize className="mr-2 h-4 w-4" />
                  Open plan
                </Button>
                <p className="text-xs text-muted-foreground">
                  Version {plan.version} · Created {formatDateTime(plan.createdAt)}
                </p>
              </Stack>
              <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
                <span className="font-mono break-all">{canonicalPlanId}</span>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-6 w-6 rounded-full border border-border text-muted-foreground hover:text-foreground hover:border-foreground"
                  onClick={() => handleCopyPlanId(plan)}
                  aria-label="Copy plan ID"
                >
                  <IconCopy className="h-4 w-4" />
                </Button>
              </div>
            </CardContent>
          </Card>
        )
      })}
      </div>

      <CreatePlanModal
        projectId={projectIdNum}
        open={createModalOpen}
        onOpenChange={setCreateModalOpen}
        onCreated={refetch}
      />
      <EditPlanModal
        plan={planToEdit}
        open={editModalOpen}
        onOpenChange={setEditModalOpen}
        onUpdated={refetch}
      />
    </PageContainer>
  )
}
