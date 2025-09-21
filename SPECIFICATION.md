# Layercake plan

## Goals and high level requirements

The overall goal is to change this pipeline graph transformation and creation tool to an interactive application that can be used to create and edit a graph visually, spawn child graphs from a parent graph, with changes higher up in the hierarchy propagating downwards. Multiple instances of the layercake tool can run at the same time and synchronise changes in real time, updating the user interface in real time. A desktop instance can connect to a server instance (any instance to any instance).

Agentic AI with tooling exposed via Model Context Protocol for agentic collaboration is a key goal.

Building this as a distributed and collaborative graph editing platform is a later goal and not a priority for version 1.0.0

### Layercake plans and graphs

 - All aspected of the layercake process from ingestion to transformation to renderings are stored as a DAG. This is the layercake plan. If an upstream node (e.g. representing ingestion of a CSV as a nodeset) changes, all downstream nodes are updated
 - Graph data can be imported from CSVs (the existing layercake source format), REST endpoints, SQL querie (react component name : PlanVisualEditor)
 - The layercake plan (PlanDAG) can be edited (visually using react-flow / xflow, component name PlanVisualEditor), with the inputs, graph hierarchy and exports of different graphs in the hierarchy all represented and editable from the same interface
 - Graphs (or more specifically LayercakeGraphs) can be edited using the spreadsheet editor component that has three tabs for nodes, edges, layers (react component name : GraphSpreadsheetEditor) OR using another instance of react-flow / xflow for graph editing (react component name : GraphVisualEditor)
 - Changes to Graphs are tracked and shared as CRDT objects and stored in a database table (Project -> Changes)
 - Each Project has a Plan DAG. The Plan DAG is a JSON object that is an attribute of the Project table
 - Plans contains graph metadata and the relationships between graph copies. Plans do not contain actual graph data (e.g. nodes, edges), graph data is in the Graph table. Plan DAG steps can be (but are not limited to) nodes that are InputNode (from file, etc), MergeNode (from InputNode and/or GraphNode, output GraphNode), CopyNode (from GraphNode, output GraphNode), OutputNode (from GraphNode, output OutputNode (render target)), GraphNode (which are references and metadata for a graph instance), TransformNode (from graph, output graph) that contain the existing graph transformation options in the current YAML. The Plan DAG is generally editing in PlanVisualEditor, with popup items to edit node attributes (example: InputNode editor will have a selectable type for File that allows for selecting file location and data type (nodes, layers, edges)) and node metadata (example: node label)
 - A key function is that edits to graphs (via GraphVisualEditor or GraphSpreadsheetEditor) are tracked and reproducible, so re-applying inputs and re-running the DAG, will re-run the edits (if they are still applicable, removing them from tracking otherwise)

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
  - for REST APIs (if present) utoipa with utoipa-swagger-ui
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

### Testing

 - All functionality must have test coverage

## Requirements

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
 
### Distributed synchronisation

 - TASK : research what a fast, efficient, mature, well-maintained CRDT library is and make a recommendation for a CRDT library
 - Import objects can be synchronised by bulk copy and overwriting
 - Instances do not auto-discover. Currently only instances that have been configured to connect will synchronise, or single user instances that have authenticated to a multi-user instance

### Authentication

 - In single user mode there is a default user with a profile that can be set up to authenticate against a remote instance of layercake server
 - In server mode layercake can run as a multi-user system, with Users belonging to Groups. All Projects belong to a Group, with Users being able to access all resources in the Group scope (Projects, Graphs, Users, etc). An RBAC system will be implemented later that will limit what Users can do (e.g. only 'admin' role can change other users)
 - In server mode, authentication to a Group can be configured for federation to e.g. a Google organization, a Keycloak realm, etc using appropriate protocols

### Agentic AI

 - All functionality must be exposed as tools via MCP. An agent must be able to access all functionality available to a human user
 - In single user mode via stdio MCP does not use authentication
 - In single user mode via http (streamable) MCP does not use authentication during development, but this will be the default for production
 - The first agent integrations will be using Claude Code and Claude Desktop

### Exports

 - Existing mermaid, plantuml, gml, json, etc targets will remain. They will be configurable OutputNodes from a GraphNode
 - An OutputNode will have existing RenderOptions editable in its node popup editor
 - Interactive visualisations (e.g. Isoflow and force 3d graph) will be available as stubs for now, not active

