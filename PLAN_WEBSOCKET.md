# WebSocket Implementation Plan for Ephemeral Data

## Problem Statement

Currently, cursor position updates in the Plan DAG editor are sent via GraphQL mutations and stored in the database, which is extremely inefficient and creates noise in the logs. Ephemeral data like cursor positions, user presence indicators, and temporary selection states should:
1. Use WebSocket connections instead of GraphQL mutations
2. Be kept in memory only (not persisted to database)
3. Be automatically cleaned up when users disconnect

## Goals

1. **Eliminate Database Storage**: Keep ephemeral data like cursor positions in memory only
2. **Reduce Network Overhead**: Eliminate GraphQL overhead for frequent, small updates
3. **Improve Performance**: Use lightweight protocols and memory-only storage
4. **Reduce Log Noise**: Move ephemeral data out of GraphQL mutation logs
5. **Enable Real-time Features**: Support for live collaboration features like cursor following
6. **Maintain Compatibility**: Keep existing GraphQL for persistent data operations

## Architecture Overview

### WebSocket Endpoint
- **Route**: `/ws/collaboration/{projectId}`
- **Protocol**: Custom JSON-based protocol over WebSocket
- **Authentication**: JWT token via query parameter or subprotocol
- **Scope**: Project-specific collaboration sessions with document-level tracking
- **Data Storage**: All ephemeral data kept in memory only (DashMap/HashMap)
- **Cleanup**: Automatic removal of user data on disconnect
- **Multi-Document Support**: Track cursor positions per document type (canvas, spreadsheet, VR, etc.)

### Message Types

```typescript
// Document types for different editing contexts
type DocumentType = 'canvas' | 'spreadsheet' | '3d' | 'timeline' | 'code_editor';

// Position data varies by document type
interface CanvasPosition {
  type: 'canvas';
  x: number;
  y: number;
  zoom?: number;
}

interface SpreadsheetPosition {
  type: 'spreadsheet';
  row: number;
  column: number;
  sheet?: string;
}

interface ThreeDPosition {
  type: '3d';
  x: number;
  y: number;
  z: number;
  rotation?: { x: number; y: number; z: number };
  scale?: number;
  viewport?: string;  // Named view/scene/room within 3D space
}

interface TimelinePosition {
  type: 'timeline';
  timestamp: number;
  track?: number;
}

interface CodePosition {
  type: 'code_editor';
  line: number;
  column: number;
  file?: string;
}

type CursorPosition = CanvasPosition | SpreadsheetPosition | ThreeDPosition | TimelinePosition | CodePosition;

// Outbound (Client → Server)
interface CursorUpdateMessage {
  type: 'cursor_update';
  data: {
    documentId: string;           // Specific document within project
    documentType: DocumentType;   // Type of document being edited
    position: CursorPosition;     // Position data specific to document type
    selectedNodeId?: string;      // Selected element in document
    timestamp: number;
  };
}

interface JoinSessionMessage {
  type: 'join_session';
  data: {
    userId: string;
    userName: string;
    avatarColor: string;
    documentId?: string;    // Optional: join specific document
  };
}

interface DocumentSwitchMessage {
  type: 'switch_document';
  data: {
    documentId: string;
    documentType: DocumentType;
  };
}

interface LeaveSessionMessage {
  type: 'leave_session';
  data: {
    documentId?: string;    // Leave specific document or entire project
  };
}

// Inbound (Server → Client)
interface UserPresenceMessage {
  type: 'user_presence';
  data: {
    userId: string;
    userName: string;
    avatarColor: string;
    isOnline: boolean;
    lastActive: string;
    // Document-specific cursor positions
    documents: {
      [documentId: string]: {
        documentType: DocumentType;
        position?: CursorPosition;
        selectedNodeId?: string;
        lastActiveInDocument: string;
      };
    };
  };
}

interface BulkPresenceMessage {
  type: 'bulk_presence';
  data: UserPresenceMessage['data'][];
}

interface DocumentActivityMessage {
  type: 'document_activity';
  data: {
    documentId: string;
    activeUsers: {
      userId: string;
      userName: string;
      position?: CursorPosition;
      selectedNodeId?: string;
    }[];
  };
}
```

## Current State Analysis

### Existing Database Ephemeral Data to Remove
The following database storage of ephemeral data should be eliminated:

1. **GraphQL Mutations**
   - `UPDATE_CURSOR_POSITION` - Currently stores cursor positions in database
   - `JOIN_PROJECT_COLLABORATION` - May store session data in database
   - `LEAVE_PROJECT_COLLABORATION` - May store session data in database

