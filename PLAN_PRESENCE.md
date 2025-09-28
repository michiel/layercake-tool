# Frontend Presence System Integration Plan

## Overview

This document outlines the implementation plan for integrating the existing backend presence infrastructure with frontend UI/UX components based on SPECIFICATION.md requirements. The backend already has comprehensive WebSocket collaboration infrastructure - this plan focuses on UI integration and addressing frontend gaps.

## Current State Analysis

### Existing Backend Infrastructure ✅
- **WebSocket Server**: Real-time collaboration WebSocket endpoint exists
- **Message Types**: Comprehensive message types in `layercake-core/src/server/websocket/types.rs`
- **Cursor Tracking**: Full cursor position support for Canvas, Spreadsheet, 3D, Timeline, and CodeEditor contexts
- **Session Management**: CollaborationState with project and document session management
- **User Presence**: UserPresenceData and DocumentPresence structures implemented

### Existing Frontend Components ✅
- **UserPresenceIndicator**: Shows online users with avatars and status (`frontend/src/components/collaboration/UserPresenceIndicator.tsx`)
- **CollaborativeCursors**: Displays real-time cursor positions (`frontend/src/components/collaboration/CollaborativeCursors.tsx`)
- **CollaborativeCursor**: Individual cursor component with user name labels (`frontend/src/components/collaboration/CollaborativeCursor.tsx`)
- **useCollaborationV2**: WebSocket-based collaboration hook (`frontend/src/hooks/useCollaborationV2.ts`)
- **WebSocket Service**: Dedicated WebSocket service (`frontend/src/services/websocket/WebSocketCollaborationService.ts`)
- **Type Definitions**: Comprehensive WebSocket types (`frontend/src/types/websocket.ts`)

### Integration Status ✅
- **Plan DAG Editor**: Already integrated with collaborative cursors via `CollaborationManager.tsx`
- **Real-time Updates**: WebSocket connection and cursor broadcasting working
- **Multi-document Support**: Document switching and per-document presence tracking implemented

## Missing Components (Frontend Gaps)

### 1. Top Bar Integration ❌
**SPECIFICATION.md Requirements:**
- Top bar with Layercake icon and title (left)
- User presence indicator with online user count and hover card (right)
- Theme toggle icon (right)
- Online/offline status indicator (right)

**Current Status:** No top bar component exists that integrates presence data

### 2. Project-level Presence Display ❌
**SPECIFICATION.md Requirements:**
- "On the frontend, user presence shown on a per-project basis in the top bar (icon with the number of active users, click on icon to list active users by name in a Mantine Hover Card)"

**Current Status:** UserPresenceIndicator exists but not integrated into a top bar

### 3. User Menu/Profile ❌
**Current Status:** No user authentication or profile management UI

## Phase 1: Create Top Bar with Integrated Presence
### 1.1 Create Top Bar Layout Component

```typescript
// frontend/src/components/layout/TopBar.tsx

import React from 'react';
import { Group, Text, ActionIcon, useMantineColorScheme } from '@mantine/core';
import { IconSun, IconMoon, IconWifi, IconWifiOff } from '@tabler/icons-react';
import { UserPresenceIndicator } from '../collaboration/UserPresenceIndicator';
import { ConnectionState } from '../../types/websocket';

interface TopBarProps {
  projectId?: number;
  connectionState?: ConnectionState;
  users?: any[]; // Use existing UserPresenceData type
  currentUserId?: string;
}

export const TopBar: React.FC<TopBarProps> = ({
  projectId,
  connectionState = ConnectionState.DISCONNECTED,
  users = [],
  currentUserId
}) => {
  const { colorScheme, toggleColorScheme } = useMantineColorScheme();
  const isDark = colorScheme === 'dark';
  const isOnline = connectionState === ConnectionState.CONNECTED;

  return (
    <Group h={60} px="md" justify="space-between"
           style={{ borderBottom: '1px solid var(--mantine-color-gray-3)' }}>
      {/* Left side - Logo and title */}
      <Group gap="sm">
        {/* Simple text logo for now - can be replaced with icon later */}
        <Text size="xl" fw={700}>🍰 Layercake</Text>
      </Group>

      {/* Right side - Controls */}
      <Group gap="sm">
        {/* User presence indicator (only show if in a project) */}
        {projectId && (
          <UserPresenceIndicator
            users={users}
            connectionState={connectionState}
            currentUserId={currentUserId}
            onReconnect={() => {/* TODO: implement reconnect */}}
          />
        )}

        {/* Theme toggle */}
        <ActionIcon variant="subtle" size="lg" onClick={() => toggleColorScheme()}>
          {isDark ? <IconSun size="1.2rem" /> : <IconMoon size="1.2rem" />}
        </ActionIcon>

        {/* Online status indicator */}
        <ActionIcon variant="subtle" size="lg" color={isOnline ? "green" : "red"}>
          {isOnline ? <IconWifi size="1.2rem" /> : <IconWifiOff size="1.2rem" />}
        </ActionIcon>
      </Group>
    </Group>
  );
};
```

### 1.2 Enhance Existing UserPresenceIndicator

