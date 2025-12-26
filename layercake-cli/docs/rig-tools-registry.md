# Rig Tools Registry

This document tracks Rig tool bindings that replace the legacy MCP tool set.

## Core Tool Targets (Stage 1)

| Tool | Service Target | Status |
| --- | --- | --- |
| `list_projects` | `ProjectService::list_projects` | Planned |
| `get_project` | `ProjectService::get_project` | Planned |
| `list_graphs` | `GraphService::list_graphs` | Planned |
| `get_graph` | `GraphService::get_graph` | Planned |

## Notes

- Tool bindings should live under `layercake-cli/src/console`.
- Remote mode should proxy these tools to server GraphQL endpoints.
- Keep tool payloads JSON-serializable and stable for agent usage.
