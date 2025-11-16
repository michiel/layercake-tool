import React, { useMemo, useState } from 'react'
import { Routes, Route, useNavigate, useParams, useLocation } from 'react-router-dom'
import { IconGraph, IconServer, IconDatabase, IconPlus, IconSettings, IconFileDatabase, IconTrash, IconDownload, IconChevronLeft, IconChevronRight, IconFolderPlus, IconNetwork, IconBooks, IconMessageDots, IconAdjustments, IconUpload, IconHierarchy2, IconChevronDown } from '@tabler/icons-react'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { Breadcrumbs } from './components/common/Breadcrumbs'
import { PlanVisualEditor } from './components/editors/PlanVisualEditor/PlanVisualEditor'
import { ErrorBoundary } from './components/common/ErrorBoundary'
import { DataSetsPage } from './components/datasets/DataSetsPage'
import { DataSetEditor } from './components/datasets/DataSetEditor'
import { LibraryPage } from './components/library/LibraryPage'
import { CreateProjectModal } from './components/project/CreateProjectModal'
import { TopBar } from './components/layout/TopBar'
import { useCollaborationV2 } from './hooks/useCollaborationV2'
import { useConnectionStatus } from './hooks/useConnectionStatus'
import { ProjectChatPage } from './pages/ProjectChatPage'
import { ChatLogsPage } from './pages/ChatLogsPage'
import { SourceManagementPage } from './pages/SourceManagementPage'
import { KnowledgeBasePage } from './pages/KnowledgeBasePage'
import { DatasetCreationPage } from './pages/DatasetCreationPage'
import { ProjectArtefactsPage } from './pages/ProjectArtefactsPage'
import { getOrCreateSessionId } from './utils/session'
import { Group, Stack } from './components/layout-primitives'
import { Button } from './components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from './components/ui/card'
import { Badge } from './components/ui/badge'
import { Alert, AlertDescription } from './components/ui/alert'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from './components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './components/ui/select'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from './components/ui/tooltip'
import { Separator } from './components/ui/separator'
import { Spinner } from './components/ui/spinner'
import { ChatProvider } from './components/chat/ChatProvider'
import { useRegisterChatContext } from './hooks/useRegisterChatContext'
import { cn } from './lib/utils'
import { useTagsFilter } from './hooks/useTagsFilter'
import { EXPORT_PROJECT_ARCHIVE, EXPORT_PROJECT_AS_TEMPLATE } from './graphql/libraryItems'
import { VALIDATE_AND_MIGRATE_PLAN_DAG } from './graphql/plan-dag'
import { showErrorNotification, showSuccessNotification } from './utils/notifications'

// Collaboration Context for providing project-level collaboration to all pages
const CollaborationContext = React.createContext<any>(null)
export const useCollaboration = () => React.useContext(CollaborationContext)

type ProjectNavChild = {
  key: string
  label: string
  route: string
  isActive: () => boolean
}

type ProjectNavSection = {
  key: string
  label: string
  icon: React.ReactNode
  route: string
  isActive: () => boolean
  children?: ProjectNavChild[]
}

// Query to fetch projects
const GET_PROJECTS = gql`
  query GetProjects($tags: [String!]) {
    projects(tags: $tags) {
      id
      name
      description
      tags
      createdAt
      updatedAt
    }
  }
`

// Mutation to delete a project
const DELETE_PROJECT = gql`
  mutation DeleteProject($id: ID!) {
    deleteProject(id: $id)
  }
`

const GET_SAMPLE_PROJECTS = gql`
  query GetSampleProjects {
    sampleProjects {
      key
      name
      description
    }
  }
`

const CREATE_SAMPLE_PROJECT = gql`
  mutation CreateSampleProject($sampleKey: String!) {
    createSampleProject(sampleKey: $sampleKey) {
      id
      name
      description
    }
  }
`

// Query to fetch Plan DAG for download
const GET_PLAN_DAG = gql`
  query GetPlanDag($projectId: Int!) {
    getPlanDag(projectId: $projectId) {
      version
      nodes {
        id
        nodeType
        position
        metadata
        config
      }
      edges {
        id
        source
        target
        metadata
      }
      metadata
    }
  }
`

// Query to fetch aggregate project statistics for overview
const GET_PROJECT_STATS = gql`
  query GetProjectStats($projectId: Int!) {
    projectStats(projectId: $projectId) {
      projectId
      documents {
        total
        indexed
        notIndexed
      }
      knowledgeBase {
        fileCount
        chunkCount
        lastIndexedAt
      }
      datasets {
        total
        byType
      }
    }
  }
`

