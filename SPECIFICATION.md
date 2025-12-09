# Layercake Specification

> **Note**: This document represents the original product vision and specification from the project's inception. Many features described here have been implemented. For current implementation details, see [README.md](README.md), [BUILD.md](BUILD.md), and documentation in the `docs/` directory.

## Goals and high level requirements

The overall goal is to change this pipeline graph transformation and creation tool to an interactive application that can be used to create and edit a graph visually, spawn child graphs from a parent graph, with changes higher up in the hierarchy propagating downwards. Multiple instances of the layercake tool can run at the same time and synchronise changes in real time, updating the user interface in real time. A desktop instance can connect to a server instance (any instance to any instance).

Agentic AI with tooling exposed via Model Context Protocol for agentic collaboration is a key goal.

Building this as a distributed and collaborative graph editing platform is a later goal and not a priority for version 1.0.0

## Data model relationships between entities

### Project

One project has one,
 - PlanDAG

One project has many,
 - Datasource
 - LayercakeGraph

### PlanDAG

One PlanDAG has many,
 - PlanNode
 - PlanEdge

### LayercakeGraph

One LayercakeGraph has many,
 - LcEdge
 - LcNode
 - LcLayer
 - LcGraphEdit

## Details

### Layercake plans and graphs

 - All aspected of the layercake process from ingestion to transformation to renderings are stored as a DAG. This is the layercake plan. If an upstream node (e.g. representing ingestion of a CSV as a nodeset) changes, all downstream nodes are updated
 - Each Project has DataSets. DataSets are tables belonging to a project that are a raw import of their source (attr: blob), and contain a graph_json attribute that imports the raw attributes to the appropriate graph_json={{nodes:[], edges:[], layers:[]}} attributes
 - DataSets can be referenced as a node in the PlanDAG, and can be used as source nodes for an edge with a target node GraphNode. GraphNodes can have multiple DataSetNodes
 - DataSets have their own data management page(s) for CRUD operations under a Project
 - Graph data can be imported from CSVs (the existing layercake source format), REST endpoints, SQL querie (react component name : PlanVisualEditor)
 - The layercake plan (PlanDAG) can be edited (visually using react-flow / xflow, component name PlanVisualEditor), with the inputs, graph hierarchy and exports of different graphs in the hierarchy all represented and editable from the same interface
 - Graphs (or more specifically LayercakeGraphs) can be edited using the spreadsheet editor component that has three tabs for nodes, edges, layers (react component name : GraphSpreadsheetEditor) OR using another instance of react-flow / xflow for graph editing (react component name : GraphVisualEditor)
 - Changes to Graphs are tracked and shared as CRDT objects and stored in a database table (Project -> Changes)
 - Each Project has a Plan DAG. The Plan DAG is a JSON object that is an attribute of the Project table
 - Plans contains graph metadata and the relationships between graph copies. Plans do not contain actual graph data (e.g. nodes, edges), graph data is in the Graph table. Plan DAG steps can be (but are not limited to) nodes that are InputNode (from file, etc), MergeNode (from InputNode and/or GraphNode, output GraphNode), OutputNode (from GraphNode, output OutputNode (render target)), GraphNode (which are references and metadata for a graph instance), TransformNode (from graph, output graph) that contain the existing graph transformation options in the current YAML. The Plan DAG is generally editing in PlanVisualEditor, with popup items to edit node attributes (example: InputNode editor will have a selectable type for File that allows for selecting file location and data type (nodes, layers, edges)) and node metadata (example: node label)
 - A key function is that edits to graphs (via GraphVisualEditor or GraphSpreadsheetEditor) are tracked and reproducible, so re-applying inputs and re-running the DAG, will re-run the edits (if they are still applicable, removing them from tracking otherwise)
 - LcGraph entities have many LcGraphEdit entities called GraphEdits. Each GraphEdit describes a change operation made to a node/layer/edge of a Graph instance. So if a node is renamed, this is a GraphEdit. If a layer is added, this is a GraphEdit, if an edge is removed this is a GraphEdit, etc. When an upstream source is updated or a graphnode is regenerated directly, the ordered list of GraphEdits is replayed and applied to the updated GraphData. GraphEdits are keyed to type(node/edge/layer) and id. If an edit has no match in the updated dataset, the edit is discarded.  the goal is to allow a user to edit a graph instance (via different frontend graph editors) and not lose the changes if the upstream data refreshes 

