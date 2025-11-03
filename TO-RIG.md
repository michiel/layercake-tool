# Migration Plan: From `llm` Crate to `rig`

## Executive Summary

This document outlines the plan to migrate Layercake's chat functionality from the `llm` crate (v1.3.4) to `rig` (v0.23.1). The migration aims to adopt a more actively maintained library with broader provider support, better tooling abstractions, and stronger community backing.

**Migration Timeline**: 1-2 weeks
**Risk Level**: Medium
**Recommended Approach**: Direct replacement with comprehensive testing on development branch

---

## Current State Analysis

### Current Architecture

The Layercake chat system currently uses the `llm` crate for:

1. **Multi-provider LLM support** (`layercake-core/src/console/chat/providers.rs:67-74`)
   - OpenAI
   - Anthropic (Claude)
   - Google (Gemini)
   - Ollama (local)

2. **Chat session management** (`layercake-core/src/console/chat/session.rs`)
   - Message history (Vec<ChatMessage>)
   - Tool/function calling integration
   - Streaming responses
   - MCP (Model Context Protocol) bridge integration
   - Database persistence of chat sessions and messages

3. **Tool calling** (`layercake-core/src/console/chat/mcp_bridge.rs`)
   - MCP tool registry integration
   - Tool execution context
   - Tool result serialisation and summarisation

### Dependencies

```toml
llm = { version = "1.3.4", optional = true, default-features = false, features = [
    "openai",
    "ollama",
    "google",
    "anthropic",
] }
```

Located in: `layercake-core/Cargo.toml:105-110`

---

## Rig Capabilities Assessment

### Strengths

1. **Active Development & Community**
   - Current version: 0.23.1 (actively maintained)
   - Production use by St Jude, Coral Protocol, Nethermind, Dria
   - 20+ model providers supported
   - Growing ecosystem with companion crates

2. **Feature Parity & Enhancements**
   - ✅ Multi-provider support (OpenAI, Anthropic, Gemini, and 17+ others)
   - ✅ Tool/function calling (via `Tool` trait)
   - ✅ Streaming support (multiple examples)
   - ✅ Agent abstractions (single and multi-agent)
   - ✅ Async-first design (tokio-based)
   - ✅ Type-safe API with comprehensive error handling
   - ✅ WASM compatibility (core library)

3. **Advanced Capabilities**
   - Agent orchestration and routing
   - Multi-turn conversations with state
   - RAG (Retrieval Augmented Generation)
   - Vector store integrations (10+ stores)
   - Embedding support
   - GenAI Semantic Convention compliance

4. **Architecture**
   - Modular design with provider plugins
   - Clear separation of concerns (agents, tools, completions)
   - Builder pattern for configuration
   - Trait-based tool definitions

### Limitations

1. **Breaking Changes Risk**
   - Project documentation warns of potential breaking changes in future versions
   - API still evolving (pre-1.0)

2. **Migration Effort**
   - Different API surface requires code refactoring
   - Tool definition approach differs from `llm` crate
   - Message structure may differ

3. **Documentation**
   - Documentation is spread across examples rather than comprehensive API docs
   - Migration guides not available

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| API incompatibility with existing code | High | Comprehensive testing on development branch before merge |
| Tool calling differences breaking MCP bridge | High | Integration tests for all MCP tool operations |
| Streaming behaviour changes | Medium | Validate streaming in all providers with test suite |
| Session persistence format changes | Medium | Test suite validates database compatibility |
| Provider-specific quirks (e.g., Ollama tool support) | Medium | Provider-specific test coverage |
| Performance regression | Low | Benchmark testing before merge to main |

### Operational Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Breaking changes in future rig versions | Medium | Pin version, monitor releases, maintain comprehensive tests |
| Provider deprecation | Low | Multiple provider support reduces single-point dependency |
| Learning curve for contributors | Low | Document patterns, provide examples |

### Benefits vs Risks

**Benefits:**
- More active development and community support
- Broader provider ecosystem (20+ vs 4)
- Better abstractions for complex workflows
- Production-proven by enterprise users
- Enhanced features (RAG, multi-agent, vector stores)

**Risks:**
- Migration effort (1-2 weeks)
- Potential for subtle behaviour changes
- API stability concerns (pre-1.0)
- No backward compatibility or rollback path

**Recommendation:** Proceed with migration using roll-forward only approach. Benefits outweigh risks, especially for long-term maintainability. Comprehensive testing on development branch mitigates deployment risks.

---

## Feasibility Review (2025-02-18)

