import React from 'react'
import { Routes, Route, useNavigate, useParams, useLocation } from 'react-router-dom'
import { AppShell, Group, Title, Stack, Button, Container, Text, Card, Badge, Alert } from '@mantine/core'
import { IconGraph, IconServer, IconDatabase, IconPlus, IconSettings, IconPlayerPlay, IconAlertCircle } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { ConnectionStatus } from './components/common/ConnectionStatus'
import { Breadcrumbs } from './components/common/Breadcrumbs'
import { PlanVisualEditor } from './components/editors/PlanVisualEditor/PlanVisualEditor'
import { ErrorBoundary } from './components/common/ErrorBoundary'

// Health check query to verify backend connectivity
const HEALTH_CHECK = gql`
  query HealthCheck {
    projects {
      id
      name
    }
  }
`

// Query to fetch projects
const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`

// Layout wrapper component
const AppLayout = ({ children }: { children: React.ReactNode }) => {
  const navigate = useNavigate()
  const location = useLocation()

  // Get current route info for navigation highlighting
  const isActiveRoute = (path: string) => {
    if (path === '/') return location.pathname === '/'
    return location.pathname.startsWith(path)
  }

  // Extract project info from current path for navbar
  const projectId = location.pathname.match(/\/projects\/(\d+)/)?.[1]

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{ width: 250, breakpoint: 'sm' }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md" justify="space-between">
          <Group style={{ cursor: 'pointer' }} onClick={() => navigate('/')}>
            <IconGraph size={28} />
            <Title order={2}>Layercake</Title>
          </Group>
          <ConnectionStatus />
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        <Title order={4} mb="md">Navigation</Title>
        <Stack gap="xs">
          <Button
            variant={isActiveRoute('/') ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconServer size={16} />}
            onClick={() => navigate('/')}
          >
            Home
          </Button>
          <Button
            variant={isActiveRoute('/projects') ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconDatabase size={16} />}
            onClick={() => navigate('/projects')}
          >
            Projects
          </Button>

          {/* Project-specific navigation - only show when in a project */}
          {projectId && (
            <>
              <div style={{ height: '1px', backgroundColor: '#e9ecef', margin: '8px 0' }} />
              <Button
                variant={isActiveRoute(`/projects/${projectId}/plan`) ? 'filled' : 'light'}
                fullWidth
                leftSection={<IconGraph size={16} />}
                onClick={() => navigate(`/projects/${projectId}/plan`)}
              >
                Plan Editor
              </Button>
              <Button
                variant={isActiveRoute(`/projects/${projectId}/graph`) ? 'filled' : 'light'}
                fullWidth
                leftSection={<IconGraph size={16} />}
                onClick={() => navigate(`/projects/${projectId}/graph`)}
                disabled
              >
                Graph Editor
              </Button>
            </>
          )}
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        {children}
      </AppShell.Main>
    </AppShell>
  )
}

// Home page component
const HomePage = () => {
  const navigate = useNavigate()
  const { loading: healthLoading, error: healthError } = useQuery(HEALTH_CHECK, {
    errorPolicy: 'all',
    notifyOnNetworkStatusChange: true,
  })

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  return (
    <Container size="xl">
      <Breadcrumbs currentPage="Home" onNavigate={handleNavigate} />

      <Title order={1} mb="xl">Welcome to Layercake</Title>

      <Text size="lg" mb="md">
        Interactive graph transformation and visualization tool with real-time collaboration.
      </Text>

      <Group mb="xl">
        <div>
          <Text fw={500}>Status:</Text>
          <Text size="sm" c={healthError ? 'red' : healthLoading ? 'yellow' : 'green'}>
            {healthError ? 'Backend Disconnected' : healthLoading ? 'Connecting...' : 'Connected'}
          </Text>
        </div>

        <div>
          <Text fw={500}>Mode:</Text>
          <Text size="sm">
            {window.location.protocol === 'file:' ? 'Desktop (Tauri)' : 'Web Browser'}
          </Text>
        </div>
      </Group>

      <Title order={2} mb="md">Getting Started</Title>

      <Group mb="md">
        <Button
          size="lg"
          leftSection={<IconDatabase size={20} />}
          onClick={() => navigate('/projects')}
        >
          Browse Projects
        </Button>
        <Button
          size="lg"
          variant="outline"
          leftSection={<IconGraph size={20} />}
          onClick={() => navigate('/projects')}
        >
          Create New Project
        </Button>
      </Group>

      <Text size="sm" c="dimmed">
        Phase 2.3: Frontend-Backend Integration Complete - Real-time Plan DAG editor ready
      </Text>
    </Container>
  )
}

// Projects list page component
const ProjectsPage = () => {
  const navigate = useNavigate()
  const { data: projectsData, loading: projectsLoading, error: projectsError } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS, {
    errorPolicy: 'all',
  })

  const projects = projectsData?.projects || []

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  const handleProjectSelect = (projectId: number) => {
    navigate(`/projects/${projectId}`)
  }

  return (
    <Container size="xl">
      <Breadcrumbs currentPage="Projects" onNavigate={handleNavigate} />

      <Group justify="space-between" mb="md">
        <Title order={1}>Projects</Title>
        <Button leftSection={<IconPlus size={16} />}>
          New Project
        </Button>
      </Group>

      {projectsLoading && <Text>Loading projects...</Text>}

      {projectsError && (
        <Text c="red" mb="md">
          Error loading projects: {projectsError.message}
        </Text>
      )}

      {projects.length === 0 && !projectsLoading && !projectsError && (
        <Card withBorder p="xl" radius="md">
          <Stack align="center" gap="md">
            <IconGraph size={48} color="gray" />
            <Title order={3}>No Projects Yet</Title>
            <Text ta="center" c="dimmed">
              Create your first project to start building Plan DAGs and transforming graphs.
            </Text>
            <Button leftSection={<IconPlus size={16} />}>
              Create First Project
            </Button>
          </Stack>
        </Card>
      )}

      {projects.length > 0 && (
        <Stack gap="md">
          {projects.map((project: any) => (
            <Card
              key={project.id}
              withBorder
              p="md"
              radius="md"
              style={{ cursor: 'pointer' }}
              onClick={() => handleProjectSelect(project.id)}
            >
              <Group justify="space-between" align="flex-start">
                <div style={{ flex: 1 }}>
                  <Group gap="sm" mb="xs">
                    <Title order={4}>{project.name}</Title>
                    <Badge variant="light" size="sm">
                      ID: {project.id}
                    </Badge>
                  </Group>
                  {project.description && (
                    <Text size="sm" c="dimmed" mb="sm">
                      {project.description}
                    </Text>
                  )}
                  <Text size="xs" c="dimmed">
                    Created: {new Date(project.createdAt).toLocaleDateString()}
                  </Text>
                </div>
                <Group gap="xs">
                  <Button
                    variant="light"
                    size="sm"
                    leftSection={<IconGraph size={14} />}
                    onClick={(e) => {
                      e.stopPropagation()
                      navigate(`/projects/${project.id}/plan`)
                    }}
                  >
                    Plan Editor
                  </Button>
                  <Button
                    variant="light"
                    size="sm"
                    leftSection={<IconSettings size={14} />}
                    onClick={(e) => {
                      e.stopPropagation()
                      handleProjectSelect(project.id)
                    }}
                  >
                    Settings
                  </Button>
                </Group>
              </Group>
            </Card>
          ))}
        </Stack>
      )}
    </Container>
  )
}

// Project detail page component
const ProjectDetailPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const { data: projectsData } = useQuery<{
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

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    )
  }

  const projectActions = [
    {
      title: 'Plan Editor',
      description: 'Create and edit Plan DAGs with visual node-based interface',
      icon: <IconGraph size={20} />,
      onClick: () => navigate(`/projects/${projectId}/plan`),
      primary: true,
    },
    {
      title: 'Graph Editor',
      description: 'Visualize and edit graph data structures',
      icon: <IconDatabase size={20} />,
      onClick: () => navigate(`/projects/${projectId}/graph`),
      disabled: true, // Coming in Phase 3
    },
    {
      title: 'Execute Plans',
      description: 'Run Plan DAG transformations and generate outputs',
      icon: <IconPlayerPlay size={20} />,
      onClick: () => {},
      disabled: true, // Coming in Phase 3
    },
    {
      title: 'Project Settings',
      description: 'Configure project settings and permissions',
      icon: <IconSettings size={20} />,
      onClick: () => {},
      disabled: true,
    },
  ]

  return (
    <Container size="xl">
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        currentPage="Overview"
        onNavigate={handleNavigate}
      />

      <Group justify="space-between" mb="xl">
        <div>
          <Title order={1}>{selectedProject.name}</Title>
          <Group gap="sm" mt="xs">
            <Badge variant="light">ID: {selectedProject.id}</Badge>
            <Badge color="green" variant="light">Active</Badge>
          </Group>
        </div>
      </Group>

      <Title order={2} mb="md">Project Tools</Title>

      <Stack gap="md">
        {projectActions.map((action) => (
          <Card
            key={action.title}
            withBorder
            p="md"
            radius="md"
            style={{
              cursor: action.disabled ? 'not-allowed' : 'pointer',
              opacity: action.disabled ? 0.6 : 1,
            }}
            onClick={action.disabled ? undefined : action.onClick}
          >
            <Group justify="space-between" align="flex-start">
              <Group align="flex-start" gap="md">
                {action.icon}
                <div>
                  <Title order={4} mb="xs">
                    {action.title}
                    {action.disabled && (
                      <Badge size="xs" color="gray" ml="sm">
                        Coming Soon
                      </Badge>
                    )}
                  </Title>
                  <Text size="sm" c="dimmed">
                    {action.description}
                  </Text>
                </div>
              </Group>
              {!action.disabled && (
                <Button
                  variant={action.primary ? 'filled' : 'light'}
                  size="sm"
                  leftSection={action.icon}
                  onClick={(e) => {
                    e.stopPropagation()
                    action.onClick()
                  }}
                >
                  Open
                </Button>
              )}
            </Group>
          </Card>
        ))}
      </Stack>
    </Container>
  )
}

// Plan editor page component
const PlanEditorPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const { data: projectsData } = useQuery<{
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

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    )
  }

  return (
    <Stack gap={0} style={{ height: '100vh' }}>
      <div style={{ padding: '16px', borderBottom: '1px solid #e9ecef' }}>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          currentPage="Plan Editor"
          onNavigate={handleNavigate}
        />
      </div>
      <div style={{ flex: 1, overflow: 'hidden' }}>
        <ErrorBoundary>
          <PlanVisualEditor
            projectId={selectedProject.id}
            onNodeSelect={(nodeId) => console.log('Selected node:', nodeId)}
            onEdgeSelect={(edgeId) => console.log('Selected edge:', edgeId)}
          />
        </ErrorBoundary>
      </div>
    </Stack>
  )
}

// Graph editor page component (placeholder)
const GraphEditorPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const { data: projectsData } = useQuery<{
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

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    )
  }

  return (
    <Container size="xl">
      <Breadcrumbs
        projectName={selectedProject.name}
        projectId={selectedProject.id}
        currentPage="Graph Editor"
        onNavigate={handleNavigate}
      />

      <Title order={1} mb="md">Graph Editor</Title>

      <Alert
        icon={<IconAlertCircle size={16} />}
        title="Coming Soon"
        color="blue"
        mb="md"
      >
        The Graph Editor will be available in Phase 3 of the development roadmap.
        This will include graph visualization, interactive editing, and advanced graph operations.
      </Alert>

      <Text c="dimmed">
        Future features will include:
      </Text>
      <ul>
        <li>Interactive graph visualization</li>
        <li>Node and edge editing capabilities</li>
        <li>Graph layout algorithms</li>
        <li>Import/export functionality</li>
        <li>Collaborative editing</li>
      </ul>
    </Container>
  )
}

// Main App component with routing
function App() {
  return (
    <ErrorBoundary>
      <AppLayout>
        <Routes>
          <Route path="/" element={
            <ErrorBoundary>
              <HomePage />
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
          <Route path="/projects/:projectId/plan" element={
            <ErrorBoundary>
              <PlanEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/graph" element={
            <ErrorBoundary>
              <GraphEditorPage />
            </ErrorBoundary>
          } />
          <Route path="*" element={
            <ErrorBoundary>
              <Container size="xl">
                <Title order={1}>Page Not Found</Title>
                <Text mb="md">The page you're looking for doesn't exist.</Text>
                <Button onClick={() => window.location.href = '/'}>
                  Go Home
                </Button>
              </Container>
            </ErrorBoundary>
          } />
        </Routes>
      </AppLayout>
    </ErrorBoundary>
  )
}

export default App