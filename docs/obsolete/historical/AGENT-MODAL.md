## Objective
Design updates that make the assistant chat persistent, globally accessible, and context-aware while fixing the “Assistant thinking…” status bug.

## Current Pain Points
- Status indicator always reads “Assistant thinking…” even when idle.
- `useChatSession` creates a fresh session for every mount, so reloads lose conversation history.
- Chat UI only lives on the chat page; users can’t interact elsewhere in the app.
- Assistant lacks page-level context (plan editor, graph editor, etc.) when receiving messages.

## Implementation Summary
- Introduced `chatSessionStore` (Zustand + localStorage) to persist sessions per provider and hydrate `useChatSession` across reloads.
- Rewrote `useChatSession` to reuse stored sessions, gate new `startChatSession` calls, prefix outbound messages with the active page context, and accurately toggle the “Assistant thinking…” indicator.
- Added `chatContextStore`, `useRegisterChatContext`, and page-level registrations (plan editor, graph editor, graphs, plan nodes, chat) so the assistant receives route-aware descriptions and project IDs.
- Built a shared `ChatProvider` that centralizes provider selection, chat runtime, and exposes a global floating modal button implemented via `AssistantModalPrimitive`.
- Migrated the chat page to consume the shared runtime (`AssistantThread`) and removed duplicate assistant-ui wiring; both page and modal now show the same conversation state.

## Approach
1. **Fix Status Indicator**
   - However caused: `chat.isAwaitingAssistant` never resets or is inferred incorrectly.
   - Update `useChatSession` to flip the flag once an assistant message arrives and ensure it doesn’t default to true on mount.

2. **Persist Single Chat Session**
   - Introduce state keyed by provider in a central store (e.g., React context or Zustand). Load once per browser session and reuse.
   - Modify `useChatSession` to accept an optional `persistedSessionId`. Only call `startChatSession` when no active session exists or when explicitly restarting / switching providers.
   - Save retrieved `sessionId` and metadata in local storage (provider, model, last activity) so reloads rehydrate the same session.
   - Ensure GraphQL `startChatSession` handles `sessionId` reuse gracefully (already supported).

3. **Global Modal Chat**
   - Add a floating button (bottom-right) using shadcn `Sheet` / assistant-ui modal pattern.
   - Embed assistant-ui `Thread` inside the modal; reuse the shared `useChatSession` state.
   - Maintain consistent runtime across page navigations; modal should mirror page chat view.
   - Use assistant-ui modal example: https://www.assistant-ui.com/examples/modal as reference (import or replicate `AssistantModalPrimitive` components).

4. **Context Injection**
   - Create a context service capturing current route and relevant metadata (project ID, view type).
   - On message send, enhance payload with a system prefix or metadata field “current_view” before invoking `sendChatMessage`.
   - For assistant-ui runtime, insert a hidden system message describing context when provider changes or route changes.
   - Keep context updates lightweight (watch route via React Router’s `useLocation`).

## Implementation Steps
1. Update `useChatSession`
   - Add persistent store (e.g., `chatSessionStore` using Zustand) to hold session info by provider.
   - Load existing session ID before creating new one; wire into GraphQL.
   - Fix `isAwaitingAssistant` logic.
2. Persist across reloads
   - Save session data in `localStorage` on updates; read during initialization.
   - Handle provider switch / restart by clearing relevant store entry.
3. Global modal
   - Implement floating button component.
   - Use assistant-ui `AssistantModalPrimitive` to render the existing `ThreadView`.
   - Ensure consistent runtime by wrapping at app level (e.g., context provider in `App.tsx`).
4. Context awareness
   - Build `usePageContext` hook returning structured metadata.
   - Extend `sendMessage` pipeline to attach context (system message update or metadata field).
   - Consider storing context in `chat_session` metadata via GraphQL if needed.
5. UI adjustments
   - Update status badge to reflect corrected `isAwaitingAssistant`.
   - Ensure modal and page views share the same status & provider controls.

## Risks & Mitigations
- **State sync complexity**: Shared runtime between modal and page may desync. Mitigate by centralizing chat runtime in a context provider.
- **Context overloading**: Too frequent context updates could spam messages. Include minimal data and only on route change.
- **Storage edge cases**: LocalStorage may contain stale sessions after server resets; detect invalid sessions by inspecting `chatHistory` response and fall back to new session.
- **Modal UX**: Ensure accessible focus management and keyboard shortcuts (assistant-ui primitives handle much of this).

## Deliverables
- ✅ `useChatSession` persistence + status fix (`frontend/src/hooks/useChatSession.ts`, `frontend/src/state/chatSessionStore.ts`).
- ✅ Global modal button and shared runtime (`frontend/src/components/chat/ChatProvider.tsx`, `frontend/src/components/chat/ChatModal.tsx`, `frontend/src/components/chat/AssistantThread.tsx`).
- ✅ Route-aware context registration (`frontend/src/hooks/useRegisterChatContext.ts`, page updates).
- ✅ Documentation kept current with implemented architecture.
