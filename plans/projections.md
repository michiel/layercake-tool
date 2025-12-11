# Projections Implementation Plan

## Overview
- Add a new navigation section under Workbench named **Projections** that lists saved projections and provides actions to open or export them.
- A Projection belongs to a Project (`project_id`) and references an existing graph (`graph_id`) that must belong to the same project; persists metadata: `id`, `name`, `projection_type` (`force3d` default, `layer3d`).
- Each Projection materializes a `ProjectionGraph` (shallow copy/clone of the source graph held in memory) and a `ProjectionState` (projection-type-specific, in-memory, mutable settings).
- Projection UI runs as a separately built frontend served at `/projections` with its own GraphQL endpoint `/projections/graphql` (plus websocket subscriptions). The main app launches it via an **Open** button (new tab/window; Tauri opens new window).
- Interactions (viewing, variable changes, render settings) flow through GraphQL subscriptions that stream graph and state updates.
- Add **Export** action to produce a standalone ZIP (HTML + JS + embedded graph/state JSON) for offline viewing (similar to https://github.com/michiel/standalone-pack).
- First projection type to ship: `force3d` using https://github.com/vasturiano/3d-force-graph; `layer3d` scaffolding only.

## Backend (API + DB + Services)
- **Schema**
  - New table `projections`: `id` (pk), `project_id`, `graph_id` (FK -> graph_data.id, enforce same project via FK check/trigger), `name`, `projection_type` (enum text), `created_at`, `updated_at`. Optional `settings_json` for persisted defaults (projection-specific).
  - No duplication of graph data on disk; `ProjectionGraph` lives in memory when opened.
  - Migration to add FK/indices; seed default projection for existing graphs? (optional).
- **GraphQL endpoint `/projections/graphql`**
  - Separate schema/server using existing stack (Axum + async-graphql + async-graphql-axum) mounted at `/projections/graphql` with websocket subscriptions.
  - Types: `Projection`, `ProjectionState`, `ProjectionGraph` view (nodes/edges shallow copy from graph_data).
  - Queries: `projections(projectId)`, `projection(id)`, `projectionState(id)`, `projectionGraph(id)` (resolves by pulling graph_data + cached projection graph).
  - Mutations: `createProjection`, `updateProjection(name,type,settings)`, `deleteProjection`, `saveProjectionState(id, stateJson)`, `refreshProjectionGraph(id)` (clone latest graph_data).
  - Subscriptions: `projectionStateUpdated(id)`, `projectionGraphUpdated(id)` to push changes to UI (viewing and interactive changes to graph/render state).
- **Services**
  - ProjectionService to load/clone graph_data into `ProjectionGraph` (in-memory DTO), manage state, and publish subscription events.
  - Export service: builds bundle (HTML+JS+JSON) using shared projection UI assets and embedded graph/state; zip and return download URL/path.
  - If multiple projection types: registry/factory to resolve type-specific renderer/state defaults.
- **Auth/Access**
  - Reuse existing session/project guards; ensure `/projections/graphql` enforces project membership.

## Frontend: Main App (Workbench)
- Navigation: new **Projections** section under Workbench (list view per project).
- List view: table/cards of projections with `name`, `type`, `graph name`, `updated_at`, actions (`Open`, `Export`, `Delete`).
- Create flow: button to create projection selecting graph and type (`force3d` default). On save, call `createProjection` and show in list.
- Open button: opens `/projections/{id}` in new tab/window; in Tauri use new window pointing to projection build.
- Export button: calls export API; download ZIP.
- Keep this UI lightweight; heavy rendering happens in projection-specific build.

## Frontend: Projection Builds (served at /projections)
- Separate Vite build output (e.g., `frontend-projections/`) served at `/projections` and also used for offline export.
- Shared client lib to talk to `/projections/graphql` (queries, mutations, subscriptions).
- Route `/projections/:id` bootstraps:
  - Fetch projection metadata/state/graph.
  - Connect websocket subscription for live updates.
  - Render specific projection component based on `projection_type`.
- **force3d implementation**
  - Integrate `3d-force-graph` (vasturiano) to render nodes/edges.
  - Map `ProjectionGraph` nodes/edges to library format; apply controls for camera, forces, labels, layers.
  - Bind UI controls to `ProjectionState` (force params, filters, color/size mappings); persist via `saveProjectionState`.
  - Listen to `projectionGraphUpdated`/`projectionStateUpdated` to live-update scene.
- **layer3d scaffold**
  - Stub component and default state; render placeholder until implemented.

## Export Flow
- Backend mutation `exportProjection(id)`:
  - Fetch projection + graph + state; run export service to package:
    - `index.html` references bundled JS/CSS (from projection build) and inlines JSON payload (graph + state).
    - Assets copied from projection build or embedded via base64; ensure offline-compatible resource loading.
  - Zip as `<projection-name>-projection.zip`; return download link/path. Reuse standalone-pack pattern.
- Frontend: trigger download from list via Export button; show toast on completion/error.

## Tauri Support
- `Open` opens new window pointing to local `/projections/{id}` route; ensure assets packaged in Tauri build.
- Wire websocket client to Tauri env; verify CORS/IPC allowances.

## Dev & Build Pipeline
- Add new package/workspace for projection UI build (or sub-build in frontend); scripts:
  - `npm run projections:dev`, `npm run projections:build`, `npm run projections:serve`.
  - Backend: optional flag to serve `/projections` static assets and `/projections/graphql`.
- Update docs (README/BUILD) for new commands and navigation.

## Milestones
1) Schema + migrations + ProjectionService + GraphQL API scaffolding (queries/mutations/subscriptions).  
2) Workbench UI list/create/open/export wiring to new API.  
3) Projection build setup + shared GraphQL client + `force3d` renderer with state persistence.  
4) Export packaging end-to-end and download flow.  
5) Tauri window wiring and packaging of projection assets.  
6) Hardening: auth, limits, perf (lazy-load graph), telemetry/logging, tests.  

