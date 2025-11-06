# Chat Lifecycle Review

## Lifecycle Overview
- **Frontend flow**: `useChatSession` starts a session via `startChatSession`, subscribes to `chatEvents`, renders messages, and posts user input through `sendChatMessage`.
- **GraphQL layer**: `ChatManager::start_session` creates a chat runtime, `enqueue_message` pushes user prompts into the runtime worker, and `subscribe` exposes a broadcast stream of `ChatEvent`s.
- **Runtime loop**: `ChatSession::send_message_with_observer` streams assistant/tool events back through the broadcast channel while also invoking MCP tools when requested.

## Root Cause
- Switching providers tears down the active web component, creating a fresh GraphQL subscription. The user mutation often reaches the backend before the new subscription handshake finishes, so the `tokio::broadcast` channel has no active receiver when the first assistant/tool events are emitted. Because broadcast channels don’t replay past messages, the UI never sees the response even though the backend completed the exchange.

## Fixes & Refactors
- **Backend buffering**: `ChatManager` now records a bounded history of `ChatEvent`s per session and replays it when `subscribe` is called. This guarantees that responses produced before the subscriber is attached remain visible after provider switches or reconnects.
- **Simplified subscription handling**: The frontend hook now performs a single `client.subscribe` call keyed by the session id, removes the delayed activation logic, and always tears down stale observers. This reduces state churn and ensures the subscription starts immediately after a session is created.
- **Safety improvements**: Both ToolInvocation and AssistantMessage events are captured in the history buffer so downstream consumers always remain in sync, even across UI remounts or slow WebSocket negotiations.
- **Build cleanup**: Removed redundant `NODE_ENV` overrides in `.env` files so `npm run build` stops emitting errors about unsupported production settings.
- **Ollama compatibility**: Detect Ollama servers that reject tool/function calls (typically older versions) and automatically retry without tool integration so the chat still succeeds while surfacing an assistant notice about the downgrade.

## Remaining Follow-ups
- The history buffer is capped at 64 events and is per-session; we should adjust the size if longer conversations are introduced or persist transcripts server-side if we need auditability.
- Frontend state resets completely on restart/provider change. If we want to preserve prior transcripts per provider, we may need a higher-level store.
- **Backend enum naming**: The `ChatEventKind` enum in `layercake-core/src/graphql/types/chat.rs` should use `#[graphql(rename_all = "PascalCase")]` to match frontend expectations. Currently sends `ASSISTANT_MESSAGE` but frontend types expect `AssistantMessage`. Frontend now handles both formats defensively.

## Validation
- `cargo test -p layercake-core`
