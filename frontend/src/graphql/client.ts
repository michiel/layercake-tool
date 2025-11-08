import { ApolloClient, InMemoryCache, split, from } from '@apollo/client'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { onError } from '@apollo/client/link/error'
import { setContext } from '@apollo/client/link/context'
import { createClient } from 'graphql-ws'
import UploadHttpLink from 'apollo-upload-client/UploadHttpLink.mjs'
import { getServerInfo, isTauriApp, waitForServer } from '../utils/tauri'
import { getOrCreateSessionId } from '../utils/session'
import { extractGraphQLErrorMessage } from '../utils/errorHandling'
import { showErrorNotification } from '../utils/notifications'

// Store server configuration
let serverConfig: { url: string; secret: string; wsUrl: string } | null = null

// Initialize server configuration for Tauri
export async function initializeTauriServer(): Promise<void> {
  if (!isTauriApp()) {
    console.log('[GraphQL] Not running in Tauri, using web mode configuration')
    return
  }

  console.log('[GraphQL] Initializing Tauri server connection...')

  // Wait for the server to be ready
  const isReady = await waitForServer()
  if (!isReady) {
    throw new Error('Failed to connect to embedded server')
  }

  // Get server info
  const info = await getServerInfo()
  if (!info) {
    throw new Error('Failed to get server information')
  }

  serverConfig = {
    url: info.url,
    secret: info.secret,
    wsUrl: info.url.replace('http', 'ws'),
  }

  console.log('[GraphQL] Tauri server configured:', { url: serverConfig.url })

  // Create the Apollo client now that we have the server config
  if (!apolloClientInstance) {
    console.log('[GraphQL] Creating Apollo Client for Tauri mode')
    apolloClientInstance = createApolloClient()
  }
}

// GraphQL endpoints - configurable for different environments
const getGraphQLEndpoints = () => {
  // Use Tauri server if configured
  if (serverConfig) {
    console.log('[GraphQL] Using Tauri server config:', serverConfig.url)
    return {
      httpUrl: `${serverConfig.url}/graphql`,
      wsUrl: `${serverConfig.wsUrl}/graphql/ws`,
      secret: serverConfig.secret,
    }
  }

  // Otherwise use environment variables (web mode)
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001'
  console.log('[GraphQL] Using web mode config:', baseUrl)
  return {
    httpUrl: `${baseUrl}/graphql`,
    wsUrl: `${baseUrl.replace('http', 'ws')}/graphql/ws`,
    secret: null,
  }
}

// Create Apollo Client - must be called after initializeTauriServer() in Tauri mode
let apolloClientInstance: ApolloClient | null = null

