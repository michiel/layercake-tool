## Chat Loop Review

### Current Flow Summary
1. `ChatProvider` owns a Zustand store (`chatSessionStore`) and exposes the runtime via React context.
2. `useChatSession`:
   - Reads/writes Zustand state.
   - Starts sessions with `startChatSession` and fetches history.
   - Subscribes to `chatEvents`.
   - Pushes user messages via `sendChatMessage`.
3. `ChatProvider` also maintains a separate `useState` (`runtimeMessages`) mirroring Zustand messages, feeding them into `useExternalStoreRuntime`.
4. The UI (page + modal) renders `AssistantThread`, which wraps assistant-ui primitives. The composer’s `onNew` calls back into `chat.sendMessage`.

### Issues Observed
| Area | Problem | Impact |
|------|---------|--------|
| State hydration | `useChatSession` short-circuits until `hydrated` is true, but UI rendering order makes this hard to reason about. | Composer ends up disabled or silent when hydration runs late. |
| Duplicate state | Messages are stored in Zustand **and** in local component state (`runtimeMessages`). | Extra conversions, possible divergence, makes debugging difficult. |
| Context registration | `useRegisterChatContext` is invoked conditionally in multiple pages. | Hook-order warnings (PlanEditorPage) and duplicated logic. |
| Message conversion | `convertMessages` recreated in `ChatProvider`, `ProjectChatPage`, `ChatModal`. | Repetition and inconsistent formatting. |
| Suggestions / status | Same suggestion arrays duplicated between modal and page. | Hard to maintain, inconsistent behaviour. |
| Disabled composer logic | Composer gating uses `loading || isAwaitingAssistant`, but `loading` is true until hydration, leaving send button inert (root cause of “typing does nothing”). | User input ignored without visual feedback. |
| Context injection | Each `sendMessage` prepends `"Context: ..."` text, baking UX concerns into transport. | Makes prompts noisy and couples UI with server contract. |

### Refactor Plan
1. **Centralize runtime state**
   - Replace the local `runtimeMessages` state with a derived selector from Zustand (single source of truth).
   - Provide helper selectors for `messages`, `status`, `sessionId`.

2. **Explicit hydration lifecycle**
   - Record hydration timestamp in Zustand.
   - Expose a `hydrationStatus` enum (`pending`, `ready`, `error`) via context.
   - Render skeletons/disabled controls using that enum instead of guessing with `loading`.

3. **Decouple context metadata**
   - Introduce a `chatContext` store slice (already present) but send metadata separately via `sendChatMessage` variables (e.g., `context` field) instead of string prefix. _Pending backend update – still using prefix._

4. **Consolidate view scaffolding**
   - Extract `ChatView` component used by both modal and page; accepts `suggestions`, `showHeader`.
   - Move suggestion copy into a single config.

5. **Simplify hook usage**
   - Wrap route registration with a `ChatContextBoundary` component near the router to avoid per-page hook duplication.

6. **Improve composer UX**
   - Keep composer enabled once session established; only disable while `isAwaitingAssistant` is true.
   - Show a small status chip (“Connected”, “Sending…”) instead of disabling input on hydration.
   - Add error toast if `sendChatMessage` fails.

7. **Testing & diagnostics**
   - Instrument `useChatSession` with debug logging behind `import.meta.env.DEV`.
   - Add unit tests for `chatSessionStore` reducers (hydration, append, reset).
   - Write Cypress smoke tests covering send/reload/resume flows.