**Current component** (`frontend/src/components/collaboration/UserPresenceIndicator.tsx`) needs modification to match SPECIFICATION.md requirements:

**Required Changes:**
- Move from current location to be used in top bar
- Add hover card with user list (currently shows tooltip)
- Add user count badge on icon
- Match specification design

```typescript
// Modify existing frontend/src/components/collaboration/UserPresenceIndicator.tsx
// Add hover card functionality to replace tooltip
// Add user count badge display as per specification
```

### 1.3 Connection Status Integration

**Note:** Connection status is already handled by existing UserPresenceIndicator component. The SPECIFICATION.md requires "red/green icon only" which can be extracted from the existing ConnectionState logic.

**Implementation:** Use existing `ConnectionState` enum from `frontend/src/types/websocket.ts`
```

### 1.4 Theme Toggle Integration

**Implementation:** Use Mantine's built-in `useMantineColorScheme` hook, as shown in TopBar component above.

**Note:** No new component needed - integrate directly into TopBar using Mantine's theming system.
```

### 1.5 User Authentication (Future)

**Current Status:** No user authentication system exists in the current codebase.

**SPECIFICATION.md Requirements:** No explicit user menu mentioned, focus on presence functionality.

**Decision:** Skip user menu for now, focus on presence integration. Can be added later when authentication is implemented.
```

### 2.1 Find Current Application Layout

**Task:** Locate where the application renders the main layout and identify where to inject the TopBar component.

**Investigation needed:**
- Find main App component (`frontend/src/App.tsx`)
- Identify current layout structure
- Determine where TopBar should be rendered

### 2.2 Update Main Layout to Include TopBar

**Current Status:** Need to examine existing layout and add TopBar at the top level.

**Steps:**
1. Import TopBar component
2. Add TopBar above current main content
3. Pass project context and collaboration data
4. Ensure responsive layout
```

### 2.3 Project Context Integration

**Current Status:** Need to determine how project ID is passed through the application.

**Questions to investigate:**
- How is current project determined?
- Where is project context stored?
- How to pass projectId to TopBar?

**Implementation:**
- Add project context provider if needed
- Pass projectId from routing/state management
- Connect collaboration hooks to TopBar
```

## Phase 3: Implementation Steps

### 3.1 Immediate Tasks (Week 1)

**Day 1-2: Layout Investigation**
1. Examine `frontend/src/App.tsx` to understand current layout
2. Identify where project context is managed
3. Determine how to access current project ID
4. Check existing routing structure

**Day 3-4: TopBar Creation**
1. Create `frontend/src/components/layout/TopBar.tsx`
2. Integrate with existing UserPresenceIndicator
3. Add theme toggle using Mantine's built-in system
4. Add connection status indicator

**Day 5: Integration**
1. Modify main App component to include TopBar
2. Connect TopBar to existing collaboration hooks
3. Test presence functionality in top bar
```

### 3.2 Enhancement Tasks (Week 2)

**Enhance UserPresenceIndicator:**
1. Modify existing component to include hover card
2. Add user count badge
3. Improve styling to match specification

**Project Context Integration:**
1. Ensure project ID is properly passed to TopBar
2. Connect to existing `useCollaborationV2` hook
3. Test with multiple users/projects

### 3.3 Testing and Validation (Week 3)

**Functional Testing:**
1. Test presence display with multiple users
2. Verify connection status indicators
3. Test theme toggle functionality
4. Verify hover card behavior

**Integration Testing:**
1. Test with existing Plan DAG editor
2. Verify cursor tracking still works
3. Test WebSocket connection/reconnection
4. Test across different browsers

## Implementation Notes

### Existing Components to Leverage ✅
- `UserPresenceIndicator` - Modify for top bar use
- `useCollaborationV2` - Use for presence data
- `ConnectionState` enum - Use for status indicators
- Mantine theme system - Use for dark/light toggle

### New Components to Create ❌
- `TopBar` - Main layout component
- Layout integration in main App component

### Backend Integration ✅
- WebSocket infrastructure exists and works
- Message types are comprehensive
- Session management is implemented
- No backend changes needed

## Success Criteria

1. **Top Bar Display**: Top bar shows on all pages with Layercake branding
2. **Presence Integration**: User count and online status visible in top bar
3. **Hover Functionality**: Clicking presence icon shows hover card with user list
4. **Theme Toggle**: Dark/light mode toggle works seamlessly
5. **Connection Status**: Online/offline status is clearly indicated
6. **No Regression**: Existing cursor tracking and collaboration features continue to work
7. **Responsive Design**: Top bar works on different screen sizes

## Current Collaboration System Architecture

**Frontend Flow:**
```
App → useCollaborationV2 → WebSocketCollaborationService → Backend WebSocket
                        ↓
              UserPresenceIndicator (in editors)
                        ↓
              CollaborativeCursors (in plan editor)
```

**Required Integration:**
```
App → TopBar → UserPresenceIndicator (modified) → useCollaborationV2 → ...
```

This plan focuses on UI integration and leverages the substantial existing collaboration infrastructure, requiring minimal new backend work and primarily frontend component integration and enhancement.

