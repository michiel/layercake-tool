import { useState, useEffect, useRef, useCallback } from 'react';
import {
  ConnectionState,
  UserPresenceData,
  DocumentActivityData,
  WebSocketConfig,
  UseWebSocketCollaboration,
  JoinSessionData,
  CursorUpdateData,
  DocumentSwitchData
} from '../types/websocket';
import { WebSocketCollaborationService } from '../services/websocket/WebSocketCollaborationService';

interface UseWebSocketCollaborationOptions {
  projectId: number;
  enabled?: boolean;
  serverUrl?: string;
  token?: string;
  maxReconnectAttempts?: number;
  reconnectInterval?: number;
}

export function useWebSocketCollaboration(
  options: UseWebSocketCollaborationOptions
): UseWebSocketCollaboration {
  const { projectId, enabled = true, serverUrl, token } = options;

  const [connectionState, setConnectionState] = useState<ConnectionState>(ConnectionState.DISCONNECTED);
  const [users, setUsers] = useState<UserPresenceData[]>([]);
  const [currentDocument, setCurrentDocument] = useState<DocumentActivityData | undefined>();
  const [error, setError] = useState<string | undefined>();

  const serviceRef = useRef<WebSocketCollaborationService | null>(null);
  const isInitializedRef = useRef(false);
  const effectRunCountRef = useRef(0);
  const isCleanedUpRef = useRef(false);

  // Get server URL from environment or use provided
  const getServerUrl = useCallback(() => {
    if (serverUrl) return serverUrl;

    // Try to get from environment variables
    const envUrl = import.meta.env.VITE_SERVER_URL ||
                   import.meta.env.VITE_API_URL ||
                   'http://localhost:3000';

    return envUrl;
  }, [serverUrl]);

  // Initialize service
  useEffect(() => {
    if (!enabled) {
      console.log('[useWebSocketCollaboration] Disabled, skipping initialization');
      return;
    }

    // Track effect invocations
    effectRunCountRef.current += 1;
    const currentRun = effectRunCountRef.current;

    console.log(`[useWebSocketCollaboration] Effect run #${currentRun}`);

    // Only initialize on first run, skip StrictMode double-invocation
    if (currentRun > 1 && !isCleanedUpRef.current) {
      console.log('[useWebSocketCollaboration] Skipping re-initialization (StrictMode double-invocation)');
      return;
    }

    // Check if already initialized
    if (serviceRef.current) {
      console.log('[useWebSocketCollaboration] Service already exists, skipping re-initialization');
      return;
    }

    console.log('[useWebSocketCollaboration] Initializing WebSocket service for project:', projectId);
    isCleanedUpRef.current = false;

    const config: WebSocketConfig = {
      url: getServerUrl(),
      projectId,
      token,
      maxReconnectAttempts: options.maxReconnectAttempts,
      reconnectInterval: options.reconnectInterval
    };

    const service = new WebSocketCollaborationService(config);

    // Set up event handlers
    service.setOnConnectionStateChange((state) => {
      setConnectionState(state);

      // Clear error when successfully connected
      if (state === ConnectionState.CONNECTED) {
        setError(undefined);
      }
    });

    service.setOnUserPresence((data) => {
      setUsers(prevUsers => {
        const userIndex = prevUsers.findIndex(u => u.userId === data.userId);
        if (userIndex >= 0) {
          const newUsers = [...prevUsers];
          newUsers[userIndex] = data;
          return newUsers;
        } else {
          return [...prevUsers, data];
        }
      });
    });

    service.setOnBulkPresence((data) => {
      setUsers(data);
    });

    service.setOnDocumentActivity((data) => {
      setCurrentDocument(data);
    });

    service.setOnError((errorMessage) => {
      setError(errorMessage);
    });

    serviceRef.current = service;
    isInitializedRef.current = true;

    // Start connection
    service.connect();

    // Cleanup on unmount
    return () => {
      console.log(`[useWebSocketCollaboration] Cleanup for run #${currentRun}`);
      isCleanedUpRef.current = true;

      if (serviceRef.current) {
        serviceRef.current.destroy();
        serviceRef.current = null;
      }
      isInitializedRef.current = false;
    };
  }, [enabled, projectId, getServerUrl, token, options.maxReconnectAttempts, options.reconnectInterval]);

  // Actions
  const joinSession = useCallback((userData: JoinSessionData) => {
    if (serviceRef.current) {
      serviceRef.current.joinSession(userData);
    }
  }, []);

  const leaveSession = useCallback((documentId?: string) => {
    if (serviceRef.current) {
      serviceRef.current.leaveSession(documentId);
    }
  }, []);

  const updateCursorPosition = useCallback((data: Omit<CursorUpdateData, 'timestamp'>) => {
    if (serviceRef.current && serviceRef.current.isConnected()) {
      serviceRef.current.updateCursorPosition(data);
    }
  }, []);

  const switchDocument = useCallback((data: DocumentSwitchData) => {
    if (serviceRef.current) {
      serviceRef.current.switchDocument(data);
    }
  }, []);

  const reconnect = useCallback(() => {
    if (serviceRef.current) {
      serviceRef.current.reconnect();
    }
  }, []);

  // Enhanced cursor update throttling with position diffing
  const throttledCursorUpdateRef = useRef<{
    timer: number | null;
    lastUpdate: number;
    lastPosition: { x: number; y: number } | null;
    pendingUpdate: Omit<CursorUpdateData, 'timestamp'> | null;
  }>({
    timer: null,
    lastUpdate: 0,
    lastPosition: null,
    pendingUpdate: null
  });

  const throttledUpdateCursorPosition = useCallback((data: Omit<CursorUpdateData, 'timestamp'>) => {
    const throttle = throttledCursorUpdateRef.current;
    const now = Date.now();
    const throttleMs = 250; // Increased from 100ms to 250ms to reduce network load

    // Extract position based on document type
    const currentPosition = data.position.type === 'canvas'
      ? { x: data.position.x, y: data.position.y }
      : data.position.type === 'spreadsheet'
      ? { x: data.position.column, y: data.position.row }
      : data.position.type === '3d'
      ? { x: data.position.x, y: data.position.y }
      : data.position.type === 'timeline'
      ? { x: data.position.timestamp, y: 0 }
      : { x: data.position.column, y: data.position.line };

    // Skip update if position hasn't changed significantly (minimum 10px movement for canvas)
    if (throttle.lastPosition && data.position.type === 'canvas') {
      const deltaX = Math.abs(currentPosition.x - throttle.lastPosition.x);
      const deltaY = Math.abs(currentPosition.y - throttle.lastPosition.y);
      const minMovement = 10; // pixels

      if (deltaX < minMovement && deltaY < minMovement) {
        return; // Skip insignificant movements
      }
    }

    throttle.pendingUpdate = data;

    if (now - throttle.lastUpdate >= throttleMs) {
      updateCursorPosition(data);
      throttle.lastUpdate = now;
      throttle.lastPosition = currentPosition;
      throttle.pendingUpdate = null;
    } else if (!throttle.timer) {
      throttle.timer = setTimeout(() => {
        if (throttle.pendingUpdate) {
          updateCursorPosition(throttle.pendingUpdate);
          throttle.lastUpdate = Date.now();

          // Extract position from pending update
          const pendingPosition = throttle.pendingUpdate.position.type === 'canvas'
            ? { x: throttle.pendingUpdate.position.x, y: throttle.pendingUpdate.position.y }
            : throttle.pendingUpdate.position.type === 'spreadsheet'
            ? { x: throttle.pendingUpdate.position.column, y: throttle.pendingUpdate.position.row }
            : throttle.pendingUpdate.position.type === '3d'
            ? { x: throttle.pendingUpdate.position.x, y: throttle.pendingUpdate.position.y }
            : throttle.pendingUpdate.position.type === 'timeline'
            ? { x: throttle.pendingUpdate.position.timestamp, y: 0 }
            : { x: throttle.pendingUpdate.position.column, y: throttle.pendingUpdate.position.line };

          throttle.lastPosition = pendingPosition;
          throttle.pendingUpdate = null;
        }
        throttle.timer = null;
      }, throttleMs - (now - throttle.lastUpdate));
    }
  }, [updateCursorPosition]);

  return {
    connectionState,
    isConnected: connectionState === ConnectionState.CONNECTED,
    users,
    currentDocument,
    error,
    joinSession,
    leaveSession,
    updateCursorPosition: throttledUpdateCursorPosition,
    switchDocument,
    reconnect
  };
}