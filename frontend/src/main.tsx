import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { ApolloProvider } from '@apollo/client/react'
import { MantineProvider, Loader, Center, Stack, Text } from '@mantine/core'
import { Notifications } from '@mantine/notifications'
import { apolloClient, initializeTauriServer } from './graphql/client'
import { isTauriApp } from './utils/tauri'
import App from './App'

// Mantine CSS
import '@mantine/core/styles.css'
import '@mantine/dates/styles.css'
import '@mantine/notifications/styles.css'

const root = ReactDOM.createRoot(document.getElementById('root')!)

// Check if running in Tauri and initialize server connection
if (isTauriApp()) {
  // Show loading state while waiting for server
  root.render(
    <React.StrictMode>
      <MantineProvider>
        <Notifications />
        <Center style={{ height: '100vh' }}>
          <Stack align="center" gap="md">
            <Loader size="xl" />
            <Text size="lg" fw={500}>
              Starting Layercake...
            </Text>
            <Text size="sm" c="dimmed">
              Connecting to embedded server
            </Text>
          </Stack>
        </Center>
      </MantineProvider>
    </React.StrictMode>
  )

  // Initialize Tauri server connection
  initializeTauriServer()
    .then(async () => {
      // Clear all caches on Tauri startup to ensure fresh state
      console.log('[Tauri] Clearing all caches on startup')

      // Clear browser caches
      if ('caches' in window) {
        try {
          const names = await caches.keys()
          await Promise.all(names.map(name => caches.delete(name)))
          console.log('[Tauri] Cleared browser caches:', names)
        } catch (error) {
          console.warn('[Tauri] Failed to clear browser caches:', error)
        }
      }

      // Clear local storage
      try {
        localStorage.clear()
        console.log('[Tauri] Cleared localStorage')
      } catch (error) {
        console.warn('[Tauri] Failed to clear localStorage:', error)
      }

      // Clear session storage
      try {
        sessionStorage.clear()
        console.log('[Tauri] Cleared sessionStorage')
      } catch (error) {
        console.warn('[Tauri] Failed to clear sessionStorage:', error)
      }

      // Server is ready, render the app
      root.render(
        <React.StrictMode>
          <ApolloProvider client={apolloClient}>
            <MantineProvider>
              <Notifications />
              <BrowserRouter>
                <App />
              </BrowserRouter>
            </MantineProvider>
          </ApolloProvider>
        </React.StrictMode>
      )
    })
    .catch((error) => {
      // Show error state
      root.render(
        <React.StrictMode>
          <MantineProvider>
            <Notifications />
            <Center style={{ height: '100vh' }}>
              <Stack align="center" gap="md" style={{ maxWidth: 500 }}>
                <Text size="xl" fw={700} c="red">
                  Failed to connect to server
                </Text>
                <Text size="sm" ta="center">
                  {error.message || 'Unknown error occurred'}
                </Text>
                <Text size="xs" c="dimmed" ta="center">
                  Please try restarting the application. If the problem persists, check the application logs.
                </Text>
              </Stack>
            </Center>
          </MantineProvider>
        </React.StrictMode>
      )
    })
} else {
  // Web mode - render normally
  root.render(
    <React.StrictMode>
      <ApolloProvider client={apolloClient}>
        <MantineProvider>
          <Notifications />
          <BrowserRouter>
            <App />
          </BrowserRouter>
        </MantineProvider>
      </ApolloProvider>
    </React.StrictMode>
  )
}
