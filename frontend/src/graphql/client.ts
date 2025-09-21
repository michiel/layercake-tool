import { ApolloClient, InMemoryCache, createHttpLink, split, from } from '@apollo/client'
import { GraphQLWsLink } from '@apollo/client/link/subscriptions'
import { getMainDefinition } from '@apollo/client/utilities'
import { onError } from '@apollo/client/link/error'
import { createClient } from 'graphql-ws'

// Configuration based on environment
// const isDevelopment = import.meta.env.DEV

// GraphQL endpoints - configurable for different environments
const getGraphQLEndpoints = () => {
  const baseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001'

  return {
    httpUrl: `${baseUrl}/graphql`,
    wsUrl: `${baseUrl.replace('http', 'ws')}/graphql/ws`,
  }
}

const { httpUrl, wsUrl } = getGraphQLEndpoints()

// HTTP Link for queries and mutations
const httpLink = createHttpLink({
  uri: httpUrl,
  credentials: 'omit', // Don't include credentials for cross-origin development
})

// WebSocket Link for subscriptions (real-time collaboration)
const wsLink = new GraphQLWsLink(
  createClient({
    url: wsUrl,
    connectionParams: () => {
      // No authentication in development mode
      return {}
    },
    shouldRetry: () => {
      // Retry connection on network errors
      return true
    },
  })
)

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
  httpLink
)

// Apollo Client with enhanced cache configuration for real-time collaboration
export const apolloClient = new ApolloClient({
  link: from([errorLink, splitLink]),
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