```yaml
plan:
 inputs:
  - id: ID
    inputType: CSVNodesFromFile
    file: import/nodes.csv
  - id: ID
    inputType: CSVEdgesFromFile
    file: import/nodes.csv
  graphs:
   - id: 1
     parent: null
     label: "Current State"
   - id: 2
     parent: 1
     label: "Target State (Option A)"
   - id: 3
     parent: 1
     label: "Target State (Option B)"
   - id: 4
     parent: 2
     label: "Target State (Option A-1)"
   - id: 5
     parent: 2
     label: "Target State (Option A-2)"
  exports:
   - id: ID
     graph: 1
     exporter: "GML"
     filename: "out/ref-model-labels-chopped.gml"
     graph_config:
       node_label_max_length: 2
       node_label_insert_newlines_at: 1
       edge_label_max_length: 2
       edge_label_insert_newlines_at: 1
   - id: ID
     graph: 2
     exporter: "PlantUML"
     filename: "out/ref-model-render-options.puml"
     exporter: "PlantUML"
     graph_config:
       node_label_max_length: 2
     render_config:
       orientation: "LR"
       contain_nodes: false
```

## Technology

- Rust
  - tauri version 2 for the desktop version
  - axum for http server
  - tokio for async
  - for GraphQL : async-graphql with async-graphql-axum
  - for MCP : https://github.com/michiel/axum-mcp
  - tower
  - for database management : sea-orm
  - for logging and tracing : tracing, tracing-subscriber, tracing-tree
  - clap.rs for command line parameters and arguments
- Frontend / Typescript
  - React
  - Mantine frontend
  - Apollo for graphql
  - Editing with
    - https://reactflow.dev/
  - Visualisations with
    - https://github.com/vasturiano/3d-force-graph
    - https://github.com/markmanx/isoflow


## Code

 - The root of the project is a cargo workspace
 - Use project crates inside the workspace to isolate functionality and minimise compilation time
 - The tauri project has its own crate (src-tauri)
 - The web interface has its own subdirectory (frontend)
 - The code consistently logs using tracing at TRACING, DEBUG, INFO, WARN and ERROR, with the default level being INFO. This can be overridden via config, command line flags or then RUST_LOG environment variable, with different log levels configurable on a crate/module basis if necessary
 - The web interface will communicate with the backend via GraphQL
 - Layercake plan files can be represented as a YAML file. The Layercake binary will have a command for running the plan, similar to the current implementation ()
NC
### Testing

 - All functionality must have test coverage

## Requirements

### Presence

 - All user presence is tracked and shared on a per-project basis. cursor position is tracked on a per-project-per-document basis. position can be different depending on the document, example: current implementation tracks position data on canvas, it should also be able to track position in (for example) spreadsheet coordinates, or in a 3d rendering of a VR journey
 - User presence like cursor information is ephemeral data and is communicated between the client and server using a direct, raw websocket connection and NOT via GraphQL mutations
 - On the frontend, user presence shown on a per-project basis in the top bar (icon with the number of active users, click on icon to list active users by name in a Mantine Hover Card)
 - On the frontend, user presence shown on a per-project-per-document basis by the active user cursor positioning (e.g. presence cursors on the canvas in Plan DAG Editor)

### Frontend

#### Top bar

 - Items on the top bar,
  * (left) Top left the Layercake icon and title
  * (right) Icon for light/dark theme switching
  * (right) Icon for online/offline status indication (no text, red/green icon only)
  * (right) Account name + avator, onclick dropdown menu with links to Settings, Profile