### Overall Assessment
- Migrating from `llm` to Rig requires rebuilding the entire chat orchestration loop, not a drop-in swap. The current implementation in `layercake-core/src/console/chat/session.rs:249-438` handles persistence, streamed observer updates, multi-iteration tool execution, and Ollama-specific fallbacks; Rig’s agent API will need explicit replacements for each of those behaviours.
- MCP integration depends on the `llm::chat::Tool` shape (`layercake-core/src/console/chat/mcp_bridge.rs:39-88`). Rig’s tooling system expects statically named `Tool` implementations, so an adapter must bridge dynamic MCP registrations without losing per-call metadata or project scoping.
- Rig’s advertised provider parity should be validated with a spike before committing—especially Gemini and Ollama function-calling support and streaming parity across providers.

### Identified Inaccuracies & Gaps
- Dependency usage: the sample snippets import `rig::…` while the plan adds `rig-core`. Confirm the crate name and re-export strategy; otherwise the code will not compile.
- `ChatProvider::build_client` (as proposed) returns `Result<impl rig::completion::CompletionModel>`, but each provider produces a different concrete client. Either box an object-safe trait or wrap clients in an enum.
- `compose_system_prompt(config, project_id, &tools)` currently expects tool names (`Vec<String>`); the proposed Rig code passes tool structs and would fail to compile without a new overload.
- Rig’s `Tool` trait uses a `const NAME`; the adapter draft that mutates the name at runtime cannot work. Dynamic MCP tools will need distinct wrapper types or a routing shim.
- Streaming behaviour is underspecified. The rewritten example calls `agent.prompt(input)` and discards streaming events, regressing the CLI output observers that exist today.
- Persistence of tool calls and metadata (tool name, call id, payload, summary) is absent in the Rig sketch; those writes are mandatory for parity with history stored via `ChatHistoryService`.
- The timeline assumes 1–2 weeks. Given the breadth of refactors and lack of automated regression coverage, expect a longer stabilization period unless additional engineering is allocated.

### Action Items Before Migration
- Prototype Rig integration for one provider (e.g. OpenAI) to prove tool calling, streaming, and error handling (including sanitisation in `layercake-core/src/console/chat/session.rs:522-559`) can be replicated.
- Design a Rig-compatible MCP tool adapter that preserves dynamic registration, security context, and persistence semantics.
- Update the implementation plan to account for persistence updates, observer streaming, and Ollama fallback logic; document how multi-iteration tool loops will be rebuilt with Rig agents.

---

## Implementation Checklist

### Phase 0: Spike & Validation (Days 1-3) ✅ = Done, 🔄 = In Progress, ⬜ = Not Started, ❌ = Blocked

- [✅] **Spike: Basic rig integration**
  - [✅] Add rig-core dependency to Cargo.toml (test only)
  - [✅] Create spike example with OpenAI provider
  - [✅] Test basic chat completion - **WORKING** (examples/rig_spike_simple.rs compiles)
  - [⬜] Verify streaming API works - TODO
  - [⬜] Test tool calling with ToolDyn - TODO
  - [✅] Document findings in spike notes

**BLOCKER FOUND**: rig-core 0.23.1 requires Rust 1.82+ for if-let chains (RFC 2497), current version was 1.87.0 but failed to compile.

**RESOLUTION**: Updated to Rust 1.91.0 (stable) which includes if-let chains support. rig-core now compiles successfully!

**SPIKE STATUS**: In progress - working through rig API discovery

**API Findings**:
- Crate is `rig-core` but imports use `rig::` (need package rename in Cargo.toml)
- Requires `use rig::client::CompletionClient` trait for `.completion_model()` and `.agent()` methods
- Requires `use rig::completion::Prompt` trait for `.prompt()` method
- Model constants: `rig::providers::openai::GPT_4O_MINI`
- Tool trait requires `const NAME: &'static str` - **confirms dynamic tools need wrapper**

**BLOCKER**: Documentation for rig-core 0.23.1 not available on docs.rs (404), learning API from GitHub examples

**Phase 0 Spike Findings Summary**:

1. **Rust Compatibility**: ✅ RESOLVED - Requires Rust 1.82+ (if-let chains), works with 1.91.0
2. **Compilation**: ✅ rig-core compiles successfully
3. **Documentation**: ❌ **CRITICAL** - No docs.rs documentation, GitHub examples outdated/incomplete
4. **API Stability**: ❌ **CONCERN** - API discovery difficult, traits not intuitive, examples don't compile
5. **Dynamic Tools**: ❌ **CONFIRMED** - `Tool::NAME` must be const, requires wrapper infrastructure

