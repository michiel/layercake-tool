use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
pub struct SchemaDescription {
    pub entity: String,
    pub fields: Vec<FieldSchema>,
    pub example: serde_json::Value,
}

#[derive(Serialize)]
pub struct FieldSchema {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: bool,
    pub description: String,
    pub values: Option<Vec<String>>,
}

pub fn get_node_create_schema(node_type: Option<&str>) -> SchemaDescription {
    let base_fields = vec![
        FieldSchema {
            name: "nodeType".to_string(),
            field_type: "string".to_string(),
            required: true,
            description: "Type of node to create".to_string(),
            values: Some(vec![
                "DataSetNode".to_string(),
                "GraphNode".to_string(),
                "GraphArtefactNode".to_string(),
                "TreeArtefactNode".to_string(),
                "ProjectionNode".to_string(),
                "StoryNode".to_string(),
            ]),
        },
        FieldSchema {
            name: "position".to_string(),
            field_type: "Position".to_string(),
            required: true,
            description: "Canvas position {x: number, y: number}".to_string(),
            values: None,
        },
        FieldSchema {
            name: "metadata".to_string(),
            field_type: "object".to_string(),
            required: true,
            description: "Node metadata (label, description)".to_string(),
            values: None,
        },
        FieldSchema {
            name: "config".to_string(),
            field_type: "object".to_string(),
            required: true,
            description: "Node-specific configuration".to_string(),
            values: None,
        },
    ];

    let example = match node_type {
        Some("GraphNode") => json!({
            "nodeType": "GraphNode",
            "position": {"x": 100.0, "y": 200.0},
            "metadata": {
                "label": "My Graph",
                "description": "Graph description"
            },
            "config": {"metadata": {}}
        }),
        Some("DataSetNode") => json!({
            "nodeType": "DataSetNode",
            "position": {"x": 100.0, "y": 200.0},
            "metadata": {"label": "Dataset"},
            "config": {"dataSetId": 123}
        }),
        Some("GraphArtefactNode") => json!({
            "nodeType": "GraphArtefactNode",
            "position": {"x": 100.0, "y": 200.0},
            "metadata": {"label": "Artefact"},
            "config": {
                "renderTarget": "Mermaid",
                "renderConfig": {
                    "orientation": "LR",
                    "containNodes": false
                },
                "outputPath": "",
                "graphConfig": {}
            }
        }),
        _ => json!({
            "nodeType": "GraphNode",
            "position": {"x": 100.0, "y": 200.0},
            "metadata": {"label": "Example"},
            "config": {}
        }),
    };

    SchemaDescription {
        entity: "nodes".to_string(),
        fields: base_fields,
        example,
    }
}

pub fn get_edge_create_schema() -> SchemaDescription {
    let fields = vec![
        FieldSchema {
            name: "source".to_string(),
            field_type: "string".to_string(),
            required: true,
            description: "Source node ID".to_string(),
            values: None,
        },
        FieldSchema {
            name: "target".to_string(),
            field_type: "string".to_string(),
            required: true,
            description: "Target node ID".to_string(),
            values: None,
        },
        FieldSchema {
            name: "metadata".to_string(),
            field_type: "object".to_string(),
            required: true,
            description: "Edge metadata (label, data_type)".to_string(),
            values: None,
        },
    ];

    let example = json!({
        "source": "dataset_abc123",
        "target": "graph_def456",
        "metadata": {
            "label": "Data",
            "data_type": "GraphData"
        }
    });

    SchemaDescription {
        entity: "edges".to_string(),
        fields,
        example,
    }
}

pub fn get_available_actions(entity: &str) -> Vec<String> {
    match entity {
        "datasets" => vec!["list".to_string(), "get".to_string()],
        "plans" => vec!["list".to_string(), "get".to_string()],
        "nodes" => vec![
            "list".to_string(),
            "get".to_string(),
            "create".to_string(),
            "update".to_string(),
            "delete".to_string(),
            "move".to_string(),
            "traverse".to_string(),
            "search".to_string(),
            "batch".to_string(),
            "clone".to_string(),
        ],
        "edges" => vec![
            "create".to_string(),
            "update".to_string(),
            "delete".to_string(),
        ],
        "exports" => vec!["download".to_string()],
        "schema" => vec!["get".to_string(), "list".to_string()],
        "analysis" => vec!["get".to_string()],
        "annotations" => vec![
            "create".to_string(),
            "list".to_string(),
            "get".to_string(),
            "update".to_string(),
            "delete".to_string(),
        ],
        _ => vec![],
    }
}

pub fn get_node_types() -> Vec<String> {
    vec![
        "DataSetNode".to_string(),
        "GraphNode".to_string(),
        "GraphArtefactNode".to_string(),
        "TreeArtefactNode".to_string(),
        "ProjectionNode".to_string(),
        "StoryNode".to_string(),
        "SequenceArtefactNode".to_string(),
    ]
}

pub fn get_export_formats() -> Vec<String> {
    vec![
        "Mermaid".to_string(),
        "DOT".to_string(),
        "JSON".to_string(),
        "CSV".to_string(),
    ]
}