2. **Database Tables/Columns**
   - `user_presence` table (if exists) - Contains cursor positions and online status
   - Any cursor position columns in user or session tables
   - Temporary collaboration state data

3. **Backend Code**
   - GraphQL resolvers that persist ephemeral data
   - Database models for presence/cursor data
   - Any ORM code saving ephemeral state

**Goal**: Replace all of the above with memory-only WebSocket storage

## Data Architecture Concepts

### Two-Level Tracking System

**Project-Level User Presence** (Per-Project):
- User is online/offline in the project
- User's basic information (name, avatar color)
- Last activity timestamp in the project
- Shared across all documents in the project

**Document-Level Cursor Tracking** (Per-Project-Per-Document):
- Cursor position specific to document type
- Selected elements within the document
- Last activity timestamp in the specific document
- Isolated per document - user can have different positions in different documents

### Multi-Document Support Examples

1. **Canvas Document**: `{ documentId: "plan-dag-canvas", type: "canvas", position: { x: 100, y: 200, zoom: 1.5 } }`
2. **Spreadsheet Document**: `{ documentId: "project-data", type: "spreadsheet", position: { row: 5, column: 3, sheet: "metrics" } }`
3. **VR Journey Document**: `{ documentId: "vr-walkthrough", type: "vr_journey", position: { x: 10, y: 0, z: 5, scene: "office-floor-2" } }`
4. **Timeline Document**: `{ documentId: "project-timeline", type: "timeline", position: { timestamp: 1640995200, track: 2 } }`
5. **Code Editor Document**: `{ documentId: "config-yaml", type: "code_editor", position: { line: 42, column: 8, file: "config.yaml" } }`

This allows users to seamlessly switch between different types of documents within the same project while maintaining their cursor positions in each context.

## Implementation Plan

### Phase 1: Backend WebSocket Infrastructure ✅ COMPLETED
**Duration**: 2-3 days

1. **Create WebSocket Handler Module** ✅
   - File: `layercake-core/src/server/websocket/`
   - Implement WebSocket connection management
   - Handle project-specific session rooms
   - Add authentication middleware (TODO: JWT validation)

2. **Message Protocol Implementation** ✅
   - Define message types and serialization
   - Implement message routing and validation
   - Add rate limiting for cursor updates (max 20 updates/second)

3. **Multi-Document Session Management** ✅
   - Track active users per project in memory-only data structures
   - Store document-specific cursor positions and selections per user
   - Support multiple document types (canvas, spreadsheet, 3D, timeline, code)
   - Handle automatic cleanup on disconnect (no database persistence)
   - Implement heartbeat/keepalive mechanism with memory cleanup
   - Data structure: `DashMap<ProjectId, DashMap<UserId, DashMap<DocumentId, DocumentState>>>`

4. **Integration with Axum Server** ✅
   - Add WebSocket route to existing server: `/ws/collaboration`
   - Configure CORS for WebSocket connections
   - Add WebSocket endpoint to server startup logs

### Phase 2: Frontend WebSocket Client
**Duration**: 2-3 days

1. **WebSocket Service Layer**
   - File: `frontend/src/services/websocket/`
   - Implement connection management with auto-reconnection
   - Handle message queuing during disconnections
   - Add connection state management (connecting/connected/disconnected)

2. **Collaboration Hook Refactoring**
   - Update `useCollaboration` hook to use WebSocket
   - Implement throttled cursor position broadcasting
   - Add user presence state management
   - Maintain backwards compatibility with GraphQL mutations

3. **Error Handling & Fallbacks**
   - Graceful degradation when WebSocket unavailable
   - Fallback to GraphQL for critical operations
   - Connection retry logic with exponential backoff

### Phase 3: Database Cleanup & Migration
**Duration**: 1-2 days

1. **Remove Database Storage of Ephemeral Data**
   - Remove `UPDATE_CURSOR_POSITION` mutation from GraphQL schema
   - Remove `user_presence` table or cursor position columns if they exist
   - Clean up unused GraphQL subscription code for presence
   - Update database migrations to remove ephemeral data columns

2. **Backend Cleanup**
   - Remove any database persistence code for cursor positions
   - Ensure all ephemeral data is memory-only
   - Add memory cleanup on server restart (expected behavior)

3. **Performance Optimization**
   - Implement cursor update batching (collect updates over 50ms intervals)
   - Add compression for message payloads
   - Optimize JSON serialization
   - Add memory usage monitoring for session data