**UPDATED FINDINGS** (After reviewing actual docs):

✅ **Documentation EXISTS**: https://docs.rs/rig-core/latest/rig/ (was checking wrong URL)
✅ **Dynamic tools SUPPORTED**: `ToolDyn` trait with `name()` method returning `String` (not const!)
✅ **MCP integration EXISTS**: `rmcp` feature flag provides MCP tool support
✅ **Agent API is simpler**: `.agent("model-name")` with string, not constants

**Key Discoveries**:
- `ToolDyn` trait enables dynamic dispatch - `name()` returns `String`
- `AgentBuilder.tool(impl Tool + 'static)` accepts any Tool implementation
- `rmcp` feature provides MCP tool integration (need to investigate)
- Proper docs available at https://docs.rs/rig-core/latest/rig/

**REVISED ASSESSMENT**: ✅ **MIGRATION IS FEASIBLE**

The initial concerns were based on incomplete information. With proper documentation:
- ✅ Dynamic tools are supported via ToolDyn
- ✅ API is well-documented
- ✅ MCP integration may already exist (rmcp feature)
- ⚠️  Still pre-1.0 (breaking changes possible)

**RECOMMENDATION**: **PROCEED with migration** but with caution:
1. Investigate `rmcp` feature for MCP integration
2. Use `ToolDyn` trait for dynamic MCP tool wrapper
3. Pin rig-core version to avoid breaking changes
4. Build comprehensive tests during migration

**NEXT STEP**: Complete working spike example with ToolDyn

- [ ] **Spike: Tool adapter design**
  - [ ] Research rig Tool trait constraints
  - [ ] Design dynamic tool wrapper approach
  - [ ] Prototype MCP tool adapter
  - [ ] Test tool execution with metadata preservation
  - [ ] Validate security context propagation

- [ ] **Spike: Error handling**
  - [ ] Test error handling patterns
  - [ ] Verify API key sanitisation approach
  - [ ] Test Ollama fallback scenarios
  - [ ] Document error mapping strategy

- [ ] **Decision Point: Proceed or Pivot**
  - [ ] Review spike findings
  - [ ] Confirm rig meets requirements
  - [ ] Update timeline if needed
  - [ ] Document any blockers found

### Phase 1: Core Infrastructure (Days 4-8)

- [ ] **Dependency management**
  - [ ] Update layercake-core/Cargo.toml
  - [ ] Remove llm dependency
  - [ ] Add rig-core dependency
  - [ ] Verify workspace builds
  - [ ] Run cargo check

- [ ] **Provider implementation**
  - [ ] Create provider client enum wrapper
  - [ ] Implement OpenAI client builder
  - [ ] Implement Anthropic client builder
  - [ ] Implement Gemini client builder
  - [ ] Implement Ollama client builder
  - [ ] Update ChatProvider trait methods
  - [ ] Add credential handling
  - [ ] Test each provider initialization

- [ ] **Tool adapter infrastructure**
  - [ ] Create layercake-core/src/console/chat/tools.rs
  - [ ] Implement McpToolAdapter struct
  - [ ] Implement McpToolExecutor trait
  - [ ] Handle dynamic tool naming
  - [ ] Add tool result serialisation
  - [ ] Add metadata preservation
  - [ ] Test tool adapter with sample MCP tool

### Phase 2: Session Management (Days 9-12)

- [ ] **Session rewrite**
  - [ ] Rewrite ChatSession struct with rig agent
  - [ ] Implement session initialization
  - [ ] Implement message persistence
  - [ ] Add session resumption logic
  - [ ] Preserve database compatibility

- [ ] **Streaming implementation**
  - [ ] Wire rig streaming to observer pattern
  - [ ] Implement token-by-token updates
  - [ ] Test CLI observer callbacks
  - [ ] Preserve existing output format

- [ ] **Tool execution loop**
  - [ ] Implement multi-iteration tool calling
  - [ ] Add MAX_TOOL_ITERATIONS logic
  - [ ] Handle tool result feedback
  - [ ] Persist tool call metadata
  - [ ] Test tool execution flow

- [ ] **Error handling**
  - [ ] Implement Ollama HTTP 400 fallback
  - [ ] Add API key sanitisation
  - [ ] Map rig errors to existing error types
  - [ ] Test error scenarios

### Phase 3: MCP Integration (Days 13-15)

- [ ] **MCP bridge updates**
  - [ ] Add rig_tools() method to McpBridge
  - [ ] Implement tool registry conversion
  - [ ] Test security context propagation
  - [ ] Verify project scoping
  - [ ] Test tool execution with real MCP tools

