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
    if (!enabled || isInitializedRef.current) return;

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
      service.destroy();
      serviceRef.current = null;
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

  // Throttled cursor update to prevent spam
  const throttledCursorUpdateRef = useRef<{
    timer: number | null;
    lastUpdate: number;
    pendingUpdate: Omit<CursorUpdateData, 'timestamp'> | null;
  }>({
    timer: null,
    lastUpdate: 0,
    pendingUpdate: null
  });

  const throttledUpdateCursorPosition = useCallback((data: Omit<CursorUpdateData, 'timestamp'>) => {
    const throttle = throttledCursorUpdateRef.current;
    const now = Date.now();
    const throttleMs = 100; // Update at most every 100ms

    throttle.pendingUpdate = data;

    if (now - throttle.lastUpdate >= throttleMs) {
      updateCursorPosition(data);
      throttle.lastUpdate = now;
      throttle.pendingUpdate = null;
    } else if (!throttle.timer) {
      throttle.timer = setTimeout(() => {
        if (throttle.pendingUpdate) {
          updateCursorPosition(throttle.pendingUpdate);
          throttle.lastUpdate = Date.now();
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