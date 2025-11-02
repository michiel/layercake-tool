# Layercake Data Model

Layercake is a graph analysis and transformation tool. The data model consists of:

## Core Entities

- **Projects**: Top-level containers for all data and plans
- **Data Sources**: CSV/JSON files containing nodes and edges
- **Graphs**: Computed graph structures with nodes, edges, and layers
- **Plans**: YAML definitions of data transformations
- **Plan DAG**: Visual workflow representation with nodes and edges

## Graph Components

- **Nodes**: Graph vertices with attributes (id, label, layer, weight, attrs)
- **Edges**: Connections between nodes (id, source, target, label, weight)
- **Layers**: Groupings of nodes/edges for visualisation

## Workflow

1. Import data sources (nodes.csv, edges.csv)
2. Create plan with transformation steps
3. Execute plan to generate computed graphs
4. Export results in various formats (GraphML, GEXF, etc.)

When users ask about their data, use the appropriate tools to query graphs, analyse structures, or inspect plan configurations.
