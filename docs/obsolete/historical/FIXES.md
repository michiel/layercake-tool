## Plan: Project-Scoped Chat Sessions

1. **Establish authenticated project context**
   - Replace the dummy “default user” plumbing in chat entry points with the actual session/user extracted via `AuthorizationService`.
   - Persist the active user’s project membership in the GraphQL context so downstream services can verify access.

2. **Scope chat runtime + MCP usage**
   - Adjust the console/chat session bootstrap to mint a `SecurityContext` bound to the authenticated user and project (no more `SecurityContext::system()`).
   - Ensure tool lists/execute paths respect the scoped context and hide blacklisted project-management tools from chats.

3. **Persist and render vetted chat history**
   - Store assistant messages with the already-formatted summary instead of raw tool payloads; keep structured tool data separately if needed.
   - Update the React client to read existing sessions, display message history, and send replies without creating duplicate sessions.

4. **Tighten MCP agent flows (blockers)**
   - Pass the calling user’s ID into `create_mcp_agent`/`revoke_mcp_agent`/`regenerate_mcp_agent_key`, reusing the authorization checks.
   - Add minimal coverage or manual verification steps so project-scoped agents can authenticate and operate only within their project.

5. **Regression guardrails**
   - Re-enable or replace the currently-disabled service tests (fix migration harness or add integration tests using `cargo test -p layercake-core` harness).
   - Smoke-test the chat UI and MCP access paths end-to-end for a sample project before merging.
