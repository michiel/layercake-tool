# Plan DAG JSON Schema Design

## Overview

Based on the SPECIFICATION.md requirements and existing transformation system, the Plan DAG must be stored as a JSON object in the Project table with 6 node types that integrate with the existing plan execution system.

## JSON Schema Structure

### Complete Plan DAG JSON

```json
{
  "version": "1.0",
  "nodes": [
    {
      "id": "input_1",
      "type": "InputNode",
      "position": { "x": 100, "y": 100 },
      "metadata": {
        "label": "CSV Import",
        "description": "Import nodes from CSV file"
      },
      "config": {
        "inputType": "CSVNodesFromFile",
        "source": "import/nodes.csv",
        "dataType": "Nodes",
        "outputGraphRef": "graph_main"
      }
    },
    {
      "id": "graph_1",
      "type": "GraphNode",
      "position": { "x": 300, "y": 100 },
      "metadata": {
        "label": "Main Graph",
        "description": "Primary graph instance"
      },
      "config": {
        "graphId": 123,
        "isReference": true,
        "metadata": {
          "nodeCount": 150,
          "edgeCount": 200,
          "lastModified": "2025-01-20T10:30:00Z"
        }
      }
    },
    {
      "id": "transform_1",
      "type": "TransformNode",
      "position": { "x": 500, "y": 100 },
      "metadata": {
        "label": "Limit Depth",
        "description": "Limit partition depth to 3"
      },
      "config": {
        "inputGraphRef": "graph_1",
        "outputGraphRef": "graph_limited",
        "transformType": "PartitionDepthLimit",
        "transformConfig": {
          "maxPartitionDepth": 3,
          "generateHierarchy": false,
          "invertGraph": false
        }
      }
    },
    {
      "id": "merge_1",
      "type": "MergeNode",
      "position": { "x": 700, "y": 200 },
      "metadata": {
        "label": "Merge Graphs",
        "description": "Combine multiple graph sources"
      },
      "config": {
        "inputRefs": ["input_1", "graph_limited"],
        "outputGraphRef": "graph_merged",
        "mergeStrategy": "Union",
        "conflictResolution": "PreferFirst"
      }
    },
    {
      "id": "copy_1",
      "type": "CopyNode",
      "position": { "x": 900, "y": 100 },
      "metadata": {
        "label": "Create Variant",
        "description": "Create modified copy for scenario analysis"
      },
      "config": {
        "sourceGraphRef": "graph_merged",
        "outputGraphRef": "graph_variant_a",
        "copyType": "DeepCopy",
        "preserveMetadata": true
      }
    },
    {
      "id": "output_1",
      "type": "OutputNode",
      "position": { "x": 1100, "y": 100 },
      "metadata": {
        "label": "Export DOT",
        "description": "Generate Graphviz DOT output"
      },
      "config": {
        "sourceGraphRef": "graph_variant_a",
        "renderTarget": "DOT",
        "outputPath": "output/variant_a.dot",
        "renderConfig": {
          "containNodes": true,
          "orientation": "TB"
        },
        "graphConfig": {
          "generateHierarchy": false,
          "maxPartitionDepth": null,
          "maxPartitionWidth": null,
          "invertGraph": false,
          "nodeLabelMaxLength": 20,
          "nodeLabelInsertNewlinesAt": 10,
          "edgeLabelMaxLength": 15,
          "edgeLabelInsertNewlinesAt": 8
        }
      }
    }
  ],
  "edges": [
    {
      "id": "edge_1",
      "source": "input_1",
      "target": "merge_1",
      "metadata": {
        "label": "Nodes",
        "dataType": "GraphData"
      }
    },
    {
      "id": "edge_2",
      "source": "graph_1",
      "target": "transform_1",
      "metadata": {
        "label": "Graph Reference",
        "dataType": "GraphReference"
      }
    },
    {
      "id": "edge_3",
      "source": "transform_1",
      "target": "merge_1",
      "metadata": {
        "label": "Transformed",
        "dataType": "GraphData"
      }
    },
    {
      "id": "edge_4",
      "source": "merge_1",
      "target": "copy_1",
      "metadata": {
        "label": "Merged Graph",
        "dataType": "GraphData"
      }
    },
    {
      "id": "edge_5",
      "source": "copy_1",
      "target": "output_1",
      "metadata": {
        "label": "Graph Copy",
        "dataType": "GraphData"
      }
    }
  ],
  "metadata": {
    "createdAt": "2025-01-20T10:00:00Z",
    "lastModified": "2025-01-20T10:30:00Z",
    "version": "1.0",
    "description": "Main data processing pipeline"
  }
}
```

