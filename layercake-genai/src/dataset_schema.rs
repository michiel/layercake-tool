/// Dataset generation schema for structured LLM output
///
/// This module defines JSON schemas and types for generating datasets via LLM
/// structured output (rig-core 0.25+ GenerationConfig.response_json_schema).
///
/// The schema ensures type-safe, validated dataset generation that directly
/// deserializes into Layercake's Graph format without manual parsing.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Graph structure matching layercake-core's Graph type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub layers: Vec<Layer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<String>,
}

/// Node structure matching layercake-core's Node type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub is_partition: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to: Option<String>,
    pub weight: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
}

/// Edge structure matching layercake-core's Edge type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub weight: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
}

/// Layer structure matching layercake-core's Layer type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: String,
    pub label: String,
    pub background_color: String,
    pub text_color: String,
    pub border_color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<i32>,
}

/// JSON Schema for dataset generation
///
/// This schema constrains LLM output to valid Graph structures,
/// eliminating parsing errors and ensuring data integrity.
pub fn dataset_generation_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Name of the dataset"
            },
            "nodes": {
                "type": "array",
                "description": "List of graph nodes",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier for the node"
                        },
                        "label": {
                            "type": "string",
                            "description": "Display label for the node"
                        },
                        "layer": {
                            "type": "string",
                            "description": "Layer ID this node belongs to"
                        },
                        "is_partition": {
                            "type": "boolean",
                            "description": "Whether this node represents a partition/group"
                        },
                        "belongs_to": {
                            "type": ["string", "null"],
                            "description": "Parent node ID if this is a child node"
                        },
                        "weight": {
                            "type": "integer",
                            "description": "Node weight/importance (default: 1)"
                        },
                        "comment": {
                            "type": ["string", "null"],
                            "description": "Optional comment or description"
                        }
                    },
                    "required": ["id", "label", "layer", "is_partition", "weight"],
                    "additionalProperties": false
                }
            },
            "edges": {
                "type": "array",
                "description": "List of graph edges/relationships",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier for the edge"
                        },
                        "source": {
                            "type": "string",
                            "description": "Source node ID"
                        },
                        "target": {
                            "type": "string",
                            "description": "Target node ID"
                        },
                        "label": {
                            "type": "string",
                            "description": "Relationship label"
                        },
                        "layer": {
                            "type": "string",
                            "description": "Layer ID this edge belongs to"
                        },
                        "weight": {
                            "type": "integer",
                            "description": "Edge weight/importance (default: 1)"
                        },
                        "comment": {
                            "type": ["string", "null"],
                            "description": "Optional comment or description"
                        }
                    },
                    "required": ["id", "source", "target", "label", "layer", "weight"],
                    "additionalProperties": false
                }
            },
            "layers": {
                "type": "array",
                "description": "List of layer definitions",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "Unique identifier for the layer"
                        },
                        "label": {
                            "type": "string",
                            "description": "Display label for the layer"
                        },
                        "background_color": {
                            "type": "string",
                            "description": "CSS color for layer background (hex or named)"
                        },
                        "text_color": {
                            "type": "string",
                            "description": "CSS color for layer text (hex or named)"
                        },
                        "border_color": {
                            "type": "string",
                            "description": "CSS color for layer border (hex or named)"
                        },
                        "alias": {
                            "type": ["string", "null"],
                            "description": "Optional layer alias"
                        }
                    },
                    "required": ["id", "label", "background_color", "text_color", "border_color"],
                    "additionalProperties": false
                }
            },
            "annotations": {
                "type": ["string", "null"],
                "description": "Optional markdown annotations for the dataset"
            }
        },
        "required": ["name", "nodes", "edges", "layers"],
        "additionalProperties": false
    })
}

/// Wrapper for dataset generation response
///
/// This matches the JSON schema above and deserializes directly
/// into a Graph structure that can be serialized to YAML.
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetGenerationResponse {
    /// Dataset name
    pub name: String,
    /// Graph nodes
    pub nodes: Vec<Node>,
    /// Graph edges
    pub edges: Vec<Edge>,
    /// Layer definitions
    pub layers: Vec<Layer>,
    /// Optional annotations
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<String>,
}

impl From<DatasetGenerationResponse> for Graph {
    fn from(response: DatasetGenerationResponse) -> Self {
        Graph {
            name: response.name,
            nodes: response.nodes,
            edges: response.edges,
            layers: response.layers,
            annotations: response.annotations,
        }
    }
}

impl DatasetGenerationResponse {
    /// Convert to Graph and serialize as YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        let graph = Graph {
            name: self.name.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            layers: self.layers.clone(),
            annotations: self.annotations.clone(),
        };
        serde_yaml::to_string(&graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_has_required_fields() {
        let schema = dataset_generation_schema();

        // Verify top-level structure
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"].is_object());
        assert!(schema["properties"]["nodes"].is_object());
        assert!(schema["properties"]["edges"].is_object());
        assert!(schema["properties"]["layers"].is_object());

        // Verify required fields
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
        assert!(required.contains(&json!("nodes")));
        assert!(required.contains(&json!("edges")));
        assert!(required.contains(&json!("layers")));
    }

    #[test]
    fn test_node_schema_fields() {
        let schema = dataset_generation_schema();
        let node_schema = &schema["properties"]["nodes"]["items"];

        // Verify node has all required fields
        let required = node_schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
        assert!(required.contains(&json!("label")));
        assert!(required.contains(&json!("layer")));
        assert!(required.contains(&json!("is_partition")));
        assert!(required.contains(&json!("weight")));
    }

    #[test]
    fn test_deserialize_valid_response() {
        let json = r##"{
            "name": "Test Dataset",
            "nodes": [
                {
                    "id": "node1",
                    "label": "Node 1",
                    "layer": "layer1",
                    "is_partition": false,
                    "belongs_to": null,
                    "weight": 1,
                    "comment": null
                }
            ],
            "edges": [],
            "layers": [
                {
                    "id": "layer1",
                    "label": "Layer 1",
                    "background_color": "#ffffff",
                    "text_color": "#000000",
                    "border_color": "#cccccc",
                    "alias": null
                }
            ],
            "annotations": null
        }"##;

        let response: DatasetGenerationResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.name, "Test Dataset");
        assert_eq!(response.nodes.len(), 1);
        assert_eq!(response.layers.len(), 1);
    }

    #[test]
    fn test_convert_to_graph() {
        let response = DatasetGenerationResponse {
            name: "Test".to_string(),
            nodes: vec![],
            edges: vec![],
            layers: vec![],
            annotations: Some("Test annotations".to_string()),
        };

        let graph = Graph::from(response);
        assert_eq!(graph.name, "Test");
        assert_eq!(graph.annotations, Some("Test annotations".to_string()));
    }

    #[test]
    fn test_to_yaml_conversion() {
        let response = DatasetGenerationResponse {
            name: "Test Dataset".to_string(),
            nodes: vec![],
            edges: vec![],
            layers: vec![],
            annotations: None,
        };

        let yaml = response.to_yaml().unwrap();
        assert!(yaml.contains("name: Test Dataset"));
        assert!(yaml.contains("nodes: []"));
    }
}