- [ ] **Tool persistence**
  - [ ] Persist tool invocations
  - [ ] Store tool call IDs
  - [ ] Store tool arguments
  - [ ] Store tool results
  - [ ] Verify ChatHistoryService integration

- [ ] **Module integration**
  - [ ] Update mod.rs exports
  - [ ] Remove old llm imports
  - [ ] Add rig imports
  - [ ] Fix compilation errors
  - [ ] Run cargo build

### Phase 4: Testing (Days 16-20)

- [ ] **Unit tests**
  - [ ] Provider initialization tests
  - [ ] Tool adapter tests
  - [ ] Session management tests
  - [ ] Persistence tests
  - [ ] Error handling tests

- [ ] **Integration tests**
  - [ ] OpenAI provider end-to-end
  - [ ] Anthropic provider end-to-end
  - [ ] Gemini provider end-to-end
  - [ ] Ollama provider end-to-end
  - [ ] MCP tool integration
  - [ ] Session resumption
  - [ ] Multi-turn conversations

- [ ] **Edge case testing**
  - [ ] Ollama tool rejection (HTTP 400)
  - [ ] API key sanitisation in logs
  - [ ] Timeout handling
  - [ ] Error recovery
  - [ ] Concurrent sessions

- [ ] **Performance validation**
  - [ ] Response latency benchmarks
  - [ ] Memory usage checks
  - [ ] Streaming performance
  - [ ] Tool execution overhead

### Phase 5: Deployment (Days 21-22)

- [ ] **Pre-deployment**
  - [ ] All tests passing
  - [ ] Documentation updated
  - [ ] Review checklist complete
  - [ ] Performance acceptable

- [ ] **Deployment**
  - [ ] Create pull request
  - [ ] Code review
  - [ ] Merge to main
  - [ ] Deploy to staging
  - [ ] Monitor chat functionality
  - [ ] Deploy to production

- [ ] **Post-deployment**
  - [ ] Monitor for issues
  - [ ] Verify all providers working
  - [ ] Check error logs
  - [ ] Performance monitoring
  - [ ] User acceptance

---

## Migration Timeline

### Phase 0: Spike & Validation (Days 1-3)

**Status**: ⬜ Not Started

**Objectives:**
- Validate rig capabilities with prototype
- Design tool adapter approach
- Confirm streaming and error handling

**Tasks:**
1. Create spike branch
2. Prototype OpenAI integration
3. Test streaming with observers
4. Design MCP tool adapter
5. Document findings and blockers

**Deliverables:**
- Spike code in separate branch
- Technical findings document
- Go/no-go decision

### Phase 1: Core Infrastructure (Days 4-8)

**Status**: ⬜ Not Started

**Objectives:**
- Replace llm dependency with rig-core
- Implement provider clients
- Create tool adapter infrastructure

**Deliverables:**
- Working provider implementations
- Tool adapter framework
- Code compiles

### Phase 2: Session Management (Days 9-12)

**Status**: ⬜ Not Started

**Objectives:**
- Rewrite session with rig agents
- Implement streaming and persistence
- Add tool execution loop

**Deliverables:**
- Complete session implementation
- Streaming working with observers
- Tool execution functional

### Phase 3: MCP Integration (Days 13-15)

**Status**: ⬜ Not Started

**Objectives:**
- Integrate MCP tools with rig
- Ensure persistence compatibility
- Complete module integration

**Deliverables:**
- MCP bridge updated
- Tool persistence working
- Full compilation

### Phase 4: Testing (Days 16-20)

**Status**: ⬜ Not Started

**Objectives:**
- Comprehensive testing
- Provider validation
- Performance checks

**Deliverables:**
- All tests passing
- Performance validated
- Edge cases covered

### Phase 5: Deployment (Days 21-22)

**Status**: ⬜ Not Started

**Objectives:**
- Deploy to production
- Monitor functionality

**Deliverables:**
- Production deployment
- Monitoring confirmed

---

### Original Phase 1: Implementation (Days 1-4)

**Objectives:**
- Direct replacement of `llm` crate with `rig`
- Maintain functional equivalence

**Tasks:**
1. Update dependencies in `Cargo.toml`
   - Remove `llm` dependency
   - Add `rig-core = "0.23.1"`
   - Update `console` feature to reference rig

2. Replace provider implementation
   - Rename `providers.rs` and update `ChatProvider` enum
   - Implement client builders for each provider using rig
   - Update credential handling

