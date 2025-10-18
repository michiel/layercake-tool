import React, { useState } from 'react'
import { Routes, Route, useNavigate, useParams, useLocation } from 'react-router-dom'
import { AppShell, Group, Title, Stack, Button, Container, Text, Card, Badge, Alert, Modal, Select, FileButton, ActionIcon, Tooltip } from '@mantine/core'
import { IconGraph, IconServer, IconDatabase, IconPlus, IconSettings, IconFileDatabase, IconTrash, IconFileImport, IconDownload, IconChevronLeft, IconChevronRight, IconFolderPlus } from '@tabler/icons-react'
import { useQuery, useMutation } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { Breadcrumbs } from './components/common/Breadcrumbs'
import { PlanVisualEditor } from './components/editors/PlanVisualEditor/PlanVisualEditor'
import { ErrorBoundary } from './components/common/ErrorBoundary'
import { DataSourcesPage } from './components/datasources/DataSourcesPage'
import { DataSourceEditor } from './components/datasources/DataSourceEditor'
import { CreateProjectModal } from './components/project/CreateProjectModal'
import { TopBar } from './components/layout/TopBar'
import { useCollaborationV2 } from './hooks/useCollaborationV2'
import { useConnectionStatus } from './hooks/useConnectionStatus'

// Collaboration Context for providing project-level collaboration to all pages
const CollaborationContext = React.createContext<any>(null)
export const useCollaboration = () => React.useContext(CollaborationContext)

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
    planDag(projectId: $projectId) {
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

// Generate a unique session ID for this browser tab/window
// This ensures each browser session is tracked separately
const generateSessionId = () => {
  // Use crypto.randomUUID if available, otherwise fallback to timestamp + random
  if (crypto.randomUUID) {
    return `user-${crypto.randomUUID()}`;
  }
  return `user-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
};

// Layout wrapper component
const AppLayout = ({ children }: { children: React.ReactNode }) => {
  const navigate = useNavigate()
  const location = useLocation()

  // Generate stable session ID (only once per component mount)
  const [sessionId] = useState(() => generateSessionId());

  // Navigation collapse state
  const [navCollapsed, setNavCollapsed] = useState(false);

  // Get current route info for navigation highlighting
  const isActiveRoute = (path: string) => {
    if (path === '/') return location.pathname === '/'
    return location.pathname.startsWith(path)
  }

  // Extract project info from current path for navbar
  const projectIdMatch = location.pathname.match(/\/projects\/(\d+)/)
  const projectId = projectIdMatch ? parseInt(projectIdMatch[1]) : undefined

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

  return (
    <AppShell
      header={{ height: 60 }}
      navbar={{ width: navCollapsed ? 60 : 250, breakpoint: 'sm' }}
      padding="md"
      h="100vh"
    >
      <AppShell.Header>
        <TopBar
          projectId={projectId}
          connectionState={connectionStatus.state}
          users={collaboration.users}
          currentUserId={sessionId}
          onNavigateHome={() => navigate('/')}
        />
      </AppShell.Header>

      <AppShell.Navbar p={navCollapsed ? 'xs' : 'md'}>
        <Stack gap="xs" style={{ height: '100%' }}>
          <Group justify="space-between" mb="xs">
            {!navCollapsed && <Title order={4}>Navigation</Title>}
            <Tooltip label={navCollapsed ? "Expand" : "Collapse"} position="right">
              <ActionIcon
                variant="subtle"
                onClick={() => setNavCollapsed(!navCollapsed)}
                size="sm"
              >
                {navCollapsed ? <IconChevronRight size={16} /> : <IconChevronLeft size={16} />}
              </ActionIcon>
            </Tooltip>
          </Group>

          <Stack gap="xs" style={{ flex: 1 }}>
            <Tooltip label="Home" position="right" disabled={!navCollapsed}>
              <Button
                variant={isActiveRoute('/') ? 'filled' : 'light'}
                fullWidth={!navCollapsed}
                leftSection={navCollapsed ? undefined : <IconServer size={16} />}
                onClick={() => navigate('/')}
                px={navCollapsed ? 'xs' : undefined}
                style={navCollapsed ? { justifyContent: 'center' } : undefined}
              >
                {navCollapsed ? <IconServer size={16} /> : 'Home'}
              </Button>
            </Tooltip>
            <Tooltip label="Projects" position="right" disabled={!navCollapsed}>
              <Button
                variant={isActiveRoute('/projects') ? 'filled' : 'light'}
                fullWidth={!navCollapsed}
                leftSection={navCollapsed ? undefined : <IconDatabase size={16} />}
                onClick={() => navigate('/projects')}
                px={navCollapsed ? 'xs' : undefined}
                style={navCollapsed ? { justifyContent: 'center' } : undefined}
              >
                {navCollapsed ? <IconDatabase size={16} /> : 'Projects'}
              </Button>
            </Tooltip>

            {/* Project-specific navigation - only show when in a project */}
            {projectId && (
              <>
                <div style={{ height: '1px', backgroundColor: '#e9ecef', margin: '8px 0' }} />
                <Tooltip label="Project" position="right" disabled={!navCollapsed}>
                  <Button
                    variant={isActiveRoute(`/projects/${projectId}`) && !isActiveRoute(`/projects/${projectId}/plan`) && !isActiveRoute(`/projects/${projectId}/datasources`) && !isActiveRoute(`/projects/${projectId}/graphs`) ? 'filled' : 'light'}
                    fullWidth={!navCollapsed}
                    leftSection={navCollapsed ? undefined : <IconFolderPlus size={16} />}
                    onClick={() => navigate(`/projects/${projectId}`)}
                    px={navCollapsed ? 'xs' : undefined}
                    style={navCollapsed ? { justifyContent: 'center' } : undefined}
                  >
                    {navCollapsed ? <IconFolderPlus size={16} /> : 'Project'}
                  </Button>
                </Tooltip>
                <Tooltip label="Plan" position="right" disabled={!navCollapsed}>
                  <Button
                    variant={isActiveRoute(`/projects/${projectId}/plan`) ? 'filled' : 'light'}
                    fullWidth={!navCollapsed}
                    leftSection={navCollapsed ? undefined : <IconGraph size={16} />}
                    onClick={() => navigate(`/projects/${projectId}/plan`)}
                    px={navCollapsed ? 'xs' : undefined}
                    style={navCollapsed ? { justifyContent: 'center' } : undefined}
                  >
                    {navCollapsed ? <IconGraph size={16} /> : 'Plan'}
                  </Button>
                </Tooltip>
                <Tooltip label="Data Sources" position="right" disabled={!navCollapsed}>
                  <Button
                    variant={isActiveRoute(`/projects/${projectId}/datasources`) ? 'filled' : 'light'}
                    fullWidth={!navCollapsed}
                    leftSection={navCollapsed ? undefined : <IconFileDatabase size={16} />}
                    onClick={() => navigate(`/projects/${projectId}/datasources`)}
                    px={navCollapsed ? 'xs' : undefined}
                    style={navCollapsed ? { justifyContent: 'center' } : undefined}
                  >
                    {navCollapsed ? <IconFileDatabase size={16} /> : 'Data Sources'}
                  </Button>
                </Tooltip>
                <Tooltip label="Graphs" position="right" disabled={!navCollapsed}>
                  <Button
                    variant={isActiveRoute(`/projects/${projectId}/graphs`) ? 'filled' : 'light'}
                    fullWidth={!navCollapsed}
                    leftSection={navCollapsed ? undefined : <IconGraph size={16} />}
                    onClick={() => navigate(`/projects/${projectId}/graphs`)}
                    px={navCollapsed ? 'xs' : undefined}
                    style={navCollapsed ? { justifyContent: 'center' } : undefined}
                  >
                    {navCollapsed ? <IconGraph size={16} /> : 'Graphs'}
                  </Button>
                </Tooltip>
              </>
            )}
          </Stack>

          <div>
            <div style={{ height: '1px', backgroundColor: '#e9ecef', margin: '8px 0' }} />
            <Tooltip label="Database Settings" position="right" disabled={!navCollapsed}>
              <Button
                variant={isActiveRoute('/settings/database') ? 'filled' : 'light'}
                fullWidth={!navCollapsed}
                leftSection={navCollapsed ? undefined : <IconSettings size={16} />}
                onClick={() => navigate('/settings/database')}
                px={navCollapsed ? 'xs' : undefined}
                style={navCollapsed ? { justifyContent: 'center' } : undefined}
              >
                {navCollapsed ? <IconSettings size={16} /> : 'Database Settings'}
              </Button>
            </Tooltip>
          </div>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        <CollaborationContext.Provider value={collaboration}>
          {children}
        </CollaborationContext.Provider>
      </AppShell.Main>
    </AppShell>
  )
}

// Home page component
const HomePage = () => {
  const navigate = useNavigate()
  const [createModalOpened, setCreateModalOpened] = useState(false)
  const [sampleModalOpened, setSampleModalOpened] = useState(false)
  const [selectedSampleKey, setSelectedSampleKey] = useState<string | null>(null)
  const [sampleError, setSampleError] = useState<string | null>(null)

  const { data: projectsData } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

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
    <div style={{ width: '100%', height: '100%' }}>
      {/* Action buttons section */}
      <div style={{ padding: '3rem 2rem', backgroundColor: '#f8f9fa', borderBottom: '1px solid #dee2e6' }}>
        <Group justify="center" gap="xl">
          <Button
            size="xl"
            variant="filled"
            leftSection={<IconDatabase size={24} />}
            onClick={() => navigate('/projects')}
            style={{ minWidth: 240, height: 80, fontSize: '1.1rem' }}
          >
            Browse Projects
          </Button>
          <Button
            size="xl"
            variant="filled"
            color="blue"
            leftSection={<IconPlus size={24} />}
            onClick={handleCreateProject}
            style={{ minWidth: 240, height: 80, fontSize: '1.1rem' }}
          >
            Start New Project
          </Button>
          <Button
            size="xl"
            variant="filled"
            color="teal"
            leftSection={<IconFolderPlus size={24} />}
            onClick={handleOpenSampleModal}
            style={{ minWidth: 240, height: 80, fontSize: '1.1rem' }}
          >
            Import Sample Project
          </Button>
        </Group>
      </div>

      {/* Recent projects section */}
      <div style={{ padding: '2rem' }}>
        <Title order={2} mb="xl" style={{ textAlign: 'center' }}>
          Recent Projects
        </Title>

        {recentProjects.length === 0 ? (
          <Card withBorder p="xl" radius="md" style={{ maxWidth: 600, margin: '0 auto' }}>
            <Stack align="center" gap="md">
              <IconGraph size={48} color="gray" />
              <Title order={3}>No Projects Yet</Title>
              <Text ta="center" c="dimmed">
                Create your first project to get started with Layercake.
              </Text>
            </Stack>
          </Card>
        ) : (
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(320px, 1fr))',
            gap: '1.5rem',
            maxWidth: 1600,
            margin: '0 auto'
          }}>
            {recentProjects.map((project) => (
              <Card
                key={project.id}
                withBorder
                padding="lg"
                radius="md"
                shadow="sm"
                style={{ cursor: 'pointer', height: '100%', display: 'flex', flexDirection: 'column' }}
                onClick={() => navigate(`/projects/${project.id}`)}
              >
                <Card.Section withBorder inheritPadding py="xs" style={{ backgroundColor: '#f8f9fa' }}>
                  <Group justify="space-between">
                    <Group gap="xs">
                      <IconGraph size={20} />
                      <Text fw={600}>{project.name}</Text>
                    </Group>
                    <Badge variant="light" size="sm">
                      ID: {project.id}
                    </Badge>
                  </Group>
                </Card.Section>

                <Stack gap="sm" mt="md" style={{ flex: 1 }}>
                  {project.description && (
                    <Text size="sm" c="dimmed" lineClamp={2}>
                      {project.description}
                    </Text>
                  )}

                  <div style={{ marginTop: 'auto' }}>
                    <Text size="xs" c="dimmed">
                      Updated {new Date(project.updatedAt).toLocaleDateString()}
                    </Text>
                  </div>
                </Stack>

                <Card.Section withBorder inheritPadding py="xs" mt="md">
                  <Group gap="xs" justify="flex-end">
                    <Button
                      size="xs"
                      variant="light"
                      leftSection={<IconGraph size={14} />}
                      onClick={(e) => {
                        e.stopPropagation()
                        navigate(`/projects/${project.id}/plan`)
                      }}
                    >
                      Plan
                    </Button>
                    <Button
                      size="xs"
                      variant="light"
                      leftSection={<IconFileDatabase size={14} />}
                      onClick={(e) => {
                        e.stopPropagation()
                        navigate(`/projects/${project.id}/datasources`)
                      }}
                    >
                      Data
                    </Button>
                  </Group>
                </Card.Section>
              </Card>
            ))}
          </div>
        )}
      </div>

      <CreateProjectModal
        opened={createModalOpened}
        onClose={() => setCreateModalOpened(false)}
        onSuccess={handleProjectCreated}
      />

      <Modal
        opened={sampleModalOpened}
        onClose={handleSampleModalClose}
        title="Import Sample Project"
        size="md"
      >
        <Stack gap="md">
          <Text size="sm" c="dimmed">
            Select one of the bundled samples to create a project preloaded with data sources and a starter DAG.
          </Text>

          <Select
            label="Sample Project"
            placeholder={sampleProjectsLoading ? 'Loading samples...' : 'Select a sample'}
            data={sampleOptions}
            value={selectedSampleKey}
            onChange={setSelectedSampleKey}
            disabled={sampleProjectsLoading || sampleOptions.length === 0}
          />

          {selectedSample?.description && (
            <Text size="sm" c="dimmed">
              {selectedSample.description}
            </Text>
          )}

          {sampleError && (
            <Alert color="red" title="Cannot create sample project">
              {sampleError}
            </Alert>
          )}

          <Group justify="flex-end" gap="xs">
            <Button variant="subtle" onClick={handleSampleModalClose} disabled={createSampleLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateSampleProject}
              loading={createSampleLoading}
              disabled={!selectedSampleKey || sampleProjectsLoading}
            >
              Create Sample Project
            </Button>
          </Group>
        </Stack>
      </Modal>
    </div>
  )
}

// Projects list page component
const ProjectsPage = () => {
  const navigate = useNavigate()
  const [createModalOpened, setCreateModalOpened] = useState(false)
  const [importModalOpened, setImportModalOpened] = useState(false)
  const [selectedProjectForImport, setSelectedProjectForImport] = useState<number | null>(null)
  const [importFile, setImportFile] = useState<File | null>(null)
  const [importLoading, setImportLoading] = useState(false)
  const [importError, setImportError] = useState<string | null>(null)
  const [sampleModalOpened, setSampleModalOpened] = useState(false)
  const [selectedSampleKey, setSelectedSampleKey] = useState<string | null>(null)
  const [sampleError, setSampleError] = useState<string | null>(null)

  const { data: projectsData, loading: projectsLoading, error: projectsError, refetch } = useQuery<{
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

  const [importPlanYaml] = useMutation(gql`
    mutation ImportPlanYaml($projectId: Int!, $yamlContent: String!) {
      importPlanYaml(projectId: $projectId, yamlContent: $yamlContent) {
        success
        message
        nodeCount
        edgeCount
      }
    }
  `)

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

  const handleImportClick = () => {
    setImportModalOpened(true)
  }

  const handleImportProject = async () => {
    if (!selectedProjectForImport || !importFile) {
      setImportError('Please select a project and upload a YAML file')
      return
    }

    setImportLoading(true)
    setImportError(null)

    try {
      const yamlContent = await importFile.text()
      const result = await importPlanYaml({
        variables: {
          projectId: selectedProjectForImport,
          yamlContent,
        },
      })

      if ((result.data as any)?.importPlanYaml?.success) {
        setImportModalOpened(false)
        setImportFile(null)
        setSelectedProjectForImport(null)
        navigate(`/projects/${selectedProjectForImport}/plan`)
      }
    } catch (error: any) {
      setImportError(error.message || 'Failed to import plan')
    } finally {
      setImportLoading(false)
    }
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
    <Container size="xl">
      <Breadcrumbs currentPage="Projects" onNavigate={handleNavigate} />

      <Group justify="space-between" mb="md">
        <Title order={1}>Projects</Title>
        <Group gap="xs">
          <Button leftSection={<IconPlus size={16} />} onClick={handleCreateProject}>
            New Project
          </Button>
          <Button
            variant="light"
            leftSection={<IconFolderPlus size={16} />}
            onClick={handleOpenSampleModal}
          >
            Add Sample Project
          </Button>
          <Button
            variant="light"
            leftSection={<IconFileImport size={16} />}
            onClick={handleImportClick}
          >
            Import Plan
          </Button>
        </Group>
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
            <Button leftSection={<IconPlus size={16} />} onClick={handleCreateProject}>
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
                    Plan
                  </Button>
                  <Button
                    variant="light"
                    size="sm"
                    color="red"
                    leftSection={<IconTrash size={14} />}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteProject(project.id);
                    }}
                  >
                    Delete
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

      <CreateProjectModal
        opened={createModalOpened}
        onClose={() => setCreateModalOpened(false)}
        onSuccess={handleProjectCreated}
      />

      <Modal
        opened={sampleModalOpened}
        onClose={handleSampleModalClose}
        title="Add Sample Project"
        size="md"
      >
        <Stack gap="md">
          <Text size="sm" c="dimmed">
            Select one of the bundled samples to create a project preloaded with data sources and a starter DAG.
          </Text>

          {sampleProjectsError && (
            <Alert color="red" title="Unable to load samples">
              {sampleProjectsError.message}
            </Alert>
          )}

          <Select
            label="Sample Project"
            placeholder={sampleProjectsLoading ? 'Loading samples...' : 'Select a sample'}
            data={sampleOptions}
            value={selectedSampleKey}
            onChange={setSelectedSampleKey}
            disabled={sampleProjectsLoading || sampleOptions.length === 0}
          />

          {selectedSample?.description && (
            <Text size="sm" c="dimmed">
              {selectedSample.description}
            </Text>
          )}

          {sampleError && (
            <Alert color="red" title="Cannot create sample project">
              {sampleError}
            </Alert>
          )}

          <Group justify="flex-end" gap="xs">
            <Button variant="subtle" onClick={handleSampleModalClose} disabled={createSampleLoading}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateSampleProject}
              loading={createSampleLoading}
              disabled={!selectedSampleKey || sampleProjectsLoading}
            >
              Create Sample Project
            </Button>
          </Group>
        </Stack>
      </Modal>

      <Modal
        opened={importModalOpened}
        onClose={() => {
          setImportModalOpened(false)
          setImportFile(null)
          setSelectedProjectForImport(null)
          setImportError(null)
        }}
        title="Import Plan from YAML"
        size="md"
      >
        <Stack gap="md">
          <Text size="sm" c="dimmed">
            Import a plan.yaml file to automatically create a DAG structure with nodes and edges.
          </Text>

          <Select
            label="Target Project"
            placeholder="Select a project"
            data={projects.map(p => ({ value: p.id.toString(), label: p.name }))}
            value={selectedProjectForImport?.toString() || null}
            onChange={(value) => setSelectedProjectForImport(value ? parseInt(value) : null)}
            required
          />

          <div>
            <Text size="sm" fw={500} mb={4}>
              YAML File
            </Text>
            <FileButton
              onChange={setImportFile}
              accept=".yaml,.yml"
            >
              {(props) => (
                <Button {...props} variant="light" fullWidth>
                  {importFile ? importFile.name : 'Select YAML file'}
                </Button>
              )}
            </FileButton>
            {importFile && (
              <Text size="xs" c="dimmed" mt={4}>
                Selected: {importFile.name} ({(importFile.size / 1024).toFixed(2)} KB)
              </Text>
            )}
          </div>

          {importError && (
            <Alert color="red" title="Import Failed">
              {importError}
            </Alert>
          )}

          <Group justify="flex-end" gap="xs">
            <Button
              variant="subtle"
              onClick={() => {
                setImportModalOpened(false)
                setImportFile(null)
                setSelectedProjectForImport(null)
                setImportError(null)
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={handleImportProject}
              loading={importLoading}
              disabled={!selectedProjectForImport || !importFile}
            >
              Import
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Container>
  )
}

// Project detail page component
const ProjectDetailPage = () => {
  const navigate = useNavigate()
  const { projectId } = useParams<{ projectId: string }>()
  const { data: projectsData, loading: projectsLoading } = useQuery<{
    projects: Array<{
      id: number
      name: string
      description: string
      createdAt: string
      updatedAt: string
    }>
  }>(GET_PROJECTS)

  const { data: planDagData } = useQuery(GET_PLAN_DAG, {
    variables: { projectId: parseInt(projectId || '0') },
    skip: !projectId,
  })

  const projects = projectsData?.projects || []
  const selectedProject = projects.find((p: any) => p.id === parseInt(projectId || '0'))
  const planDag = (planDagData as any)?.planDag

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

  // Show loading state while projects are being fetched
  if (projectsLoading) {
    return (
      <Container size="xl">
        <Text>Loading project...</Text>
      </Container>
    )
  }

  // Only show "not found" if loading is complete and project doesn't exist
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
      title: 'Data Sources',
      description: 'Manage CSV and JSON files that serve as input data for your Plan DAGs',
      icon: <IconFileDatabase size={20} />,
      onClick: () => navigate(`/projects/${projectId}/datasources`),
    },
    {
      title: 'Plan',
      description: 'Create and edit Plan DAGs with visual node-based interface',
      icon: <IconGraph size={20} />,
      onClick: () => navigate(`/projects/${projectId}/plan`),
      primary: true,
    },
    {
      title: 'Graphs',
      description: 'Manage graph entities for this project',
      icon: <IconDatabase size={20} />,
      onClick: () => navigate(`/projects/${projectId}/graphs`),
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
                <Group gap="xs">
                  {action.title === 'Plan' && planDag && (
                    <Tooltip label="Download Plan DAG as YAML">
                      <ActionIcon
                        variant="subtle"
                        size="lg"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleDownloadYAML()
                        }}
                      >
                        <IconDownload size="1.2rem" />
                      </ActionIcon>
                    </Tooltip>
                  )}
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
                </Group>
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

  const handleNavigate = (route: string) => {
    navigate(route)
  }

  // Show loading state while projects are being fetched
  if (projectsLoading) {
    return (
      <Container size="xl">
        <Text>Loading project...</Text>
      </Container>
    )
  }

  // Only show "not found" if loading is complete and project doesn't exist
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

  const searchParams = new URLSearchParams(location.search)
  const focusNodeId = searchParams.get('focusNode') || undefined

  return (
    <Stack gap={0} style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <div style={{ padding: '8px 16px', borderBottom: '1px solid #e9ecef', flexShrink: 0 }}>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          currentPage="Plan"
          onNavigate={handleNavigate}
        />
      </div>
      <div style={{ flex: 1, overflow: 'hidden' }}>
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
    </Stack>
  )
}

import { GraphsPage } from './components/graphs/GraphsPage'
import { GraphEditorPage } from './pages/GraphEditorPage'
import { DatabaseSettings } from './components/settings/DatabaseSettings'

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
          <Route path="/projects/:projectId/graphs" element={
            <ErrorBoundary>
              <GraphsPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/graphs/:graphId/edit" element={
            <ErrorBoundary>
              <GraphEditorPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/datasources" element={
            <ErrorBoundary>
              <DataSourcesPage />
            </ErrorBoundary>
          } />
          <Route path="/projects/:projectId/datasources/:dataSourceId/edit" element={
            <ErrorBoundary>
              <DataSourceEditor />
            </ErrorBoundary>
          } />
          <Route path="/settings/database" element={
            <ErrorBoundary>
              <DatabaseSettings />
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
