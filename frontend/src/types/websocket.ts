// WebSocket types for collaboration

export type DocumentType = 'canvas' | 'spreadsheet' | '3d' | 'timeline' | 'code_editor';

export interface CanvasPosition {
  type: 'canvas';
  x: number;
  y: number;
  zoom?: number;
}

export interface SpreadsheetPosition {
  type: 'spreadsheet';
  row: number;
  column: number;
  sheet?: string;
}

export interface ThreeDPosition {
  type: '3d';
  x: number;
  y: number;
  z: number;
  rotation?: { x: number; y: number; z: number };
  scale?: number;
  viewport?: string;
}

export interface TimelinePosition {
  type: 'timeline';
  timestamp: number;
  track?: number;
}

export interface CodeEditorPosition {
  type: 'code_editor';
  line: number;
  column: number;
  file?: string;
}

export type CursorPosition = CanvasPosition | SpreadsheetPosition | ThreeDPosition | TimelinePosition | CodeEditorPosition;

// Client → Server messages
export interface JoinSessionData {
  userId: string;
  userName: string;
  avatarColor: string;
  documentId?: string;
}

export interface CursorUpdateData {
  documentId: string;
  documentType: DocumentType;
  position: CursorPosition;
  selectedNodeId?: string;
  timestamp: number;
}

export interface DocumentSwitchData {
  documentId: string;
  documentType: DocumentType;
}

export interface LeaveSessionData {
  documentId?: string;
}

export type ClientMessage =
  | { type: 'join_session'; data: JoinSessionData }
  | { type: 'cursor_update'; data: CursorUpdateData }
  | { type: 'switch_document'; data: DocumentSwitchData }
  | { type: 'leave_session'; data: LeaveSessionData }
  | { type: 'ping' };

// Server → Client messages
export interface DocumentPresence {
  documentType: DocumentType;
  position?: CursorPosition;
  selectedNodeId?: string;
  lastActiveInDocument: string;
}

export interface UserPresenceData {
  userId: string;
  userName: string;
  avatarColor: string;
  isOnline: boolean;
  lastActive: string;
  documents: Record<string, DocumentPresence>;
}

export interface DocumentUser {
  userId: string;
  userName: string;
  position?: CursorPosition;
  selectedNodeId?: string;
}

export interface DocumentActivityData {
  documentId: string;
  activeUsers: DocumentUser[];
}

export type ServerMessage =
  | { type: 'user_presence'; data: UserPresenceData }
  | { type: 'bulk_presence'; data: UserPresenceData[] }
  | { type: 'document_activity'; data: DocumentActivityData }
  | { type: 'error'; message: string }
  | { type: 'pong' };

// WebSocket connection states
export enum ConnectionState {
  CONNECTING = 'connecting',
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  RECONNECTING = 'reconnecting',
  ERROR = 'error'
}

// Configuration
export interface WebSocketConfig {
  url: string;
  projectId: number;
  token?: string;
  maxReconnectAttempts?: number;
  reconnectInterval?: number;
  heartbeatInterval?: number;
  messageQueueSize?: number;
}

// Hook return types
export interface UseWebSocketCollaboration {
  connectionState: ConnectionState;
  isConnected: boolean;
  users: UserPresenceData[];
  currentDocument?: DocumentActivityData;
  error?: string;

  // Actions
  joinSession: (userData: JoinSessionData) => void;
  leaveSession: (documentId?: string) => void;
  updateCursorPosition: (data: Omit<CursorUpdateData, 'timestamp'>) => void;
  switchDocument: (data: DocumentSwitchData) => void;
  reconnect: () => void;
}

// Message queue item
export interface QueuedMessage {
  message: ClientMessage;
  timestamp: number;
  retries: number;
}