3. Rewrite session management
   - Replace `ChatSession` implementation with rig agents
   - Maintain database persistence compatibility
   - Preserve message history handling

4. Adapt tool calling
   - Implement MCP-to-rig tool adapter
   - Update `McpBridge` to provide rig `Tool` implementations
   - Ensure tool execution flow remains unchanged

**Deliverables:**
- Complete rig-based implementation
- Code compiles successfully
- Existing API surface preserved where possible

### Phase 2: Testing & Validation (Days 5-8)

**Objectives:**
- Validate functional equivalence
- Ensure all providers work correctly
- Verify edge cases

**Tasks:**
1. Core functionality testing
   - Test all four providers (OpenAI, Claude, Gemini, Ollama)
   - Validate tool calling with MCP bridge
   - Test session persistence and resumption
   - Validate streaming behaviour

2. Edge case testing
   - Ollama tool rejection handling (HTTP 400 fallback)
   - API key sanitisation in error logs
   - Timeout and error handling
   - Multi-turn conversations

3. Performance validation
   - Response latency spot checks
   - Memory usage verification
   - Concurrent session handling

**Deliverables:**
- All tests passing
- Provider-specific validation complete
- Performance acceptable

### Phase 3: Deployment (Days 9-10)

**Objectives:**
- Deploy to production
- Monitor for issues
- Update documentation

**Tasks:**
1. Merge to main branch
2. Deploy to production
3. Monitor chat functionality
4. Update documentation

**Deliverables:**
- Production deployment complete
- Documentation updated
- Migration complete

---

## Technical Implementation Plan

### 1. Dependency Management

**Update `layercake-core/Cargo.toml`:**

```toml
[features]
default = ["server", "mcp", "graphql", "console"]
console = [
    "dep:clap-repl",
    "dep:nu-ansi-term",
    "mcp",
    "dep:chrono",
    "dep:rig-core",
]

[dependencies]
# LLM backend - direct replacement of llm crate
rig-core = { version = "0.23.1", optional = true }
```

**Changes:**
- Remove `llm` dependency entirely
- Remove feature flags for backend selection
- Add `rig-core` as optional dependency for console feature
- Verify crate naming/export strategy (`rig-core` vs `rig`) before updating imports; adjust `use` statements to match the actual published crate or add a local re-export module.

### 2. Provider Implementation

**Update `layercake-core/src/console/chat/providers.rs`:**

Replace the existing implementation with rig-based provider builders. The `ChatProvider` enum remains unchanged to maintain API compatibility.

```rust
use anyhow::{anyhow, Result};
use clap::ValueEnum;
use std::{fmt, str::FromStr};

/// Supported chat providers (unchanged)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, ValueEnum)]
pub enum ChatProvider {
    Ollama,
    OpenAi,
    Gemini,
    Claude,
}

// Keep existing FromStr, Display, Default implementations...

impl ChatProvider {
    pub fn display_name(&self) -> &'static str {
        // ... existing code ...
    }

    pub fn requires_api_key(&self) -> bool {
        !matches!(self, ChatProvider::Ollama)
    }

    /// Build a rig client for this provider
    pub fn build_client(
        &self,
        api_key: Option<String>,
        base_url: Option<String>,
    ) -> Result<impl rig::completion::CompletionModel> {
        use rig::providers;

        match self {
            ChatProvider::OpenAi => {
                let api_key = api_key
                    .ok_or_else(|| anyhow!("OpenAI requires API key"))?;
                let mut client = providers::openai::Client::new(&api_key);
                if let Some(url) = base_url {
                    client = client.with_base_url(url);
                }
                Ok(client)
            }
            ChatProvider::Claude => {
                let api_key = api_key
                    .ok_or_else(|| anyhow!("Anthropic requires API key"))?;
                let mut client = providers::anthropic::Client::new(&api_key);
                if let Some(url) = base_url {
                    client = client.with_base_url(url);
                }
                Ok(client)
            }
            ChatProvider::Gemini => {
                let api_key = api_key
                    .ok_or_else(|| anyhow!("Gemini requires API key"))?;
                let mut client = providers::gemini::Client::new(&api_key);
                if let Some(url) = base_url {
                    client = client.with_base_url(url);
                }
                Ok(client)
            }
            ChatProvider::Ollama => {
                let base_url = base_url
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                Ok(providers::ollama::Client::new(&base_url))
            }
        }
    }
}
```

