import { useCallback, useRef } from 'react'

/**
 * Generates a unique client ID for this browser session
 */
const generateClientId = (): string => {
  if (typeof window !== 'undefined') {
    let clientId = sessionStorage.getItem('graphql-client-id')
    if (!clientId) {
      clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
      sessionStorage.setItem('graphql-client-id', clientId)
    }
    return clientId
  }
  // Fallback for server-side rendering
  return `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
}

/**
 * Hook to filter out GraphQL subscription updates that originated from this client
 * This prevents clients from reacting to their own mutations via subscriptions
 */
export const useSubscriptionFilter = () => {
  const clientIdRef = useRef<string>(generateClientId())

  const filterSubscriptionData = useCallback((subscriptionData: any) => {
    // Check if this update originated from this client
    const updateClientId = subscriptionData?.data?.mutation?.clientId ||
                          subscriptionData?.data?.clientId ||
                          subscriptionData?.clientId

    if (updateClientId === clientIdRef.current) {
      console.log('[SubscriptionFilter] Filtering out own mutation from subscription:', {
        clientId: clientIdRef.current,
        updateType: subscriptionData?.data?.mutation?.type || 'unknown'
      })
      return null // Filter out own updates
    }

    console.log('[SubscriptionFilter] Processing remote subscription update:', {
      fromClient: updateClientId,
      currentClient: clientIdRef.current,
      updateType: subscriptionData?.data?.mutation?.type || 'unknown'
    })

    return subscriptionData
  }, [])

  const getClientId = useCallback(() => clientIdRef.current, [])

  return {
    clientId: clientIdRef.current,
    filterSubscriptionData,
    getClientId
  }
}

/**
 * Context provider for Apollo Client to include client ID in mutations
 */
export const createMutationContext = (clientId: string) => ({
  clientId,
  timestamp: Date.now()
})

/**
 * Higher-order function to wrap subscription data handlers with filtering
 */
export const withSubscriptionFilter = (
  handler: (data: any) => void,
  clientId: string
) => {
  return (subscriptionData: any) => {
    const updateClientId = subscriptionData?.data?.mutation?.clientId ||
                          subscriptionData?.data?.clientId ||
                          subscriptionData?.clientId

    // Only process updates from other clients
    if (updateClientId && updateClientId !== clientId) {
      handler(subscriptionData)
    }
  }
}