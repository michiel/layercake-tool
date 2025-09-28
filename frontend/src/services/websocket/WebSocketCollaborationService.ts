import {
  WebSocketConfig,
  ConnectionState,
  ClientMessage,
  ServerMessage,
  UserPresenceData,
  DocumentActivityData,
  QueuedMessage,
  JoinSessionData,
  CursorUpdateData,
  DocumentSwitchData
} from '../../types/websocket';

export class WebSocketCollaborationService {
  private ws: WebSocket | null = null;
  private config: WebSocketConfig;
  private connectionState: ConnectionState = ConnectionState.DISCONNECTED;
  private reconnectAttempts = 0;
  private reconnectTimer: number | null = null;
  private heartbeatTimer: number | null = null;
  private messageQueue: QueuedMessage[] = [];
  private isIntentionalDisconnect = false;

  // Event handlers
  private onConnectionStateChange: (state: ConnectionState) => void = () => {};
  private onUserPresence: (data: UserPresenceData) => void = () => {};
  private onBulkPresence: (data: UserPresenceData[]) => void = () => {};
  private onDocumentActivity: (data: DocumentActivityData) => void = () => {};
  private onError: (error: string) => void = () => {};

  constructor(config: WebSocketConfig) {
    this.config = {
      maxReconnectAttempts: 10,
      reconnectInterval: 1000,
      heartbeatInterval: 30000,
      messageQueueSize: 100,
      ...config
    };
  }

  // Event handlers setters
  setOnConnectionStateChange(handler: (state: ConnectionState) => void) {
    this.onConnectionStateChange = handler;
  }

  setOnUserPresence(handler: (data: UserPresenceData) => void) {
    this.onUserPresence = handler;
  }

  setOnBulkPresence(handler: (data: UserPresenceData[]) => void) {
    this.onBulkPresence = handler;
  }

  setOnDocumentActivity(handler: (data: DocumentActivityData) => void) {
    this.onDocumentActivity = handler;
  }

  setOnError(handler: (error: string) => void) {
    this.onError = handler;
  }

  // Connection management
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    this.isIntentionalDisconnect = false;
    this.setConnectionState(ConnectionState.CONNECTING);

    const wsUrl = this.buildWebSocketUrl();

