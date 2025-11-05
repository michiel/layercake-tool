## Objective
- Determine whether [assistant-ui](https://github.com/assistant-ui/assistant-ui) can replace the current Layercake chat experience (Vite/React frontend + GraphQL subscriptions) without regressing functionality or developer ergonomics.

## Current State Review
- Audit `frontend/src/pages/ProjectChatPage.tsx` and `frontend/src/hooks/useChatSession.ts` to document data flow (start-session mutation, message mutation, events subscription, local message state).
- Capture styling/theming requirements coming from Mantine components, project branding, and responsiveness targets.
- Catalog backend contracts used by the UI (`startChatSession`, `sendChatMessage`, `chatEvents`), including payload shape, streaming semantics, and error handling expectations.
- Note persistence/metadata features the UI surfaces (session titles, provider selection, tool call display) to compare with assistant-ui capabilities.

## assistant-ui Capability Assessment
- Review assistant-ui primitives (providers, hooks, UI components) and identify how they handle:
  - Multi-provider selection and dynamic model metadata.
  - Streaming tool call events or non-text messages.
  - External state management (Apollo, GraphQL, custom transport).
  - Theming and design system integration (Mantine vs. Tailwind/Headless UI).
- Determine whether assistant-ui expects an OpenAI-compatible API or can be driven by custom transports—identify adapter points for our GraphQL mutations/subscriptions.
- Check licensing, bundle size impact, and minimum React version to confirm compatibility with our frontend stack.

## Integration Spike Plan
- Build a thin prototype page under `frontend/src/pages/assistant-ui-spike/` consuming mock data that mirrors our GraphQL outputs. Aim to render:
  - User and assistant messages.
  - Tool invocation cards using assistant-ui's structured output support (verify feature availability).
  - Provider/model switcher.
- Implement an adapter layer translating `ChatEvent` subscription payloads into assistant-ui's message schema; stub required interfaces (message IDs, timestamps, roles).
- Validate streaming behavior: ensure incremental token updates or server-sent events can be mapped onto subscription updates without duplicating work.

## Backend Alignment Tasks
- Confirm `ChatManager` can emit message deltas compatible with assistant-ui expectations (e.g., incremental tokens vs. full messages). If gaps exist, outline required API changes (new fields, event types).
- Assess whether we need REST endpoints or can keep GraphQL; identify any rate/connection constraints for WebSocket subscriptions when embedding assistant-ui.
- Plan migration strategy for persisted metadata (session list, tool summaries) if assistant-ui favors a different storage contract.

## Adoption Path & Risks
- Draft a phased rollout:
  1. Prototype route gated behind a feature flag.
  2. Internal QA with real projects.
  3. Full replacement once parity is confirmed.
- Enumerate risks:
  - Feature gaps (tool summaries, multi-provider support).
  - Styling divergence from Mantine.
  - Increased bundle size or dependency conflicts.
- Identify mitigation strategies (custom wrappers, fallback to existing UI, contributing upstream patches).

## Decision Checklist
- Functional parity verified (messages, tools, provider switching, session history).
- Performance acceptable (benchmark load times, render speed).
- Accessibility & internationalization maintained.
- Developer experience: documentation, customization, testability.
- If all criteria met, produce adoption RFC with timelines and required code changes; otherwise, document blockers and alternatives (enhancing current UI, building custom components).

---

## Evaluation Summary (2025-02-14)

### Current UI Capabilities
- Frontend state lives in `useChatSession`, backed by `startChatSession` / `sendChatMessage` mutations and `chatEvents` subscription (assistant/tool messages as discrete GraphQL events).
- Mantine components provide theming, layout, and UX polish (badges, alerts, scroll area, input bar).
- Chat metadata: provider switcher, model badge, tool invocation summaries, restart/reset flows, and persisted session IDs from the backend.

### assistant-ui Fit Analysis
- **Runtime options**: `LocalRuntime` can call our GraphQL API through fetch, but we need streaming semantics; `ExternalStoreRuntime` lets us reuse Apollo-managed state and push GraphQL subscription updates directly into assistant-ui.
- **Tool calls**: Library supports tool-call message parts and custom renderers; we can map our `ToolInvocation` event summary into assistant-ui `ToolCallMessagePart` with the same payload we persist today.
- **Styling/theming**: Defaults rely on shadcn/tailwind CSS via `@assistant-ui/styles`. Mantine can host assistant-ui components, but we either accept their styling or build wrappers that map Mantine tokens to CSS vars. Expect effort to align typography, spacing, dark mode, and Mantine theme overrides.
- **Transport expectations**: Most recipes assume REST or OpenAI-compatible streaming. For GraphQL we’ll build a runtime adapter that:
  - Calls `startChatSession` / `sendChatMessage`.
  - Subscribes to `chatEvents` and updates assistant-ui message store (either via `useExternalStoreRuntime` or a custom `LocalRuntime` adapter that forwards incremental events).
  - Translates GraphQL payloads (assistant text, tool summaries) into assistant-ui message/part schema.
- **Persistence**: assistant-ui is stateless by default; we already persist sessions in Layercake DB. We can keep existing GraphQL endpoints and skip Assistant Cloud.
- **Dependencies**: Requires React 18+, Zustand, Radix primitives, and CSS bundle; all compatible with our Vite setup. Need to audit bundle size impact once integrated.

### Prototype Scope
- Build a feature-flagged page using `AssistantRuntimeProvider + Thread`, wired to a lightweight adapter that consumes existing GraphQL hooks.
- Implement adapter utilities:
  - Event translator (GraphQL `ChatEvent` → assistant-ui `ThreadMessage` + parts).
  - Message send handler that mirrors `useChatSession.sendMessage`.
- Wrap default components with Mantine containers to maintain layout parity.
- Validate streaming/perf by replaying an existing session via mocked events.

### Risks & Mitigations
- **Styling drift**: High likelihood default assistant-ui appearance clashes with Mantine. Mitigate by overriding CSS variables and, if necessary, composing primitives with Mantine components.
- **Feature parity**: Need custom UI for tool summaries and provider switcher. assistant-ui primitives allow custom headers/footers; ensure we can slot provider selector without fighting internal layouts.
- **Testing**: Introduce React Testing Library snapshots to ensure event translation works; keep integration behind feature flag until stable.
- **bundle size**: Monitor via `npm run analyze` to ensure acceptable growth; tree-shake unused primitives.

### Recommendation
assistant-ui can replace the bespoke chat view with manageable engineering effort:
- Integration complexity: **Medium** (adapter development + styling work).
- Key blockers: none identified; all required hooks exposed via `LocalRuntime`/`ExternalStoreRuntime`.
- Next step: implement prototype branch following “Integration Spike Plan”, evaluate styling workload, and run UX review with the product team before deciding on full migration.
