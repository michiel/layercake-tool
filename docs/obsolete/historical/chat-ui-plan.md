# Project Chat UI Plan

## Scope
- Add a `/project/:id/chat` route that mounts an LLM-backed assistant panel inside the existing frontend shell.
- Reuse the new GraphQL mutations/subscriptions (`startChatSession`, `sendChatMessage`, `chatEvents`) to drive the conversation loop.
- Provide tooling discovery (available MCP actions) and clearly differentiate assistant messages vs. tool outputs.

## Milestones

1. **Routing & Layout**
   - Extend the project router to include a `chat` child route.
   - Reuse sidebar/header chrome so the chat page inherits project context (breadcrumbs, selectors).
   - Create a responsive layout with a scrollable transcript pane and a fixed composer.

2. **GraphQL Wiring**
   - Add typed hooks (Urql/Apollo) for the chat mutations/subscriptions.
   - Implement a `useChatSession(projectId, provider)` hook that
     - lazily calls `startChatSession`
     - exposes methods for sending user turns
     - streams `chatEvents` into local state.
   - Handle reconnection/token expiry by propagating errors from the subscription stream into UI toasts.

3. **Transcript Rendering**
   - Model messages as `{id, role, text, toolName?}` where tool invocations render with badges (e.g., `Tool: list_projects`).
   - Add incremental rendering for streaming updates (append chunks as they arrive, collapse consecutive assistant messages).
   - Provide state indicators: “Connecting…”, “Waiting for assistant…”, “Executing tool…”.

4. **Composer Features**
   - Support provider selection dropdown (default from backend).
   - Enforce optimistic send (display pending bubble while awaiting response).
   - Add keyboard shortcuts (`Shift+Enter` for newline, `Enter` to send) and message history recall.

5. **MCP Tool Awareness (Phase 2)**
   - Surface a collapsible sidebar listing available tools/prompts from the GraphQL session payload (follow-up schema work may expose this metadata).
   - Allow inserting tool name scaffolding into the composer to guide the model.

6. **Testing & Telemetry**
   - Add component tests for the chat hook using mocked GraphQL responses.
   - Record analytics events (session started, tool executed, errors) to understand usage patterns.

## Open Questions
- Do we need per-user session persistence (e.g., resume chat after navigation) or can we create a fresh session per visit?
- Should tool outputs be copyable/downloadable if they return large payloads (e.g., graph exports)?
- How do we surface authentication errors when the backend rejects a chat request (missing API keys, etc.)?
