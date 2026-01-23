# Chat Console Implementation Plan

## Goals & Scope
- Introduce `layercake console` as a REPL-style entrypoint that wraps existing CLI capabilities and adds a `chat` workflow backed by configured LLM providers.
- Reuse the existing database/services layer plus the MCP module so that AI-assisted operations can trigger the same tools used by external MCP clients.
- Lay the groundwork for exposing the chat experience via GraphQL so the web UI can attach to the same orchestration logic.

## Architectural Overview
- **CLI integration (`layercake-core/src/main.rs`, new `console` module):** extend the clap command tree with a `Console` subcommand that boots a REPL built with `clap-repl`. Shared command definitions live in `console::commands`, with asynchronous handlers that call into existing services.
- **Runtime context (`layercake-core/src/console/context.rs`):** maintain application state for the REPL (selected project, cached graph info, handles to database/services, logger, configuration).
- **Chat engine (`layercake-core/src/console/chat/`):** provide a `ChatSession` struct that owns conversation state, the active LLM provider client, and an MCP bridge exposing `LayercakeServerState` tools/prompts/resources.
- **Provider abstraction (`layercake-core/src/console/chat/providers.rs`):** define a trait (`ChatModelClient`) whose implementations wrap the `llm` crate for Ollama, OpenAI/Codex, Google Gemini, and Anthropic Claude. Configuration resolves provider selection, API keys, model names, temperature, etc.
- **GraphQL exposure (`layercake-core/src/graphql/`):** surface the chat workflow behind a mutation + subscription pair (start chat session, stream responses) that delegates to the same chat service code used by the console.
- **Configuration (`resources/config/*.toml` or env vars):** document and load provider settings (preferred model, base URL, credentials) via a new `ChatConfig` struct, respecting existing config conventions.

## Implementation Steps

1. **Project plumbing & dependencies**
   - Add `clap-repl` and `llm` to `layercake-core/Cargo.toml`; enable the appropriate `llm` backends via feature flags so each provider is available at runtime.
   - Extend feature flags: reuse `cli` flag or introduce `console` to keep the binary lean when the REPL is not required.
   - Confirm Tokio runtime requirements (`clap-repl` is sync); plan to drive async handlers via `tokio::runtime::Handle::current().block_on` or spawn tasks.

2. **Console entrypoint**
   - Update `Commands` enum in `layercake-core/src/main.rs` with `Console` (optionally variant arguments for profile/db path).
   - When `Console` is selected, initialize logging, load configuration, open database connection (reusing helper from server initialization), construct shared service bundle, then start the REPL loop.
   - Extract any shared setup logic (database URL resolution, service wiring) into reusable helpers (e.g., `services::bootstrap::new_service_container`).

3. **REPL command surface**
   - Implement `console::commands` mapping to Clap subcommands: `list-projects`, `use-project <id>`, `list-graphs [--project <id>]`, `show-graph <graph_id>`, `chat`, `exit`.
   - Each handler should:
     - Use the async service layer (`project_service`, `graph_service`, etc.) to fetch data.
     - Render human-friendly tables (leverage `tabled` or simple formatting).
     - Update `ConsoleContext` (selected project/graph) when appropriate.
   - Ensure errors bubble through the REPL with readable messages and no panic.

4. **Chat session orchestration**
   - Add `ChatSession` struct with fields: active project, conversation history (Vec of `{role, content}`), optional graph context, MCP bridge handle, LLM client.
   - Implement session lifecycle:
     - On `chat` command, instantiate session with the selected project (required) and load provider settings.
     - Prompt user input via REPL; send user message to `ChatSession::reply`.
     - `reply` gathers MCP tool metadata (available tools/prompts) from `LayercakeServerState`, assembles a system prompt describing available actions, and calls the configured `ChatModelClient`.
     - When the LLM response includes structured tool calls (depending on provider), execute via MCP tool registry and feed results back into the history.
   - Provide buffered responses for now; defer streaming support until UI integration requires it.

