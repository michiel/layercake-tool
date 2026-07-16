import { ApolloClient, InMemoryCache, split, from } from '@apollo/client'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { onError } from '@apollo/client/link/error'
import { setContext } from '@apollo/client/link/context'
import { createClient } from 'graphql-ws'
import UploadHttpLink from 'apollo-upload-client/UploadHttpLink.mjs'
import { getOrCreateSessionId } from '../utils/session'
import { extractGraphQLErrorMessage } from '../utils/errorHandling'
import { showErrorNotification } from '../utils/notifications'

// Apollo Client 4.2+ requires any non-default `errorPolicy` used in
// `defaultOptions` to be declared here for type safety. We default to
// `errorPolicy: 'all'` (below) so partial data still renders when a request
// also returns errors — declare that on each operation kind.
declare module '@apollo/client' {
  namespace ApolloClient {
    namespace DeclareDefaultOptions {
      interface WatchQuery {
        errorPolicy: 'all'
      }
      interface Query {
        errorPolicy: 'all'
      }
      interface Mutate {
        errorPolicy: 'all'
      }
    }
  }
}

export type GraphQLEndpointOverride = {
  httpPath?: string;
  wsPath?: string;
};

/**
 * Resolve the API base URL.
 *
 * When the SPA is served by the layercake-server binary it is same-origin
 * with the API, so endpoints are relative to `window.location`. The
 * `VITE_API_BASE_URL` env var overrides this for local development, where the
 * Vite dev server runs on a different port than the Rust server.
 */
const getApiBaseUrl = (): string => {
  const override = import.meta.env.VITE_API_BASE_URL
  if (override) {
    return override.replace(/\/+$/, '')
  }
  // Same-origin: the server hosting this page also serves the API.
  return typeof window !== 'undefined' ? window.location.origin : ''
}

// GraphQL endpoints - relative to the serving origin (or VITE_API_BASE_URL in dev)
export const getGraphQLEndpoints = (override?: GraphQLEndpointOverride) => {
  const httpPath = override?.httpPath || '/graphql';
  const wsPath = override?.wsPath || '/graphql/ws';

  const baseUrl = getApiBaseUrl()
  const wsBase = baseUrl.replace(/^http/, 'ws')
  return {
    httpUrl: `${baseUrl}${httpPath}`,
    wsUrl: `${wsBase}${wsPath}`,
  }
}

// Lazily-created singleton Apollo Client (see the exported proxy below).
let apolloClientInstance: ApolloClient | null = null

function createApolloClient(override?: GraphQLEndpointOverride): ApolloClient {
  console.log('[GraphQL] Creating Apollo Client with endpoints:', getGraphQLEndpoints(override))

  // Attach the session id to every request.
  const authLink = setContext((_, { headers }) => {
    const sessionId = getOrCreateSessionId()
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
      const { httpUrl } = getGraphQLEndpoints(override)
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
  const { wsUrl: currentWsUrl } = getGraphQLEndpoints(override)
  console.log('[GraphQL WebSocket] Creating client with URL:', currentWsUrl)

  console.log('[GraphQL WebSocket] Creating WebSocket client for:', currentWsUrl)

  const wsClient = createClient({
    url: currentWsUrl,
    connectionParams: () => {
      const sessionId = getOrCreateSessionId()
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
// This ensures the client is only created when first accessed.
export const apolloClient = new Proxy({} as ApolloClient, {
  get(_target, prop, receiver) {
    if (!apolloClientInstance) {
      console.log('[GraphQL] Creating Apollo Client (lazy initialization)')
      apolloClientInstance = createApolloClient()
    }
    return Reflect.get(apolloClientInstance, prop, receiver)
  },
}) as ApolloClient

// Create an Apollo client for an alternate endpoint (e.g., projections)
export const createApolloClientForEndpoint = (
  override?: GraphQLEndpointOverride
): ApolloClient => createApolloClient(override)

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
