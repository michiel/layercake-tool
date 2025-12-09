# Gemini Code Assistant Context

## Project Overview

This project, named "Layercake", is a collaborative, real-time, interactive graph editing platform. It allows users to create, edit, and manage complex graph structures through a visual interface. The core of the project is the "Plan DAG," a directed acyclic graph that defines the entire data flow, from data ingestion to transformation and export.

The project is built with a Rust backend and a React frontend. The backend uses Axum for the web server, SeaORM for database interaction, and async-graphql for the GraphQL API. The frontend is built with React, TypeScript, Vite, Mantine for UI components, and React Flow for graph visualization.

## Key Features

*   **Plan DAG:** A visual editor for defining data pipelines, including data sources, transformations, and outputs.
*   **LayercakeGraph:** First-class graph objects that can be copied, transformed, and merged within the Plan DAG.
*   **Real-time Collaboration:** Multiple users can edit the same project simultaneously, with changes propagated in real-time via WebSockets and GraphQL subscriptions.
*   **Agentic AI:** All functionality is exposed via the Model Context Protocol (MCP) to allow AI agents to interact with the system.
*   **Tauri Desktop App:** A desktop version of the application is available, built with Tauri.

## Project Structure

The project is a Rust workspace with the following main components:

*   `layercake-core`: The core Rust crate containing the main business logic, including:
    *   `main.rs`: The entry point for the CLI application.
    *   `lib.rs`: The main library file, which defines the module structure.
    *   `server`: The Axum web server, including the GraphQL API and WebSocket handling.
    *   `database`: Database entities and migrations using SeaORM.
    *   `plan`: Logic for creating, parsing, and executing Layercake plans.
    *   `graph`: Graph data structures and algorithms.
    *   `collaboration`: Real-time collaboration features.
    *   `graphql`: The GraphQL schema and resolvers.
    *   `mcp`: The Model Context Protocol implementation.
*   `src-tauri`: The Tauri application, which wraps the web frontend.
*   `frontend`: The React frontend application, with the following structure:
    *   `src/main.tsx`: The entry point for the React application.
    *   `src/App.tsx`: The main application component.
    *   `src/graphql`: Apollo Client setup, queries, mutations, and subscriptions.
    *   `src/components`: React components for the UI.

## Building and Running

### CLI

To run the CLI application:

```bash
cargo run -- -p sample/kvm_control_flow/plan.yaml
```

### Frontend

To run the frontend in development mode:

```bash
cd frontend
npm install
npm run dev
```

### Tauri

To build and run the Tauri application:

```bash
cargo tauri dev
```

## Key Dependencies

### Backend (Rust)

*   `tokio`: Asynchronous runtime.
*   `axum`: Web framework.
*   `sea-orm`: Database ORM.
*   `async-graphql`: GraphQL server.
*   `clap`: Command-line argument parsing.
*   `serde`: Serialization and deserialization.
*   `tauri`: Desktop application framework.

### Frontend (TypeScript/React)

*   `react`: UI library.
*   `react-router-dom`: Routing.
*   `@apollo/client`: GraphQL client.
*   `@mantine/core`: UI component library.
*   `reactflow`: Graph visualization.
*   `vite`: Build tool.

## Development Conventions

*   The project uses `rustfmt` for code formatting.
*   The frontend code is written in TypeScript.
*   The project uses `npm` for package management.
*   The project uses `Vite` for building the frontend.
*   The project uses `GraphQL` for the API.
*   The project uses `WebSockets` for real-time collaboration.
*   The project uses `Tauri` for the desktop application.
*   The project uses `SeaORM` for database access.
*   The project uses `Axum` for the web server.