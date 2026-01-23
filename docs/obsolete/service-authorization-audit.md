# Service Authorization Audit

This document tracks which core services require actor-based authorization.

## Status

- Initial sweep completed with placeholders.
- Update each row with required roles/scopes as policies are formalized.

## Services

| Service | Actor Required | Notes |
| --- | --- | --- |
| `ProjectService` | Yes | Requires project-level access checks. |
| `GraphService` | Yes | Mutations should require write access. |
| `GraphEditService` | Yes | Edits should require write access. |
| `PlanService` | Yes | Plan edits require write access. |
| `PlanDagService` | Yes | DAG edits require write access. |
| `DataSetService` | Yes | Data imports should require write access. |
| `LibraryItemService` | Yes | Library changes should require write access. |
| `CodeAnalysisService` | Yes | Restricted to project collaborators. |
| `SystemSettingsService` | Yes | Admin-only. |