## Node Type Definitions

### 1. InputNode

Imports data from external sources (CSV, REST, SQL) and creates graph data.

```typescript
interface InputNodeConfig {
  inputType: "CSVNodesFromFile" | "CSVEdgesFromFile" | "CSVLayersFromFile" | "REST" | "SQL";
  source: string;              // File path, URL, or connection string
  dataType: "Nodes" | "Edges" | "Layers";
  outputGraphRef: string;      // Reference to output graph

  // CSV-specific options (optional)
  csvOptions?: {
    skipRows?: number;
    separator?: string;
    encoding?: string;
  };

  // REST-specific options (optional)
  restOptions?: {
    method?: "GET" | "POST";
    headers?: Record<string, string>;
    authentication?: {
      type: "bearer" | "basic" | "apikey";
      credentials: string;
    };
  };

  // SQL-specific options (optional)
  sqlOptions?: {
    connectionString: string;
    query: string;
    parameters?: Record<string, any>;
  };
}
```

### 2. GraphNode

References existing graph instances with metadata.

```typescript
interface GraphNodeConfig {
  graphId: number;             // Database ID of the graph
  isReference: boolean;        // true for references, false for embedded copies

  metadata: {
    nodeCount?: number;
    edgeCount?: number;
    layerCount?: number;
    lastModified?: string;     // ISO timestamp
    tags?: string[];
    customProperties?: Record<string, any>;
  };

  // Optional: For embedded copies
  embeddedGraph?: {
    name: string;
    nodes: any[];              // Full graph data if embedded
    edges: any[];
    layers: any[];
  };
}
```

### 3. TransformNode

Applies transformations using existing transformation system.

```typescript
interface TransformNodeConfig {
  transforms: GraphTransform[];
}

type GraphTransformKind =
  | 'PartitionDepthLimit'
  | 'PartitionWidthLimit'
  | 'NodeLabelMaxLength'
  | 'NodeLabelInsertNewlines'
  | 'EdgeLabelMaxLength'
  | 'EdgeLabelInsertNewlines'
  | 'InvertGraph'
  | 'GenerateHierarchy'
  | 'AggregateEdges';

interface GraphTransform {
  kind: GraphTransformKind;
  params: GraphTransformParams;
}

interface GraphTransformParams {
  maxPartitionDepth?: number;
  maxPartitionWidth?: number;
  nodeLabelMaxLength?: number;
  nodeLabelInsertNewlinesAt?: number;
  edgeLabelMaxLength?: number;
  edgeLabelInsertNewlinesAt?: number;
  enabled?: boolean;
}
```

> **Notes**
> - `AggregateEdges` is appended by default and can be disabled via `params.enabled = false`.
> - Transformations are executed strictly in array order, allowing users to mix depth/width limits, label adjustments, and inversion in a single node.

### 4. MergeNode

Combines multiple graph sources (InputNodes and/or GraphNodes).

```typescript
interface MergeNodeConfig {
  inputRefs: string[];         // Array of InputNode or GraphNode IDs
  outputGraphRef: string;      // Target graph reference
  mergeStrategy: MergeStrategy;
  conflictResolution?: ConflictResolution;

  // Advanced merge options
  mergeOptions?: {
    preserveNodeIds?: boolean;
    preserveEdgeIds?: boolean;
    mergeMetadata?: boolean;
    layerMergeStrategy?: "union" | "intersection" | "first_wins";
  };
}

type MergeStrategy =
  | "Union"                    // Combine all nodes and edges
  | "Intersection"             // Only nodes/edges present in all inputs
  | "Difference"               // Remove second input from first
  | "Custom";                  // Custom merge logic

type ConflictResolution =
  | "PreferFirst"              // Use data from first input in conflicts
  | "PreferLast"               // Use data from last input in conflicts
  | "Merge"                    // Attempt to merge conflicting data
  | "Error";                   // Fail on conflicts
```

