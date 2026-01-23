# Code & Infra Correlation Plan

## Objectives
- Extend `layercake-code-analysis` to ingest IaC (Terraform, CloudFormation/SAM, Bicep, CDK Py/TS) and normalize resources into a Resource Graph.
- Correlate application code (Python/JS/TS) with infra resources (env vars, handlers, imports) and emit a unified graph/dataset consumable by Layercake.
- Keep compatibility with existing code-analysis outputs (dataset graph + markdown annotations) and reuse the graph serialization pipeline.

## Scope & Deliverables
- New infra scanner module in `layercake-code-analysis` with format-specific parsers and a shared `ResourceNode`/`GraphEdge` model.
- InfrastructureGraph builder that normalizes IDs, resource types, source file paths (relative), and dependencies.
- Correlation linker that maps code analysis findings to infra resources (env var bindings, handler files, resource references).
- Markdown reporting section summarizing detected resources, unresolved references, and correlation matches.
- Fixture-backed tests for each supported format and end-to-end correlation tests.
- Graph adapter that converts infra + code correlation into the Layercake `Graph` struct for dataset import, with proper layers/attributes and annotations.

## Work Plan
1) Modeling & Plumbing
   - Add infra domain structs (`ResourceType`, `ResourceNode`, `EdgeType`, `GraphEdge`, `InfrastructureGraph`) with serde support and slugified IDs consistent with code graphs.
   - Define a `CorrelationReport` bridging code `AnalysisResult` to infra resources (matches, orphans, unresolved refs).
   - Expose new public entrypoints (e.g., `analyze_infra` + `correlate_code_infra`) and wire into the CLI/Graph builder.
2) Parsers by Format
   - Terraform: use `hcl-rs` to extract resources, names, properties, and `depends_on`/attribute refs.
   - CloudFormation/SAM: use `serde_yaml` to parse `Resources`, handle intrinsic refs (`Ref`, `GetAtt`) as edges.
   - Bicep: use `tree-sitter-bicep` queries to locate `resource` declarations, names, types, and dependencies.
   - CDK Python: extend existing rustpython traversal to detect construct instantiations (module + class) and logical IDs.
   - CDK TypeScript: integrate `oxc_parser`/`swc` to find `new s3.Bucket(...)` patterns and extract scope/id/type.
3) Graph Construction
   - Normalize all parsed resources into `ResourceNode`s; add edges for `DependsOn`, implicit `References`, and logical grouping.
   - Ensure adjacency validation (no dangling IDs) and deterministic ordering for serialization.
   - Serialize to Layercake `Graph` using existing graph adapter (layers for infra/dependency/reference).
4) Correlation Logic
   - Extend code analyzers to surface env var usage and handler file paths (Python/JS/TS).
   - Link code findings to infra nodes (env var names, handler filenames, module names) and emit `CorrelationReport`.
   - Add `CodeLink` edges between code nodes and infra nodes; include unresolved matches in the annotation.
5) Reporting & Dataset Output
   - Update markdown reporter to add infra summary, correlation matches, and unresolved items.
   - Attach infra graph and correlation results to the code analysis dataset/annotation without duplicating attributes.
6) Layercake Graph Transformation
   - Add an adapter that maps `InfrastructureGraph` + correlation edges into `layercake_core::graph::Graph`:
     - Node layers: `infra` (resources), `infra-partition` (modules/dirs), `codelink` (links), reuse existing code layers for call/dataflow.
     - Edge layers: `infra-depends`, `infra-ref`, `infra-code-link` (code→infra), ensuring IDs are slugified, lowercase, unique, non-partition endpoints where required.
     - Preserve key attributes (type, properties) as graph attributes; avoid duplicating `id/label/layer/weight`.
     - Include top-level scope nodes for infra partitions and ensure all resource nodes `belongs_to` a partition.
   - Export the combined graph into datasets using existing dataset ingestion path; ensure annotations include the infra/correlation markdown.
   - Add validation to reject graphs with dangling edges or missing `belongs_to` and surface errors in reports.
7) Testing & Fixtures
   - Add sample IaC fixtures (tf, cfn, bicep, CDK Py/TS) and golden outputs for parsed resources and correlation.
   - Add integration tests covering end-to-end CLI run producing graphs/datasets.
8) CLI & Configuration
   - Introduce CLI flags to enable/disable infra scanning and correlation targets.
   - Document format support, limitations, and performance notes in `docs/code-analysis-infra.md`.

## Risks & Mitigations
- **Parser diversity**: Use format-specific crates with tolerant error handling and surface per-file failures in the report.
- **ID collisions**: Centralize slugification and validation when inserting nodes/edges.
- **Correlation accuracy**: Start with env-var and handler-file heuristics; gate experimental matches behind feature flags/config.

## Status
- Implemented: infra domain model, graph adapter to Layercake Graph, CLI/reporting hooks, Terraform/CFN/Bicep/CDK (Py/TS) parsers, dataset graph merge, basic correlation (env vars, handlers, files), tests, and service wiring to include infra graphs + annotations on run.
- Pending: deeper correlation heuristics (config/env var resolution), more robust code→infra edge mapping, feature flags/toggles, and expanded fixtures/golden outputs.
