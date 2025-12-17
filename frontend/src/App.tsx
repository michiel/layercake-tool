import React, { useEffect, useMemo, useState } from 'react'
import { Routes, Route, Navigate, useNavigate, useParams, useLocation, Link } from 'react-router-dom'
import { IconGraph, IconServer, IconDatabase, IconPlus, IconSettings, IconFileDatabase, IconTrash, IconDownload, IconChevronLeft, IconChevronRight, IconFolderPlus, IconBooks, IconAdjustments, IconHierarchy2, IconChevronDown, IconUpload, IconFlask } from '@tabler/icons-react'
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
import { KnowledgeBasePage } from './pages/KnowledgeBasePage'
import { CodeAnalysisPage } from './pages/CodeAnalysisPage'
import { CodeAnalysisDetailPage } from './pages/CodeAnalysisDetailPage'
import { DatasetCreationPage } from './pages/DatasetCreationPage'
import { ProjectArtefactsPage } from './pages/ProjectArtefactsPage'
import { ProjectLayersPage } from './pages/ProjectLayersPage'
import { getOrCreateSessionId } from './utils/session'
import { Group, Stack } from './components/layout-primitives'
import { Button } from './components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from './components/ui/card'
import { Badge } from './components/ui/badge'
import { Alert, AlertDescription } from './components/ui/alert'
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from './components/ui/dialog'
import { Tabs, TabsContent, TabsList, TabsTrigger } from './components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './components/ui/select'
import { Switch } from './components/ui/switch'
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from './components/ui/tooltip'
import { Separator } from './components/ui/separator'
import { Spinner } from './components/ui/spinner'
import { ChatProvider } from './components/chat/ChatProvider'
import { useRegisterChatContext } from './hooks/useRegisterChatContext'
import { cn } from './lib/utils'
import { useTagsFilter } from './hooks/useTagsFilter'
import { EXPORT_PROJECT_ARCHIVE, EXPORT_PROJECT_AS_TEMPLATE, RESET_PROJECT } from './graphql/libraryItems'
import { LIST_PLANS, GET_PLAN } from './graphql/plans'
import { LIST_STORIES, type Story } from './graphql/stories'
import { showErrorNotification, showSuccessNotification } from './utils/notifications'
import { PlansPage } from './components/plans/PlansPage'
import { ProjectionsPage } from './pages/workbench/ProjectionsPage'
import { ProjectionViewerPage } from './pages/projections/ProjectionViewerPage'
import { ProjectionEditPage } from './pages/projections/ProjectionEditPage'
import type { Plan } from './types/plan'

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
      importExportPath
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

const IMPORT_PROJECT_ARCHIVE = gql`
  mutation ImportProjectArchive($fileContent: String!, $name: String) {
    importProjectArchive(fileContent: $fileContent, name: $name) {
      id
      name
      description
      importExportPath
    }
  }
`

const IMPORT_PROJECT_FROM_DIRECTORY = gql`
  mutation ImportProjectFromDirectory($path: String!, $name: String, $keepConnection: Boolean) {
    importProjectFromDirectory(path: $path, name: $name, keepConnection: $keepConnection) {
      id
      name
      description
      importExportPath
    }
  }
`

const EXPORT_PROJECT_TO_DIRECTORY = gql`
  mutation ExportProjectToDirectory(
    $projectId: Int!
    $path: String!
    $includeKnowledgeBase: Boolean
    $keepConnection: Boolean
  ) {
    exportProjectToDirectory(
      projectId: $projectId
      path: $path
      includeKnowledgeBase: $includeKnowledgeBase
      keepConnection: $keepConnection
    )
  }
`

const REIMPORT_PROJECT = gql`
  mutation ReimportProject($projectId: Int!) {
    reimportProject(projectId: $projectId) {
      id
      name
      description
      importExportPath
    }
  }
`

