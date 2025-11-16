import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { ApolloProvider } from '@apollo/client/react'
import { apolloClient, initializeTauriServer } from './graphql/client'
import { isTauriApp } from './utils/tauri'
import App from './App'
import { Toaster } from '@/components/ui/sonner'
import { Spinner } from '@/components/ui/spinner'
import { Stack } from '@/components/layout-primitives'
import { ThemeProvider } from '@/components/theme/theme-provider'
import { TagsFilterProvider } from '@/hooks/useTagsFilter'

// Tailwind CSS
import './index.css'

const root = ReactDOM.createRoot(document.getElementById('root')!)

// Check if running in Tauri and initialize server connection
if (isTauriApp()) {
  // Show loading state while waiting for server
  root.render(
    <React.StrictMode>
      <Toaster />
      <div className="flex items-center justify-center h-screen">
        <Stack gap="md" className="items-center">
          <Spinner className="h-12 w-12" />
          <p className="text-lg font-medium">
            Starting Layercake...
          </p>
          <p className="text-sm text-muted-foreground">
            Connecting to embedded server
          </p>
        </Stack>
      </div>
    </React.StrictMode>
  )

  // Initialize Tauri server connection
  initializeTauriServer()
    .then(async () => {
      // Clear all caches on Tauri startup to ensure fresh state
      console.log('[Tauri] Clearing all caches on startup')

      // Preserve theme before clearing storage
      const themeKey = 'layercake-theme'
      const savedTheme = typeof localStorage !== 'undefined' ? localStorage.getItem(themeKey) : null

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
        if (savedTheme) {
          localStorage.setItem(themeKey, savedTheme)
          console.log('[Tauri] Restored theme preference')
        }
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
            <Toaster />
            <BrowserRouter>
              <ThemeProvider
                attribute="class"
                defaultTheme="light"
                enableSystem={false}
                storageKey="layercake-theme"
              >
                <TagsFilterProvider>
                  <App />
                </TagsFilterProvider>
              </ThemeProvider>
            </BrowserRouter>
          </ApolloProvider>
        </React.StrictMode>
      )
    })
    .catch((error) => {
      // Show error state
      root.render(
        <React.StrictMode>
          <Toaster />
          <div className="flex items-center justify-center h-screen">
            <Stack gap="md" className="items-center max-w-[500px]">
              <p className="text-xl font-bold text-red-600">
                Failed to connect to server
              </p>
              <p className="text-sm text-center">
                {error.message || 'Unknown error occurred'}
              </p>
              <p className="text-xs text-muted-foreground text-center">
                Please try restarting the application. If the problem persists, check the application logs.
              </p>
            </Stack>
          </div>
        </React.StrictMode>
      )
    })
} else {
  // Web mode - render normally
  root.render(
    <React.StrictMode>
      <ApolloProvider client={apolloClient}>
        <Toaster />
        <BrowserRouter>
          <ThemeProvider
            attribute="class"
            defaultTheme="light"
            enableSystem={false}
            storageKey="layercake-theme"
          >
            <TagsFilterProvider>
              <App />
            </TagsFilterProvider>
          </ThemeProvider>
        </BrowserRouter>
      </ApolloProvider>
    </React.StrictMode>
  )
}
