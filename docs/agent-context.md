# Agent Context Modernisation Plan

This note reviews how we currently construct Rig agents, highlights gaps in the existing system prompt, and outlines a concrete plan (including a new prompt template) to ensure every agent carries the full Layercake mental model: plan DAG semantics, node/graph execution, artefact generation, and per-project context.

## 1. Current State

### 1.1 Rig usage
- `layercake-core/src/console/chat/session.rs` builds providers with `rig::agents::AgentBuilder`, but only feeds a minimal prompt string and does **not** project the stored conversation/plan metadata into structured `Message` objects. Tool definitions are advertised, yet the builder never receives the complete engineering context.
- `compose_system_prompt` in the same file only appends `"Current project ID: {project_id}"` to any static text from `ChatConfig`. No project structure, DAG summary, or execution guidance reaches the LLM.
- MCP/Rig integration does not currently fetch plan DAG state, graph stats, or artefact inventories before the first turn, so every agent is flying blind and must rediscover how Layercake works from user chatter.

### 1.2 System prompt gaps
- Agents have no canonical description of the plan DAG itself (node types, dependencies, execution ordering) or how datasets propagate into graph construction.
- Artefact creation (graph/tree nodes, exports) is undocumented in the prompt despite being a critical user-visible feature.
- Current project data (name, tags, last run, plan summary) is missing, so multi-project operators cannot rely on the assistant to remember which DAG or dataset set they are editing.

## 2. Requirements for Rig Agents

Every Rig-powered agent MUST:
1. Recall the plan DAG lifecycle: node metadata, execution order, dataset ingestion, merge/transform nodes, and how graphs are materialised into `graph_nodes`, `graph_edges`, and `graph_layers`.
2. Describe graph structures and artefact generation (Graph/Tree nodes, export previews, library publishing) without rediscovering them from scratch.
3. Operate with explicit knowledge of the current project (ID, name, tags, latest plan revision, high-level stats) so responses are scoped correctly.
4. Reference available tooling (MCP + Rig tool calls) confidently, including limitations (e.g., maximum tool iterations, when to fall back).

## 3. Proposed System Prompt Template

Populate dynamic tokens (`{{...}}`) from database/services before invoking Rig:

```text
You are a senior Layercake engineer assisting with project {{project_id}} ({{project_name}}).

Plan DAG model:
- Nodes: DataSetNode (ingest existing dataset by ID), GraphNode (build graph from upstream datasets/graphs), MergeNode (combine upstream graphs), TransformNode/FilterNode (post-process graphs), GraphArtefactNode/TreeArtefactNode (export/visualise outputs), Chat nodes, etc.
- Execution happens through DagExecutor (resolves dependencies, executes nodes + affected downstream nodes).
- Graph materialisation stores rows in graph_nodes / graph_edges / graph_layers with dataset provenance.

Graph construction & artefacts:
- GraphBuilder builds graphs from datasets (handles duplicate edge IDs, layer metadata).
- MergeBuilder merges graphs/datasets; enforce unique edge IDs per graph.
- Artefact nodes (GraphArtefactNode/TreeArtefactNode) export previews (Mermaid, DOT, JSON) and support library publishing.
- Library supports datasets, full projects, and project templates via library_items table.

Project context:
- Name: {{project_name}}
- Tags: {{project_tags}}
- Active plan: {{plan_name}} ({{plan_node_count}} nodes / {{plan_edge_count}} edges, last updated {{plan_updated_at}})
- Graph stats: {{graph_summary}}
- Dataset summary: {{dataset_summary}}

Instructions:
1. Always ground responses in the Layercake architecture above.
2. When planning changes, describe which subsystems (plan DAG, datasets, graphs, artefacts, collaboration) are touched and why.
3. When execution steps are required, detail node execution order and downstream graph impact.
4. Prefer Rig/MCP tool usage for repository inspection, file edits, or running evaluations; describe the tool intent before calling it.
5. Keep answers scoped to project {{project_id}} unless the user explicitly switches context.
```

## 4. Implementation Plan

1. **Prompt Assembly**
   - Extend `compose_system_prompt` (`layercake-core/src/console/chat/session.rs`) to fetch project metadata (name, description, tags) plus plan DAG summary via existing services (`ProjectService`, `PlanDagService`, `GraphService`).
   - Inject repository + pipeline descriptions as static sections (stored under `resources/system-prompts/agent-context.md` or similar) to avoid duplicating strings in code.
   - Populate dynamic placeholders (`{{project_*}}`, `{{plan_*}}`, etc.) before handing the final string to the Rig agent builder.

2. **Context Fetchers**
   - Add helper(s) in `ChatSession` to load:
     - Project record (name, tags, updatedAt).
     - Latest plan DAG statistics (`PlanDagService::load_plan_dag`).
     - Aggregate dataset/graph stats (reuse `projectStats` resolver logic by calling the same service layer).
   - Cache these summaries per session turn to avoid repeated DB hits.

3. **Rig Builder Wiring**
   - Ensure `call_rig_agent` (session.rs) uses the enriched prompt when calling `.preamble(...)`.
   - Convert chat history to structured Rig `Message` instances so the new prompt isn’t flattened into user text.
   - Confirm `.rmcp_tools` continues to advertise Layercake tools; add documentation of expected tools within the prompt’s instructions only for human readability (not as a compatibility hack).

4. **Observability & Validation**
   - Trace when enriched prompts are built and log high-level stats (project name, node counts) to verify coverage.
   - Add integration test coverage (or snapshot test) ensuring `compose_system_prompt` outputs all key sections when fed mock project/plan metadata.
   - Update developer docs (`docs/RIG-TOOL-PLAN.md`) to reference this `docs/agent-context.md` so prompt evolution is centralised.

Delivering the above guarantees that every Rig agent shares a consistent, richly detailed Layercake mental model before the first tool call, dramatically reducing redundant user exposition and improving plan/DAG reasoning quality.