### 5. CopyNode

Creates copies of graphs for scenario analysis.

```typescript
interface CopyNodeConfig {
  sourceGraphRef: string;      // Source graph reference
  outputGraphRef: string;      // Target graph reference
  copyType: CopyType;
  preserveMetadata?: boolean;

  copyOptions?: {
    copyName?: string;         // Name for the copied graph
    copyDescription?: string;  // Description for the copied graph
    preserveIds?: boolean;     // Keep original node/edge IDs
    copyLayers?: boolean;      // Include layer definitions
  };
}

type CopyType =
  | "DeepCopy"                 // Full independent copy
  | "ShallowCopy"              // Reference-based copy
  | "PartialCopy";             // Copy with filters
```

### 6. OutputNode

Exports graphs to various formats using existing export system.

```typescript
interface OutputNodeConfig {
  sourceGraphRef: string;      // Source graph reference
  renderTarget: RenderTarget;
  outputPath: string;          // File path or URL for output

  // Render configuration (maps to ExportProfileRenderConfig)
  renderConfig?: {
    containNodes?: boolean;
    orientation?: "LR" | "TB";
    customTemplate?: string;   // For custom exports
    partials?: Record<string, string>;
  };

  // Graph configuration (maps to ExportProfileGraphConfig)
  graphConfig?: {
    generateHierarchy?: boolean;
    maxPartitionDepth?: number | null;
    maxPartitionWidth?: number | null;
    invertGraph?: boolean;
    nodeLabelMaxLength?: number;
    nodeLabelInsertNewlinesAt?: number;
    edgeLabelMaxLength?: number;
    edgeLabelInsertNewlinesAt?: number;
  };
}

type RenderTarget =
  | "DOT"                      // Graphviz DOT
  | "GML"                      // Graph Modeling Language
  | "JSON"                     // JSON export
  | "PlantUML"                 // PlantUML format
  | "Mermaid"                  // Mermaid diagrams
  | "CSVNodes"                 // CSV node export
  | "CSVEdges"                 // CSV edge export
  | "CSVMatrix"                // CSV adjacency matrix
  | "JSGraph"                  // JavaScript graph format
  | "Custom";                  // Custom template export
```

## Integration with Existing System

### Mapping to Current Plan Structure

The new Plan DAG nodes map to existing structures:

```rust
// Transform existing Plan -> Plan DAG conversion
impl Plan {
    pub fn to_plan_dag(&self) -> PlanDAG {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_counter = 0;

        // Convert import profiles to InputNodes
        for import_profile in &self.import.profiles {
            nodes.push(PlanDAGNode {
                id: format!("input_{}", node_counter),
                node_type: PlanDAGNodeType::InputNode,
                config: serde_json::to_value(InputNodeConfig {
                    input_type: match import_profile.filetype {
                        ImportFileType::Nodes => "CSVNodesFromFile".to_string(),
                        ImportFileType::Edges => "CSVEdgesFromFile".to_string(),
                        ImportFileType::Layers => "CSVLayersFromFile".to_string(),
                    },
                    source: import_profile.filename.clone(),
                    data_type: match import_profile.filetype {
                        ImportFileType::Nodes => "Nodes".to_string(),
                        ImportFileType::Edges => "Edges".to_string(),
                        ImportFileType::Layers => "Layers".to_string(),
                    },
                    output_graph_ref: format!("graph_{}", node_counter),
                    csv_options: None,
                    rest_options: None,
                    sql_options: None,
                }).unwrap(),
                position: Position { x: 100.0 * node_counter as f64, y: 100.0 },
                metadata: NodeMetadata {
                    label: format!("Import {}", import_profile.filename),
                    description: Some(format!("Import {} from CSV",
                        match import_profile.filetype {
                            ImportFileType::Nodes => "nodes",
                            ImportFileType::Edges => "edges",
                            ImportFileType::Layers => "layers",
                        }
                    )),
                },
            });
            node_counter += 1;
        }

        // Convert export profiles to TransformNode + OutputNode chains
        for export_profile in &self.export.profiles {
            // Add TransformNode if graph_config is present
            if let Some(graph_config) = export_profile.graph_config {
                nodes.push(PlanDAGNode {
                    id: format!("transform_{}", node_counter),
                    node_type: PlanDAGNodeType::TransformNode,
                    config: serde_json::to_value(TransformNodeConfig {
                        input_graph_ref: format!("graph_{}", node_counter - 1),
                        output_graph_ref: format!("graph_transformed_{}", node_counter),
                        transform_type: "Custom".to_string(),
                        transform_config: TransformConfig::from_export_graph_config(graph_config),
                    }).unwrap(),
                    // ... rest of node configuration
                });
                node_counter += 1;
            }

            // Add OutputNode
            nodes.push(PlanDAGNode {
                id: format!("output_{}", node_counter),
                node_type: PlanDAGNodeType::OutputNode,
                config: serde_json::to_value(OutputNodeConfig {
                    source_graph_ref: format!("graph_transformed_{}", node_counter - 1),
                    render_target: export_profile.exporter.to_string(),
                    output_path: export_profile.filename.clone(),
                    render_config: export_profile.render_config.map(|rc| rc.into()),
                    graph_config: export_profile.graph_config.map(|gc| gc.into()),
                }).unwrap(),
                // ... rest of node configuration
            });
            node_counter += 1;
        }

        PlanDAG { nodes, edges, metadata: PlanDAGMetadata::default() }
    }
}
```

