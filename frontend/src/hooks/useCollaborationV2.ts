import { useCallback, useRef, useState, useMemo } from 'react';
import { useMutation } from '@apollo/client/react';
import {
  JOIN_PROJECT_COLLABORATION,
  LEAVE_PROJECT_COLLABORATION,
} from '../graphql/plan-dag';

import { useWebSocketCollaboration } from './useWebSocketCollaboration';
import {
  ConnectionState,
  UserPresenceData,
  DocumentActivityData,
  DocumentType
} from '../types/websocket';

interface UseCollaborationV2Options {
  projectId: number;
  documentId?: string;
  documentType?: DocumentType;
  enableWebSocket?: boolean;
  userInfo?: {
    id: string;
    name: string;
    avatarColor: string;
  };
}

export const useCollaborationV2 = (options: UseCollaborationV2Options) => {
  const {
    projectId,
    documentId = 'plan-dag-canvas',
    documentType = 'canvas',
    enableWebSocket = true,
    userInfo
  } = options;

  // WebSocket collaboration
  const webSocket = useWebSocketCollaboration({
    projectId,
    enabled: enableWebSocket
  });

  // GraphQL fallback mutations (cursor position mutation removed - using WebSocket only)
  const [joinCollaborationGraphQL] = useMutation(JOIN_PROJECT_COLLABORATION);
  const [leaveCollaborationGraphQL] = useMutation(LEAVE_PROJECT_COLLABORATION);

  // Local state for fallback mode
  const [isJoined, setIsJoined] = useState(false);

  // Determine if we should use WebSocket or GraphQL fallback
  const useWebSocketMode = useMemo(() => {
    return enableWebSocket &&
           webSocket.connectionState === ConnectionState.CONNECTED;
  }, [enableWebSocket, webSocket.connectionState]);

  // Auto-join session when WebSocket connects and user info is available
  const hasAutoJoinedRef = useRef(false);
  if (useWebSocketMode && userInfo && !hasAutoJoinedRef.current) {
    webSocket.joinSession({
      userId: userInfo.id,
      userName: userInfo.name,
      avatarColor: userInfo.avatarColor,
      documentId
    });
    hasAutoJoinedRef.current = true;
  }

  // Reset auto-join flag when disconnected
  if (webSocket.connectionState === ConnectionState.DISCONNECTED) {
    hasAutoJoinedRef.current = false;
  }

  // Broadcast cursor position
  const broadcastCursorPosition = useCallback((positionX: number, positionY: number, selectedNodeId?: string) => {
    // Validate inputs
    if (typeof positionX !== 'number' || typeof positionY !== 'number' ||
        isNaN(positionX) || isNaN(positionY) ||
        !isFinite(positionX) || !isFinite(positionY)) {
      console.warn('Invalid cursor position values:', { positionX, positionY });
      return;
    }

    if (useWebSocketMode) {
      // Use WebSocket
      const position = documentType === 'canvas'
        ? { type: 'canvas' as const, x: positionX, y: positionY }
        : documentType === 'spreadsheet'
        ? { type: 'spreadsheet' as const, row: Math.floor(positionY), column: Math.floor(positionX) }
        : documentType === '3d'
        ? { type: '3d' as const, x: positionX, y: positionY, z: 0 }
        : documentType === 'timeline'
        ? { type: 'timeline' as const, timestamp: Math.floor(positionX) }
        : { type: 'code_editor' as const, line: Math.floor(positionY), column: Math.floor(positionX) };

      webSocket.updateCursorPosition({
        documentId,
        documentType,
        position,
        selectedNodeId
      });
    } else {
      // WebSocket not available - cursor position updates disabled
      // NOTE: GraphQL cursor position mutations have been removed as part of WebSocket migration
      console.debug('Cursor position update skipped - WebSocket not connected and GraphQL fallback removed');
    }
  }, [
    useWebSocketMode,
    webSocket,
    documentId,
    documentType,
    projectId
  ]);

  // Join project collaboration
  const joinProject = useCallback(async () => {
    if (useWebSocketMode && userInfo) {
      // Already auto-joined when WebSocket connected
      webSocket.joinSession({
        userId: userInfo.id,
        userName: userInfo.name,
        avatarColor: userInfo.avatarColor,
        documentId
      });
      return Promise.resolve();
    } else {
      // Fallback to GraphQL
      try {
        const result = await joinCollaborationGraphQL({
          variables: { projectId }
        });
        setIsJoined(true);
        return result;
      } catch (error) {
        console.error('Failed to join collaboration via GraphQL:', error);
        throw error;
      }
    }
  }, [
    useWebSocketMode,
    userInfo,
    webSocket,
    documentId,
    joinCollaborationGraphQL,
    projectId
  ]);

  // Leave project collaboration
  const leaveProject = useCallback(async () => {
    if (useWebSocketMode) {
      webSocket.leaveSession(documentId);
      return Promise.resolve();
    } else {
      // Fallback to GraphQL
      try {
        const result = await leaveCollaborationGraphQL({
          variables: { projectId }
        });
        setIsJoined(false);
        return result;
      } catch (error) {
        console.error('Failed to leave collaboration via GraphQL:', error);
        throw error;
      }
    }
  }, [
    useWebSocketMode,
    webSocket,
    documentId,
    leaveCollaborationGraphQL,
    projectId
  ]);

  // Switch to different document
  const switchDocument = useCallback((newDocumentId: string, newDocumentType: DocumentType) => {
    if (useWebSocketMode) {
      webSocket.switchDocument({
        documentId: newDocumentId,
        documentType: newDocumentType
      });
    }
    // Note: GraphQL mode doesn't support multi-document collaboration
  }, [useWebSocketMode, webSocket]);

  // Get collaboration status
  const getCollaborationStatus = () => {
    if (useWebSocketMode) {
      return {
        mode: 'websocket' as const,
        connected: webSocket.isConnected,
        connectionState: webSocket.connectionState,
        users: webSocket.users,
        currentDocument: webSocket.currentDocument,
        error: webSocket.error
      };
    } else {
      return {
        mode: 'graphql' as const,
        connected: isJoined,
        connectionState: isJoined ? ConnectionState.CONNECTED : ConnectionState.DISCONNECTED,
        users: [] as UserPresenceData[],
        currentDocument: undefined as DocumentActivityData | undefined,
        error: undefined
      };
    }
  };

  return {
    // Core actions
    broadcastCursorPosition,
    joinProject,
    leaveProject,
    switchDocument,

    // Status
    ...getCollaborationStatus(),

    // WebSocket specific actions
    reconnect: webSocket.reconnect,

    // Compatibility with existing useCollaboration hook
    // (keeping the same interface for backwards compatibility)
    isWebSocketMode: useWebSocketMode,
  };
};