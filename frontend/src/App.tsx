// React import not needed with new JSX transform
import { AppShell, Container, Title, Text, Button, Group } from '@mantine/core'
import { IconGraph, IconServer, IconDatabase } from '@tabler/icons-react'
import { useQuery } from '@apollo/client/react'
import { gql } from '@apollo/client'
import { ConnectionStatus } from './components/common/ConnectionStatus'

// Temporary mock query until we have the actual GraphQL schema
const HEALTH_CHECK = gql`
  query HealthCheck {
    health {
      status
      timestamp
    }
  }
`

function App() {
  // Mock health check query - will be replaced with actual schema
  const { loading, error } = useQuery(HEALTH_CHECK, {
    errorPolicy: 'ignore', // Ignore errors for now since backend isn't ready
  })

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
        <Button variant="light" fullWidth mb="sm" leftSection={<IconDatabase size={16} />}>
          Projects
        </Button>
        <Button variant="light" fullWidth mb="sm" leftSection={<IconGraph size={16} />}>
          Plan Editor
        </Button>
        <Button variant="light" fullWidth mb="sm" leftSection={<IconGraph size={16} />}>
          Graph Editor
        </Button>
      </AppShell.Navbar>

      <AppShell.Main>
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
            <Button size="lg" leftSection={<IconDatabase size={20} />}>
              Create New Project
            </Button>
            <Button size="lg" variant="outline" leftSection={<IconServer size={20} />}>
              Connect to Server
            </Button>
          </Group>

          <Text size="sm" c="dimmed">
            Phase 1: Frontend Foundation - Tauri Desktop Application with Apollo Client integration
          </Text>
        </Container>
      </AppShell.Main>
    </AppShell>
  )
}

export default App