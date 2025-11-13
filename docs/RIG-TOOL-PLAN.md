# Technical Plan: Refactoring Tool Use with `rig`

This document outlines a technical plan for refactoring the tool-use implementation in `layercake-tool` to leverage the built-in features of the `rig` crate. This will result in a more robust, reliable, and maintainable implementation that is compatible with all supported LLMs (OpenAI, Anthropic, Gemini, and Ollama).

## 1. Project Overview

The current implementation of tool use in `layercake-tool` relies on a custom and brittle approach to parsing tool invocations from the raw text response of the LLM. This approach is not ideal and does not take full advantage of the `rig` crate's built-in tooling features.

This plan proposes a refactoring of the `ChatSession` implementation to use `rig`'s tooling features more effectively. This will involve passing tool definitions to the `rig` agent and handling structured tool calls from the agent's response.

## 2. High-Level Plan

The refactoring will be done in the following phases:

1.  **Refactor `ChatSession` to use `rig`'s tool-use features.**
2.  **Extend the solution to all supported LLMs.**
3.  **Clean up the code by removing redundant functions.**

## 3. Detailed Technical Plan

### Phase 1: Refactor `ChatSession` for `rig` Tool-Use

**File:** `layercake-core/src/console/chat/session.rs`

1.  **Update `call_rig_agent` to Pass Tool Definitions:**
    *   Modify the `call_rig_agent` function to pass the `rmcp_tools` to the `rig` agent.
    *   The `rig` agent builder has a `rmcp_tools` method that can be used for this purpose.

    ```rust
    // In `call_rig_agent` function:

    // For each provider (OpenAI, Anthropic, Gemini, Ollama)
    let builder = client.agent(model);

    #[cfg(feature = "rmcp")]
    if let Some(ref rmcp_client) = self.rmcp_client {
        if !self.rmcp_tools.is_empty() {
            builder = builder.rmcp_tools(self.rmcp_tools.clone(), rmcp_client.peer().to_owned());
        }
    }

    let agent = builder.build();
    ```

2.  **Update `resolve_conversation` to Handle Structured Tool Calls:**
    *   The `invoke_agent_with_retries` function returns a `String`. This should be changed to return a `rig::completion::CompletionResponse`.
    *   The `resolve_conversation` function should be updated to handle the `CompletionResponse`.
    *   The `CompletionResponse` will contain a `tool_calls` field with a list of `rig::message::ToolCall` objects.

3.  **Update `handle_tool_invocation` to Use `rig::message::ToolCall`:**
    *   Modify the `handle_tool_invocation` function to accept a `rig::message::ToolCall` object instead of a `ParsedToolInvocation` object.
    *   The `ToolCall` object will contain the `name` and `arguments` of the function to be called.

    ```rust
    // In `resolve_conversation` function, after getting the response from the agent:
    if !response.choice.is_empty() {
        let choice = response.choice.first();
        if let Some(AssistantContent::ToolCall(tool_call)) = choice {
            self.handle_tool_invocation(tool_call, observer).await?;
            continue;
        }
    }

    // ...

    // Modify the signature of `handle_tool_invocation`
    async fn handle_tool_invocation<F>(
        &mut self,
        tool_call: rig::message::ToolCall,
        observer: &mut F,
    ) -> Result<()>
    where
        F: FnMut(ChatEvent),
    {
        // ...
        let execution_args = tool_call.function.arguments.clone();
        let result = self
            .bridge
            .execute_tool(&tool_call.function.name, &self.security, execution_args.clone())
            .await
            // ...
    }
    ```

### Phase 2: Extend the Solution to All Supported LLMs

The changes in Phase 1 should be applied to all supported LLM providers:

*   **OpenAI:** The `rig` crate has excellent support for OpenAI, so the changes should be straightforward.
*   **Anthropic:** The `rig` crate supports Anthropic, and the changes should be similar to OpenAI.
*   **Gemini:** The `rig` crate supports Gemini, and the changes should be similar to OpenAI.
*   **Ollama:** The `rig` crate has specific support for Ollama's tooling features, so the changes should work seamlessly.

### Phase 3: Code Cleanup

1.  **Remove Redundant Functions:**
    *   Remove the `extract_tool_invocation` and `parse_tool_command` functions from `session.rs`. These functions are no longer needed, as the `rig` crate handles the parsing of structured tool calls.

2.  **Simplify the System Prompt:**
    *   Modify the `compose_system_prompt` function to remove the list of tool names. The tool definitions are now passed to the `rig` agent separately, so there's no need to include them in the system prompt.

## 4. Testing

After implementing the changes, it's crucial to test the tool-use functionality with all supported LLMs. The tests should cover:

*   **Single tool calls:** Verify that the LLM can call a single tool with the correct arguments.
*   **Multiple tool calls:** Verify that the LLM can call multiple tools in a single turn.
*   **Tool calls with no arguments:** Verify that the LLM can call tools that don't have any arguments.
*   **Error handling:** Verify that the system handles errors gracefully when a tool call fails.

## 5. Conclusion

By following this technical plan, the developers of `layercake-tool` can create a much more robust, reliable, and maintainable tool-use implementation for their chat feature. This will improve the user experience and make it easier to support new LLM providers and tools in the future.