    try {
      this.ws = new WebSocket(wsUrl);
      this.setupWebSocketHandlers();
    } catch (error) {
      this.handleConnectionError('Failed to create WebSocket connection');
    }
  }

  disconnect(): void {
    this.isIntentionalDisconnect = true;
    this.clearReconnectTimer();
    this.clearHeartbeatTimer();

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    this.setConnectionState(ConnectionState.DISCONNECTED);
  }

  reconnect(): void {
    this.disconnect();
    this.reconnectAttempts = 0;
    this.connect();
  }

  // Message sending
  sendMessage(message: ClientMessage): boolean {
    if (this.ws?.readyState === WebSocket.OPEN) {
      try {
        this.ws.send(JSON.stringify(message));
        return true;
      } catch (error) {
        this.queueMessage(message);
        return false;
      }
    } else {
      this.queueMessage(message);
      return false;
    }
  }

  // High-level actions
  joinSession(data: JoinSessionData): void {
    this.sendMessage({ type: 'join_session', data });
  }

  updateCursorPosition(data: Omit<CursorUpdateData, 'timestamp'>): void {
    const message = {
      type: 'cursor_update' as const,
      data: {
        ...data,
        timestamp: Date.now()
      }
    };
    this.sendMessage(message);
  }

  switchDocument(data: DocumentSwitchData): void {
    this.sendMessage({ type: 'switch_document', data });
  }

  leaveSession(documentId?: string): void {
    this.sendMessage({
      type: 'leave_session',
      data: { documentId }
    });
  }

  // Connection state
  getConnectionState(): ConnectionState {
    return this.connectionState;
  }

  isConnected(): boolean {
    return this.connectionState === ConnectionState.CONNECTED;
  }

  // Private methods
  private buildWebSocketUrl(): string {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const baseUrl = this.config.url.replace(/^https?:/, protocol);
    const url = new URL(`${baseUrl}/ws/collaboration`);

    url.searchParams.set('project_id', this.config.projectId.toString());
    if (this.config.token) {
      url.searchParams.set('token', this.config.token);
    }

    return url.toString();
  }

  private setupWebSocketHandlers(): void {
    if (!this.ws) return;

    this.ws.onopen = () => {
      // WebSocket connected successfully
      this.setConnectionState(ConnectionState.CONNECTED);
      this.reconnectAttempts = 0;
      this.startHeartbeat();
      this.flushMessageQueue();
    };

    this.ws.onclose = (event) => {
      this.clearHeartbeatTimer();

      if (!this.isIntentionalDisconnect && event.code !== 1000) {
        this.handleConnectionError('Connection lost');
        this.scheduleReconnect();
      } else {
        this.setConnectionState(ConnectionState.DISCONNECTED);
      }
    };

    this.ws.onerror = () => {
      this.handleConnectionError('WebSocket error occurred');
    };

    this.ws.onmessage = (event) => {
      try {
        const message: ServerMessage = JSON.parse(event.data);
        this.handleServerMessage(message);
      } catch (error) {
        this.onError('Failed to parse WebSocket message');
      }
    };
  }

  private handleServerMessage(message: ServerMessage): void {
    switch (message.type) {
      case 'user_presence':
        this.onUserPresence(message.data);
        break;
      case 'bulk_presence':
        this.onBulkPresence(message.data);
        break;
      case 'document_activity':
        this.onDocumentActivity(message.data);
        break;
      case 'error':
        this.onError(message.message);
        break;
      default:
        this.onError(`Unknown message type: ${(message as any).type}`);
    }
  }

  private setConnectionState(state: ConnectionState): void {
    if (this.connectionState !== state) {
      this.connectionState = state;
      this.onConnectionStateChange(state);
    }
  }

  private handleConnectionError(error: string): void {
    this.setConnectionState(ConnectionState.ERROR);
    this.onError(error);
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= (this.config.maxReconnectAttempts || 10)) {
      this.handleConnectionError('Max reconnection attempts reached');
      return;
    }

    this.setConnectionState(ConnectionState.RECONNECTING);

    const delay = Math.min(
      (this.config.reconnectInterval || 1000) * Math.pow(2, this.reconnectAttempts),
      30000 // Max 30 seconds
    );

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private startHeartbeat(): void {
    this.clearHeartbeatTimer();

    this.heartbeatTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        // Send ping message to keep connection alive
        try {
          this.ws.send(JSON.stringify({ type: 'ping' }));
        } catch (error) {
          // Heartbeat failed - connection will be handled by onclose
        }
      }
    }, this.config.heartbeatInterval || 30000);
  }

  private clearHeartbeatTimer(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  private queueMessage(message: ClientMessage): void {
    const queuedMessage: QueuedMessage = {
      message,
      timestamp: Date.now(),
      retries: 0
    };

    this.messageQueue.push(queuedMessage);

    // Limit queue size
    const maxSize = this.config.messageQueueSize || 100;
    if (this.messageQueue.length > maxSize) {
      this.messageQueue = this.messageQueue.slice(-maxSize);
    }
  }

  private flushMessageQueue(): void {
    const messagesToSend = [...this.messageQueue];
    this.messageQueue = [];

    for (const queuedMessage of messagesToSend) {
      // Skip messages that are too old (>5 minutes)
      if (Date.now() - queuedMessage.timestamp > 300000) {
        continue;
      }

      if (!this.sendMessage(queuedMessage.message)) {
        // If sending fails, re-queue the message
        queuedMessage.retries++;
        if (queuedMessage.retries < 3) {
          this.queueMessage(queuedMessage.message);
        }
      }
    }
  }

  // Cleanup
  destroy(): void {
    this.disconnect();
    this.messageQueue = [];
  }
}