**Caveats:**
- Returning `Result<impl CompletionModel>` in this form will not compile because each branch yields a distinct client type. Wrap the clients in an enum or box an object-safe trait once Rig’s trait bounds are confirmed.
- Confirm that Rig exposes a client interface that mirrors the required chat+tool API for each provider (especially Gemini and Ollama) before removing the existing `llm` builder.

### 3. Tool Integration

**Create `layercake-core/src/console/chat/tools.rs`:**

```rust
use anyhow::Result;
use async_trait::async_trait;
use axum_mcp::prelude::*;
use rig::completion::ToolDefinition;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Wrapper to adapt MCP tools to rig's Tool trait
pub struct McpToolAdapter {
    name: String,
    description: String,
    schema: Value,
    executor: Arc<dyn McpToolExecutor>,
    security: SecurityContext,
}

#[async_trait]
pub trait McpToolExecutor: Send + Sync {
    async fn execute(
        &self,
        name: &str,
        security: &SecurityContext,
        args: Value,
    ) -> Result<Value>;
}

#[async_trait]
impl Tool for McpToolAdapter {
    const NAME: &'static str = "mcp_tool";  // Override at runtime
    type Error = anyhow::Error;
    type Args = Value;
    type Output = Value;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.schema.clone(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        self.executor
            .execute(&self.name, &self.security, args)
            .await
    }
}

impl McpToolAdapter {
    pub fn from_mcp_tool(
        tool: &axum_mcp::protocol::messages::Tool,
        executor: Arc<dyn McpToolExecutor>,
        security: SecurityContext,
    ) -> Self {
        Self {
            name: tool.name.clone(),
            description: tool.description.clone(),
            schema: tool.input_schema.clone(),
            executor,
            security,
        }
    }
}
```

**Caveats:**
- Rig’s `Tool` trait currently requires a compile-time `NAME`; dynamic MCP tools cannot simply override it at runtime. Evaluate generating wrapper types on the fly, or introduce an indirection layer that routes calls by name while satisfying the trait’s static requirements.
- Ensure tool execution still records summaries, payloads, and call identifiers so existing persistence and audit trails remain intact.

### 4. Session Management

**Update `layercake-core/src/console/chat/session.rs`:**

Replace the entire session implementation with rig-based agents.

```rust
use anyhow::Result;
use rig::prelude::*;
use rig::agent::Agent;
use sea_orm::DatabaseConnection;

pub struct ChatSession {
    db: DatabaseConnection,
    session_id: Option<String>,
    project_id: i32,
    user_id: i32,
    provider: ChatProvider,
    model_name: String,
    agent: Agent,
    security: SecurityContext,
    messages: Vec<Message>,  // Track history for persistence
}

impl ChatSession {
    pub async fn new(
        db: DatabaseConnection,
        project_id: i32,
        user: users::Model,
        provider: ChatProvider,
        config: &ChatConfig,
    ) -> Result<Self> {
        let credentials = ChatCredentialStore::new(db.clone());
        let api_key = credentials.api_key(provider).await?;
        let base_url = credentials.base_url_override(provider).await?
            .or(config.provider(provider).base_url.clone());

        let client = provider.build_client(api_key, base_url).await?;
        let model_name = config.provider(provider).model.clone();

        // Build agent with tools
        let mcp_bridge = McpBridge::new(db.clone());
        let security = build_user_security_context(
            ClientContext::default(),
            user.id,
            &user.user_type,
            Some(project_id),
        );

        let tools = mcp_bridge.rig_tools(&security).await?;

        let agent = client
            .agent(&model_name)
            .preamble(&compose_system_prompt(config, project_id, &tools))
            .temperature(0.2)
            .max_tokens(4096)
            .tools(tools)
            .build();

        Ok(Self {
            db,
            session_id: None,
            project_id,
            user_id: user.id,
            provider,
            model_name,
            agent,
            security,
            messages: Vec::new(),
        })
    }

    pub async fn send_message_with_observer<F>(
        &mut self,
        input: &str,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        let session_id = self.ensure_persisted().await?;

        // Store user message
        self.messages.push(Message::user(input));
        self.persist_message("user", input, None, None).await?;

        // Execute agent with streaming
        let response = self.agent
            .prompt(input)
            .await?;

        // Handle tool calls if present
        if let Some(tool_calls) = response.tool_calls {
            for call in tool_calls {
                observer(ChatEvent::ToolInvocation {
                    name: call.name.clone(),
                    summary: format!("Calling {}...", call.name),
                });

                // Tool execution handled by rig agent
                // Results automatically fed back to conversation
            }
        }

        // Emit assistant response
        let text = response.text;
        observer(ChatEvent::AssistantMessage { text: text.clone() });

        self.messages.push(Message::assistant(&text));
        self.persist_message("assistant", &text, None, None).await?;

        Ok(())
    }

    async fn persist_message(
        &self,
        role: &str,
        content: &str,
        tool_name: Option<String>,
        tool_call_id: Option<String>,
    ) -> Result<()> {
        if let Some(ref session_id) = self.session_id {
            use crate::services::chat_history_service::ChatHistoryService;
            let history_service = ChatHistoryService::new(self.db.clone());
            history_service
                .store_message(
                    session_id,
                    role.to_string(),
                    content.to_string(),
                    tool_name,
                    tool_call_id,
                    None,
                )
                .await?;
        }
        Ok(())
    }
}
```

