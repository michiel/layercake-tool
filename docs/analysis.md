# Static Analysis Uplift Plan

Objective: enable automatic generation of node/edge CSV inventories (data/compute/platform) for diverse codebases (Python, JS/TS, CDK, Terraform, Bicep, generic YAML), relying only on static analysis and reusable heuristics.

## Current State (layercake-code-analysis/)
- Language analyzers: Python and JS parse functions/imports, data/control flows, env vars, entry points.
- Graph conversion: turns `AnalysisResult` into Layercake graph with scopes/files/dirs, imports/dataflow/controlflow layers.
- Options: include/exclude infra/imports/data/control, function coalescing, support-file filters.
- Gaps: no infra-as-code parsers; limited semantic tagging of data vs compute; file/dir scope only; no AWS/CDK/TF resource extraction; no weight normalization in exports.

## Goals
1. Generate CSV node/edge inventories matching prompts without manual editing (CLI opt-in flag).
2. Support multi-language projects (Python, JS/TS) plus IaC (CDK, Terraform, Bicep, SAM templates).
3. Identify platform resources (AWS services) and map compute/data nodes to them.
4. Derive data/control/import flows and annotate origins for traceability.
5. Provide stable, documented APIs for downstream UI/exports; frontend calls should hydrate graphs directly without intermediate CSV.

## Technical Plan
### 1) Core Model Extensions
- Add typed enums for node kinds (Compute, Data, Platform, Scope, Test, Support) and edge kinds (DataFlow, ControlFlow, Import, InfraBinding, InfraFlow).
- Extend `AnalysisResult` with `resources` (parsed infra resources), `bindings` (code → resource), and `artifacts` (files/directories with role tags).
- Preserve provenance: each item carries `source_file`, `line`, `extractor`.

### 2) Language Analyzer Enhancements
- **Python/JS/TS**: enhance parsers to detect handler entry points (Lambda-style, FastAPI/Express endpoints), HTTP verbs/paths, env var usage, and IO operations (files, S3 clients) to infer data nodes. Add lightweight static call graph to improve controlflow edges. Normalize identifiers (lowercase, sanitized IDs).
- **Test detection**: tag files/dirs with test heuristics (`test`, `spec`, `__tests__`, fixtures/mocks) and mark nodes as Test/Support but keep optional exclusion.

### 3) IaC & Config Parsers
- **CDK (TS/Python)**: parse synthesized `cdk.out` if present; otherwise parse constructs from source using AST/regex to extract resource types/ids and logical names.
- **Terraform**: parse `.tf` to extract resource blocks, types, names, and references; map common AWS resources (lambda, api_gateway, dynamodb, s3, cognito, iam, cloudwatch).
- **Bicep**: parse to resources with type strings.
- **SAM/CloudFormation**: parse `template.yaml`/`template.json` for resources and metadata (Events → API paths).
- **Manifest/config**: infer service usage from `package.json` deps (`@aws-sdk/*`, `aws-cdk-lib`), `requirements.txt`/`pyproject` (`boto3`, `awscli`), and project layout.
- Produce `Resource` structs (id, type, name, attributes, file, line) and `Binding` links between resources (e.g., Lambda to IAM role, API to Lambda).

### 4) Correlation & Heuristics
- Map compute nodes to resources via filename/path matches (handler paths), env vars (`TABLE`, `QUEUE_URL`), SDK client constructor arguments, and IaC logical names.
- Generate data nodes from API shapes and persistent stores (DynamoDB table → record/list nodes; S3 bucket → object node).
- Coalesce functions into file-level compute nodes by default for CSV export; keep per-function edges optional.
- Compute `relative_weight` for edges based on frequency or confidence (1–6) for rendering.

### 5) CSV Export Pipeline (CLI only)
- CLI flag `--csv` on `ca report` emits:
  - `nodes.csv`: id, label (human-readable), layer, is_partition, belongs_to, comment (provenance).
  - `edges.csv`: id, source, target, layer, label, relative_weight, comment.
- Ensure a single `root_scope` partition; every node has `belongs_to` set (file→dir→root).
- Strip labels from coalesced edges if noisy; keep comments for traceability.
- Frontend: no intermediate CSV; service returns graph directly for dataset hydration.

### 6) API & Config
- Extend `CodeAnalysisOptions` with:
  - `include_tests`, `include_support`
  - `coalesce_functions`, `emit_csv`
  - `include_iac` (cdk/terraform/bicep/sam)
- Add defaults and validation; document in README.
- Introduce analysis types: `code` (existing) and `solution` (new profile type for CSV/report generation). Profiles store `analysis_type`; frontend chooses type first (default `code`).

### 7) Frontend Exposure
- Profile creation: select analysis type (`Code analysis` default, `Solution analysis` for CSV/report flow).
- Profile edit: toggles for include/exclude tests/support, include IaC, coalesce; CLI CSV export is not used in frontend path.
- Dataset viewer: allow downloading latest CSV outputs (if produced by CLI); display provenance comments on hover.

### 8) Testing Strategy
- Add fixtures for:
  - SAM CRUD (existing) → verify compute/data/platform nodes and bindings.
  - Mixed JS/TS + CDK sample.
  - Terraform-only infra sample.
  - Bicep/ARM minimal template.
  - Synthetic test/support-heavy repo to validate exclusions.
- Snapshot CSV outputs and graph annotations; unit tests for resource parsers and correlation heuristics.

## Dataset Annotations Upgrade
- Add structured annotations to datasets: ordered list stored in DB (`title`, `date`, `body` markdown).
- Migrate `data_sets` with JSON column for annotations; expose via GraphQL/REST; update dataset services to append, replace, and retrieve annotations.
- Use annotations to store analysis report markdown (including warnings/diagnostics) instead of overloading description.

## Risks & Tradeoffs
- **Heuristic accuracy**: mapping code to resources via env vars/paths can mislink. Mitigation: keep confidence scores, allow opting out of bindings, log diagnostics.
- **Parser maintenance**: IaC formats evolve; keep parsers lightweight and fail-soft (skip invalid blocks, annotate warnings).
- **Performance**: AST + IaC parsing across large repos can be slow. Mitigation: parallel traversal, file filters, optional depth limits.
- **Noise vs. fidelity**: Coalescing removes detail; keep a flag to retain per-function nodes when needed.
- **Multi-language edge cases**: CDK/Bicep/Terraform parsers may miss custom constructs; provide extensible matcher registry.

## Recommendations
- Start with Terraform/CFN/SAM parsing (clear schemas) and JS/TS AST enhancements, then add CDK/Bicep heuristics.
- Make CSV export opt-in per profile initially; log diagnostics (unmatched resources, skipped files) into annotations.
- Normalize IDs early (lowercase, sanitized) and ensure root scope/belongs_to invariants to avoid DAG errors.
- Version the CSV schema in annotations to help downstream consumers.