### Database Storage

```sql
-- Updated projects table with Plan DAG JSON
CREATE TABLE projects (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    group_id INTEGER NOT NULL,
    plan_dag TEXT NOT NULL,      -- JSON: Complete Plan DAG structure
    created_by INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (group_id) REFERENCES groups(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- JSON validation constraint (SQLite 3.38+)
-- ALTER TABLE projects ADD CHECK (json_valid(plan_dag));
```

### Rust Data Structures

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanDAG {
    pub version: String,
    pub nodes: Vec<PlanDAGNode>,
    pub edges: Vec<PlanDAGEdge>,
    pub metadata: PlanDAGMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanDAGNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: PlanDAGNodeType,
    pub position: Position,
    pub metadata: NodeMetadata,
    pub config: serde_json::Value,  // Node-specific configuration
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlanDAGNodeType {
    InputNode,
    GraphNode,
    TransformNode,
    MergeNode,
    CopyNode,
    OutputNode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NodeMetadata {
    pub label: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanDAGEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub metadata: EdgeMetadata,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EdgeMetadata {
    pub label: String,
    pub data_type: String,  // "GraphData", "GraphReference", etc.
}
```

## Validation and Execution

### Plan DAG Validation

```rust
impl PlanDAG {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check for cycles
        if self.has_cycles() {
            errors.push(ValidationError::CyclicDependency);
        }

        // Validate node references
        for edge in &self.edges {
            if !self.nodes.iter().any(|n| n.id == edge.source) {
                errors.push(ValidationError::InvalidNodeReference(edge.source.clone()));
            }
            if !self.nodes.iter().any(|n| n.id == edge.target) {
                errors.push(ValidationError::InvalidNodeReference(edge.target.clone()));
            }
        }

        // Validate node configurations
        for node in &self.nodes {
            if let Err(e) = self.validate_node_config(node) {
                errors.push(ValidationError::InvalidNodeConfig(node.id.clone(), e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn get_execution_order(&self) -> Result<Vec<String>, ExecutionError> {
        // Topological sort of DAG nodes
        // Returns ordered list of node IDs for execution
    }
}
```

## Schema Evolution

### Version Management

```json
{
  "version": "1.0",
  "schemaVersion": "2025.1",
  "migrations": [
    {
      "fromVersion": "1.0",
      "toVersion": "1.1",
      "description": "Added support for SQL input nodes",
      "migrationScript": "migrations/plan_dag_1_0_to_1_1.sql"
    }
  ]
}
```

This JSON schema design provides:

1. **Full Integration**: Maps existing Plan structure to new Plan DAG format
2. **Type Safety**: Strong typing for all node configurations
3. **Extensibility**: Easy to add new node types and configurations
4. **Validation**: Comprehensive validation and execution ordering
5. **Backward Compatibility**: Migration path from existing YAML plans
6. **Performance**: JSON storage with optional indexing on node types

The schema supports all 6 required node types while maintaining integration with the existing robust transformation and export system.