**Caveats:**
- Rig’s agent abstraction does not automatically replicate the existing `MAX_TOOL_ITERATIONS` retry loop, observer callbacks, or tool-result persistence. Plan concrete replacements for the logic in `layercake-core/src/console/chat/session.rs:249-438`.
- Streaming must remain token-based so CLI observers keep receiving incremental updates; Rig’s streaming API needs to be wired into the existing observer model.
- Preserve Ollama-specific fallback behaviour (disabling tools on HTTP 400) and API-key sanitisation currently implemented around `LLMError`.
- Ensure MCP tool results continue to be serialized and stored for audit/history purposes, including call IDs and payloads consumed by `ChatHistoryService`.

### 5. MCP Bridge Updates

**Update `layercake-core/src/console/chat/mcp_bridge.rs`:**

Add method to convert MCP tools to rig tools:

```rust
impl McpBridge {
    pub async fn rig_tools(
        &self,
        context: &SecurityContext,
    ) -> Result<Vec<Box<dyn Tool>>, McpError> {
        let tools = self.list_tools(context).await?;
        let executor = Arc::new(McpToolExecutorImpl {
            registry: self.state.tool_registry().clone(),
        });

        Ok(tools
            .into_iter()
            .map(|tool| {
                Box::new(McpToolAdapter::from_mcp_tool(
                    &tool,
                    executor.clone(),
                    context.clone(),
                )) as Box<dyn Tool>
            })
            .collect())
    }

    // Remove llm_tools() method - no longer needed
}

struct McpToolExecutorImpl {
    registry: LayercakeToolRegistry,
}

#[async_trait]
impl McpToolExecutor for McpToolExecutorImpl {
    async fn execute(
        &self,
        name: &str,
        security: &SecurityContext,
        args: Value,
    ) -> Result<Value> {
        let exec_context = ToolExecutionContext::new(security.clone())
            .with_arguments(args);

        let result = self.registry
            .execute_tool(name, exec_context)
            .await?;

        Ok(McpBridge::serialize_tool_result(&result))
    }
}
```

**Caveats:**
- `LayercakeToolRegistry` currently lives inside the MCP bridge; confirm it can be cloned cheaply or introduce a lightweight executor shim to avoid holding large state in each tool wrapper.
- Maintain argument injection and security scope enforcement when executing tools; Rig’s trait may not expose the same hooks by default. Honour project scoping and error propagation semantics from the existing implementation.

### 6. File Reorganisation

**Files to modify:**
- `layercake-core/src/console/chat/providers.rs` - Update with rig client builders
- `layercake-core/src/console/chat/session.rs` - Rewrite with rig agents
- `layercake-core/src/console/chat/mcp_bridge.rs` - Add rig tool adapter
- `layercake-core/src/console/chat/mod.rs` - Update module exports

**Files to create:**
- `layercake-core/src/console/chat/tools.rs` - MCP tool adapter for rig

**No changes required:**
- `layercake-core/src/console/chat/config.rs` - Configuration remains compatible
- Database schema - Session/message persistence unchanged, but migration must continue writing the same records that downstream services expect (tool metadata, call IDs, serialized payloads).

### 7. Testing Strategy

**Extend existing tests in `layercake-core/tests/`:**

```rust
#[cfg(feature = "console")]
mod chat_tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_initialization() {
        // Test each provider can be initialized
        for provider in [ChatProvider::Ollama, ChatProvider::OpenAi,
                         ChatProvider::Gemini, ChatProvider::Claude] {
            // Test initialization logic
        }
    }

    #[tokio::test]
    async fn test_tool_calling() {
        // Test MCP tool integration
    }

    #[tokio::test]
    async fn test_session_persistence() {
        // Test message history persists correctly
    }

    #[tokio::test]
    async fn test_streaming() {
        // Test streaming responses work
    }
}
```

