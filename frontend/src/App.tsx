import React, { useState } from 'react'
import { AppShell, Group, Title, Stack, Button, Container, Text, Card, Badge, Alert } from '@mantine/core'
import { IconGraph, IconServer, IconDatabase, IconPlus, IconSettings, IconPlayerPlay, IconAlertCircle } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { ConnectionStatus } from './components/common/ConnectionStatus'
import { Breadcrumbs } from './components/common/Breadcrumbs'
import { PlanVisualEditor } from './components/editors/PlanVisualEditor/PlanVisualEditor'

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

type ViewType = 'home' | 'projects' | 'project-detail' | 'plan-editor' | 'graph-editor'

interface ProjectContext {
  id: number
  name: string
}

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('home')
  const [selectedProject, setSelectedProject] = useState<ProjectContext | null>(null)

  // Health check query to verify backend connectivity
  const { loading: healthLoading, error: healthError } = useQuery(HEALTH_CHECK, {
    errorPolicy: 'all',
    notifyOnNetworkStatusChange: true,
  })

  const { data: projectsData, loading: projectsLoading, error: projectsError } = useQuery(GET_PROJECTS, {
    errorPolicy: 'all',
  })

  const projects = projectsData?.projects || []

  // Navigation handler
  const handleNavigate = (route: string) => {
    if (route === 'home') {
      setCurrentView('home')
      setSelectedProject(null)
    } else if (route === 'projects') {
      setCurrentView('projects')
      setSelectedProject(null)
    } else if (route.startsWith('project-')) {
      const projectId = parseInt(route.replace('project-', ''))
      const project = projects.find((p: any) => p.id === projectId)
      if (project) {
        setSelectedProject({ id: project.id, name: project.name })
        setCurrentView('project-detail')
      }
    }
  }

  // Project selection handler
  const handleProjectSelect = (projectId: number, projectName: string) => {
    setSelectedProject({ id: projectId, name: projectName })
    setCurrentView('project-detail')
  }

  // Navigation state
  const getActiveNavItem = () => {
    if (currentView === 'home') return 'home'
    if (currentView === 'projects') return 'projects'
    return null // No nav item active when in project context
  }

  const activeNavItem = getActiveNavItem()

  const renderCurrentView = () => {
    switch (currentView) {
      case 'home':
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
                onClick={() => setCurrentView('projects')}
              >
                Browse Projects
              </Button>
              <Button
                size="lg"
                variant="outline"
                leftSection={<IconGraph size={20} />}
                onClick={() => setCurrentView('projects')}
              >
                Create New Project
              </Button>
            </Group>

            <Text size="sm" c="dimmed">
              Phase 2.3: Frontend-Backend Integration Complete - Real-time Plan DAG editor ready
            </Text>
          </Container>
        )

      case 'projects':
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
                    onClick={() => handleProjectSelect(project.id, project.name)}
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
                            setSelectedProject({ id: project.id, name: project.name })
                            setCurrentView('plan-editor')
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
                            handleProjectSelect(project.id, project.name)
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

      case 'project-detail':
        if (!selectedProject) return null

        const projectActions = [
          {
            title: 'Plan Editor',
            description: 'Create and edit Plan DAGs with visual node-based interface',
            icon: <IconGraph size={20} />,
            onClick: () => setCurrentView('plan-editor'),
            primary: true,
          },
          {
            title: 'Graph Editor',
            description: 'Visualize and edit graph data structures',
            icon: <IconDatabase size={20} />,
            onClick: () => setCurrentView('graph-editor'),
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

      case 'plan-editor':
        if (!selectedProject) return null

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
              <PlanVisualEditor
                projectId={selectedProject.id}
                onNodeSelect={(nodeId) => console.log('Selected node:', nodeId)}
                onEdgeSelect={(edgeId) => console.log('Selected edge:', edgeId)}
              />
            </div>
          </Stack>
        )

      case 'graph-editor':
        if (!selectedProject) return null

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

      default:
        return (
          <Container size="xl">
            <Title order={1}>Page Not Found</Title>
          </Container>
        )
    }
  }

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{ width: 250, breakpoint: 'sm' }}
      padding="md"
    >
      <AppShell.Header>
        <Group h="100%" px="md" justify="space-between">
          <Group style={{ cursor: 'pointer' }} onClick={() => handleNavigate('home')}>
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
            variant={activeNavItem === 'home' ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconServer size={16} />}
            onClick={() => handleNavigate('home')}
          >
            Home
          </Button>
          <Button
            variant={activeNavItem === 'projects' ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconDatabase size={16} />}
            onClick={() => handleNavigate('projects')}
          >
            Projects
          </Button>

          {/* Project-specific navigation - only show when in a project */}
          {selectedProject && (
            <>
              <div style={{ height: '1px', backgroundColor: '#e9ecef', margin: '8px 0' }} />
              <Button
                variant={currentView === 'plan-editor' ? 'filled' : 'light'}
                fullWidth
                leftSection={<IconGraph size={16} />}
                onClick={() => setCurrentView('plan-editor')}
              >
                Plan Editor
              </Button>
              <Button
                variant={currentView === 'graph-editor' ? 'filled' : 'light'}
                fullWidth
                leftSection={<IconGraph size={16} />}
                onClick={() => setCurrentView('graph-editor')}
                disabled
              >
                Graph Editor
              </Button>
            </>
          )}
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        {renderCurrentView()}
      </AppShell.Main>
    </AppShell>
  )
}

export default App