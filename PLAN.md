# LayercakeGraphEditor Implementation Plan

This plan outlines the steps to implement the `LayercakeGraphEditor` component, integrate it into the frontend, and provide a way to edit individual graphs.

## 1. Implement LayercakeGraphEditor Component

**Location:** `frontend/src/components/graphs/LayercakeGraphEditor.tsx`

**Description:** This component will visualize an `LcGraph` object using `xyflow/react-flow`.

**Key Features:**
-   **React Flow Setup:** Initialize `ReactFlowProvider` and `ReactFlow` component.
-   **Graph Rendering:**
    -   Take an `LcGraph` object as input.
    -   Convert `LcGraph` nodes and edges into `react-flow` compatible nodes and edges.
    -   Handle `belongs_to` relationships by creating nested sub-flows. This will involve dynamically creating parent nodes for each `belongs_to` group and nesting child nodes within them.
    -   Render edges with arrows indicating direction (source to target).
-   **Dynamic Layout:**
    -   Utilize a layout algorithm (e.g., `dagre` or `elkjs`) to arrange nodes and edges automatically.
    -   The layout should prioritize a top-to-bottom direction.
    -   Ensure sufficient spacing between nodes for readability.
    -   The layout should be performed on the initial render of the graph.
-   **Interactivity:**
    -   Enable basic `react-flow` interactivity (panning, zooming).
    -   (Future consideration) Allow node dragging within sub-flows.
-   **Styling:** Apply basic styling for nodes, edges, and sub-flows.

## 2. Add "Edit Graph" Button to GraphsPage

**Location:** `frontend/src/components/graphs/GraphsPage.tsx`

**Description:** Add an "Edit Graph" button to the actions column of the graphs table.

**Changes:**
-   Modify the `Table.Td` for actions to include a new `ActionIcon` with an "Edit" icon.
-   This button will navigate to the new graph editing route (e.g., `/projects/:projectId/graphs/:graphId/edit`).

## 3. Create New Route for Graph Editing

**Location:** `frontend/src/App.tsx` (or relevant routing configuration)

**Description:** Define a new route that will render the `GraphEditorPage` component.

**Changes:**
-   Add a new `Route` entry: `<Route path="/projects/:projectId/graphs/:graphId/edit" element={<GraphEditorPage />} />`.

## 4. Implement GraphEditorPage Component

**Location:** `frontend/src/pages/GraphEditorPage.tsx` (new file)

**Description:** This page will fetch the `LcGraph` data and render the `LayercakeGraphEditor`.

**Key Features:**
-   **Route Parameter Handling:** Extract `projectId` and `graphId` from the URL parameters.
-   **Data Fetching:**
    -   Use `useQuery` to fetch the `LcGraph` data using a new GraphQL query (e.g., `GET_GRAPH_DETAILS`).
    -   Handle loading and error states.
-   **Component Rendering:** Pass the fetched `LcGraph` data to the `LayercakeGraphEditor` component.
-   **Breadcrumbs:** Integrate breadcrumbs for navigation.
-   **Basic UI:** Provide a title and potentially a "Save" button (initially non-functional, for future implementation).

## 5. Define New GraphQL Query for Graph Details

**Location:** `frontend/src/graphql/graphs.ts`

**Description:** Add a new GraphQL query to fetch the detailed `LcGraph` object, including its nodes, edges, and layers.

**Changes:**
-   Add `GET_GRAPH_DETAILS` query that fetches `id`, `name`, `nodeId`, `executionState`, `nodeCount`, `edgeCount`, `createdAt`, `updatedAt`, and importantly, `layers`, `graphNodes`, and `graphEdges`.
-   Define corresponding TypeScript interfaces for `GraphNode` and `GraphEdge` if not already present.

## 6. Backend GraphQL Resolver for Graph Details

**Location:** `layercake-core/src/graphql/queries/mod.rs` and `layercake-core/src/graphql/types/graph.rs`

**Description:** Implement resolvers to fetch `graphNodes` and `graphEdges` for the `Graph` type.

**Changes:**
-   In `layercake-core/src/graphql/types/graph.rs`, add `graphNodes` and `graphEdges` fields to the `Graph` struct.
-   Implement resolvers for these fields to fetch data from `graph_nodes::Entity` and `graph_edges::Entity` respectively, filtering by `graph_id`.

## 7. Frontend Styling and Utilities

**Location:** `frontend/src/styles/reactFlow.css` (new file) and `frontend/src/utils/graphUtils.ts` (new file)

**Description:** Add necessary styling for React Flow and utility functions for graph manipulation.

**Changes:**
-   `reactFlow.css`: Basic styles for React Flow elements.
-   `graphUtils.ts`: Helper functions for converting `LcGraph` data to `react-flow` elements, handling sub-flows, and performing layout.

## 8. Verification

-   Run `cargo check` and `npm run dev` to ensure no compilation errors.
-   Navigate to the `/projects/:projectId/graphs` page and verify the "Edit Graph" button is present.
-   Click the "Edit Graph" button and verify that the `GraphEditorPage` loads and displays the graph correctly.
-   Check the browser console for any errors related to GraphQL queries or React Flow rendering.
