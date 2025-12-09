# Next Steps: Code/Solution Analysis Refinement

## Recommendations
- Separate analysis types and options:
  - Define distinct option structs for `code` vs `solution` analysis.
  - UI should gate fields per type; backend should store structured options per type instead of one shared JSON blob.
- Solution analysis should avoid function-level detail:
  - Build a solution-mode graph directly (no function nodes), keeping only entries, files, external calls, infra/resource links, exits.
  - Add exit-point detection and external-call detection (HTTP, SDK/boto3) in analyzers.
- Preserve richness when mapping to Layercake:
  - Keep node attributes (complexity, return_type, args, file, line, env var usage).
  - External calls as dedicated nodes (layer `external_call`) with attributes (service/method/path).
  - Infra correlation edges carry confidence and handler hints; resource nodes keep handler_path.
  - Ensure `belongs_to` set for all nodes; keep hierarchy intact.
  - Do not drop edge labels/weights when coalescing; retain merged labels in attributes.
- Reports remain annotations only; dataset descriptions should not be overwritten.
- Infra parsing/correlation:
  - Keep handler hints; extend resource-type-specific edges where possible (refs inferred already).
  - Surface correlation confidence in UI (done).
- Defaults per type:
  - Solution: include_infra=true, include_imports=false, include_data_flow/control_flow=false.
  - Code: current defaults.

## Plan
1) Add structured per-type options and wire through backend/GraphQL/UI.
2) Implement solution-mode graph builder (no functions), add exit + external_call nodes/edges, carry rich attributes.
3) Preserve attributes/labels in coalescing and merging; ensure annotations only (no dataset description overwrite).
4) Extend analyzers for external-call detection and exit-point tagging; propagate into graphs.
5) Refine infra correlation edges with confidence/handler_path attributes; ensure `belongs_to` completeness.
6) Update UI to gate options per analysis type and display external calls/exits in the result viewer.

Progress:
- [x] 1) Structured options
- [x] 2) Solution-mode graph builder + attributes (baseline solution graph without functions, includes entries/exits/external calls)
- [ ] 3) Preserve labels/annotations only
- [x] 4) External call + exit detection (Python + JS/TS heuristics added)
- [ ] 5) Infra correlation attributes/belongs_to (handler/key hints + inferred refs added; belong-to audit pending)
- [ ] 6) UI gating & viewer enhancements (partial: viewer shows external calls/exits)
