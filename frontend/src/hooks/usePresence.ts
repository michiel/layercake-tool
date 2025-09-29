import { useState, useEffect, useRef, useCallback } from 'react'
import { PresenceService } from '../services/PresenceService'
import { UserPresenceData, ConnectionState } from '../types/websocket'

interface UsePresenceOptions {
  projectId: number
  currentUser: UserPresenceData
  serverUrl?: string
  autoConnect?: boolean
}

interface UsePresenceResult {
  // Connection state
  connectionState: ConnectionState
  isConnected: boolean
  error: Error | null

  // Presence data
  users: UserPresenceData[]
  onlineCount: number

  // Actions
  connect: () => Promise<void>
  disconnect: () => void
  broadcastCursor: (x: number, y: number, selectedNodeId?: string) => void
  broadcastStatus: (status: 'active' | 'idle' | 'away') => void
  broadcastViewport: (viewport: { x: number, y: number, zoom: number }) => void
}

/**
 * Hook for WebSocket-based presence functionality
 * Handles only ephemeral presence data - completely separate from GraphQL
 */
export const usePresence = (options: UsePresenceOptions): UsePresenceResult => {
  const {
    projectId,
    currentUser,
    serverUrl = process.env.VITE_SERVER_URL || 'ws://localhost:3001',
    autoConnect = true
  } = options

  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected')
  const [users, setUsers] = useState<UserPresenceData[]>([])
  const [error, setError] = useState<Error | null>(null)

  const presenceServiceRef = useRef<PresenceService | null>(null)

  // Initialize presence service
  useEffect(() => {
    if (!presenceServiceRef.current) {
      console.log('[usePresence] Initializing PresenceService for project:', projectId)

      presenceServiceRef.current = new PresenceService(
        serverUrl,
        projectId,
        currentUser
      )

      // Set up event listeners
      const unsubscribers = [
        presenceServiceRef.current.onConnectionChange(setConnectionState),
        presenceServiceRef.current.onPresenceUpdate(setUsers),
        presenceServiceRef.current.onError(setError),
        presenceServiceRef.current.onUserJoin((user) => {
          console.log('[usePresence] User joined:', user.id)
          setUsers(prev => {
            const exists = prev.find(u => u.id === user.id)
            if (exists) return prev
            return [...prev, user]
          })
        }),
        presenceServiceRef.current.onUserLeave((userId) => {
          console.log('[usePresence] User left:', userId)
          setUsers(prev => prev.filter(u => u.id !== userId))
        }),
        presenceServiceRef.current.onCursorMove((userId, x, y) => {
          setUsers(prev => prev.map(user =>
            user.id === userId
              ? { ...user, cursorPosition: { x, y }, lastActivity: Date.now() }
              : user
          ))
        })
      ]

      // Cleanup function
      return () => {
        console.log('[usePresence] Cleaning up PresenceService')
        unsubscribers.forEach(unsub => unsub())
        presenceServiceRef.current?.disconnect()
        presenceServiceRef.current = null
      }
    }
  }, [projectId, serverUrl, currentUser.id])

  // Auto-connect if enabled
  useEffect(() => {
    if (autoConnect && presenceServiceRef.current && connectionState === 'disconnected') {
      connect().catch(error => {
        console.error('[usePresence] Auto-connect failed:', error)
      })
    }
  }, [autoConnect, connectionState])

  // Actions
  const connect = useCallback(async (): Promise<void> => {
    if (!presenceServiceRef.current) {
      throw new Error('PresenceService not initialized')
    }

    try {
      setError(null)
      await presenceServiceRef.current.connect()
    } catch (error) {
      const presenceError = error instanceof Error ? error : new Error('Connection failed')
      setError(presenceError)
      throw presenceError
    }
  }, [])

  const disconnect = useCallback((): void => {
    presenceServiceRef.current?.disconnect()
    setUsers([])
    setError(null)
  }, [])

  const broadcastCursor = useCallback((x: number, y: number, selectedNodeId?: string): void => {
    presenceServiceRef.current?.broadcastCursorPosition(x, y, selectedNodeId)
  }, [])

  const broadcastStatus = useCallback((status: 'active' | 'idle' | 'away'): void => {
    presenceServiceRef.current?.broadcastUserStatus(status)
  }, [])

  const broadcastViewport = useCallback((viewport: { x: number, y: number, zoom: number }): void => {
    presenceServiceRef.current?.broadcastViewportChange(viewport)
  }, [])

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      presenceServiceRef.current?.disconnect()
    }
  }, [])

  return {
    // Connection state
    connectionState,
    isConnected: connectionState === 'connected',
    error,

    // Presence data
    users,
    onlineCount: users.length,

    // Actions
    connect,
    disconnect,
    broadcastCursor,
    broadcastStatus,
    broadcastViewport
  }
}