// Layout wrapper component
const AppLayout = ({ children }: { children: React.ReactNode }) => {
  const navigate = useNavigate()
  const location = useLocation()

  // Generate stable session ID (only once per component mount)
  const [sessionId] = useState(() => getOrCreateSessionId());

  // Navigation collapse state
  const [navCollapsed, setNavCollapsed] = useState(false);

  // Track which navigation sections are collapsed
  const [collapsedSections, setCollapsedSections] = useState<Set<string>>(new Set());

  // Toggle section collapse state
  const toggleSectionCollapse = (sectionKey: string) => {
    setCollapsedSections(prev => {
      const next = new Set(prev)
      if (next.has(sectionKey)) {
        next.delete(sectionKey)
      } else {
        next.add(sectionKey)
      }
      return next
    })
  }

  // Get current route info for navigation highlighting
  const isActiveRoute = (path: string) => {
    if (path === '/') return location.pathname === '/'
    return location.pathname === path
  }

  const isActiveRoutePrefix = (path: string) => {
    if (path === '/') return location.pathname === '/'
    if (!location.pathname.startsWith(path)) {
      return false
    }
    const nextChar = location.pathname.charAt(path.length)
    return nextChar === '' || nextChar === '/'
  }

  // Extract project info from current path for navbar
  const projectId = useMemo(() => {
    const match = location.pathname.match(/\/projects\/(\d+)/)
    return match ? parseInt(match[1], 10) : undefined
  }, [location.pathname])

  // Initialize collaboration only if we're in a project context
  const collaboration = useCollaborationV2({
    projectId: projectId || 0,
    documentId: 'project-global',
    documentType: 'canvas',
    enableWebSocket: !!projectId,
    userInfo: {
      id: sessionId,
      name: 'Anonymous User',
      avatarColor: '#3b82f6'
    }
  })

  // Get overall connection status (GraphQL + WebSocket)
  const connectionStatus = useConnectionStatus({
    websocketConnectionState: collaboration.connectionState,
    enableWebSocket: !!projectId
  })

  const makeRouteMatcher = (path: string, options?: { prefix?: boolean }) => () =>
    options?.prefix ? isActiveRoutePrefix(path) : isActiveRoute(path)

  const projectNavSections = useMemo<ProjectNavSection[]>(() => {
    if (!projectId) {
      return []
    }

    const createSection = ({
      matchOptions,
      ...section
    }: Omit<ProjectNavSection, 'isActive'> & { matchOptions?: { prefix?: boolean } }) => {
      const matcher = makeRouteMatcher(section.route, matchOptions)
      return {
        ...section,
        isActive: () => {
          if (section.children?.some(child => child.isActive())) {
            return false
          }
          return matcher()
        },
      }
    }

    const dataAcquisitionChildren: ProjectNavChild[] = [
      {
        key: 'source-management',
        label: 'Document management',
        route: `/projects/${projectId}/data-acquisition/source-management`,
        isActive: makeRouteMatcher(`/projects/${projectId}/data-acquisition/source-management`),
      },
      {
        key: 'knowledge-base',
        label: 'Knowledge base',
        route: `/projects/${projectId}/data-acquisition/knowledge-base`,
        isActive: makeRouteMatcher(`/projects/${projectId}/data-acquisition/knowledge-base`),
      },
      {
        key: 'dataset-creation',
        label: 'Data set creation',
        route: `/projects/${projectId}/data-acquisition/datasets`,
        isActive: makeRouteMatcher(`/projects/${projectId}/data-acquisition/datasets`),
      },
      {
        key: 'data-sets',
        label: 'Data sets',
        route: `/projects/${projectId}/datasets`,
        isActive: makeRouteMatcher(`/projects/${projectId}/datasets`),
      },
    ]

    const graphCreationChildren: ProjectNavChild[] = [
      {
        key: 'plan',
        label: 'Plan',
        route: `/projects/${projectId}/plan`,
        isActive: makeRouteMatcher(`/projects/${projectId}/plan`),
      },
      {
        key: 'plan-nodes',
        label: 'Plan nodes',
        route: `/projects/${projectId}/plan-nodes`,
        isActive: makeRouteMatcher(`/projects/${projectId}/plan-nodes`),
      },
      {
        key: 'graphs',
        label: 'Graphs',
        route: `/projects/${projectId}/graphs`,
        isActive: makeRouteMatcher(`/projects/${projectId}/graphs`),
      },
    ]

    const chatChildren: ProjectNavChild[] = [
      {
        key: 'chat-logs',
        label: 'Chat logs',
        route: `/projects/${projectId}/chat/logs`,
        isActive: makeRouteMatcher(`/projects/${projectId}/chat/logs`),
      },
    ]

    return [
      createSection({
        key: 'overview',
        label: 'Project overview',
        icon: <IconFolderPlus className="h-4 w-4" />,
        route: `/projects/${projectId}`,
      }),
      createSection({
        key: 'data-acquisition',
        label: 'Data acquisition',
        icon: <IconDatabase className="h-4 w-4" />,
        route: dataAcquisitionChildren[0].route,
        children: dataAcquisitionChildren,
      }),
      createSection({
        key: 'graph-creation',
        label: 'Workbench',
        icon: <IconGraph className="h-4 w-4" />,
        route: graphCreationChildren[0].route,
        children: graphCreationChildren,
      }),
      createSection({
        key: 'artefacts',
        label: 'Artefacts',
        icon: <IconHierarchy2 className="h-4 w-4" />,
        route: `/projects/${projectId}/artefacts`,
      }),
      createSection({
        key: 'chat',
        label: 'Chat',
        icon: <IconMessageDots className="h-4 w-4" />,
        route: `/projects/${projectId}/chat`,
        matchOptions: { prefix: true },
        children: chatChildren,
      }),
    ]
  }, [projectId, location.pathname])

  const activeProjectNavKey = useMemo(() => {
    for (const section of projectNavSections) {
      const activeChild = section.children?.find(child => child.isActive())
      if (activeChild) {
        return `${section.key}:${activeChild.key}`
      }
      if (section.isActive()) {
        return section.key
      }
    }
    return undefined
  }, [projectNavSections])

  const projectsButtonActive = location.pathname.startsWith('/projects') && !activeProjectNavKey

  // Initialize collapsed sections - collapse all sections by default unless they have an active child
  React.useEffect(() => {
    const sectionsWithChildren = projectNavSections.filter(s => s.children && s.children.length > 0)
    const newCollapsed = new Set<string>()

    sectionsWithChildren.forEach(section => {
      const hasActiveChild = section.children!.some(child => child.isActive())
      if (!hasActiveChild) {
        newCollapsed.add(section.key)
      }
    })

    setCollapsedSections(newCollapsed)
  }, [projectId]) // Only re-run when project changes, not on every route change

  return (
    <div className="flex h-screen flex-col overflow-hidden">
      {/* Top Bar */}
      <header className="flex items-center border-b px-4">
        <div className="w-full max-w-full">
          <TopBar
            projectId={projectId}
            connectionState={connectionStatus.state}
            users={collaboration.users}
            currentUserId={sessionId}
            onNavigateHome={() => navigate('/')}
          />
        </div>
      </header>

      <div className="flex flex-1 min-h-0">
        {/* Sidebar Navigation */}
        <aside
          className={cn(
            'flex flex-col border-r transition-all duration-200',
            navCollapsed ? 'w-[60px]' : 'w-[250px]'
          )}
        >
          <div className={cn(navCollapsed ? 'p-2' : 'p-4', 'min-h-0 flex-1 overflow-y-auto')}>
          <Stack gap="xs" className="h-full">
            <TooltipProvider>
              <Group justify="between" className="mb-2">
                {!navCollapsed && <h4 className="text-lg font-semibold">Navigation</h4>}
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => setNavCollapsed(!navCollapsed)}
                      className="h-8 w-8"
                    >
                      {navCollapsed ? <IconChevronRight className="h-4 w-4" /> : <IconChevronLeft className="h-4 w-4" />}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">
                    {navCollapsed ? "Expand" : "Collapse"}
                  </TooltipContent>
                </Tooltip>
              </Group>

              <Stack gap="xs" className="flex-1">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/') ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      onClick={() => navigate('/')}
                    >
                      {navCollapsed ? (
                        <IconServer className="h-4 w-4" />
                      ) : (
                        <>
                          <IconServer className="h-4 w-4 mr-2" />
                          Home
                        </>
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Home</TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/library') ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      onClick={() => navigate('/library')}
                    >
                      {navCollapsed ? (
                        <IconBooks className="h-4 w-4" />
                      ) : (
                        <>
                          <IconBooks className="h-4 w-4 mr-2" />
                          Library
                        </>
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Library</TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={projectsButtonActive ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      onClick={() => navigate('/projects')}
                    >
                      {navCollapsed ? (
                        <IconDatabase className="h-4 w-4" />
                      ) : (
                        <>
                          <IconDatabase className="h-4 w-4 mr-2" />
                          Projects
                        </>
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Projects</TooltipContent>
                </Tooltip>

                {/* Project-specific navigation - only show when in a project */}
                {projectId && projectNavSections.length > 0 && (
                  <>
                    <Separator className="my-2" />
                    <Stack gap="xs">
                      {projectNavSections.map((section) => {
                        const sectionActive = section.isActive()
                        const hasChildren = section.children && section.children.length > 0
                        const hasActiveChild = hasChildren && section.children!.some(child => child.isActive())
                        const isExpanded = hasActiveChild || !collapsedSections.has(section.key)
                        const highlightChild = !sectionActive && hasActiveChild

                        return (
                          <div key={section.key}>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <div className="flex items-center gap-1">
                                  <Button
                                    variant={sectionActive ? 'default' : 'ghost'}
                                    className={cn(
                                      navCollapsed ? 'justify-center px-2' : 'justify-start',
                                      hasChildren && !navCollapsed ? 'flex-1' : 'w-full',
                                      highlightChild ? 'text-primary' : undefined
                                    )}
                                    aria-current={sectionActive || hasActiveChild ? 'page' : undefined}
                                    onClick={() => navigate(section.route)}
                                  >
                                    {section.icon}
                                    {!navCollapsed && <span className="ml-2">{section.label}</span>}
                                  </Button>
                                  {!navCollapsed && hasChildren && (
                                    <Button
                                      variant="ghost"
                                      size="icon"
                                      className="h-9 w-9 shrink-0"
                                      onClick={(e) => {
                                        e.stopPropagation()
                                        toggleSectionCollapse(section.key)
                                      }}
                                    >
                                      {isExpanded ? (
                                        <IconChevronDown className="h-4 w-4" />
                                      ) : (
                                        <IconChevronRight className="h-4 w-4" />
                                      )}
                                    </Button>
                                  )}
                                </div>
                              </TooltipTrigger>
                              <TooltipContent side="right">{section.label}</TooltipContent>
                            </Tooltip>
                            {!navCollapsed && hasChildren && isExpanded && (
                              <Stack gap="xs" className="pl-6 mt-1">
                                {section.children!.map((child) => {
                                  const childActive = child.isActive()
                                  return (
                                    <Button
                                      key={child.key}
                                      size="sm"
                                      variant={childActive ? 'default' : 'ghost'}
                                      aria-current={childActive ? 'page' : undefined}
                                      className="justify-start text-sm"
                                      onClick={() => navigate(child.route)}
                                    >
                                      {child.label}
                                    </Button>
                                  )
                                })}
                              </Stack>
                            )}
                          </div>
                        )
                      })}
                    </Stack>
                  </>
                )}
              </Stack>

              <div>
                <Separator className="my-2" />
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/settings/database') ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      onClick={() => navigate('/settings/database')}
                    >
                      {navCollapsed ? (
                        <IconSettings className="h-4 w-4" />
                      ) : (
                        <>
                          <IconSettings className="h-4 w-4 mr-2" />
                          Database Settings
                        </>
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Database Settings</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/settings/system') ? 'default' : 'ghost'}
                      className={`${navCollapsed ? 'justify-center px-2' : 'w-full justify-start'} mt-2`}
                      onClick={() => navigate('/settings/system')}
                    >
                      {navCollapsed ? (
                        <IconAdjustments className="h-4 w-4" />
                      ) : (
                        <>
                          <IconAdjustments className="h-4 w-4 mr-2" />
                          System Settings
                        </>
                      )}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">System Settings</TooltipContent>
                </Tooltip>
              </div>
            </TooltipProvider>
          </Stack>
          </div>
        </aside>

        {/* Main Content Area */}
        <div className="flex flex-1 flex-col overflow-hidden">
          <main className="flex-1 overflow-auto p-4">
            <CollaborationContext.Provider value={collaboration}>
              {children}
            </CollaborationContext.Provider>
          </main>
        </div>
      </div>
    </div>
  )
}

// Home page component
const HomePage = () => {
  const navigate = useNavigate()
  const { activeTags } = useTagsFilter()
  const [createModalOpened, setCreateModalOpened] = useState(false)
  const [sampleModalOpened, setSampleModalOpened] = useState(false)
  const [selectedSampleKey, setSelectedSampleKey] = useState<string | null>(null)
  const [sampleError, setSampleError] = useState<string | null>(null)

  const { data: projectsData } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      tags: string[]
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS, {
    variables: {
      tags: activeTags.length > 0 ? activeTags : null
    }
  })

  const { data: sampleProjectsData, loading: sampleProjectsLoading } = useQuery<{
    sampleProjects: Array<{
      key: string
      name: string
      description?: string | null
    }>
  }>(GET_SAMPLE_PROJECTS)

  const [createSampleProject, { loading: createSampleLoading }] = useMutation(CREATE_SAMPLE_PROJECT, {
    onCompleted: (result) => {
      const project = (result as any)?.createSampleProject
      if (project) {
        navigate(`/projects/${project.id}`)
        setSampleModalOpened(false)
        setSelectedSampleKey(null)
        setSampleError(null)
      }
    },
    onError: (error) => {
      setSampleError(error.message)
    },
    refetchQueries: [{ query: GET_PROJECTS }]
  })

  // Get 5 most recent projects
  const recentProjects = [...(projectsData?.projects || [])]
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    .slice(0, 5)

  const handleCreateProject = () => {
    setCreateModalOpened(true)
  }

  const handleProjectCreated = (project: { id: number; name: string; description?: string }) => {
    navigate(`/projects/${project.id}`)
  }

  const handleOpenSampleModal = () => {
    setSampleError(null)
    setSampleModalOpened(true)
  }

  const handleSampleModalClose = () => {
    setSampleModalOpened(false)
    setSelectedSampleKey(null)
    setSampleError(null)
  }

  const handleCreateSampleProject = async () => {
    if (!selectedSampleKey) {
      setSampleError('Please select a sample project')
      return
    }

    setSampleError(null)

    try {
      await createSampleProject({
        variables: {
          sampleKey: selectedSampleKey,
        },
      })
    } catch (error) {
      console.error('Failed to create sample project', error)
    }
  }

  const sampleOptions =
    sampleProjectsData?.sampleProjects?.map(sample => ({
      value: sample.key,
      label: sample.name,
      description: sample.description ?? undefined,
    })) ?? []

  const selectedSample = sampleOptions.find(option => option.value === selectedSampleKey)

  return (
    <div className="w-full h-full">
      {/* Action buttons section */}
      <div className="py-12 px-8 bg-muted/50 border-b">
        <Group justify="center" gap="xl">
          <Button
            size="lg"
            onClick={() => navigate('/projects')}
            className="min-w-[240px] h-20 text-lg"
          >
            <IconDatabase className="mr-2 h-6 w-6" />
            Browse Projects
          </Button>
          <Button
            size="lg"
            onClick={handleCreateProject}
            className="min-w-[240px] h-20 text-lg"
          >
            <IconPlus className="mr-2 h-6 w-6" />
            Start New Project
          </Button>
          <Button
            size="lg"
            onClick={handleOpenSampleModal}
            variant="secondary"
            className="min-w-[240px] h-20 text-lg"
          >
            <IconFolderPlus className="mr-2 h-6 w-6" />
            Import Sample Project
          </Button>
        </Group>
      </div>

      {/* Recent projects section */}
      <div className="p-8">
        <h2 className="text-2xl font-bold text-center mb-6">
          Recent Projects
        </h2>

        {recentProjects.length === 0 ? (
          <Card className="max-w-[600px] mx-auto border p-6">
            <div className="flex flex-col items-center gap-4">
              <IconGraph size={48} className="text-muted-foreground" />
              <h3 className="text-xl font-bold">No Projects Yet</h3>
              <p className="text-center text-muted-foreground">
                Create your first project to get started with Layercake.
              </p>
            </div>
          </Card>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fit,minmax(320px,1fr))] gap-6 max-w-[1600px] mx-auto">
            {recentProjects.map((project) => (
              <Card
                key={project.id}
                className="cursor-pointer h-full flex flex-col border shadow-sm hover:shadow-md transition-shadow"
                onClick={() => navigate(`/projects/${project.id}`)}
              >
                <div className="border-b bg-muted/50 p-3">
                  <Group justify="between">
                    <Group gap="xs">
                      <IconGraph className="h-5 w-5" />
                      <p className="font-semibold">{project.name}</p>
                    </Group>
                    <Badge variant="secondary">
                      ID: {project.id}
                    </Badge>
                  </Group>
                </div>

                <Stack gap="sm" className="flex-1 p-4">
                  {project.description && (
                    <p className="text-sm text-muted-foreground line-clamp-2">
                      {project.description}
                    </p>
                  )}

                  <div className="mt-auto">
                    <p className="text-xs text-muted-foreground">
                      Updated {new Date(project.updatedAt).toLocaleDateString()}
                    </p>
                  </div>
                </Stack>

                <div className="border-t p-3">
                  <Group gap="xs" justify="end">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={(e) => {
                        e.stopPropagation()
                        navigate(`/projects/${project.id}/plan`)
                      }}
                    >
                      <IconGraph className="mr-2 h-3.5 w-3.5" />
                      Plan
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={(e) => {
                        e.stopPropagation()
                        navigate(`/projects/${project.id}/datasets`)
                      }}
                    >
                      <IconFileDatabase className="mr-2 h-3.5 w-3.5" />
                      Data
                    </Button>
                  </Group>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>

      <CreateProjectModal
        opened={createModalOpened}
        onClose={() => setCreateModalOpened(false)}
        onSuccess={handleProjectCreated}
        defaultTags={activeTags}
      />

      <Dialog open={sampleModalOpened} onOpenChange={(open) => !open && handleSampleModalClose()}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Import Sample Project</DialogTitle>
          </DialogHeader>
          <Stack gap="md" className="py-4">
            <p className="text-sm text-muted-foreground">
              Select one of the bundled samples to create a project preloaded with data sets and a starter DAG.
            </p>

            <div className="space-y-2">
              <label className="text-sm font-medium">Sample Project</label>
              <Select
                value={selectedSampleKey || ''}
                onValueChange={setSelectedSampleKey}
                disabled={sampleProjectsLoading || sampleOptions.length === 0}
              >
                <SelectTrigger>
                  <SelectValue placeholder={sampleProjectsLoading ? 'Loading samples...' : 'Select a sample'} />
                </SelectTrigger>
                <SelectContent>
                  {sampleOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {selectedSample?.description && (
              <p className="text-sm text-muted-foreground">
                {selectedSample.description}
              </p>
            )}

            {sampleError && (
              <Alert variant="destructive">
                <AlertDescription>
                  <p className="font-semibold mb-1">Cannot create sample project</p>
                  <p className="text-sm">{sampleError}</p>
                </AlertDescription>
              </Alert>
            )}
          </Stack>
          <DialogFooter>
            <Button variant="ghost" onClick={handleSampleModalClose} disabled={createSampleLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateSampleProject}
              disabled={!selectedSampleKey || sampleProjectsLoading || createSampleLoading}
            >
              {createSampleLoading && <span className="mr-2">...</span>}
              Create Sample Project
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// Projects list page component
const ProjectsPage = () => {
  const navigate = useNavigate()
  const { activeTags } = useTagsFilter()
  const [createModalOpened, setCreateModalOpened] = useState(false)
  const [sampleModalOpened, setSampleModalOpened] = useState(false)
  const [selectedSampleKey, setSelectedSampleKey] = useState<string | null>(null)
  const [sampleError, setSampleError] = useState<string | null>(null)

  const { data: projectsData, loading: projectsLoading, error: projectsError, refetch } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      tags: string[]
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS, {
    variables: {
      tags: activeTags.length > 0 ? activeTags : null
    },
    errorPolicy: 'all',
  })

  const { data: sampleProjectsData, loading: sampleProjectsLoading, error: sampleProjectsError } = useQuery<{
    sampleProjects: Array<{
      key: string
      name: string
      description?: string | null
    }>
  }>(GET_SAMPLE_PROJECTS)

  const projects = projectsData?.projects || []

  const [deleteProject] = useMutation(DELETE_PROJECT, {
    refetchQueries: [{ query: GET_PROJECTS }],
  });

  const [createSampleProject, { loading: createSampleLoading }] = useMutation(CREATE_SAMPLE_PROJECT, {
    onCompleted: (result) => {
      const project = (result as any)?.createSampleProject
      if (project) {
        refetch()
        navigate(`/projects/${project.id}`)
        setSampleModalOpened(false)
        setSelectedSampleKey(null)
        setSampleError(null)
      }
    },
    onError: (error) => {
      setSampleError(error.message)
    }
  })

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleProjectSelect = (projectId: number) => {
    navigate(`/projects/${projectId}`)
  }

  const handleCreateProject = () => {
    setCreateModalOpened(true)
  }

  const handleProjectCreated = (project: { id: number; name: string; description?: string }) => {
    refetch() // Refresh the projects list
    navigate(`/projects/${project.id}`) // Navigate to the new project
  }

  const handleDeleteProject = (projectId: number) => {
    if (window.confirm('Are you sure you want to delete this project? This action cannot be undone.')) {
      deleteProject({ variables: { id: projectId } });
    }
  };

  const handleOpenSampleModal = () => {
    setSampleError(null)
    setSampleModalOpened(true)
  }

  const handleSampleModalClose = () => {
    setSampleModalOpened(false)
    setSelectedSampleKey(null)
    setSampleError(null)
  }

  const handleCreateSampleProject = async () => {
    if (!selectedSampleKey) {
      setSampleError('Please select a sample project')
      return
    }

    setSampleError(null)

    try {
      await createSampleProject({
        variables: {
          sampleKey: selectedSampleKey,
        },
      })
    } catch (error) {
      // Errors are reported via the mutation's onError handler
      console.error('Failed to create sample project', error)
    }
  }

  const sampleOptions =
    sampleProjectsData?.sampleProjects?.map(sample => ({
      value: sample.key,
      label: sample.name,
      description: sample.description ?? undefined,
    })) ?? []

  const selectedSample = sampleOptions.find(option => option.value === selectedSampleKey)

  return (
    <PageContainer>
      <Breadcrumbs currentPage="Projects" onNavigate={handleNavigate} />

      <Group justify="between" className="mb-4">
        <h1 className="text-3xl font-bold">Projects</h1>
        <Group gap="xs">
          <Button onClick={handleCreateProject}>
            <IconPlus className="mr-2 h-4 w-4" />
            New Project
          </Button>
          <Button variant="secondary" onClick={handleOpenSampleModal}>
            <IconFolderPlus className="mr-2 h-4 w-4" />
            Add Sample Project
          </Button>
        </Group>
      </Group>

      {projectsLoading && <p>Loading projects...</p>}

      {projectsError && (
        <p className="text-destructive mb-4">
          Error loading projects: {projectsError.message}
        </p>
      )}

      {projects.length === 0 && !projectsLoading && !projectsError && (
        <Card className="border p-6">
          <div className="flex flex-col items-center gap-4">
            <IconGraph size={48} className="text-muted-foreground" />
            <h3 className="text-xl font-bold">No Projects Yet</h3>
            <p className="text-center text-muted-foreground">
              Create your first project to start building Plan DAGs and transforming graphs.
            </p>
            <Button onClick={handleCreateProject}>
              <IconPlus className="mr-2 h-4 w-4" />
              Create First Project
            </Button>
          </div>
        </Card>
      )}

      {projects.length > 0 && (
        <Stack gap="md">
          {projects.map((project: any) => (
            <Card
              key={project.id}
              className="border p-4 cursor-pointer hover:shadow-md transition-shadow"
              onClick={() => handleProjectSelect(project.id)}
            >
              <Group justify="between" align="start">
                <div className="flex-1">
                  <Group gap="sm" className="mb-2">
                    <h4 className="text-lg font-semibold">{project.name}</h4>
                    <Badge variant="secondary">
                      ID: {project.id}
                    </Badge>
                  </Group>
                  {project.description && (
                    <p className="text-sm text-muted-foreground mb-2">
                      {project.description}
                    </p>
                  )}
                  <p className="text-xs text-muted-foreground">
                    Created: {new Date(project.createdAt).toLocaleDateString()}
                  </p>
                </div>
                <Group gap="xs">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation()
                      navigate(`/projects/${project.id}/plan`)
                    }}
                  >
                    <IconGraph className="mr-2 h-3.5 w-3.5" />
                    Plan
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-destructive hover:text-destructive/80"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteProject(project.id);
                    }}
                  >
                    <IconTrash className="mr-2 h-3.5 w-3.5" />
                    Delete
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation()
                      navigate(`/projects/${project.id}/edit`)
                    }}
                  >
                    <IconSettings className="mr-2 h-3.5 w-3.5" />
                    Settings
                  </Button>
                </Group>
              </Group>
            </Card>
          ))}
        </Stack>
      )}

      <CreateProjectModal
        opened={createModalOpened}
        onClose={() => setCreateModalOpened(false)}
        onSuccess={handleProjectCreated}
        defaultTags={activeTags}
      />

      <Dialog open={sampleModalOpened} onOpenChange={(open) => !open && handleSampleModalClose()}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>Add Sample Project</DialogTitle>
          </DialogHeader>
          <Stack gap="md" className="py-4">
            <p className="text-sm text-muted-foreground">
              Select one of the bundled samples to create a project preloaded with data sets and a starter DAG.
            </p>

            {sampleProjectsError && (
              <Alert variant="destructive">
                <AlertDescription>
                  <p className="font-semibold mb-1">Unable to load samples</p>
                  <p className="text-sm">{sampleProjectsError.message}</p>
                </AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium">Sample Project</label>
              <Select
                value={selectedSampleKey || ''}
                onValueChange={setSelectedSampleKey}
                disabled={sampleProjectsLoading || sampleOptions.length === 0}
              >
                <SelectTrigger>
                  <SelectValue placeholder={sampleProjectsLoading ? 'Loading samples...' : 'Select a sample'} />
                </SelectTrigger>
                <SelectContent>
                  {sampleOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {selectedSample?.description && (
              <p className="text-sm text-muted-foreground">
                {selectedSample.description}
              </p>
            )}

            {sampleError && (
              <Alert variant="destructive">
                <AlertDescription>
                  <p className="font-semibold mb-1">Cannot create sample project</p>
                  <p className="text-sm">{sampleError}</p>
                </AlertDescription>
              </Alert>
            )}
          </Stack>
          <DialogFooter>
            <Button variant="ghost" onClick={handleSampleModalClose} disabled={createSampleLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateSampleProject}
              disabled={!selectedSampleKey || sampleProjectsLoading || createSampleLoading}
            >
              {createSampleLoading && <span className="mr-2">...</span>}
              Create Sample Project
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

    </PageContainer>
  )
}

// Project detail page component
const ProjectDetailPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const projectIdNum = parseInt(projectId || '0')

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

  const { data: planDagData, refetch: refetchPlanDag } = useQuery(GET_PLAN_DAG, {
    variables: { projectId: projectIdNum },
    skip: !projectId,
  })

  const { data: projectStatsData } = useQuery<{
    projectStats: {
      projectId: number
      documents: {
        total: number
        indexed: number
        notIndexed: number
      }
      knowledgeBase: {
        fileCount: number
        chunkCount: number
        lastIndexedAt?: string | null
      }
      datasets: {
        total: number
        byType: Record<string, number>
      }
    }
  }>(GET_PROJECT_STATS, {
    variables: { projectId: projectIdNum },
    skip: !projectId,
  })

  const projects = projectsData?.projects || []
  const selectedProject = projects.find((p: any) => p.id === projectIdNum)
  const planDag = (planDagData as any)?.planDag

  const [exportDialogOpen, setExportDialogOpen] = useState(false)
  const [activeExportTab, setActiveExportTab] = useState<'archive' | 'template'>('archive')
  const [exportProjectArchiveMutation, { loading: exportArchiveLoading }] = useMutation(EXPORT_PROJECT_ARCHIVE)
  const [exportProjectAsTemplateMutation, { loading: exportTemplateLoading }] = useMutation(
    EXPORT_PROJECT_AS_TEMPLATE
  )
  const [validatePlanDagMutation, { loading: validatePlanDagLoading }] = useMutation(
    VALIDATE_AND_MIGRATE_PLAN_DAG
  )

  // Extract stats from single query
  const stats = projectStatsData?.projectStats
  const totalFiles = stats?.documents.total || 0
  const indexedFiles = stats?.documents.indexed || 0
  const notIndexedFiles = stats?.documents.notIndexed || 0
  const kbFileCount = stats?.knowledgeBase.fileCount || 0
  const kbChunkCount = stats?.knowledgeBase.chunkCount || 0
  const kbLastUpdate = stats?.knowledgeBase.lastIndexedAt
  const totalDatasets = stats?.datasets.total || 0
  const datasetsByType = stats?.datasets.byType || {}

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  // Download Plan DAG as YAML
  const handleDownloadYAML = () => {
    if (!planDag || !selectedProject) return

    // Convert Plan DAG to YAML-like structure
    const yamlContent = convertPlanDagToYAML(planDag)

    // Create filename from plan name
    const planName = planDag.metadata?.name || selectedProject.name || 'plan'
    const escapedName = planName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
    const filename = `${escapedName}-plan.yaml`

    // Create and download file
    const blob = new Blob([yamlContent], { type: 'text/yaml;charset=utf-8' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    console.log(`Downloaded Plan DAG as ${filename}`)
  }

  const handleValidateAndMigratePlan = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    try {
      const { data } = await validatePlanDagMutation({
        variables: { projectId: projectIdNum },
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
      if (typeof refetchPlanDag === 'function') {
        refetchPlanDag()
      }
      if (warningCount > 0) {
        console.warn('Plan DAG validation warnings', result?.warnings)
      }
    } catch (error: any) {
      console.error('Failed to validate/migrate plan DAG', error)
      showErrorNotification(
        'Plan validation failed',
        error?.message || 'Unable to validate or migrate the plan DAG.'
      )
    }
  }

  // Simple YAML converter
  const convertPlanDagToYAML = (dag: any): string => {
    const indent = (level: number) => '  '.repeat(level)

    const serializeValue = (value: any, level: number): string => {
      if (value === null || value === undefined) return 'null'
      if (typeof value === 'string') return `"${value.replace(/"/g, '\\"')}"`
      if (typeof value === 'number' || typeof value === 'boolean') return String(value)
      if (Array.isArray(value)) {
        if (value.length === 0) return '[]'
        return '\n' + value.map(item =>
          `${indent(level)}- ${serializeValue(item, level + 1).trim()}`
        ).join('\n')
      }
      if (typeof value === 'object') {
        const entries = Object.entries(value)
        if (entries.length === 0) return '{}'
        return '\n' + entries.map(([key, val]) =>
          `${indent(level)}${key}: ${serializeValue(val, level + 1).trim()}`
        ).join('\n')
      }
      return String(value)
    }

    let yaml = '# Plan DAG Configuration\n'
    yaml += `# Generated on ${new Date().toISOString()}\n\n`
    yaml += `version: "${dag.version || '1.0.0'}"\n\n`

    if (dag.metadata) {
      yaml += 'metadata:\n'
      Object.entries(dag.metadata).forEach(([key, value]) => {
        yaml += `  ${key}: ${serializeValue(value, 2).trim()}\n`
      })
      yaml += '\n'
    }

    yaml += 'nodes:\n'
    dag.nodes.forEach((node: any) => {
      yaml += `  - id: "${node.id}"\n`
      yaml += `    nodeType: "${node.nodeType}"\n`
      if (node.position) {
        yaml += `    position:\n`
        yaml += `      x: ${node.position.x}\n`
        yaml += `      y: ${node.position.y}\n`
      }
      if (node.metadata) {
        yaml += `    metadata:\n`
        Object.entries(node.metadata).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      if (node.config) {
        const config = typeof node.config === 'string' ? JSON.parse(node.config) : node.config
        yaml += `    config:\n`
        Object.entries(config).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      yaml += '\n'
    })

    yaml += 'edges:\n'
    dag.edges.forEach((edge: any) => {
      yaml += `  - id: "${edge.id}"\n`
      yaml += `    source: "${edge.source}"\n`
      yaml += `    target: "${edge.target}"\n`
      if (edge.metadata) {
        yaml += `    metadata:\n`
        Object.entries(edge.metadata).forEach(([key, value]) => {
          yaml += `      ${key}: ${serializeValue(value, 3).trim()}\n`
        })
      }
      yaml += '\n'
    })

    return yaml
  }

  const handleExportArchive = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    try {
      const { data } = await exportProjectArchiveMutation({
        variables: { projectId: projectIdNum },
      })
      const payload = (data as any)?.exportProjectArchive
      if (!payload) {
        throw new Error('Archive payload missing')
      }
      const binary = atob(payload.fileContent)
      const bytes = new Uint8Array(binary.length)
      for (let i = 0; i < binary.length; i += 1) {
        bytes[i] = binary.charCodeAt(i)
      }
      const blob = new Blob([bytes], { type: 'application/zip' })
      const url = URL.createObjectURL(blob)
      const anchor = document.createElement('a')
      anchor.href = url
      anchor.download = payload.filename || `${selectedProject?.name ?? 'project'}.zip`
      document.body.appendChild(anchor)
      anchor.click()
      document.body.removeChild(anchor)
      URL.revokeObjectURL(url)
      showSuccessNotification('Project exported', 'Download started.')
      setExportDialogOpen(false)
    } catch (error: any) {
      console.error('Failed to export project archive', error)
      showErrorNotification('Export failed', error?.message || 'Unable to export project.')
    }
  }

  const handleExportTemplate = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    try {
      await exportProjectAsTemplateMutation({
        variables: { projectId: projectIdNum },
      })
      showSuccessNotification('Template created', 'Find it in the Library under Templates.')
      setExportDialogOpen(false)
    } catch (error: any) {
      console.error('Failed to export project as template', error)
      showErrorNotification('Template export failed', error?.message || 'Unable to export template.')
    }
  }

  // Show loading state while projects are being fetched
  if (projectsLoading) {
    return (
      <PageContainer>
        <p>Loading project...</p>
      </PageContainer>
    )
  }

  // Only show "not found" if loading is complete and project doesn't exist
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

  const planNodeCount = planDag?.nodes?.length ?? 0
  const planEdgeCount = planDag?.edges?.length ?? 0
  const planVersion = planDag?.version ?? 'n/a'

  return (
    <PageContainer>
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        currentPage="Overview"
        onNavigate={handleNavigate}
      />

      <Group justify="between" className="mb-6">
        <div>
          <h1 className="text-3xl font-bold">{selectedProject.name}</h1>
          <Group gap="sm" className="mt-2">
            <Badge variant="secondary">ID: {selectedProject.id}</Badge>
            <Badge variant="default">Active</Badge>
          </Group>
        </div>
        <Group gap="sm">
          <Button variant="outline" onClick={() => setExportDialogOpen(true)}>
            <IconDownload className="mr-2 h-4 w-4" />
            Export Project
          </Button>
          <Button
            variant="outline"
            onClick={() => navigate(`/projects/${selectedProject.id}/edit`)}
          >
            <IconSettings className="mr-2 h-4 w-4" />
            Edit Details
          </Button>
        </Group>
      </Group>

      <Stack gap="xl">
        <section>
          <Group justify="between" className="mb-4">
            <div>
              <h2 className="text-2xl font-bold">Data Acquisition</h2>
              <p className="text-muted-foreground">
                Import files, manage ingestion, and monitor the knowledge base for this project.
              </p>
            </div>
            <Group gap="xs">
              <Button variant="secondary" onClick={() => navigate(`/projects/${projectId}/datasets`)}>
                Manage data sets
              </Button>
              <Button variant="secondary" onClick={() => navigate(`/projects/${projectId}/data-acquisition/source-management`)}>
                Upload files
              </Button>
            </Group>
          </Group>
          <div className="grid gap-4 md:grid-cols-3">
            <Card className="border hover:shadow-md transition-shadow">
              <CardHeader className="cursor-pointer" onClick={() => navigate(`/projects/${projectId}/data-acquisition/source-management`)}>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconUpload className="h-4 w-4 text-primary" />
                  Document management
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Total:</span>
                    <Badge variant="secondary">{totalFiles}</Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Indexed:</span>
                    <Badge variant="default">{indexedFiles}</Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Not indexed:</span>
                    <Badge variant="outline">{notIndexedFiles}</Badge>
                  </div>
                </div>
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/data-acquisition/source-management`)}>
                  Manage documents
                </Button>
              </CardContent>
            </Card>
            <Card className="border hover:shadow-md transition-shadow">
              <CardHeader className="cursor-pointer" onClick={() => navigate(`/projects/${projectId}/data-acquisition/knowledge-base`)}>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconDatabase className="h-4 w-4 text-primary" />
                  Knowledge base
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Files:</span>
                    <Badge variant="secondary">{kbFileCount}</Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Chunks:</span>
                    <Badge variant="secondary">{kbChunkCount}</Badge>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Last update:</span>
                    <span className="text-xs text-muted-foreground">
                      {kbLastUpdate ? new Date(kbLastUpdate).toLocaleDateString() : 'Never'}
                    </span>
                  </div>
                </div>
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/data-acquisition/knowledge-base`)}>
                  View knowledge base
                </Button>
              </CardContent>
            </Card>
            <Card className="border hover:shadow-md transition-shadow">
              <CardHeader className="cursor-pointer" onClick={() => navigate(`/projects/${projectId}/datasets`)}>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconFileDatabase className="h-4 w-4 text-primary" />
                  Data sets
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <span className="text-sm text-muted-foreground">Total:</span>
                    <Badge variant="secondary">{totalDatasets}</Badge>
                  </div>
                  {Object.entries(datasetsByType).map(([type, count]) => (
                    <div key={type} className="flex justify-between items-center">
                      <span className="text-sm text-muted-foreground capitalize">{type}:</span>
                      <Badge variant="outline">{count}</Badge>
                    </div>
                  ))}
                  {totalDatasets === 0 && (
                    <p className="text-sm text-muted-foreground italic">No datasets yet</p>
                  )}
                </div>
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/datasets`)}>
                  Open data sets
                </Button>
              </CardContent>
            </Card>
          </div>
        </section>

        <section>
          <Group justify="between" className="mb-4">
            <div>
              <h2 className="text-2xl font-bold">Workbench</h2>
              <p className="text-muted-foreground">
                Design, inspect, and export the Plan DAG along with all derived graphs.
              </p>
            </div>
            <Group gap="xs">
              <Button
                variant="secondary"
                onClick={handleValidateAndMigratePlan}
                disabled={validatePlanDagLoading}
              >
                {validatePlanDagLoading && <Spinner className="mr-2 h-4 w-4" />}
                <IconAdjustments className="mr-2 h-4 w-4" />
                Validate &amp; migrate plan
              </Button>
              <Button variant="secondary" onClick={() => handleDownloadYAML()} disabled={!planDag}>
                <IconDownload className="mr-2 h-4 w-4" />
                Export plan YAML
              </Button>
              <Button variant="secondary" onClick={() => navigate(`/projects/${projectId}/plan`)}>
                Open plan editor
              </Button>
            </Group>
          </Group>
          <div className="grid gap-4 md:grid-cols-3">
            <Card className="border">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconGraph className="h-4 w-4 text-primary" />
                  Plan status
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <Group gap="sm">
                  <Badge variant="secondary">Nodes: {planNodeCount}</Badge>
                  <Badge variant="secondary">Edges: {planEdgeCount}</Badge>
                </Group>
                <p className="text-xs text-muted-foreground">
                  Version: {planVersion}
                </p>
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/plan`)}>
                  Edit plan
                </Button>
              </CardContent>
            </Card>
            <Card className="border">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconNetwork className="h-4 w-4 text-primary" />
                  Plan nodes
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Review node execution status, trace dependencies, and inspect generated outputs.
                </p>
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/plan-nodes`)}>
                  View nodes
                </Button>
              </CardContent>
            </Card>
            <Card className="border">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconDatabase className="h-4 w-4 text-primary" />
                  Graph outputs
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Browse generated graphs, download CSV exports, or open the graph editor.
                </p>
                <Group gap="xs">
                  <Button className="flex-1" variant="secondary" onClick={() => navigate(`/projects/${projectId}/graphs`)}>
                    Browse graphs
                  </Button>
                  <Button
                    className="flex-1"
                    variant="secondary"
                    disabled={!planDag?.nodes?.length}
                    onClick={() => {
                      if (planDag?.nodes?.length) {
                        navigate(`/projects/${projectId}/plan-nodes/${planDag.nodes[0].id}/edit`)
                      }
                    }}
                  >
                    Open editor
                  </Button>
                </Group>
              </CardContent>
            </Card>
          </div>
        </section>

        <section>
          <Group justify="between" className="mb-4">
            <div>
              <h2 className="text-2xl font-bold">Chat & Collaboration</h2>
              <p className="text-muted-foreground">
                Collaborate with agents and review chat logs.
              </p>
            </div>
            <Group gap="xs">
              <Button variant="secondary" onClick={() => navigate(`/projects/${projectId}/chat`)}>
                Open project chat
              </Button>
            </Group>
          </Group>
          <div className="grid gap-4 md:grid-cols-2">
            <Card className="border">
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-base font-semibold">
                  <IconMessageDots className="h-4 w-4 text-primary" />
                  Project chat
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Launch the shared agent conversation scoped to this project, or inspect previous messages.
                </p>
                <Group gap="sm">
                  <Button className="flex-1" variant="secondary" onClick={() => navigate(`/projects/${projectId}/chat`)}>
                    Join chat
                  </Button>
                  <Button className="flex-1" variant="secondary" onClick={() => navigate(`/projects/${projectId}/chat/logs`)}>
                    View logs
                  </Button>
                </Group>
              </CardContent>
            </Card>
          </div>
        </section>
      </Stack>

      <Dialog open={exportDialogOpen} onOpenChange={setExportDialogOpen}>
        <DialogContent className="sm:max-w-[520px]">
          <DialogHeader>
            <DialogTitle>Export Project</DialogTitle>
          </DialogHeader>
          <Tabs value={activeExportTab} onValueChange={(value) => setActiveExportTab(value as 'archive' | 'template')}>
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger value="archive">Export</TabsTrigger>
              <TabsTrigger value="template">Export as template</TabsTrigger>
            </TabsList>
            <TabsContent value="archive" className="mt-4 space-y-3">
              <p className="text-sm text-muted-foreground">
                Download a ZIP archive containing this project&apos;s DAG and full dataset contents.
              </p>
              <Button onClick={handleExportArchive} disabled={exportArchiveLoading}>
                {exportArchiveLoading && <Spinner className="mr-2 h-4 w-4" />}
                Download project (.zip)
              </Button>
            </TabsContent>
            <TabsContent value="template" className="mt-4 space-y-3">
              <p className="text-sm text-muted-foreground">
                Publish this project as a reusable template in the shared library (datasets are stripped to
                headers only).
              </p>
              <Button onClick={handleExportTemplate} disabled={exportTemplateLoading}>
                {exportTemplateLoading && <Spinner className="mr-2 h-4 w-4" />}
                Export as template
              </Button>
            </TabsContent>
          </Tabs>
        </DialogContent>
      </Dialog>
    </PageContainer>
  )
}

// Plan editor page component
const PlanEditorPage = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const { projectId } = useParams<{ projectId: string }>()
  const collaboration = useCollaboration() // Get project-level collaboration from context
  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

  const projects = projectsData?.projects || []
  const selectedProject = projects.find((p: any) => p.id === parseInt(projectId || '0'))

  const contextDescription = useMemo(() => {
    if (projectsLoading) {
      return 'Loading plan editor'
    }
    if (!selectedProject) {
      return projectId ? `Plan editor unavailable for project ${projectId}` : 'Plan editor'
    }
    return `Editing plan for project ${selectedProject.name} (#${selectedProject.id})`
  }, [projectsLoading, selectedProject, projectId])

  useRegisterChatContext(contextDescription, selectedProject?.id)

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  // Show loading state while projects are being fetched
  if (projectsLoading) {
    return (
      <PageContainer>
        <p>Loading project...</p>
      </PageContainer>
    )
  }

  // Only show "not found" if loading is complete and project doesn't exist
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

  const searchParams = new URLSearchParams(location.search)
  const focusNodeId = searchParams.get('focusNode') || undefined

  return (
    <div className="h-full flex flex-col gap-0">
      <div className="px-4 py-2 border-b flex-shrink-0">
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          sections={[{ title: 'Workbench', href: `/projects/${selectedProject.id}/plan` }]}
          currentPage="Plan"
          onNavigate={handleNavigate}
        />
      </div>
      <div className="flex-1 overflow-hidden">
        <ErrorBoundary>
          <PlanVisualEditor
            projectId={selectedProject.id}
            onNodeSelect={(nodeId) => console.log('Selected node:', nodeId)}
            onEdgeSelect={(edgeId) => console.log('Selected edge:', edgeId)}
            focusNodeId={focusNodeId}
            collaboration={collaboration}
          />
        </ErrorBoundary>
      </div>
    </div>
  )
}

import { PlanNodesPage } from './components/graphs/PlanNodesPage'
import { GraphsPage } from './components/graphs/GraphsPage'
import { GraphEditorPage } from './pages/GraphEditorPage'
import { DatabaseSettings } from './components/settings/DatabaseSettings'
import { SystemSettingsPage } from './pages/SystemSettingsPage'
import PageContainer from './components/layout/PageContainer'
import { EditProjectPage } from './pages/EditProjectPage'

// Main App component with routing
function App() {
  return (
    <ChatProvider>
      <ErrorBoundary>
        <AppLayout>
          <Routes>
            <Route path="/" element={
              <ErrorBoundary>
                <HomePage />
              </ErrorBoundary>
          } />
          <Route path="/library" element={
            <ErrorBoundary>
              <LibraryPage />
            </ErrorBoundary>
          } />
          <Route path="/projects" element={
            <ErrorBoundary>
              <ProjectsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId" element={
            <ErrorBoundary>
              <ProjectDetailPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/edit" element={
            <ErrorBoundary>
              <EditProjectPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/plan" element={
            <ErrorBoundary>
              <PlanEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/plan-nodes" element={
            <ErrorBoundary>
              <PlanNodesPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/graphs" element={
            <ErrorBoundary>
              <GraphsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/artefacts" element={
            <ErrorBoundary>
              <ProjectArtefactsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/chat" element={
            <ErrorBoundary>
              <ProjectChatPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/chat/logs" element={
            <ErrorBoundary>
              <ChatLogsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/plan-nodes/:graphId/edit" element={
            <ErrorBoundary>
              <GraphEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/datasets" element={
            <ErrorBoundary>
              <DataSetsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/source-management" element={
            <ErrorBoundary>
              <SourceManagementPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/knowledge-base" element={
            <ErrorBoundary>
              <KnowledgeBasePage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/datasets" element={
            <ErrorBoundary>
              <DatasetCreationPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/datasets/:dataSetId/edit" element={
            <ErrorBoundary>
              <DataSetEditor />
            </ErrorBoundary>
          } />
          <Route path="/settings/database" element={
            <ErrorBoundary>
              <DatabaseSettings />
            </ErrorBoundary>
          } />
          <Route path="/settings/system" element={
            <ErrorBoundary>
              <SystemSettingsPage />
            </ErrorBoundary>
          } />
          <Route path="*" element={
            <ErrorBoundary>
              <PageContainer>
                <h1 className="text-3xl font-bold">Page Not Found</h1>
                <p className="mb-4">The page you're looking for doesn't exist.</p>
                <Button onClick={() => window.location.href = '/'}>
                  Go Home
                </Button>
              </PageContainer>
            </ErrorBoundary>
          } />
        </Routes>
        </AppLayout>
      </ErrorBoundary>
    </ChatProvider>
  )
}

export default App
