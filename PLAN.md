# Data Model Refactoring Plan

This plan outlines the steps to refactor the database entities to match the desired data model relationships.

## 1. Project to PlanDAG (One-to-One)

The current relationship is one-to-many (a project can have multiple plans). The desired relationship is one-to-one (a project has one plan).

**File to modify:** `layercake-core/src/database/entities/projects.rs`

- **Change:** In `projects.rs`, the `Relation::Plans` will be changed from `has_many` to `has_one`.

**File to modify:** `layercake-core/src/database/entities/plans.rs`

- **Change:** In `plans.rs`, the `project_id` column should be made unique to enforce the one-to-one relationship at the database level.

## 2. LayercakeGraph to LcLayer (One-to-Many)

The current relationship is Project -> LcLayer. The desired relationship is LayercakeGraph -> LcLayer.

**File to modify:** `layercake-core/src/database/entities/layers.rs`

- **Change:** In `layers.rs`, the `project_id` field will be replaced with a `graph_id` field.
- **Change:** The `Relation::Projects` will be removed and a new `Relation::Graphs` will be added, linking to `graphs::Entity`.

**File to modify:** `layercake-core/src/database/entities/graphs.rs`

- **Change:** In `graphs.rs`, a new `Relation::Layers` will be added as `has_many = "super::layers::Entity"`.

## 3. Remove Unwanted Relationships from Project

The `Project` entity currently has relationships to `Nodes` and `Edges` which are not in the desired data model.

**File to modify:** `layercake-core/src/database/entities/projects.rs`

- **Change:** In `projects.rs`, the `Relation::Nodes` and `Relation::Edges` will be removed.

## 4. Create Migrations

After applying the entity changes, new database migrations will need to be created and applied to update the database schema accordingly. This will involve:
- Dropping foreign keys.
- Modifying columns (e.g., making `project_id` in `plans` unique, changing `project_id` to `graph_id` in `layers`).
- Creating new foreign keys.

## 5. Service Layer Implementation Plan

The service layer will need to be updated to reflect the new data model.

- **File to modify:** `layercake-core/src/services/project_service.rs` (and any other relevant services)
    - **Change:** Update functions that currently fetch multiple plans for a project to fetch a single plan.
    - **Change:** Remove functions that fetch layers directly from a project.
    - **Change:** Update any logic that relies on the old relationships.

- **File to modify:** `layercake-core/src/services/graph_service.rs` (and any other relevant services)
    - **Change:** Add functions to fetch layers associated with a graph.
    - **Change:** Update any logic that relies on the old relationships.

## 6. GraphQL Schema Implementation Plan

The GraphQL schema will need to be updated to reflect the new data model.

- **File to modify:** `layercake-core/src/graphql/schema.rs` (or wherever the schema is defined)
    - **Change:** In the `Project` type, change the `plans` field from `Vec<Plan>` to `Option<Plan>`.
    - **Change:** In the `Project` type, remove the `layers` field.
    - **Change:** In the `LayercakeGraph` type, add a `layers` field of type `Vec<Layer>`.
    - **Change:** Update all resolvers that are affected by these changes.

## 7. Frontend Implementation Plan

The frontend will need to be updated to work with the new GraphQL schema.

- **Files to modify:** `frontend/src/graphql/queries.ts`, `frontend/src/graphql/mutations.ts`, etc.
    - **Change:** Update all GraphQL queries, mutations, and subscriptions to match the new schema. For example, queries that previously fetched `project.plans` will now fetch `project.plan`. Queries that fetched `project.layers` will now need to fetch `project.graph.layers`.

- **Files to modify:** `frontend/src/components/**/*.tsx`
    - **Change:** Update all components that use the modified queries. This will likely involve changing how data is accessed and passed down to child components.
    - **Change:** For example, a component that was displaying a list of plans for a project will now only display a single plan. A component that was displaying layers will now need to get them from a graph object instead of a project object.