# Separation of Concerns Module Audit

This audit captures the current top-level module placement after Stage 1 moves.

## layercake-core (business logic)

- `app_context/`, `database/`, `services/`, `pipeline/`, `plan_dag/`, `graph.rs`, `plan.rs`
- `export/`, `plan_execution.rs`, `sequence_context.rs`, `story_types.rs`

## layercake-server (server/GraphQL)

- `server/`, `graphql/`, `collaboration/`, `mcp/`, `chat/`

## layercake-cli (CLI/console)

- `console/`, `chat_credentials_cli/`, `main.rs`

## Notes

- Core no longer owns GraphQL, server, or console modules.
- Remaining Stage 1 work: CoreError/Actor integration and error baselines.