#### DAG Plan Editor

 - The DAG plan editor has the following nodes
   * DataSetNode - this references an *existing* DataSet entity of the Project
   * GraphNode - this *creates and manages* a Graph entity of the Project
   * TransformNode - changes a Graph. Input is a GraphNode, output is a GraphNode. The configuration for the TransformNode is a list of rules that are applied in order (example: invert_graph, max_partition_width:2, max_partition_depth:3, node_label_max_length:4, edge_label_max_length:4)
   * MergeNode - merges DataSetNodes and/or GraphNodes. The output is a new GraphNode. The configuration for MergeNode are the merge rules (default: overwrite existing nodes/edges/layers with same ID)
   * OutputNode - this triggers a specific export or visualisation (example: GraphvizDOT, CSV), input is a GraphNode
 - The DAG Plan editor has a toolbar on the top. This toolbar has draggable icons for each of the node types that can be dropped on the canvas as unconfigured nodes. Unconfigured nodes are highlighted in orange. Clicking the cog icon on an uncofigured node opens the configuration dialog, which is different for each node. A DataSet node allows you to select an existing DataSet. A TransformNode allows you to create a list of rules, each of which has their own configuration
 - Nodes in the DAG Plan Editor have connectors on all 4 sides. The left and top are INPUT connectors and will only connect FROM nodes that output the correct type. the right and bottom are OUTPUT connectors and will connect TO nodes that input the correct type. Visually input and output connectors are distinct. After connecting an edge to or from a top/left/bottom/right connector, that connection has to stay consistent (NOT connect to top and render as connect to left. if visually connected to top, the render has to be top as well, etc)
 - The Plan DAG can have nodes in either CONFIGURED or UNCONFIGURED state
 - Node must meet all their requirements (including node-specific configuration that passes validation, if applicable) to be in configured state, otherwise they are in unconfigured state
 - Nodes that are not in CONFIGURED state have an orange outline
 - Node connection rules :
   * GraphNodes can have multiple inputs (of GraphNode type) but MUST have at least one to be in configured state
   * GraphNodes can have multiple outputs (of GraphNode type)
   * DataSetNodes can have multiple outputs (of GraphNode type), but cannot connect to the same target twice, and MUST have at least one output connected to be in configured state
   * TransformNodes can have only one input (GraphNode type) and multiple outputs (GraphNode type), but cannot connect to the same target twice, and MUST have one input and one output to be in configured state
   * OutputNodes can have only one input (of GraphNode type) and MUST have one input to be in configured state
   * These node connection rules also serve as part of Plan DAG validation on the backend
 - Edges can be selected and then deleted OR changed OR disconnected and reconnected
 - The raw websocket is used for bidirectional, ephemeral presence and cursor data. graphql is used for data queries and mutations. keep these separate, and review the current state on frontend and backend to ensure they are separate. a data mutation performed by the client should not result in a subsequent render update based on a subscription update for that specific mutation, as it was initiated by the same client (other clients on that subscriptions SHOULD receive and respond to the update)