5. **MCP bridge integration**
   - Reuse `LayercakeServerState` by instantiating it in console setup: obtain DB connection, tool/resource registries, and auth context (likely `SecurityContext::system()` or authenticated user).
   - Implement an adapter translating between chat tool-call schema and MCP request/response shapes; surface tool descriptions and parameters in the chat prompt.
   - Ensure MCP execution respects async boundaries and captures stdout/stderr for the chat transcript.

6. **Provider abstraction**
   - Define `enum ChatProvider { Ollama, OpenAi, Gemini, Claude }` with configuration attributes.
   - Implement trait `ChatModelClient` with async `send_messages(&self, history: &[ChatMessage]) -> Result<ChatResponse>` using the `llm` crate backends for each provider.
   - Introduce `ChatResponse` capable of representing both natural language replies and tool call directives (structured JSON). Consider using enums to reflect tool invocation vs. plain text.
   - Cache clients in `ConsoleContext` to avoid re-reading config per prompt.

7. **GraphQL exposure**
   - Add GraphQL types (`chat_session.rs`) representing messages, tool calls, and streaming chunks.
   - Implement mutation `startChatSession(projectId, provider, options)` returning a session ID plus initial state; subscribe to `chatResponses(sessionId)` for incremental updates.
   - The resolver should reuse the chat orchestration logic (possibly factor into `services::chat_service`) to avoid duplication between console and GraphQL shells.
   - Wire the new schema pieces into `layercake-core/src/graphql/schema.rs` and update the server to register the necessary data loaders.

8. **Configuration & UX polish**
   - Define `ChatConfig` (deserialize from `~/.config/layercake/chat.toml` or environment variables) with provider defaults, fallback order, and timeouts. Persist non-key credentials in a dedicated database table exposed via the service layer.
   - Document precedence (CLI flags → env vars → config/database) and provide helpful error messages when credentials are missing. API keys remain environment-driven and system-wide for now.
   - Skip transcript persistence until UI work requires it; keep chat history in-memory only.

9. **Testing & Validation**
   - Add unit tests for command parsing and `ChatConfig` loading.
   - Write integration tests using mocked LLM providers (feature-gated) ensuring chat history, tool invocations, and MCP execution paths behave correctly.
   - For GraphQL, extend schema snapshot tests and add resolver tests using async-graphql’s test harness.
   - Manual validation checklist: `cargo run -- console`, run sample commands, verify chat flows with at least one provider stub, ensure GraphQL mutation available when `graphql` feature enabled.

## Open Questions

Resolved during review:
- Provider credentials are stored in a dedicated database table; API keys are supplied via environment variables and apply system-wide for now.
- Chat transcripts remain in-memory only until UI integration lands.
- Buffered output is acceptable until UI integration; streaming can be deferred.
- Implementation will rely on the `llm` crate for all target providers.
- The console and chat context expose the full MCP tool surface; any tool added to MCP becomes available automatically.
- GraphQL chat sessions and console chats run under a system-level auth context for now.

## Risks & Mitigations
- **LLM backend coverage:** Depending solely on the `llm` crate means missing backends could block certain providers. Mitigation: validate support early and plan upstream contributions or fallback models if gaps exist.
- **Async in REPL:** `clap-repl` is synchronous; improper handling of async services could block the UI. Mitigation: encapsulate async operations behind a runtime handle and ensure long tasks yield progress indicators.
- **Credential handling:** Storing API keys locally risks leaks. Mitigation: rely on env vars or OS keyring integration and avoid writing them to logs/history.
- **MCP bridging complexity:** Mapping provider tool-call schemas to MCP requests may be non-trivial, especially across providers. Mitigation: start with a simple command catalog and iterate, adding translation layer tests.
- **GraphQL coupling:** Reusing chat logic across console and GraphQL could introduce feature-flag tangles. Mitigation: create a provider-agnostic `chat_service` module compiled whenever either `cli` or `graphql` features are enabled and guard optional pieces with cargo features.
- **User expectation alignment:** Sample commands (`list-projects`, `list-graphs`) imply functionality not currently exposed via CLI. Mitigation: confirm required command surface and adjust scope or provide placeholder implementations until services are ready.
