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
