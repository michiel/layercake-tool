import { UserPresenceData, ConnectionState } from '../types/websocket'

/**
 * Pure WebSocket-based presence service
 * Handles only ephemeral presence data (cursors, user status, etc.)
 * Completely separate from GraphQL data persistence
 */
export class PresenceService {
  private websocket: WebSocket | null = null
  private connectionState: ConnectionState = ConnectionState.DISCONNECTED
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000
  private heartbeatInterval: number | null = null

  private listeners = {
    connectionChange: new Set<(state: ConnectionState) => void>(),
    presenceUpdate: new Set<(users: UserPresenceData[]) => void>(),
    cursorMove: new Set<(userId: string, x: number, y: number) => void>(),
    userJoin: new Set<(user: UserPresenceData) => void>(),
    userLeave: new Set<(userId: string) => void>(),
    error: new Set<(error: Error) => void>()
  }

  constructor(
    private serverUrl: string,
    private projectId: number,
    private currentUser: UserPresenceData
  ) {}

  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        const wsUrl = `${this.serverUrl}/ws/presence/${this.projectId}`
        console.log('[PresenceService] Connecting to:', wsUrl)

        this.websocket = new WebSocket(wsUrl)

        this.websocket.onopen = () => {
          console.log('[PresenceService] Connected')
          this.connectionState = 'connected'
          this.reconnectAttempts = 0
          this.startHeartbeat()
          this.emit('connectionChange', this.connectionState)

          // Send initial presence data
          this.sendPresenceUpdate(this.currentUser)
          resolve()
        }

        this.websocket.onclose = (event) => {
          console.log('[PresenceService] Disconnected:', event.code, event.reason)
          this.connectionState = 'disconnected'
          this.stopHeartbeat()
          this.emit('connectionChange', this.connectionState)

          // Auto-reconnect logic
          if (this.reconnectAttempts < this.maxReconnectAttempts && !event.wasClean) {
            this.scheduleReconnect()
          }
        }

        this.websocket.onerror = (error) => {
          console.error('[PresenceService] WebSocket error:', error)
          this.emit('error', new Error('WebSocket connection failed'))
          reject(error)
        }

        this.websocket.onmessage = (event) => {
          this.handleMessage(event.data)
        }

      } catch (error) {
        reject(error)
      }
    })
  }

  disconnect(): void {
    if (this.websocket) {
      this.websocket.close(1000, 'Client disconnect')
      this.websocket = null
    }
    this.stopHeartbeat()
    this.connectionState = 'disconnected'
    this.emit('connectionChange', this.connectionState)
  }

  broadcastCursorPosition(x: number, y: number, selectedNodeId?: string): void {
    if (!this.isConnected()) return

    this.send({
      type: 'cursor_move',
      data: {
        userId: this.currentUser.id,
        x,
        y,
        selectedNodeId,
        timestamp: Date.now()
      }
    })
  }

  broadcastUserStatus(status: 'active' | 'idle' | 'away'): void {
    if (!this.isConnected()) return

    this.send({
      type: 'user_status',
      data: {
        userId: this.currentUser.id,
        status,
        timestamp: Date.now()
      }
    })
  }

  broadcastViewportChange(viewport: { x: number, y: number, zoom: number }): void {
    if (!this.isConnected()) return

    this.send({
      type: 'viewport_change',
      data: {
        userId: this.currentUser.id,
        viewport,
        timestamp: Date.now()
      }
    })
  }

  // Event subscription methods
  onConnectionChange(listener: (state: ConnectionState) => void): () => void {
    this.listeners.connectionChange.add(listener)
    return () => this.listeners.connectionChange.delete(listener)
  }

  onPresenceUpdate(listener: (users: UserPresenceData[]) => void): () => void {
    this.listeners.presenceUpdate.add(listener)
    return () => this.listeners.presenceUpdate.delete(listener)
  }

  onCursorMove(listener: (userId: string, x: number, y: number) => void): () => void {
    this.listeners.cursorMove.add(listener)
    return () => this.listeners.cursorMove.delete(listener)
  }

  onUserJoin(listener: (user: UserPresenceData) => void): () => void {
    this.listeners.userJoin.add(listener)
    return () => this.listeners.userJoin.delete(listener)
  }

  onUserLeave(listener: (userId: string) => void): () => void {
    this.listeners.userLeave.add(listener)
    return () => this.listeners.userLeave.delete(listener)
  }

  onError(listener: (error: Error) => void): () => void {
    this.listeners.error.add(listener)
    return () => this.listeners.error.delete(listener)
  }

  // Getters
  getConnectionState(): ConnectionState {
    return this.connectionState
  }

  isConnected(): boolean {
    return this.connectionState === 'connected' && this.websocket?.readyState === WebSocket.OPEN
  }

  // Private methods
  private send(message: any): void {
    if (this.isConnected() && this.websocket) {
      this.websocket.send(JSON.stringify(message))
    }
  }

  private sendPresenceUpdate(user: UserPresenceData): void {
    this.send({
      type: 'presence_update',
      data: {
        user,
        timestamp: Date.now()
      }
    })
  }

  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data)

      switch (message.type) {
        case 'presence_update':
          this.emit('presenceUpdate', message.data.users)
          break

        case 'cursor_move':
          this.emit('cursorMove', message.data.userId, message.data.x, message.data.y)
          break

        case 'user_join':
          this.emit('userJoin', message.data.user)
          break

        case 'user_leave':
          this.emit('userLeave', message.data.userId)
          break

        case 'pong':
          // Heartbeat response - connection is alive
          break

        default:
          console.warn('[PresenceService] Unknown message type:', message.type)
      }
    } catch (error) {
      console.error('[PresenceService] Failed to parse message:', error)
    }
  }

  private emit<K extends keyof typeof this.listeners>(
    event: K,
    ...args: any[]
  ): void {
    this.listeners[event].forEach((listener: any) => {
      try {
        listener(...args)
      } catch (error) {
        console.error(`[PresenceService] Error in ${event} listener:`, error)
      }
    })
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = window.setInterval(() => {
      if (this.isConnected()) {
        this.send({ type: 'ping' })
      }
    }, 30000) // 30 second heartbeat
  }

  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval)
      this.heartbeatInterval = null
    }
  }

  private scheduleReconnect(): void {
    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)

    console.log(`[PresenceService] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`)

    setTimeout(() => {
      if (this.connectionState === 'disconnected') {
        this.connectionState = 'connecting'
        this.emit('connectionChange', this.connectionState)
        this.connect().catch(error => {
          console.error('[PresenceService] Reconnection failed:', error)
        })
      }
    }, delay)
  }
}