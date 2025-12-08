# Layercake Code Analysis – Processing Specification

This document describes the major stages, inputs, and outputs of the `layercake-code-analysis` crate. It focuses on how source code is discovered, analyzed, and converted into the `AnalysisResult` model that downstream consumers (CLI, backend, frontend) turn into graphs, datasets, or CSV exports. Notes from the latest review are included as “findings” where relevant.

## Inputs
- **Root path**: Filesystem directory to analyze (typically a project root).
- **Supported languages**: Python (currently primary), with language-agnostic fallbacks for file and dependency discovery. Future language handlers plug into the same pipeline.
- **Options** (parsed by the caller and respected by the analyzer/graph stages):
  - `include_data_flow`, `include_control_flow`, `include_imports`, `include_infra` (booleans).
  - `coalesce_functions` (convert function nodes to file nodes during graph export).
  - `exclude_known_support_files`, `exclude_inferred_support` (filter out lockfiles/tests/fixtures, etc.).
  - CSV export flag (CLI) for emitting CSVs instead of graph JSON when requested.
  - **Finding:** Infra parsing and correlation are now run by default; callers only opt out by setting `include_infra=false` or profile `noInfra=true`.

## Processing Stages
1. **Workspace discovery**
   - Walk the project tree, capturing a directory and file inventory (relative paths).
   - Skip hidden/vendor/test/support files when exclusion flags are enabled.

2. **Language parsing & symbol extraction**
   - Parse supported source files to extract:
     - Functions: name, file path, line, args, return type, cyclomatic complexity.
     - Imports: module/library references per file.
     - Entry points: `if __name__ == "__main__"`-style blocks (and analogous patterns per language).
     - Environment references: detected env var names and usage contexts.
   - Maintain a per-file function table for later disambiguation (prevents merging same-named functions across files).

3. **Control-flow & data-flow inference**
   - **Control flow**: call edges between functions within a file; inter-file calls are resolved using canonical names plus file/handler hints when available.
   - **Data flow**: source→sink edges for variable flows between functions; label is the variable name when known.
   - **Import links**: library→function edges for imported modules actually used in a file.

4. **Infrastructure parsing (default)**
   - Always invoked after code analysis. `infra::analyze_infra` produces an `InfrastructureGraph` when infra/IaC files are present (e.g., AWS SAM/CloudFormation/Terraform/CDK/Bicep/Terraform).
   - Infra nodes: resources and partitions; edges: depends-on, references, and code-link placeholders; diagnostics are recorded on the graph. Handler hints (CodeUri + Handler) are stitched into resource properties to aid correlation.
   - **Finding:** infra structures are now part of `AnalysisResult` (serialized) and reused by the backend to avoid re-scans.

5. **Code↔Infra correlation (default)**
   - `infra::correlate_code_infra` matches resources to code by:
     - Handler strings (`path.func`), function names, file names, and property references (including stitched `handler_path`).
   - Produces `CorrelationReport` (matches, unresolved, warnings) with qualified `path::function` hints.
   - Correlation edges are merged later by the consumer (see Graph conversion).
   - **Finding:** correlation now prefers file-scoped handler matches (using `path::function`) to avoid fanning out onto a single handler.

6. **Statistics & annotations**
   - Source metrics (LOC by language) via `tokei`.
   - Markdown report generation via `report::markdown::MarkdownReporter`, embedding stats, findings, and CSV blocks (stripped before storing as dataset annotations).

## Outputs (`AnalysisResult`)
The analyzer returns a structured `AnalysisResult` containing:
- `functions`: list of functions with complexity, return type, args, file path, line.
- `imports`: module references by file.
- `data_flows`: source function, sink function, variable, file.
- `call_edges`: caller, callee, file.
- `entry_points`: condition/marker, file, line.
- `env_vars`: detected environment variable names/usages.
- `directories` / `files`: relative paths for hierarchy building.
- `libraries` (when applicable): derived from imports.
- `infra`: optional `InfrastructureGraph` produced by default infra scanning (resources, partitions, edges, diagnostics).
- `infra_correlation`: optional `CorrelationReport` aligning infra resources with code (matches/unresolved/warnings).
- `report`: Markdown string (rendered by the reporter, often cleaned before persistence).
- `stats`: LOC and language breakdown (from `tokei`), included in annotations.
  - **Finding:** `AnalysisResult` is `Serialize`/`Deserialize` and persisted on profiles (`last_result`) for replay and UI viewing; ensure future fields remain serialization-friendly.

### Graph Conversion (consumer stage)
Although performed in `layercake-core`, the conversion follows these rules:
- Nodes:
  - `scope`: root (Codebase/Solution), directories, files (partitions by default).
  - `function`: non-partition flow nodes with attributes `{complexity, return_type, file, line, args}`.
  - `library`, `entry`, `exit`, `infra` (when merged).
  - Label sanitization removes quotes/control chars; IDs are unique, lowercase, and safe.
- Edges:
  - `dataflow`: variable-labeled source→sink.
  - `controlflow`: caller→callee.
  - `import`: library→function (may be filtered in solution view).
  - `entry`: entry point→function.
  - Infra correlation edges (`infra-code-link`) are added when merging.
- Hierarchy:
  - Every node has `belongs_to`; a synthetic root is created if missing.
  - Files/directories are relative to the project root.
  - Partition flags mark structural nodes; flow nodes are `is_partition: false`.
- Coalescing (optional):
  - Function nodes can be rewired to their owning file nodes; duplicate edges are merged and weights summed.
  - **Finding:** Function IDs are keyed by file+name to prevent cross-file collisions (e.g., multiple `lambda_handler` functions).

### CSV Export (CLI option)
- When `--csv`/`--csv-dir` is provided, the analysis emits nodes/edges CSVs instead of (or alongside) graph JSON.
- CSV columns mirror graph fields (id, label, layer, is_partition, belongs_to, attributes, etc.) for ingestion into datasets.

## Error Handling & Warnings
- Missing paths or unreadable files return errors to the caller.
- Correlation emits warnings when infra resources cannot be linked to code.
- Label sanitization and node/edge ID normalization append annotations when modifications occur.
- Infra parsing emits diagnostics for dropped edges or malformed resources; correlation warnings are appended to annotations when merged.

## Extensibility
- New language handlers plug into stage 2 (parsing) while preserving the `AnalysisResult` schema.
- New infra providers plug into `infra::analyze_infra` and reuse correlation heuristics.
- Additional analysis dimensions (e.g., taint sources/sinks, more control-flow details) should extend `AnalysisResult` and the graph mapper in compatible ways.

## Open Gaps / Future Improvements (from review)
- Broader language coverage: JavaScript analyzer is basic; TypeScript/CDK parsing is early-stage.
- Infra scanning: Terraform/CloudFormation/CDK coverage is heuristic; needs richer resource typing and file→resource provenance.
- Correlation: add confidence scores, path normalization across OSes, and clearer diagnostics when multiple handlers match.
- Data/control flow: inter-file call resolution is best-effort; consider symbol tables per language for cross-module calls.
- Performance: parallel parsing is in place; infra scan could short-circuit when disabled to avoid overhead.
