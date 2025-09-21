import { useState } from 'react'
import { AppShell, Container, Title, Text, Button, Group, Stack } from '@mantine/core'
import { IconGraph, IconServer, IconDatabase, IconPlus } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { ConnectionStatus } from './components/common/ConnectionStatus'
import { PlanVisualEditor } from './components/editors/PlanVisualEditor/PlanVisualEditor'

// Temporary mock query until we have the actual GraphQL schema
const HEALTH_CHECK = gql`
  query HealthCheck {
    health {
      status
      timestamp
    }
  }
`

type ViewType = 'home' | 'projects' | 'plan-editor' | 'graph-editor'

function App() {
  const [currentView, setCurrentView] = useState<ViewType>('home')
  const [selectedProjectId, setSelectedProjectId] = useState<number>(1) // Mock project ID

  // Mock health check query - will be replaced with actual schema
  const { loading, error } = useQuery(HEALTH_CHECK, {
    errorPolicy: 'ignore', // Ignore errors for now since backend isn't ready
    skip: true, // Skip the query entirely for frontend-only development
  })

  const renderCurrentView = () => {
    switch (currentView) {
      case 'home':
        return (
          <Container size="xl">
            <Title order={1} mb="xl">Welcome to Layercake</Title>

            <Text size="lg" mb="md">
              Interactive graph transformation and visualization tool with real-time collaboration.
            </Text>

            <Group mb="xl">
              <div>
                <Text fw={500}>Status:</Text>
                <Text size="sm" c={error ? 'red' : loading ? 'yellow' : 'green'}>
                  {error ? 'Backend Disconnected' : loading ? 'Connecting...' : 'Connected'}
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
                Create New Project
              </Button>
              <Button
                size="lg"
                variant="outline"
                leftSection={<IconGraph size={20} />}
                onClick={() => setCurrentView('plan-editor')}
              >
                Try Plan Editor
              </Button>
            </Group>

            <Text size="sm" c="dimmed">
              Phase 1.3: ReactFlow Plan DAG Visual Editor - Interactive node-based plan creation
            </Text>
          </Container>
        )

      case 'projects':
        return (
          <Container size="xl">
            <Group justify="space-between" mb="md">
              <Title order={1}>Projects</Title>
              <Button leftSection={<IconPlus size={16} />}>
                New Project
              </Button>
            </Group>
            <Text c="dimmed">Project management interface coming soon...</Text>
          </Container>
        )

      case 'plan-editor':
        return (
          <div style={{ height: '100vh' }}>
            <PlanVisualEditor
              projectId={selectedProjectId}
              onNodeSelect={(nodeId) => console.log('Selected node:', nodeId)}
              onEdgeSelect={(edgeId) => console.log('Selected edge:', edgeId)}
            />
          </div>
        )

      case 'graph-editor':
        return (
          <Container size="xl">
            <Title order={1} mb="md">Graph Editor</Title>
            <Text c="dimmed">Graph visual editor interface coming in Phase 2...</Text>
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
          <Group>
            <IconGraph size={28} />
            <Title order={2}>Layercake</Title>
          </Group>
          <ConnectionStatus />
        </Group>
      </AppShell.Header>

      <AppShell.Navbar p="md">
        <Title order={4} mb="md">Navigation</Title>
        <Stack spacing="xs">
          <Button
            variant={currentView === 'home' ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconServer size={16} />}
            onClick={() => setCurrentView('home')}
          >
            Home
          </Button>
          <Button
            variant={currentView === 'projects' ? 'filled' : 'light'}
            fullWidth
            leftSection={<IconDatabase size={16} />}
            onClick={() => setCurrentView('projects')}
          >
            Projects
          </Button>
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
          >
            Graph Editor
          </Button>
        </Stack>
      </AppShell.Navbar>

      <AppShell.Main>
        {renderCurrentView()}
      </AppShell.Main>
    </AppShell>
  )
}

export default App