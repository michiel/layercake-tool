## Summary

 - Layercake is a tool for collecting graph data points, collecting then into a graph, allowing versioning and editing of the graph, and then exporting the full graph or selected subsets of the graph to various formats that can be configured multiple times, individually.
 - Layercake is a single binary that can be used as a library, a CLI tool, or a web service. It is designed to be extensible and modular, allowing users to add their own data sources, exporters, and other components.
 - Layercake started as a plan runner, where a YAML plan defines inputs (nodes, edges, layers in CSV format) and runs a pipeline execution for the plan, generating multiple outputs that are templated using handlebars (example: PlantUML output via feeding the graph data into a PlantUML handlebars template).
 - Layercake is going to be running a server that can serve multiple projects, each with inputs, its own graph, transformations for outputs, and configured exporters, using the current plan runner as a base
 - Data will be persisted in a sqlite database via seaorm (with PostgreSQL support for production)
 - The server will expose a full set of tools for all operations via a MCP API for AI Agent and LLM interactions
 - The server will also expose a REST API for web applications and other clients to interact with the data
 - The server will also expose a GraphQL API for advanced querying and manipulation of the project, plan and graph data
 - MCP, GraphQL and REST will all use a unified backend
 - The project will remain a single binary, with the CLI tool and web service being two different modes of operation
 - CLI one-off plan execution can be performed using the same data model, possibly with an in-memory SQL database and a default project initialized specifically for the run and then discarded
 - Outputs will be written to a directory structure that can be configured per project, with the ability to override the output directory for one-off runs
 - As a new capability, outputs will also be exposed directly as inputs to web components and other clients, allowing for dynamic updates and interactions (example: a react component rendering the graph and updating as the graph changes)
 - Layercake will have a react frontend that can be used to interact with the server, allowing users to view and edit their projects, plans, and graphs as well as preview outputs and interact with the outputs dynamically

## Plan Data Format

 - Plans are stored as JSON in the database (migrated from YAML for better web integration)
 - Plan content supports schema validation for structural integrity
 - Plans support incremental editing via JSON Patch operations for real-time collaboration
 - Legacy YAML plans are automatically converted to JSON format during migration
 - Plan schema versioning enables evolution of plan structure over time

## Frontend Architecture

 - React frontend with TypeScript for type safety and modern development experience
 - Modular component architecture with dynamic loading for large editors (ReactFlow, Isoflow)
 - Component registry system enables pluggable editor components
 - Development mode: separate React dev server with API proxy for hot reload
 - Production mode: single binary serves embedded HTML shell loading CDN assets
 - Frontend assets deployed to CDN (GitHub Pages/jsdelivr) via CI/CD for global distribution

## Data Editing Interfaces

 - Multi-mode plan editor supporting three editing approaches:
   - Rich text YAML editor with syntax highlighting (legacy compatibility)
   - Visual ReactFlow editor for DAG-based plan construction
   - JSON patch editor for incremental real-time editing
 - Spreadsheet-like interface for bulk data editing with tabs for layers, nodes, and edges
 - Excel-like functionality including keyboard shortcuts, bulk operations, and data validation
 - Real-time data synchronization across all editing interfaces via GraphQL subscriptions

## Graph Operations

 - Graph versioning system with diff capabilities for tracking changes over time
 - Advanced graph transformations and filtering for subset selection
 - Connectivity analysis and path finding algorithms for graph insights
 - Interactive graph visualization with drag-and-drop editing capabilities
 - Performance optimization for large graphs (10,000+ nodes) with virtualization

## Deployment Model

 - Single binary deployment containing both backend and frontend
 - CDN-first asset delivery with local fallback for offline operation
 - Automatic cache busting using Git commit hashes for asset versioning
 - Cross-platform support (Linux, macOS, Windows) with hybrid TLS architecture
 - Development workflow enables seamless frontend/backend development with hot reload





