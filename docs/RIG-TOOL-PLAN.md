# Technical Plan: Rig-Native Tool Use with `rig`

This document captures the steps required to replace the handwritten tool-call parsing inside `layercake-tool` with the structured tool features that ship with the `rig` crate. The target outcome is a provider-agnostic chat loop that streams `ToolCall` objects from `rig`, executes them through the MCP bridge, and persists the full call/response history for every supported provider (OpenAI, Anthropic, Gemini, and Ollama).

## 1. Objective

Deliver a `ChatSession` implementation (`layercake-core/src/console/chat/session.rs`) that:

- Sends real tool definitions to every `rig` agent via `.rmcp_tools(...)`.
- Treats agent output as `CompletionResponse` objects that can contain `AssistantContent::ToolCall`.
- Executes each requested tool through the existing `McpBridge` and feeds `ToolResult` content back to the agent so the model can continue multi-turn reasoning without custom parsing.

## 2. Current State & Feasibility Review

- Conversation state is flattened into a single string (`build_conversation_prompt`) and the agent response is treated as a plain `String`. This prevents us from accessing `ToolCall` metadata returned by `rig-core 0.23.1`.
- Tool execution is detected by scanning for a ` ```tool_code` block (`extract_tool_invocation` / `parse_tool_command`). This is brittle, requires system-prompt hacks, and blocks provider features such as multiple tool calls in one turn.
- `call_rig_agent` already creates provider-specific agents and even tries to call `.rmcp_tools(...)`, but the builder is immutable and the return type discards the structured `CompletionResponse`, so the MCP capabilities are never exercised.
- `handle_tool_invocation` stores tool metadata in `ChatMessage`, yet those messages never get converted back into `rig::completion::Message::User { content: ToolResult }`, so the agent never sees the tool output it just produced.
- Feasibility: `rig-core` exposes `CompletionResponse`, `AssistantContent`, `ToolCall`, and `UserContent::ToolResult`, and the crate already implements `.rmcp_tools` for OpenAI, Anthropic, Gemini, and Ollama. No additional dependencies are needed; the work is fully achievable inside the current workspace.

## 3. Implementation Plan

### Phase 1 – Return Structured Agent Output

1. Change `call_rig_agent` so it returns `CompletionResponse<ProviderResponse>` instead of `String`. Add a small enum or wrapper to hold the provider-specific `raw_response` so we keep type safety without sprinkling generics throughout `ChatSession`.
2. Make the agent builder mutable for every provider and wire `.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned())` behind the `rmcp` feature flag. This guarantees that tools registered through the MCP server are advertised to every model.
3. Update `invoke_agent_with_retries` to propagate the full `CompletionResponse`. When we have to fall back (e.g., Ollama rejecting tools), convert the fallback response to text using `response.choice` instead of reissuing a brand-new prompt.
4. Capture `response.usage` for telemetry and include it in tracing so we can see token costs per turn.

### Phase 2 – Build Rig Prompts from Conversation History

1. Replace `build_conversation_prompt()` with `build_prompt_messages()` that converts `self.messages` into `Vec<rig::completion::Message>`. Map:
   - `ChatMessage::user` → `Message::User { content: Text }`
   - `ChatMessage::assistant` → `Message::Assistant { content: Text }`
   - `ChatMessage::tool_result` → `Message::User { content: ToolResult }`
   - Stored `tool_calls` → `Assistant` messages containing `AssistantContent::ToolCall`.
2. Construct a `Prompt` (or leverage `Prompt::try_from(messages)`) from those messages and pass it to `agent.prompt(...)` instead of concatenating strings. Keep the existing `system_prompt`/RAG context by feeding it into `agent.preamble(...)` so instructions are sent separately from user content.
3. Ensure that the RAG pipeline still injects citations: once the model produces final text, append citations just before emitting the assistant event, same as today.

### Phase 3 – Rig Tool Call Execution Loop

1. In `resolve_conversation`, inspect every `AssistantContent` returned in `response.choice`. Branch on:
   - `AssistantContent::ToolCall(call)` → enqueue into a new `handle_tool_call` helper.
   - `AssistantContent::Text(text)`/`Reasoning` → stream to the observer once no further tool calls are pending.
2. Update `handle_tool_invocation` to accept `rig::completion::ToolCall`. Execute the MCP tool through `self.bridge.execute_tool`, persist the invocation metadata, and push the result into `self.messages` via `ChatMessage::tool_result`.
3. Support multiple tool calls per turn by iterating `response.choice.into_iter()` and calling each tool sequentially before asking the agent to continue.
4. After executing a tool, produce a `Message::User { content: ToolResult { call_id, content: Text } }` so the next prompt turn contains the tool output that the agent expects.
5. Remove `extract_tool_invocation`, `parse_tool_command`, and `parse_key_value_arguments`; these are replaced entirely by `AssistantContent::ToolCall`.

### Phase 4 – Cleanup, UX, and Observability

1. Simplify `compose_system_prompt` so it no longer enumerates tool names—the agent already knows which tools exist through `.rmcp_tools`.
2. Keep emitting `ChatEvent::ToolInvocation` and persist tool metadata so the frontend can render `ToolCallMessagePart` components without change.
3. Extend tracing/metrics to log whether a turn included tool calls, how long each MCP execution took, and the per-provider token usage pulled from `CompletionResponse.usage`.
4. Document the new behavior (`docs/`, `historical/`) and delete any instructions that asked the model to emit ` ```tool_code` snippets since that format will no longer be used.

## 4. Testing & Validation

- Exercise the happy path for each provider with a tool-enabled plan (use `cargo run --bin layercake -- -p sample/kvm_control_flow_plan.yaml`).
- Manually trigger multiple tool calls by asking for chained operations and confirm that every call persists to the database and renders in the React thread UI.
- Simulate tool errors (return `Err` from the MCP bridge) and confirm the agent receives a structured error message before retrying.
- Toggle RAG on/off to ensure the new prompt builder still injects context and citations.
- Run `npm run backend:test` plus integration smoke tests to guarantee no regressions in the GraphQL chat API.

## 5. Success Criteria

- `ChatSession` no longer references `extract_tool_invocation` or the ` ```tool_code` convention; all tooling flows pass through `AssistantContent::ToolCall`.
- MCP tools are registered exactly once per session and exposed to every provider via `.rmcp_tools`, confirmed through integration traces.
- Tool invocations (name, args, result summary) show up in persisted chat history and the frontend without additional parsing logic.
- Disabling tool use mid-session (e.g., Ollama HTTP 400) still works because `invoke_agent_with_retries` can fall back to plain completions using the same `CompletionResponse` plumbing.