## Technical Examples
- **async-graphql schema snippets**
  ```rust
  #[derive(SimpleObject)]
  struct Projection {
      id: ID,
      project_id: ID,
      graph_id: ID,
      name: String,
      projection_type: String,
  }

  #[derive(SimpleObject)]
  struct ProjectionState {
      projection_id: ID,
      projection_type: String,
      state_json: serde_json::Value,
  }

  #[Object]
  impl ProjectionQuery {
      async fn projections(&self, ctx: &Context<'_>, project_id: ID) -> Result<Vec<Projection>> {
          // enforce project scoping; only return projections whose project_id matches
      }

      async fn projection_graph(&self, ctx: &Context<'_>, id: ID) -> Result<ProjectionGraph> {
          // load graph_data for the projection.graph_id (ensure same project), clone into DTO
      }
  }

  #[Subscription]
  impl ProjectionSubscription {
      async fn projection_graph_updated(&self, id: ID) -> impl Stream<Item = ProjectionGraph> {
          // tokio::sync::broadcast receiver emitting graph/render changes
      }
      async fn projection_state_updated(&self, id: ID) -> impl Stream<Item = ProjectionState> {
          // emit when state_json is updated (UI control changes)
      }
  }
  ```

- **Axum wiring for projections GraphQL**
  ```rust
  let schema = Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
      .data(projection_service.clone())
      .finish();

  let app = Router::new()
      .route("/projections/graphql", get(graphql_playground).post_service(GraphQL::new(schema.clone())))
      .route("/projections/graphql/ws", get(GraphQLSubscription::new(schema)));
  ```

- **Frontend subscriptions (urql/Apollo)**
  ```ts
  const GRAPH_SUB = gql`
    subscription ProjectionGraphUpdated($id: ID!) {
      projectionGraphUpdated(id: $id) {
        nodes { id label layer }
        edges { id source target }
      }
    }
  `;

  const STATE_SUB = gql`
    subscription ProjectionStateUpdated($id: ID!) {
      projectionStateUpdated(id: $id) {
        projectionId
        projectionType
        stateJson
      }
    }
  `;

  const graphSub = useSubscription({ query: GRAPH_SUB, variables: { id } });
  const stateSub = useSubscription({ query: STATE_SUB, variables: { id } });
  ```

- **Broadcast on state change (Rust)**
  ```rust
  pub fn update_state(&self, projection_id: i32, state: Value) {
      self.state_store.insert(projection_id, state.clone());
      let _ = self.state_tx.send((projection_id, state));
  }
  ```

## Implementation Plan (phased)
- **Phase 1: Backend foundation**
  - ✅ Add `projections` table migration (project_id + graph_id FK; service-level check keeps graph/project aligned).
  - ✅ Implement ProjectionService (state store, graph cloning, broadcast channels).
  - ✅ Wire Axum + async-graphql schema at `/projections/graphql` with queries/mutations/subscriptions; add websocket route.
- **Phase 2: Workbench UI**
  - 🚧 Projections nav added; Workbench page now links to Projections view.
  - 🚧 Projections page lists/creates/deletes projections via `/projections/graphql`; uses graph picker (project-scoped).
  - TODO: Wire Export action to backend exportProjection, add Open behavior for Tauri window.
- **Phase 3: Projection frontend build**
  - Create separate Vite build (shared GraphQL client) served at `/projections`.
  - Implement `force3d` renderer (3d-force-graph) with controls bound to ProjectionState; subscribe to graph/state updates.
  - Stub `layer3d` component and defaults.
- **Phase 4: Export pipeline**
  - Backend `exportProjection` mutation to bundle projection build + payload into ZIP; reuse standalone-pack pattern.
  - Frontend download flow with progress/toasts.
- **Phase 5: Desktop/Tauri**
  - Open projections in new Tauri window pointing to `/projections/{id}`; package projection assets in desktop build.
- **Phase 6: Hardening**
  - Add auth/authorization checks per project, backpressure on broadcasts, perf tuning (lazy load, caching), tests (schema/service, UI smoke).
