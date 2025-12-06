# Rig-Core 0.25.0 Upgrade Plan

**Date:** 2025-12-02
**Status:** In Progress
**Rig Version:** 0.25.0 (upgraded from 0.24.x)

## Executive Summary

The codebase has been successfully upgraded to rig-core 0.25.0. This document outlines the new features available, identifies opportunities for improvement, and provides a phased implementation plan to leverage the new capabilities.

**Key Takeaway:** No breaking changes affect current code, but significant new features (especially structured output) can improve reliability and reduce parsing errors in dataset generation.

---

## Changelog Summary (v0.25.0 - December 1, 2025)

### New Features
1. **Gemini Assistant Image Responses** (#1048) - Enhanced image handling capabilities
2. **Generation Config Enhancement** - Added `response_json_schema` to GenerationConfig (#1077)
3. **Provider Client Consolidation** (#1050) - Streamlined client initialisation across providers

### Key Bug Fixes
- Fixed Gemini configuration errors (#1094)
- Fixed OpenAI structured output required properties (#1090)
- Fixed RMCP derive clone issues (#1080)
- Addressed model/agent initialisation inconsistencies (#1069)

### Important Changes
- ✅ **No deprecated APIs in use** - Deprecated `DynClientBuilder` is not used in the codebase
- Added `Content-Type: application/json` header to HTTP requests (#1106)
- **Security:** Fixed potential API key leaks in client headers (#1102)
- Removed outdated models and unused chatbot module
- Enhanced request modelling for all providers (#1067)

---

## Current Rig Usage in Layercake

### Primary Usage Locations

1. **layercake-genai/src/embeddings.rs** (Lines 2-4)
   - Uses: `EmbeddingsClient`, `EmbeddingModel`, `openai::Client`, `ollama::Client`
   - Purpose: Embedding document chunks for RAG
   - Status: ✅ Working, no changes needed

2. **layercake-genai/src/dataset_generation.rs** (Lines 2-3)
   - Uses: `CompletionClient`, `Prompt`, `openai::Client`
   - Purpose: AI-powered dataset generation
   - Status: ⚠️ Can benefit from structured output

3. **layercake-core/examples/rig_spike.rs**
   - Uses: Various rig features for testing and validation
   - Purpose: Integration testing and API exploration
   - Status: ✅ Working, reference implementation

4. **layercake-core/src/console/chat/**
   - RAG context building and chat functionality
   - Integration with MCP bridge
   - Status: ✅ Working, no changes needed

---

## Implementation Plan

### ✅ Phase 0: Prerequisite Validation (COMPLETE)
- [x] Confirm upgrade to rig-core 0.25.0
- [x] Verify no deprecated APIs in use
- [x] Audit current rig usage patterns
- [x] Document breaking changes (none found)

### 🔨 Phase 1: Structured Output for Dataset Generation (HIGH PRIORITY)

**Timeline:** Week 1 (3-5 days)
**Impact:** HIGH - Eliminates parsing errors, improves reliability

#### Current State
```rust
// layercake-genai/src/dataset_generation.rs:39
let completion = agent.prompt(prompt_text).await?;
// Returns unstructured text that needs parsing
```

#### Target State
```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
struct DatasetOutput {
    nodes: Vec<NodeDefinition>,
    edges: Vec<EdgeDefinition>,
    metadata: DatasetMetadata,
}

// Configure agent with response schema
let agent = self.openai
    .agent("gpt-4o-mini")
    .preamble("You are a data acquisition specialist...")
    .generation_config(GenerationConfig {
        response_json_schema: Some(json!({
            "type": "object",
            "properties": {
                "nodes": { "type": "array", "items": {...} },
                "edges": { "type": "array", "items": {...} },
                "metadata": {...}
            },
            "required": ["nodes", "edges"]
        })),
        ..Default::default()
    })
    .build();
```

#### Benefits
- ✅ Guaranteed valid JSON/YAML output
- ✅ Eliminates parsing errors
- ✅ Better validation and error handling
- ✅ Type-safe deserialization
- ✅ Aligns with RAG-plan.md goals

#### Tasks
- [ ] Define `DatasetOutput` schema structs
- [ ] Create JSON schema for dataset format
- [ ] Update `DatasetGenerator::run()` to use structured output
- [ ] Add deserialization logic
- [ ] Update tests to validate schema compliance
- [ ] Document schema format

**Files to Modify:**
- `layercake-genai/src/dataset_generation.rs`
- Add new file: `layercake-genai/src/dataset_schema.rs`

---

### 🛡️ Phase 2: Security Audit (HIGH PRIORITY)

**Timeline:** Week 1 (1-2 days)
**Impact:** CRITICAL - Prevent API key exposure

#### Audit Checklist

1. **Header Manipulation Review**
   - [ ] Check `layercake-genai/src/services/mod.rs`
   - [ ] Check `layercake-core/src/console/chat/providers.rs`
   - [ ] Verify no custom headers expose secrets

2. **Logging Review**
   - [ ] Audit all rig client error handling
   - [ ] Ensure API keys never logged
   - [ ] Verify error messages redact sensitive data
   - [ ] Check tracing/debug output

3. **Error Handling**
   - [ ] Review `embeddings.rs` error paths (lines 84-92, 100-109)
   - [ ] Review `dataset_generation.rs` error handling
   - [ ] Sanitise error messages before display

4. **Tauri Integration**
   - [ ] Review Tauri command handlers
   - [ ] Verify secrets not exposed in IPC
   - [ ] Check frontend error displays

**Files to Audit:**
- `layercake-genai/src/embeddings.rs`
- `layercake-genai/src/dataset_generation.rs`
- `layercake-genai/src/services/mod.rs`
- `layercake-core/src/console/chat/providers.rs`
- All Tauri command handlers using rig

**Security Best Practices:**
```rust
// ❌ Bad - may log API key
.map_err(|e| {
    tracing::error!("Failed: {}", e);
    e
})

// ✅ Good - sanitised error
.map_err(|e| {
    tracing::error!("OpenAI API error (details redacted)");
    anyhow::anyhow!("Embedding generation failed")
})
```

---

### 🔧 Phase 3: Client Initialisation Review (MEDIUM PRIORITY)

**Timeline:** Week 2 (1 day)
**Impact:** MEDIUM - Code quality, maintainability

#### Current Patterns (Good)
```rust
// embeddings.rs - Correct pattern
let client = openai::Client::from_env();
let model = client.embedding_model("text-embedding-3-small");
```

#### Tasks
- [ ] Audit all provider client instantiation
- [ ] Verify consistent patterns across codebase
- [ ] Document standard initialisation patterns
- [ ] Update examples if needed

**Benefit:** Leverages #1050 (provider client consolidation) improvements

---

### 🎯 Phase 4: Enhanced RAG Integration (MEDIUM PRIORITY)

**Timeline:** Weeks 2-3
**Impact:** MEDIUM - Improves RAG reliability

Aligns with RAG-plan.md Phase 5 (Dataset generation workflows)

#### Opportunities

1. **Structured Embedding Metadata**
   - Use JSON schemas for embedding metadata
   - Validate chunk structure before storage
   - **Files:** `layercake-genai/src/embeddings.rs`

2. **Multi-Provider Support**
   - Leverage consolidated client init
   - Add Ollama embedding fallback for offline work
   - Configure provider priority
   - **Alignment:** RAG-plan.md mentions "local models via rig provider abstraction"

#### Tasks
- [ ] Define metadata schema for embeddings
- [ ] Implement provider fallback logic
- [ ] Add configuration for provider selection
- [ ] Test offline embedding workflows

---

### 🚀 Phase 5: Future Enhancements (LOW PRIORITY)

**Timeline:** Month 2+
**Impact:** LOW-MEDIUM - New capabilities

#### 1. Gemini Integration

**Feature:** Gemini assistant image responses (#1048)

**Potential Use Cases:**
- Image analysis in uploaded files
- Visual dataset generation (diagrams, charts)
- Multi-modal RAG contexts

**Prerequisites:**
- Add Gemini provider support alongside OpenAI/Ollama
- Design multi-modal chunk storage in `kb_documents` table
- Evaluate Gemini API pricing/availability

#### 2. Advanced Generation Control

**Opportunities:**
- Explore other GenerationConfig options
- Fine-tune response formats per use case
- Temperature/top_p configuration for different scenarios

**Dependencies:** User feedback from earlier phases

---

## Testing Strategy

### Unit Tests

```bash
# Test structured output
cargo test --package layercake-genai dataset_generation

# Test embeddings
cargo test --package layercake-genai embeddings

# Test security (no API key leaks in logs)
cargo test --package layercake-genai -- --nocapture 2>&1 | grep -i "sk-"
```

### Integration Tests

```bash
# Run rig integration spike
cargo run --example rig_spike

# Test full RAG workflow
cargo test --package layercake-core rag_integration_test
```

### Security Testing

1. **Log Review**
   - Run tests with `RUST_LOG=debug`
   - Grep output for API keys/secrets
   - Validate no sensitive data in errors

2. **Error Scenario Testing**
   - Test invalid API keys
   - Test network failures
   - Verify error messages are sanitised

3. **Header Inspection**
   - Monitor HTTP requests (if possible)
   - Verify headers don't leak secrets

---

## Breaking Changes Assessment

**Status:** ✅ **No breaking changes affecting current codebase**

| Change | Status | Impact |
|--------|--------|--------|
| `DynClientBuilder` deprecated | ✅ Not used | None |
| Client consolidation | ✅ Compatible | Positive (cleaner API) |
| Generation config changes | ✅ Additive | None (new features) |
| API key header fix | ✅ Transparent | Security improvement |

**Conclusion:** Migration is purely additive - new features available but not required.

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| Structured output parsing errors | MEDIUM | Comprehensive schema validation, fallback to text parsing |
| API key exposure during audit | HIGH | Careful log review, automated secret scanning |
| Performance impact of JSON validation | LOW | Benchmark before/after, lazy validation |
| Breaking existing dataset formats | MEDIUM | Version schemas, support legacy format |

---

## Success Metrics

### Phase 1 (Structured Output)
- [ ] 100% of generated datasets validate against schema
- [ ] Zero parsing errors in production
- [ ] <50ms overhead for JSON validation

### Phase 2 (Security)
- [ ] Zero API keys in logs (verified by grep)
- [ ] All error messages sanitised
- [ ] Security audit checklist 100% complete

### Phase 3 (Client Init)
- [ ] Consistent patterns across all providers
- [ ] Documentation updated
- [ ] Examples pass all tests

---

## Rollout Plan

### Week 1
- **Days 1-3:** Implement structured output (Phase 1)
- **Days 4-5:** Security audit (Phase 2)
- **Continuous:** Testing and validation

### Week 2
- **Days 1-2:** Client initialisation review (Phase 3)
- **Days 3-5:** Enhanced RAG integration (Phase 4)
- **Continuous:** Integration testing

### Week 3+
- **Ongoing:** Monitor production usage
- **Ongoing:** Collect user feedback
- **Future:** Evaluate Gemini integration (Phase 5)

---

## Documentation Updates

### Files to Update
- [ ] `docs/RAG-plan.md` - Reference structured output
- [ ] `docs/ARCHITECTURE.md` - Document rig integration patterns
- [ ] `README.md` - Update rig-core version notes
- [ ] `layercake-genai/README.md` - Dataset schema format

### New Documentation
- [ ] `docs/dataset-schema.md` - Structured output schema reference
- [ ] `docs/security-guidelines.md` - API key handling best practices

---

## References

- [Rig-Core GitHub](https://github.com/0xPlaygrounds/rig)
- [Rig-Core CHANGELOG](https://github.com/0xPlaygrounds/rig/blob/main/rig-core/CHANGELOG.md)
- [Rig-Core Documentation](https://docs.rs/rig-core)
- Internal: `docs/RAG-plan.md`
- Internal: `docs/RIG-TOOL-PLAN.md`

---

## Appendix: API Examples

### Before (Current)
```rust
// Unstructured text response
let completion = agent.prompt(prompt_text).await?;
// Manual parsing required, error-prone
```

### After (Structured Output)
```rust
// Type-safe structured response
let response: DatasetOutput = agent
    .prompt(prompt_text)
    .with_schema::<DatasetOutput>()
    .await?;
// Direct access to validated data
```

---

## Status Tracking

| Phase | Status | Start Date | Completion Date | Notes |
|-------|--------|------------|-----------------|-------|
| Phase 0 | ✅ Complete | 2025-12-02 | 2025-12-02 | No breaking changes found |
| Phase 1 | ✅ Complete | 2025-12-02 | 2025-12-02 | Structured output + rig client migration |
| Phase 2 | ⏳ Pending | - | - | Security audit |
| Phase 3 | ⏳ Pending | - | - | Client review |
| Phase 4 | ⏳ Pending | - | - | RAG enhancements |
| Phase 5 | 📋 Planned | - | - | Future features |

---

## Phase 1 Completion Summary

**Completed:** 2025-12-02
**Commits:** 2f135308, 9c4310c1, 4355f586, 6c0c57f9, 243a9080

### Delivered
- ✅ JSON schema for dataset generation (dataset_schema.rs)
- ✅ Type-safe Graph, Node, Edge, Layer structures
- ✅ YAML serialization support
- ✅ Updated DatasetGenerator with structured output
- ✅ Rig-core 0.25 client API migration (ProviderClient trait)
- ✅ Codebase-wide rig client migration (all providers)
- ✅ RMCP version conflict resolved (0.8.5 → 0.9.1)
- ✅ Ollama environment variables fixed (OLLAMA_API_BASE_URL)
- ✅ Ollama embedding dimensions specified (768 for nomic-embed-text)
- ✅ All tests passing (202 core + 7 data-acquisition)

### Files Modified
- `layercake-genai/src/dataset_schema.rs` (new, 330 lines)
- `layercake-genai/src/dataset_generation.rs` (enhanced)
- `layercake-genai/src/services/mod.rs` (rig API updates, Ollama env vars)
- `layercake-genai/src/embeddings.rs` (Ollama embedding dimensions)
- `layercake-genai/src/lib.rs` (module export)
- `layercake-genai/Cargo.toml` (serde_yaml dependency)
- `layercake-core/src/console/chat/session.rs` (all provider clients migrated)
- `layercake-core/Cargo.toml` (rmcp 0.9.1 upgrade)

### Migration Details
**Pattern Established:** DB credentials → temp env vars → from_env()
- OpenAI: Sets OPENAI_API_KEY and OPENAI_BASE_URL
- Anthropic: Sets ANTHROPIC_API_KEY and ANTHROPIC_BASE_URL
- Gemini: Sets GOOGLE_API_KEY and GEMINI_BASE_URL
- Ollama: Sets OLLAMA_API_KEY (placeholder) and OLLAMA_API_BASE_URL
  - Note: Defaults to http://localhost:11434 if no custom URL configured

**RMCP Version Fix:** Upgraded from 0.8.5 to 0.9.1 to match rig-core's
dependency, resolving type conflicts between two rmcp versions.

**Ollama Embeddings Fix:** Rig-core 0.25 requires explicit dimensions for
Ollama embedding models. Changed from `embedding_model(model)` to
`embedding_model_with_ndims(model, 768)` for nomic-embed-text models.

### Next Steps
- **Phase 2:** Security audit of API key handling
- **Testing:** Integration test with live OpenAI API (optional)
- **Documentation:** Update user guides for new schema format

---

**Last Updated:** 2025-12-02
**Next Review:** Before Phase 2 start