function createApolloClient(): ApolloClient {
  console.log('[GraphQL] Creating Apollo Client with endpoints:', getGraphQLEndpoints())

  // Create authentication link for Tauri secret
  const authLink = setContext((_, { headers }) => {
    const { secret } = getGraphQLEndpoints()
    const sessionId = getOrCreateSessionId()

    // Add secret header if available (Tauri mode)
    if (secret) {
      return {
        headers: {
          ...headers,
          'x-tauri-secret': secret,
          'x-layercake-session': sessionId,
        },
      }
    }

    return {
      headers: {
        ...headers,
        'x-layercake-session': sessionId,
      },
    }
  })

  // HTTP Link for queries and mutations with timeout using AbortController
  // Use a function to get the current endpoint (supports dynamic reconfiguration)
  const httpLink = new UploadHttpLink({
    uri: () => {
      const { httpUrl } = getGraphQLEndpoints()
      console.log('[GraphQL HTTP] Using endpoint:', httpUrl)
      return httpUrl
    },
    credentials: 'omit',
    fetch: (uri: RequestInfo | URL, options?: RequestInit) => {
      const controller = new AbortController()
      const timeout = setTimeout(() => {
        controller.abort()
      }, 30000)

      const requestOptions: RequestInit = {
        ...(options ?? {}),
        signal: controller.signal,
      }

      return fetch(uri, requestOptions).finally(() => {
        clearTimeout(timeout)
      })
    },
  })

  // WebSocket Link for subscriptions (real-time collaboration)
  const { wsUrl: currentWsUrl } = getGraphQLEndpoints()
  console.log('[GraphQL WebSocket] Creating client with URL:', currentWsUrl)

  console.log('[GraphQL WebSocket] Creating WebSocket client for:', currentWsUrl)

  const wsClient = createClient({
    url: currentWsUrl,
    connectionParams: () => {
      const { secret } = getGraphQLEndpoints()
      const sessionId = getOrCreateSessionId()
      console.log('[GraphQL WebSocket] Getting connection params, secret:', secret ? 'present' : 'none')
      // Include secret in WebSocket connection params if available (Tauri mode)
      if (secret) {
        return { 'x-tauri-secret': secret, 'x-layercake-session': sessionId }
      }
      return { 'x-layercake-session': sessionId }
    },
    shouldRetry: () => {
      // Retry connection on network errors
      console.log('[GraphQL WebSocket] Retrying connection')
      return true
    },
    retryAttempts: 10, // Retry up to 10 times
    retryWait: async (retries) => {
      // Exponential backoff with max 10 seconds
      const delay = Math.min(1000 * Math.pow(2, retries), 10000)
      console.log(`[GraphQL WebSocket] Waiting ${delay}ms before retry ${retries}`)
      await new Promise(resolve => setTimeout(resolve, delay))
    },
    lazy: true, // Connect lazily when first subscription is created (allows proper reconnection)
    keepAlive: 10000, // Send ping every 10 seconds to keep connection alive
    on: {
      connected: () => console.log('[GraphQL WebSocket] ✅ Connected to', currentWsUrl),
      connecting: () => console.log('[GraphQL WebSocket] 🔄 Connecting to', currentWsUrl),
      closed: (event) => console.log('[GraphQL WebSocket] ❌ Closed', event),
      error: (error) => console.error('[GraphQL WebSocket] ⚠️ Error', error),
    },
  })

  console.log('[GraphQL WebSocket] WebSocket client created')

  const wsLink = new GraphQLWsLink(wsClient)

  // Error handling link
  const errorLink = onError((errorResponse) => {
    const graphQLErrors = (errorResponse as any).graphQLErrors
    const networkError = (errorResponse as any).networkError
    if (graphQLErrors) {
      graphQLErrors.forEach((error: any) => {
        console.error(
          `[GraphQL error]: Message: ${error.message}, Location: ${error.locations}, Path: ${error.path}`
        )
        const message = extractGraphQLErrorMessage({ graphQLErrors: [error] })
        showErrorNotification('GraphQL error', message)
      })
    }

    if (networkError) {
      console.error(`[Network error]: ${networkError}`)
      const networkMessage = (networkError as any)?.message || 'Network request failed'
      showErrorNotification('Network error', networkMessage)
      // Handle authentication errors (disabled in development)
      // if ('statusCode' in networkError && (networkError as any).statusCode === 401) {
      //   localStorage.removeItem('auth_token')
      // }
    }
  })

  // Split link to route queries/mutations vs subscriptions
  const splitLink = split(
    ({ query }) => {
      const definition = getMainDefinition(query)
      return (
        definition.kind === 'OperationDefinition' &&
        definition.operation === 'subscription'
      )
    },
    wsLink,
    from([authLink, errorLink, httpLink]) // Apply auth and error handling to HTTP link
  )

  return new ApolloClient({
    link: splitLink,
    cache: new InMemoryCache({
    typePolicies: {
      Project: {
        fields: {
          layercakeGraphs: {
            merge(_existing = [], incoming) {
              return incoming
            },
          },
          collaborators: {
            merge(_existing = [], incoming) {
              return incoming
            },
          },
        },
      },
      LayercakeGraph: {
        fields: {
          nodeCount: {
            merge: false, // Always replace with new value
          },
          edgeCount: {
            merge: false,
          },
          lastModified: {
            merge: false,
          },
        },
      },
      PlanDagNode: {
        keyFields: ['id'],
        fields: {
          position: {
            merge: false, // Always replace position updates
          },
          metadata: {
            merge(existing, incoming) {
              return { ...existing, ...incoming }
            },
          },
        },
      },
      Query: {
        fields: {
          projects: {
            merge(_existing, incoming) {
              return incoming
            },
          },
          graphNodes: {
            keyArgs: ['projectId', 'layercakeGraphId'],
            merge(existing, incoming) {
              if (!existing) return incoming

              // Merge pagination results
              return {
                ...incoming,
                edges: [...(existing.edges || []), ...incoming.edges],
              }
            },
          },
        },
      },
    },
  }),
  defaultOptions: {
    watchQuery: {
      errorPolicy: 'all', // Show partial data even with errors
      notifyOnNetworkStatusChange: true,
    },
    query: {
      errorPolicy: 'all',
    },
    mutate: {
      errorPolicy: 'all',
    },
  },
  // Apollo Client configuration complete
})
}

// Export Apollo Client with lazy initialization
// This ensures the client is only created when first accessed,
// allowing serverConfig to be set up first in Tauri mode
export const apolloClient = new Proxy({} as ApolloClient, {
  get(_target, prop, receiver) {
    if (!apolloClientInstance) {
      console.log('[GraphQL] Creating Apollo Client (lazy initialization)')
      apolloClientInstance = createApolloClient()
    }
    return Reflect.get(apolloClientInstance, prop, receiver)
  },
}) as ApolloClient

// Helper function to handle connection state
export const getConnectionState = () => {
  // This would connect to the WebSocket state if available
  return 'connected' // Simplified for now
}

// Offline operation queue (basic implementation)
export const queueOperationForRetry = (operation: any) => {
  const queue = JSON.parse(localStorage.getItem('offline_operations') || '[]')
  queue.push({
    ...operation,
    timestamp: Date.now(),
  })
  localStorage.setItem('offline_operations', JSON.stringify(queue))
}

// Process offline operations when connection is restored
export const processOfflineOperations = async () => {
  const queue = JSON.parse(localStorage.getItem('offline_operations') || '[]')

  for (const operation of queue) {
    try {
      // This would retry the operation
      console.log('Retrying operation:', operation)
      // await apolloClient.mutate(operation)
    } catch (error) {
      console.error('Failed to retry operation:', error)
    }
  }

  // Clear processed operations
  localStorage.removeItem('offline_operations')
}
