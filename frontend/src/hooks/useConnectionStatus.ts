import { useQuery } from '@apollo/client/react'
import { useMemo } from 'react'
import { gql } from '@apollo/client'
import { ConnectionState } from '../types/websocket'

// Lightweight health check query (reuse from App.tsx)
const BACKEND_HEALTH_CHECK = gql`
  query HealthCheck {
    projects {
      id
      name
    }
  }
`

interface UseConnectionStatusProps {
  websocketConnectionState?: ConnectionState
  enableWebSocket?: boolean
}

export const useConnectionStatus = (props: UseConnectionStatusProps = {}) => {
  const { websocketConnectionState, enableWebSocket = false } = props

  // Check GraphQL backend connectivity - use same settings as HomePage
  const { loading, error, data } = useQuery(BACKEND_HEALTH_CHECK, {
    errorPolicy: 'all',
    notifyOnNetworkStatusChange: true,
    // Don't use aggressive polling, let Apollo Client handle caching
    fetchPolicy: 'cache-first'
  })

  // Determine overall connection status
  const connectionStatus = useMemo(() => {

    // If GraphQL has a hard error and no cached data, backend is disconnected
    if (error && !data) {
      return {
        state: ConnectionState.DISCONNECTED,
        isBackendConnected: false,
        isWebSocketConnected: false,
        description: 'Backend disconnected'
      }
    }

    // If we're still loading for the first time, show connecting
    if (loading && !data) {
      return {
        state: ConnectionState.CONNECTING,
        isBackendConnected: false,
        isWebSocketConnected: false,
        description: 'Connecting to backend...'
      }
    }

    // If we have data or no error, assume backend is connected
    const isBackendConnected = !!data || !error
    const isWebSocketConnected = enableWebSocket && websocketConnectionState === ConnectionState.CONNECTED

    // Default to connected state - if the app is running, backend is likely connected
    let state = ConnectionState.CONNECTED
    let description: string

    if (enableWebSocket) {
      description = isWebSocketConnected
        ? 'Connected (GraphQL + WebSocket)'
        : 'Connected (GraphQL only)'
    } else {
      description = 'Connected (GraphQL)'
    }

    return {
      state,
      isBackendConnected,
      isWebSocketConnected,
      description
    }
  }, [loading, error, data, websocketConnectionState, enableWebSocket])

  return {
    ...connectionStatus,
    // Convenient boolean getters
    isConnected: connectionStatus.state === ConnectionState.CONNECTED,
    isConnecting: connectionStatus.state === ConnectionState.CONNECTING,
    isDisconnected: connectionStatus.state === ConnectionState.DISCONNECTED,
    // GraphQL specific status
    graphqlLoading: loading,
    graphqlError: error
  }
}