#### LayercakeGraph Editor

 - The LayercakeGraphEditor is based on the same library as the DAG Plan Editor component xyflow / react-flow
 - The LayercakeGraphEditor takes a LcGraph object and renders it
 - The edges have an arrow for direction (source to target)
 - The belongs_to relationships are the (nested) groupings for which the component uses sub flows (https://reactflow.dev/examples/grouping/sub-flows )
 - The layout of the graph is performed dynamically on first render, there is sufficient spacing between the nodes for readability and the top to bottom preference is used for layout
 - The there is a panel on the right side. This panel has a vertical accordion element from mantine.
     - The first panel has a dynamic, editable form containing the properties of the node (node or partition node/subgraph) that has been selected via click. The form has the node attributes (label:string, layer:dropdown of layers + 'None' option). Changes made here will be persisted after focus is lost and reflected in the graph editor
     - The second panel is a placeholder 

#### Datasource management CRUD

 - The datasets pages shows a table of datasets
 - Rows in the table can be selected
 - If rows are selected, an export button activates, pressing the export button gives the option of xlxs or ods
 - Exported datasets are exported as individual sheets to xlxs or ods spreadsheet, with sheets have the id of the dataset as their label. this functionality is implemented on the backend
 - There is an import button on the datasets page, allowing upload of an xlxs or ods spreadsheet in the same format. new datasets are created when a sheet id does not have a corresponding dataset id yet, if a dataset does exist the import of that sheet is its new version


### Artefacts

 - The entire project (minus desktop) and all its functionality is distributed as a single binary
 - The desktop binaries are built after the main standalone binary
 - The binaries will have different defaults depending on its use (desktop app, cli, deployed in container)

### CLI usage

 - All main functionality is exposed via clap.rs commands and subcommands
 - There is a CLI command to self-update from github (layercake update) - this uses the existing capability to do this
 - There is a CLI command to start a server (layercake serve) that starts the axum web server with GraphQL API and serves up the web frontend. the server has sane defaults, but has an option for running with a config file (layercake serve --config=config.yaml)

### Desktop usage

 - Uses OS-specific user profile directories to store data and configuration, and initialises these by default if they are not present

### Data storage

 - The backend uses sqlite by default but can be configured for postgresql
 - The data model has (User <- Group -> Project -> (Import, Plan, Graph). The Graph contains JSON for the graph object(s), the Plan contains JSON for the layercake plan DAG, Import contains imported artifact snapshots (e.g. CSV imports) in serialized format

### APIs

 - GraphQL and MCP (Model Context Protocol) tools share a common backend. GraphQL implementation and MCP implementation live in separate crates and hook into the axum server
 - The common backend can contain functionality that is only exposed via GraphQL OR MCP, it does not have to be both. The MCP API is expected to be richer and allow for deeper interactions
 - GraphQL subscriptions MUST be implemented for real-time collaboration, enabling multiple users to see changes instantly
 - All communication between frontend and backend uses GraphQL only (no REST API)
 - Offline operation support with operation queuing and automatic retry on reconnection

### Distributed synchronisation

 - TASK : research what a fast, efficient, mature, well-maintained CRDT library is and make a recommendation for a CRDT library
 - All graph changes MUST be tracked as CRDT objects with vector clocks for causality tracking
 - Changes are applied using operational transform to resolve conflicts automatically
 - Import objects can be synchronised by bulk copy and overwriting
 - Instances do not auto-discover. Currently only instances that have been configured to connect will synchronise, or single user instances that have authenticated to a multi-user instance
 - Real-time synchronization between instances using GraphQL subscriptions
 - Automatic conflict detection and resolution using CRDT merge semantics
 - Change history tracking for reproducibility and rollback capability

### Authentication

 - In single user mode there is a default user with a profile that can be set up to authenticate against a remote instance of layercake server
 - In server mode layercake can run as a multi-user system, with Users belonging to Groups. All Projects belong to a Group, with Users being able to access all resources in the Group scope (Projects, Graphs, Users, etc). An RBAC system will be implemented later that will limit what Users can do (e.g. only 'admin' role can change other users)
 - In server mode, authentication to a Group can be configured for federation to e.g. a Google organization, a Keycloak realm, etc using appropriate protocols
 - JWT-based authentication with refresh token support
 - Session management with automatic token refresh
 - GraphQL context includes authenticated user information for authorization
 - Role-based access control (RBAC) with predefined roles: admin, editor, viewer
 - Group-level permissions with inheritance to projects and graphs

### Agentic AI

 - All functionality must be exposed as tools via MCP. An agent must be able to access all functionality available to a human user
 - In single user mode via stdio MCP does not use authentication
 - In single user mode via http (streamable) MCP does not use authentication during development, but this will be the default for production
 - The first agent integrations will be using Claude Code and Claude Desktop

### Exports

 - Existing mermaid, plantuml, gml, json, etc targets will remain. They will be configurable OutputNodes from a GraphNode
 - An OutputNode will have existing RenderOptions editable in its node popup editor
 - Interactive visualisations (e.g. Isoflow and force 3d graph) will be available as stubs for now, not active

## Performance and Scalability Requirements

### Real-time Collaboration Performance
 - GraphQL subscriptions must handle up to 100 concurrent users per project
 - Change propagation latency must be under 200ms for local networks
 - CRDT operations must complete within 50ms for typical graph sizes (1000 nodes)
 - Memory usage must remain under 1GB for projects with up to 10,000 nodes

### Database Performance
 - SQLite must handle up to 1M graph nodes with sub-second query times
 - PostgreSQL support for enterprise deployments with larger datasets
 - Database migrations must be versioned and automatically applied
 - Connection pooling for multi-user deployments

### Frontend Performance
 - Initial application load time under 3 seconds
 - Graph rendering performance: 60fps for graphs up to 1000 nodes
 - React component optimization with memo and useMemo for large graphs
 - Virtual scrolling for large node/edge lists in spreadsheet view