const REEXPORT_PROJECT = gql`
  mutation ReexportProject($projectId: Int!, $includeKnowledgeBase: Boolean) {
    reexportProject(projectId: $projectId, includeKnowledgeBase: $includeKnowledgeBase)
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

    const graphCreationChildren: ProjectNavChild[] = [
      {
        key: 'data-sets',
        label: 'Data sets',
        route: `/projects/${projectId}/datasets`,
        isActive: makeRouteMatcher(`/projects/${projectId}/datasets`),
      },
      {
        key: 'plans',
        label: 'Plans',
        route: `/projects/${projectId}/plans`,
        isActive: makeRouteMatcher(`/projects/${projectId}/plans`, { prefix: true }),
      },
      {
        key: 'layers',
        label: 'Layers',
        route: `/projects/${projectId}/workbench/layers`,
        isActive: makeRouteMatcher(`/projects/${projectId}/workbench/layers`),
      },
      {
        key: 'stories',
        label: 'Stories',
        route: `/projects/${projectId}/stories`,
        isActive: makeRouteMatcher(`/projects/${projectId}/stories`, { prefix: true }),
      },
    ]

    const experimentalChildren: ProjectNavChild[] = [
      {
        key: 'chat',
        label: 'Chat',
        route: `/projects/${projectId}/chat`,
        isActive: makeRouteMatcher(`/projects/${projectId}/chat`),
      },
      {
        key: 'chat-logs',
        label: 'Chat logs',
        route: `/projects/${projectId}/chat/logs`,
        isActive: makeRouteMatcher(`/projects/${projectId}/chat/logs`),
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
        key: 'code-analysis',
        label: 'Code analysis',
        route: `/projects/${projectId}/data-acquisition/code-analysis`,
        isActive: makeRouteMatcher(`/projects/${projectId}/data-acquisition/code-analysis`),
      },
      {
        key: 'graphs',
        label: 'Graphs',
        route: `/projects/${projectId}/graphs`,
        isActive: makeRouteMatcher(`/projects/${projectId}/graphs`),
      },
      {
        key: 'projections',
        label: 'Projections',
        route: `/projects/${projectId}/workbench/projections`,
        isActive: makeRouteMatcher(`/projects/${projectId}/workbench/projections`, { prefix: true }),
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
        key: 'graph-creation',
        label: 'Workbench',
        icon: <IconGraph className="h-4 w-4" />,
        route: `/projects/${projectId}/workbench`,
        children: graphCreationChildren,
      }),
      createSection({
        key: 'artefacts',
        label: 'Artefacts',
        icon: <IconHierarchy2 className="h-4 w-4" />,
        route: `/projects/${projectId}/artefacts`,
      }),
      createSection({
        key: 'experimental',
        label: 'Experimental',
        icon: <IconFlask className="h-4 w-4" />,
        route: `/projects/${projectId}/chat`,
        matchOptions: { prefix: true },
        children: experimentalChildren,
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

  const shouldHandleNavigationClick = (event: React.MouseEvent) => {
    if (event.defaultPrevented) return false
    if (event.button !== 0) return false
    if (event.metaKey || event.altKey || event.ctrlKey || event.shiftKey) return false
    return true
  }

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
                      asChild
                    >
                      <Link to="/">
                        {navCollapsed ? (
                          <IconServer className="h-4 w-4" />
                        ) : (
                          <>
                            <IconServer className="h-4 w-4 mr-2" />
                            Home
                          </>
                        )}
                      </Link>
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Home</TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/library') ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      asChild
                    >
                      <Link to="/library">
                        {navCollapsed ? (
                          <IconBooks className="h-4 w-4" />
                        ) : (
                          <>
                            <IconBooks className="h-4 w-4 mr-2" />
                            Library
                          </>
                        )}
                      </Link>
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Library</TooltipContent>
                </Tooltip>

                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={projectsButtonActive ? 'default' : 'ghost'}
                      className={navCollapsed ? 'justify-center px-2' : 'w-full justify-start'}
                      asChild
                    >
                      <Link to="/projects">
                        {navCollapsed ? (
                          <IconDatabase className="h-4 w-4" />
                        ) : (
                          <>
                            <IconDatabase className="h-4 w-4 mr-2" />
                            Projects
                          </>
                        )}
                      </Link>
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
                                    asChild
                                  >
                                    <Link
                                      to={section.route}
                                      onClick={(event) => {
                                        if (!shouldHandleNavigationClick(event)) {
                                          return
                                        }
                                        if (hasChildren) {
                                          setCollapsedSections(prev => {
                                            const next = new Set(prev)
                                            next.delete(section.key)
                                            return next
                                          })
                                        }
                                      }}
                                    >
                                      {section.icon}
                                      {!navCollapsed && <span className="ml-2">{section.label}</span>}
                                    </Link>
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
                                      asChild
                                    >
                                      <Link to={child.route}>{child.label}</Link>
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
                      asChild
                    >
                      <Link to="/settings/database">
                        {navCollapsed ? (
                          <IconSettings className="h-4 w-4" />
                        ) : (
                          <>
                            <IconSettings className="h-4 w-4 mr-2" />
                            Database Settings
                          </>
                        )}
                      </Link>
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">Database Settings</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={isActiveRoute('/settings/system') ? 'default' : 'ghost'}
                      className={`${navCollapsed ? 'justify-center px-2' : 'w-full justify-start'} mt-2`}
                      asChild
                    >
                      <Link to="/settings/system">
                        {navCollapsed ? (
                          <IconAdjustments className="h-4 w-4" />
                        ) : (
                          <>
                            <IconAdjustments className="h-4 w-4 mr-2" />
                            System Settings
                          </>
                        )}
                      </Link>
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
  const fileInputRef = React.useRef<HTMLInputElement>(null)

  const { data: projectsData } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      importExportPath?: string | null
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

  const [importProjectArchive, { loading: importLoading }] = useMutation(IMPORT_PROJECT_ARCHIVE, {
    onCompleted: (result) => {
      const project = (result as any)?.importProjectArchive
      if (project) {
        navigate(`/projects/${project.id}`)
      }
    },
    onError: (error) => {
      alert(`Failed to import project: ${error.message}`)
    },
    refetchQueries: [{ query: GET_PROJECTS }]
  })

  const [importProjectFromDirectory, { loading: importDirectoryLoading }] = useMutation(
    IMPORT_PROJECT_FROM_DIRECTORY,
    {
      onCompleted: (result) => {
        const project = (result as any)?.importProjectFromDirectory
        if (project) {
          navigate(`/projects/${project.id}`)
        }
      },
      onError: (error) => {
        alert(`Failed to import project from directory: ${error.message}`)
      },
      refetchQueries: [{ query: GET_PROJECTS }]
    }
  )

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

  const handleImportClick = () => {
    fileInputRef.current?.click()
  }

  const handleImportFromDirectory = async () => {
    const path = window.prompt('Enter the project directory to import')
    if (!path) return
    const keepConnection = window.confirm('Keep connection to this directory for re-import/re-export?')
    const nameOverride = window.prompt('Optional project name (leave blank to use export metadata)') || undefined

    try {
      await importProjectFromDirectory({
        variables: {
          path,
          name: nameOverride || null,
          keepConnection,
        },
      })
    } catch (error) {
      console.error('Failed to import project from directory', error)
    }
  }

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return

    // Reset the input so the same file can be selected again
    event.target.value = ''

    try {
      const arrayBuffer = await file.arrayBuffer()
      const bytes = new Uint8Array(arrayBuffer)
      let binary = ''
      for (let i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i])
      }
      const base64 = btoa(binary)

      await importProjectArchive({
        variables: {
          fileContent: base64,
        },
      })
    } catch (error) {
      console.error('Failed to import project', error)
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
      <div className="py-12 px-4 md:px-8 bg-muted/50 border-b">
        <div className="flex flex-col gap-4 md:flex-row md:flex-wrap md:justify-center">
          <Button
            size="lg"
            onClick={() => navigate('/projects')}
            className="w-full md:w-auto md:min-w-[220px] h-16 md:h-20 text-base md:text-lg"
          >
            <IconDatabase className="mr-2 h-5 w-5 md:h-6 md:w-6" />
            Browse Projects
          </Button>
          <Button
            size="lg"
            onClick={handleCreateProject}
            className="w-full md:w-auto md:min-w-[220px] h-16 md:h-20 text-base md:text-lg"
          >
            <IconPlus className="mr-2 h-5 w-5 md:h-6 md:w-6" />
            Start New Project
          </Button>
          <Button
            size="lg"
            onClick={handleOpenSampleModal}
            variant="secondary"
            className="w-full md:w-auto md:min-w-[220px] h-16 md:h-20 text-base md:text-lg"
          >
            <IconFolderPlus className="mr-2 h-5 w-5 md:h-6 md:w-6" />
            Import Sample Project
          </Button>
          <Button
            size="lg"
            onClick={handleImportClick}
            variant="secondary"
            className="w-full md:w-auto md:min-w-[220px] h-16 md:h-20 text-base md:text-lg"
            disabled={importLoading}
          >
            <IconUpload className="mr-2 h-5 w-5 md:h-6 md:w-6" />
            {importLoading ? 'Importing...' : 'Import Project'}
          </Button>
          <Button
            size="lg"
            onClick={handleImportFromDirectory}
            variant="secondary"
            className="w-full md:w-auto md:min-w-[220px] h-16 md:h-20 text-base md:text-lg"
            disabled={importDirectoryLoading}
          >
            <IconUpload className="mr-2 h-5 w-5 md:h-6 md:w-6" />
            {importDirectoryLoading ? 'Importing...' : 'Import (Filesystem)'}
          </Button>
          <input
            ref={fileInputRef}
            type="file"
            accept=".zip"
            onChange={handleFileChange}
            style={{ display: 'none' }}
          />
        </div>
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
                        navigate(`/projects/${project.id}/plans`)
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
      importExportPath?: string | null
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
                      navigate(`/projects/${project.id}/plans`)
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
      importExportPath?: string | null
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

  const { data: planDagData } = useQuery(GET_PLAN_DAG, {
    variables: { projectId: projectIdNum },
    skip: !projectId,
  })
  const { data: plansData, loading: plansLoading } = useQuery<{ plans: Plan[] }>(LIST_PLANS, {
    variables: { projectId: projectIdNum },
    skip: !projectId,
    fetchPolicy: 'cache-and-network',
  })
  const { data: storiesData, loading: storiesLoading } = useQuery<{ stories: Story[] }>(LIST_STORIES, {
    variables: { projectId: projectIdNum },
    skip: !projectId,
    fetchPolicy: 'cache-and-network',
  })

  const projects = projectsData?.projects || []
  const selectedProject = projects.find((p: any) => p.id === projectIdNum)
  const planDag = (planDagData as any)?.planDag
  const plans = plansData?.plans ?? []
  const stories = storiesData?.stories ?? []

  const [exportDialogOpen, setExportDialogOpen] = useState(false)
  const [includeKnowledgeBase, setIncludeKnowledgeBase] = useState(false)
  const [activeExportTab, setActiveExportTab] = useState<'archive' | 'template'>('archive')

  useEffect(() => {
    if (!exportDialogOpen) {
      setIncludeKnowledgeBase(false)
    }
  }, [exportDialogOpen])
  const [exportProjectArchiveMutation, { loading: exportArchiveLoading }] = useMutation(EXPORT_PROJECT_ARCHIVE)
  const [exportProjectAsTemplateMutation, { loading: exportTemplateLoading }] = useMutation(
    EXPORT_PROJECT_AS_TEMPLATE
  )
  const [exportProjectToDirectoryMutation, { loading: exportDirectoryLoading }] = useMutation(
    EXPORT_PROJECT_TO_DIRECTORY
  )
  const [reimportProjectMutation, { loading: reimportProjectLoading }] = useMutation(REIMPORT_PROJECT, {
    refetchQueries: [{ query: GET_PROJECTS }],
  })
  const [reexportProjectMutation, { loading: reexportProjectLoading }] = useMutation(REEXPORT_PROJECT)
  const [resetProjectMutation, { loading: resetProjectLoading }] = useMutation(RESET_PROJECT)

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

  const handleExportToDirectory = async () => {
    if (!selectedProject) return
    const path = window.prompt('Enter target directory for export')
    if (!path) return
    const keepConnection = window.confirm('Keep connection to this directory for future re-imports?')

    try {
      await exportProjectToDirectoryMutation({
        variables: {
          projectId: selectedProject.id,
          path,
          includeKnowledgeBase,
          keepConnection,
        },
      })
      showSuccessNotification('Exported project to filesystem', `Saved to ${path}`)
    } catch (error: any) {
      showErrorNotification('Failed to export project', error.message || 'Unknown error')
    }
  }

  const handleReimportFromConnection = async () => {
    if (!selectedProject?.importExportPath) return
    try {
      const { data } = await reimportProjectMutation({
        variables: { projectId: selectedProject.id },
      })
      const projectName = (data as any)?.reimportProject?.name ?? 'project'
      const newId = (data as any)?.reimportProject?.id
      if (newId && newId !== selectedProject.id) {
        navigate(`/projects/${newId}`)
      }
      showSuccessNotification('Re-import complete', `Reloaded ${projectName} from ${selectedProject.importExportPath}`)
    } catch (error: any) {
      showErrorNotification('Failed to re-import project', error.message || 'Unknown error')
    }
  }

  const handleReexportToConnection = async () => {
    if (!selectedProject?.importExportPath) return
    try {
      await reexportProjectMutation({
        variables: { projectId: selectedProject.id, includeKnowledgeBase },
      })
      showSuccessNotification('Re-export complete', `Updated ${selectedProject.importExportPath}`)
    } catch (error: any) {
      showErrorNotification('Failed to re-export project', error.message || 'Unknown error')
    }
  }

  const handleResetProject = async () => {
    if (!Number.isFinite(projectIdNum)) {
      return
    }
    if (!window.confirm('Are you sure you want to reset this project? This will re-initialise the project with fresh IDs while preserving all data.')) {
      return
    }
    try {
      const { data } = await resetProjectMutation({
        variables: { projectId: projectIdNum, includeKnowledgeBase: true },
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
      showErrorNotification(
        'Project reset failed',
        error?.message || 'Unable to reset the project.'
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
        variables: { projectId: projectIdNum, includeKnowledgeBase },
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
          <Button variant="outline" onClick={handleExportToDirectory} disabled={exportDirectoryLoading}>
            <IconDownload className="mr-2 h-4 w-4" />
            {exportDirectoryLoading ? 'Exporting...' : 'Export to filesystem'}
          </Button>
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

      {selectedProject.importExportPath && (
        <Alert className="mb-4">
          <AlertDescription className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <span className="truncate">
              Connected to <span className="font-mono">{selectedProject.importExportPath}</span>
            </span>
            <Group gap="xs" className="flex-shrink-0">
              <Button
                size="sm"
                variant="secondary"
                onClick={handleReimportFromConnection}
                disabled={reimportProjectLoading}
              >
                {reimportProjectLoading && <Spinner className="mr-2 h-4 w-4" />}
                Re-import from source
              </Button>
              <Button
                size="sm"
                variant="secondary"
                onClick={handleReexportToConnection}
                disabled={reexportProjectLoading}
              >
                {reexportProjectLoading && <Spinner className="mr-2 h-4 w-4" />}
                Re-export to source
              </Button>
            </Group>
          </AlertDescription>
        </Alert>
      )}

      <Stack gap="xl">
        <section>
          <Group justify="between" className="mb-4">
            <div>
              <h2 className="text-2xl font-bold">Project overview</h2>
              <p className="text-muted-foreground">
                Quick access to data, plan, and artefacts for this project.
              </p>
            </div>
            <Group gap="xs">
              <Button
                variant="secondary"
                onClick={handleResetProject}
                disabled={resetProjectLoading}
              >
                {resetProjectLoading && <Spinner className="mr-2 h-4 w-4" />}
                <IconAdjustments className="mr-2 h-4 w-4" />
                Reset project
              </Button>
              <Button variant="secondary" onClick={() => handleDownloadYAML()} disabled={!planDag}>
                <IconDownload className="mr-2 h-4 w-4" />
                Export plan YAML
              </Button>
            </Group>
          </Group>

          <div className="grid gap-4 lg:grid-cols-3">
            <Card className="border hover:shadow-md transition-shadow">
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
                          onClick={() => navigate(`/projects/${projectId}/plans/${plan.id}`)}
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
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/plans`)}>
                  Open plans
                </Button>
              </CardContent>
            </Card>

            <Card className="border hover:shadow-md transition-shadow">
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
                        onClick={() => navigate(`/projects/${projectId}/stories/${story.id}`)}
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
                <Button variant="secondary" className="w-full" onClick={() => navigate(`/projects/${projectId}/stories`)}>
                  Open stories
                </Button>
              </CardContent>
            </Card>

            <Card className="border hover:shadow-md transition-shadow">
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
                    onClick={() => navigate(`/projects/${projectId}/workbench/layers`)}
                  >
                    <IconHierarchy2 className="h-4 w-4" />
                    Layer palette
                  </Button>
                  <Button
                    variant="secondary"
                    className="w-full justify-start gap-2"
                    onClick={() => navigate(`/projects/${projectId}/graphs`)}
                  >
                    <IconGraph className="h-4 w-4" />
                    Generated graphs
                  </Button>
                  <Button
                    variant="secondary"
                    className="w-full justify-start gap-2"
                    onClick={() => navigate(`/projects/${projectId}/artefacts`)}
                  >
                    <IconDatabase className="h-4 w-4" />
                    Artefacts
                  </Button>
                </div>
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
              <div className="rounded-lg border p-3">
                <div className="flex items-start justify-between gap-3">
                  <div>
                    <p className="text-sm font-medium">Include knowledge base</p>
                    <p className="text-xs text-muted-foreground">
                      Attach uploaded source files, embeddings, and palette metadata stored in the knowledge base.
                    </p>
                  </div>
                  <Switch checked={includeKnowledgeBase} onCheckedChange={setIncludeKnowledgeBase} />
                </div>
              </div>
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
  const { projectId, planId } = useParams<{ projectId: string; planId: string }>()
  const projectIdNum = Number(projectId || 0)
  const planIdNum = Number(planId || 0)
  const collaboration = useCollaboration()

  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      importExportPath?: string | null
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

  const {
    data: planData,
    loading: planLoading,
    error: planError,
  } = useQuery<{ plan: Plan | null }>(GET_PLAN, {
    variables: { id: planIdNum },
    skip: !planIdNum,
    fetchPolicy: 'cache-and-network',
  })

  const projects = projectsData?.projects || []
  const selectedProject = projects.find((p: any) => p.id === projectIdNum)
  const plan = planData?.plan
  const planBelongsToProject = plan && plan.projectId === selectedProject?.id

  const contextDescription = useMemo(() => {
    if (projectsLoading || planLoading) {
      return 'Loading plan editor'
    }
    if (!selectedProject) {
      return projectId ? `Plan editor unavailable for project ${projectId}` : 'Plan editor'
    }
    if (!plan || !planBelongsToProject) {
      return `Plan editor unavailable`
    }
    return `Editing ${plan.name} for ${selectedProject.name} (#${selectedProject.id})`
  }, [projectsLoading, planLoading, selectedProject, plan, planBelongsToProject, projectId])

  useRegisterChatContext(contextDescription, selectedProject?.id)

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  if (projectsLoading || planLoading) {
    return (
      <PageContainer>
        <Group gap="sm" align="center">
          <Spinner className="h-4 w-4" />
          <span>Loading plan editor…</span>
        </Group>
      </PageContainer>
    )
  }

  if (!selectedProject) {
    return (
      <PageContainer>
        <h1 className="text-3xl font-bold">Project not found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to projects
        </Button>
      </PageContainer>
    )
  }

  if (!planIdNum) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Select a plan</h1>
        <p className="text-muted-foreground mt-2">
          Choose a plan from the plans list to start editing.
        </p>
        <Button className="mt-4" onClick={() => navigate(`/projects/${selectedProject.id}/plans`)}>
          View plans
        </Button>
      </PageContainer>
    )
  }

  if (planError || !plan || !planBelongsToProject) {
    return (
      <PageContainer>
        <h1 className="text-2xl font-bold">Plan not found</h1>
        <p className="text-muted-foreground mt-2">
          The requested plan is unavailable or does not belong to this project.
        </p>
        <Button className="mt-4" onClick={() => navigate(`/projects/${selectedProject.id}/plans`)}>
          Back to plans
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
          sections={[
            { title: 'Plans', href: `/projects/${selectedProject.id}/plans` },
            { title: plan.name, href: `/projects/${selectedProject.id}/plans/${plan.id}` },
          ]}
          currentPage="Plan editor"
          onNavigate={handleNavigate}
        />
      </div>
      <div className="flex-1 overflow-hidden">
        <ErrorBoundary>
          <PlanVisualEditor
            projectId={selectedProject.id}
            planId={planIdNum}
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

const LegacyPlanRedirect = () => {
  const { projectId } = useParams<{ projectId: string }>()
  const navigate = useNavigate()
  const projectIdNum = Number(projectId || 0)

  const { data, loading } = useQuery<{ plans: Array<{ id: number }> }>(LIST_PLANS, {
    variables: { projectId: projectIdNum },
    skip: !projectIdNum,
    fetchPolicy: 'network-only',
  })

  useEffect(() => {
    if (loading || !projectIdNum) {
      return
    }
    const plans = data?.plans ?? []
    if (plans.length > 0) {
      navigate(`/projects/${projectIdNum}/plans/${plans[0].id}`, { replace: true })
    } else {
      navigate(`/projects/${projectIdNum}/plans`, { replace: true })
    }
  }, [loading, data, navigate, projectIdNum])

  return (
    <PageContainer>
      <Group gap="sm" align="center">
        <Spinner className="h-4 w-4" />
        <span>Redirecting to plans…</span>
      </Group>
    </PageContainer>
  )
}

import { GraphsPage } from './components/graphs/GraphsPage'
import { GraphEditorPage } from './pages/GraphEditorPage'
import { DatabaseSettings } from './components/settings/DatabaseSettings'
import { SystemSettingsPage } from './pages/SystemSettingsPage'
import PageContainer from './components/layout/PageContainer'
import { EditProjectPage } from './pages/EditProjectPage'
import { WorkbenchPage } from './pages/WorkbenchPage'
import { StoriesPage } from './pages/StoriesPage'
import { StoryPage } from './pages/StoryPage'

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
          <Route path="/projects/:projectId/plans" element={
            <ErrorBoundary>
              <PlansPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/plans/:planId" element={
            <ErrorBoundary>
              <PlanEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/plan" element={
            <ErrorBoundary>
              <LegacyPlanRedirect />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/workbench" element={
            <ErrorBoundary>
              <WorkbenchPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/workbench/layers" element={
            <ErrorBoundary>
              <ProjectLayersPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/stories" element={
            <ErrorBoundary>
              <StoriesPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/stories/:storyId" element={
            <ErrorBoundary>
              <StoryPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/stories/:storyId/sequences/:sequenceId" element={
            <ErrorBoundary>
              <Navigate replace to="../?tab=sequences" />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/graphs" element={
            <ErrorBoundary>
              <GraphsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/workbench/projections" element={
            <ErrorBoundary>
              <ProjectionsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/workbench/projections/:projectionId/edit" element={
            <ErrorBoundary>
              <ProjectionEditPage />
            </ErrorBoundary>
          } />
          <Route path="/projections/:projectionId" element={
            <ErrorBoundary>
              <ProjectionViewerPage />
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
          <Route path="/projects/:projectId/graph/:graphId/edit" element={
            <ErrorBoundary>
              <GraphEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/datasets" element={
            <ErrorBoundary>
              <DataSetsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/knowledge-base" element={
            <ErrorBoundary>
              <KnowledgeBasePage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/code-analysis" element={
            <ErrorBoundary>
              <CodeAnalysisPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/data-acquisition/code-analysis/:profileId" element={
            <ErrorBoundary>
              <CodeAnalysisDetailPage />
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
