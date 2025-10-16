import { ApolloClient, InMemoryCache, createHttpLink, split, from } from '@apollo/client'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { onError } from '@apollo/client/link/error'
import { setContext } from '@apollo/client/link/context'
import { createClient } from 'graphql-ws'
import { getServerInfo, isTauriApp, waitForServer } from '../utils/tauri'

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
}

// GraphQL endpoints - configurable for different environments
const getGraphQLEndpoints = () => {
  // Use Tauri server if configured
  if (serverConfig) {
    return {
      httpUrl: `${serverConfig.url}/graphql`,
      wsUrl: `${serverConfig.wsUrl}/graphql/ws`,
      secret: serverConfig.secret,
    }
  }

  // Otherwise use environment variables (web mode)
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001'
  return {
    httpUrl: `${baseUrl}/graphql`,
    wsUrl: `${baseUrl.replace('http', 'ws')}/graphql/ws`,
    secret: null,
  }
}

// Create authentication link for Tauri secret
const authLink = setContext((_, { headers }) => {
  const { secret } = getGraphQLEndpoints()

  // Add secret header if available (Tauri mode)
  if (secret) {
    return {
      headers: {
        ...headers,
        'x-tauri-secret': secret,
      },
    }
  }

  return { headers }
})

// HTTP Link for queries and mutations with timeout using AbortController
// Use a function to get the current endpoint (supports dynamic reconfiguration)
const httpLink = createHttpLink({
  uri: () => getGraphQLEndpoints().httpUrl,
  credentials: 'omit',
  fetch: (uri, options) => {
    const controller = new AbortController()
    const timeout = setTimeout(() => {
      controller.abort()
    }, 30000) // 30 second timeout

    return fetch(uri, {
      ...options,
      signal: controller.signal,
    }).finally(() => {
      clearTimeout(timeout)
    })
  },
})

// WebSocket Link for subscriptions (real-time collaboration)
// Use lazy initialization to support dynamic endpoints
let wsClient: ReturnType<typeof createClient> | null = null

function getOrCreateWsClient() {
  if (!wsClient) {
    const { wsUrl: currentWsUrl } = getGraphQLEndpoints()

    wsClient = createClient({
      url: currentWsUrl,
      connectionParams: () => {
        const { secret } = getGraphQLEndpoints()
        // Include secret in WebSocket connection params if available (Tauri mode)
        if (secret) {
          return { 'x-tauri-secret': secret }
        }
        return {}
      },
      shouldRetry: () => {
        // Retry connection on network errors
        return true
      },
      on: {
        connected: () => console.log('[GraphQL WebSocket] Connected to', currentWsUrl),
        connecting: () => console.log('[GraphQL WebSocket] Connecting to', currentWsUrl),
        closed: (event) => console.log('[GraphQL WebSocket] Closed', event),
        error: (error) => console.error('[GraphQL WebSocket] Error', error),
      },
    })
  }
  return wsClient
}

const wsLink = new GraphQLWsLink(getOrCreateWsClient())

// Error handling link
const errorLink = onError((errorResponse) => {
  const graphQLErrors = (errorResponse as any).graphQLErrors
  const networkError = (errorResponse as any).networkError
  if (graphQLErrors) {
    graphQLErrors.forEach((error: any) => {
      console.error(
        `[GraphQL error]: Message: ${error.message}, Location: ${error.locations}, Path: ${error.path}`
      )
    })
  }

  if (networkError) {
    console.error(`[Network error]: ${networkError}`)

    // Handle authentication errors (disabled in development)
    // if ('statusCode' in networkError && (networkError as any).statusCode === 401) {
    //   localStorage.removeItem('auth_token')
    // }
  }
})

// Simple retry logic - could be enhanced with a dedicated retry link later
// For now, we'll handle retries in the error link

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

// Apollo Client with enhanced cache configuration for real-time collaboration
export const apolloClient = new ApolloClient({
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