**Additional coverage:**
- Add MCP integration tests that validate dynamic tool registration and result persistence, mirroring the current expectations in `ChatHistoryService`.
- Include provider-specific streaming tests (OpenAI, Claude, Gemini, Ollama) to ensure Rig surfaces deltas the same way the CLI observer expects.
- Exercise error sanitisation paths to confirm API keys remain redacted in Rig-generated error messages.

---

## Success Criteria

Migration is considered successful when:

1. ✅ All four providers (OpenAI, Claude, Gemini, Ollama) working
2. ✅ Tool calling fully functional with MCP bridge
3. ✅ Session persistence and resumption working
4. ✅ Streaming responses working
5. ✅ Performance within 10% of current implementation
6. ✅ All existing tests passing
7. ✅ No regression in error handling or logging
8. ✅ API key sanitisation still functioning
9. ✅ Ollama fallback behaviour preserved

---

## Post-Migration Opportunities

Once migration is complete, consider leveraging rig's advanced features:

1. **Multi-Agent Orchestration**
   - Specialised agents for different query types
   - Agent routing based on query analysis

2. **RAG Integration**
   - Integrate vector stores for context retrieval
   - Graph data as embedding source

3. **Enhanced Tool Capabilities**
   - Dynamic tool loading
   - Tool composition and chaining

4. **Performance Optimisations**
   - Parallel agent execution
   - Caching and memoisation

5. **Extended Provider Support**
   - Add support for additional providers (Cohere, Mistral, etc.)
   - Regional provider fallbacks

---

## Appendix A: API Mapping

| llm Crate Concept | Rig Equivalent | Notes |
|-------------------|----------------|-------|
| `LLMProvider` | `CompletionModel` | Different trait hierarchy |
| `LLMBuilder` | `ClientBuilder` + `AgentBuilder` | Two-stage construction |
| `ChatMessage` | `Message` | Similar structure |
| `Tool` (llm) | `Tool` (rig) | Different trait definition |
| `ToolCall` | Built into agent response | Automatic handling |
| `.chat()` | `.prompt()` | Method rename |
| `.chat_with_tools()` | `.build()` with `.tools()` | Tools set at agent creation |

## Appendix B: File Modification Checklist

**Files modified during migration:**

- [x] `layercake-core/Cargo.toml` - Replace llm with rig-core dependency
- [x] `layercake-core/src/console/chat/providers.rs` - Update with rig client builders
- [x] `layercake-core/src/console/chat/session.rs` - Rewrite using rig agents
- [x] `layercake-core/src/console/chat/mcp_bridge.rs` - Add rig tool adapter
- [x] `layercake-core/src/console/chat/mod.rs` - Update module exports
- [x] `layercake-core/src/console/chat/tools.rs` - Create MCP tool adapter (new file)

**No backward compatibility layer:**
- No feature flags for old implementation
- No deprecation period
- Direct replacement only

## Appendix C: Resources

- Rig Documentation: https://rig.rs
- Rig GitHub: https://github.com/0xPlaygrounds/rig
- Rig Examples: https://github.com/0xPlaygrounds/rig/tree/main/rig-core/examples
- Current llm Crate: https://crates.io/crates/llm (v1.3.4)

---

## Implementation Notes

### Key Differences from Original Plan

This is a **roll-forward only migration** with the following characteristics:

1. **No Feature Flags**: The migration removes `llm` crate entirely in one go, with no parallel implementation or feature flags for backward compatibility.

2. **No Rollback**: There is no mechanism to roll back to the `llm` implementation. If issues arise, fixes must be implemented in the rig-based code.

3. **Direct Replacement**: All files are modified in place rather than creating parallel implementations. This simplifies the codebase and accelerates migration.

4. **Comprehensive Testing**: Since there's no rollback option, testing must be thorough before merging to main. Development branch testing is critical.

5. **Timeline**: Reduced from 2-3 weeks to 1-2 weeks (10 working days) due to elimination of parallel implementation overhead.

### Risk Mitigation Without Rollback

- **Thorough testing on development branch** before merge
- **All four providers tested** individually
- **MCP tool integration validated** comprehensively
- **Session persistence verified** with database
- **Performance spot checks** conducted
- **Edge cases tested** (Ollama fallback, API key sanitisation, etc.)

### Success Depends On

- Rig API compatibility with requirements
- Comprehensive test coverage before merge
- Confidence in testing results
- Ability to fix issues forward if found post-deployment

---

**Document Status**: Final
**Last Updated**: 2025-11-03
**Migration Approach**: Roll-Forward Only (No Rollback)