4. **Testing & Validation**
   - Add WebSocket integration tests
   - Test connection handling under load
   - Verify cursor synchronization accuracy
   - Test memory cleanup on disconnect

## Technical Details

### Backend Dependencies
- `tokio-tungstenite` - WebSocket implementation
- `serde_json` - Message serialization
- `dashmap` - Concurrent session storage
- `tokio::time` - Rate limiting and timeouts

### Frontend Dependencies
- Native WebSocket API (no additional libraries needed)
- Existing React state management
- Integration with current `useCollaboration` hook

### Multi-Document In-Memory Data Structure
```rust
// Memory-only storage for ephemeral collaboration data
struct CollaborationState {
    // Project ID -> Project sessions
    projects: DashMap<i32, ProjectSession>,
}

struct ProjectSession {
    // User ID -> User presence (project-level)
    users: DashMap<String, UserPresence>,
    // Document ID -> Document sessions (document-level)
    documents: DashMap<String, DocumentSession>,
    // WebSocket connection management
    connections: DashMap<String, WebSocketSender>,
}

struct DocumentSession {
    document_type: DocumentType,
    // User ID -> Document-specific state
    active_users: DashMap<String, DocumentUserState>,
}

struct UserPresence {
    user_id: String,
    user_name: String,
    avatar_color: String,
    is_online: bool,
    last_active: Instant,
    // Project-level presence only - no cursor data here
}

struct DocumentUserState {
    position: Option<CursorPosition>,
    selected_node_id: Option<String>,
    last_active_in_document: Instant,
    // Document-specific ephemeral state only
}

#[derive(Clone, Debug)]
enum CursorPosition {
    Canvas { x: f64, y: f64, zoom: Option<f64> },
    Spreadsheet { row: i32, column: i32, sheet: Option<String> },
    ThreeD { x: f64, y: f64, z: f64, rotation: Option<(f64, f64, f64)>, scale: Option<f64>, viewport: Option<String> },
    Timeline { timestamp: i64, track: Option<i32> },
    CodeEditor { line: i32, column: i32, file: Option<String> },
}

#[derive(Clone, Debug)]
enum DocumentType {
    Canvas,
    Spreadsheet,
    ThreeD,
    Timeline,
    CodeEditor,
}
```

### Configuration
```toml
[server.websocket]
max_connections_per_project = 50
heartbeat_interval = 30s
cursor_update_rate_limit = 20  # per second
message_buffer_size = 100
# Memory-only settings
session_cleanup_interval = 60s      # Clean up inactive sessions
max_inactive_time = 300s            # Remove user after 5min inactive
max_documents_per_project = 100     # Limit documents tracked per project
document_cleanup_interval = 300s    # Clean up empty document sessions
```

### Security Considerations
1. **Authentication**: Validate JWT tokens on WebSocket handshake
2. **Rate Limiting**: Prevent spam with per-user message limits
3. **Project Isolation**: Ensure users only receive data for authorized projects
4. **Input Validation**: Sanitize all incoming cursor coordinates
5. **Resource Limits**: Cap maximum concurrent connections per project

## Success Metrics

1. **Performance Improvements**
   - Reduce cursor update latency by >80%
   - Eliminate GraphQL mutation log noise
   - Support >10 concurrent users per project smoothly

2. **User Experience**
   - Real-time cursor synchronization (<100ms lag)
   - Stable connections with auto-reconnection
   - Seamless integration with existing UI

3. **System Efficiency**
   - Reduce server CPU usage for ephemeral updates
   - **Eliminate database load** from cursor mutations (memory-only storage)
   - Cleaner application logs
   - **Zero database writes** for ephemeral data

## Rollout Strategy

1. **Development Environment**: Implement and test with WebSocket alongside existing GraphQL
2. **Feature Flag**: Deploy with feature flag to enable/disable WebSocket mode
3. **Gradual Migration**: Start with cursor updates, then expand to other ephemeral data
4. **Full Deployment**: Remove GraphQL mutations once WebSocket is stable
5. **Monitoring**: Track connection stability and performance metrics

## Future Enhancements

- **Binary Protocol**: Switch to MessagePack or Protocol Buffers for efficiency
- **Shared Cursors**: Implement cursor trails and selection visualization
- **Voice Chat Integration**: Add WebRTC support for voice collaboration
- **Real-time Comments**: Live comment threads on DAG nodes
- **Operational Transform**: Support for simultaneous node editing