//! Graph data management tools for MCP

use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};
use axum_mcp::prelude::*;
use chrono::Utc;
use csv::ReaderBuilder;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};
use serde_json::{json, Map, Value};

use crate::database::entities::execution_state::ExecutionState;
use crate::database::entities::{
    graph_edges::{self, Column as GraphEdgeColumn, Entity as GraphEdges},
    graph_nodes::{self, Column as GraphNodeColumn, Entity as GraphNodes},
    graphs::{self, Column as GraphColumn, Entity as Graphs},
    graph_layers::{Column as LayerColumn, Entity as Layers},
    plan_dag_nodes,
};
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use crate::plan::ExportFileType;
use crate::services::{ExportService, GraphService, ImportService, PlanDagService};
use uuid::Uuid;

/// Declare the MCP tools made available by this module.
pub fn get_graph_data_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "import_csv".to_string(),
            description: "Import graph data (nodes, edges, graph_layers) from CSV content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to import data into"
                    },
                    "nodes_csv": {
                        "type": "string",
                        "description": "CSV content describing nodes (optional)"
                    },
                    "edges_csv": {
                        "type": "string",
                        "description": "CSV content describing edges (optional)"
                    },
                    "layers_csv": {
                        "type": "string",
                        "description": "CSV content describing graph_layers (optional)"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "export_graph".to_string(),
            description: "Export graph data in a supported format".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to export"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["json", "csv_nodes", "csv_edges", "dot", "gml", "plantuml", "mermaid"],
                        "description": "Export format"
                    }
                },
                "required": ["project_id", "format"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_graph_data".to_string(),
            description: "Retrieve graph structure (nodes, edges, graph_layers)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": {
                        "type": "integer",
                        "description": "ID of the project to inspect"
                    },
                    "include_nodes": {
                        "type": "boolean",
                        "description": "Include node samples in the response (default: true)"
                    },
                    "include_edges": {
                        "type": "boolean",
                        "description": "Include edge samples in the response (default: true)"
                    },
                    "include_layers": {
                        "type": "boolean",
                        "description": "Include layer samples in the response (default: true)"
                    }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

/// Import graph data from CSV snippets.
pub async fn import_csv(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = parse_project_id(&arguments)?;
    let nodes_csv = get_optional_param(&arguments, "nodes_csv").and_then(Value::as_str);
    let edges_csv = get_optional_param(&arguments, "edges_csv").and_then(Value::as_str);
    let layers_csv = get_optional_param(&arguments, "layers_csv").and_then(Value::as_str);

    if nodes_csv.is_none() && edges_csv.is_none() && layers_csv.is_none() {
        return Err(McpError::Validation {
            message: "Provide at least one of nodes_csv, edges_csv, or layers_csv".to_string(),
        });
    }

    let graph = ensure_graph(project_id, db)
        .await
        .map_err(|e| internal_error("Failed to prepare graph for import", e))?;

    let mut nodes_imported = 0usize;
    let mut edges_imported = 0usize;
    let mut layers_imported = 0usize;

    if let Some(nodes_csv) = nodes_csv {
        nodes_imported = import_nodes_csv(db, graph.id, nodes_csv)
            .await
            .map_err(|e| internal_error("Failed to import nodes CSV", e))?;
    }

    if let Some(edges_csv) = edges_csv {
        edges_imported = import_edges_csv(db, graph.id, edges_csv)
            .await
            .map_err(|e| internal_error("Failed to import edges CSV", e))?;
    }

    if let Some(layers_csv) = layers_csv {
        let service = ImportService::new(db.clone());
        layers_imported = service
            .import_layers_from_csv(graph.id, layers_csv)
            .await
            .map_err(|e| internal_error("Failed to import graph_layers CSV", e))?;
    }

    update_graph_counts(db, graph.id)
        .await
        .map_err(|e| internal_error("Failed to update graph statistics", e))?;

    let result = json!({
        "project_id": project_id,
        "graph_id": graph.id,
        "nodes_imported": nodes_imported,
        "edges_imported": edges_imported,
        "layers_imported": layers_imported
    });

    create_success_response(&result)
}

/// Export the active graph for a project in a supported format.
pub async fn export_graph(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = parse_project_id(&arguments)?;
    let format_value = get_required_param(&arguments, "format")?;
    let format_str = format_value.as_str().ok_or_else(|| McpError::Validation {
        message: "Format must be a string".to_string(),
    })?;

    let export_format =
        parse_export_format(format_str).map_err(|message| McpError::Validation { message })?;

    let graph = find_graph(project_id, db)
        .await
        .map_err(|e| internal_error("Failed to locate graph", e))?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "export_graph".to_string(),
            message: format!("No graph found for project {}", project_id),
        })?;

    let graph_service = GraphService::new(db.clone());
    let export_service = ExportService::new(db.clone());

    let graph_data = graph_service
        .build_graph_from_dag_graph(graph.id)
        .await
        .map_err(|e| internal_error("Failed to load graph data", e))?;

    let format_lower = format_str.to_lowercase();
    let content = if format_lower == "json" {
        let summary = json!({
            "nodes": graph_data.nodes,
            "edges": graph_data.edges,
            "graph_layers": graph_data.layers,
        });
        serde_json::to_string_pretty(&summary)
            .map_err(|e| internal_error("Failed to serialize graph data", e))?
    } else {
        export_service
            .export_to_string(&graph_data, &export_format)
            .map_err(|e| internal_error("Failed to render graph content", e))?
    };

    let result = json!({
        "project_id": project_id,
        "graph_id": graph.id,
        "format": format_lower,
        "content": content
    });

    create_success_response(&result)
}

/// Retrieve graph metadata and sample records.
pub async fn get_graph_data(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let project_id = parse_project_id(&arguments)?;
    let include_nodes = get_optional_param(&arguments, "include_nodes")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let include_edges = get_optional_param(&arguments, "include_edges")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let include_layers = get_optional_param(&arguments, "include_layers")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    let graph = find_graph(project_id, db)
        .await
        .map_err(|e| internal_error("Failed to locate graph", e))?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_graph_data".to_string(),
            message: format!("No graph found for project {}", project_id),
        })?;

    let node_count = GraphNodes::find()
        .filter(GraphNodeColumn::GraphId.eq(graph.id))
        .count(db)
        .await
        .map_err(|e| internal_error("Failed to count nodes", e))?;

    let edge_count = GraphEdges::find()
        .filter(GraphEdgeColumn::GraphId.eq(graph.id))
        .count(db)
        .await
        .map_err(|e| internal_error("Failed to count edges", e))?;

    let layer_count = Layers::find()
        .filter(LayerColumn::GraphId.eq(graph.id))
        .count(db)
        .await
        .map_err(|e| internal_error("Failed to count graph_layers", e))?;

    let mut result = json!({
        "project_id": project_id,
        "graph_id": graph.id,
        "nodes": { "count": node_count },
        "edges": { "count": edge_count },
        "graph_layers": { "count": layer_count }
    });

    if include_nodes {
        let samples = GraphNodes::find()
            .filter(GraphNodeColumn::GraphId.eq(graph.id))
            .limit(SAMPLE_LIMIT as u64)
            .all(db)
            .await
            .map_err(|e| internal_error("Failed to load node samples", e))?;

        let values: Vec<Value> = samples
            .into_iter()
            .map(|node| {
                json!({
                    "id": node.id,
                    "label": node.label,
                    "layer": node.layer,
                    "belongs_to": node.belongs_to,
                    "is_partition": node.is_partition,
                    "attrs": node.attrs
                })
            })
            .collect();

        result["nodes"]["samples"] = Value::Array(values);
    }

    if include_edges {
        let samples = GraphEdges::find()
            .filter(GraphEdgeColumn::GraphId.eq(graph.id))
            .limit(SAMPLE_LIMIT as u64)
            .all(db)
            .await
            .map_err(|e| internal_error("Failed to load edge samples", e))?;

        let values: Vec<Value> = samples
            .into_iter()
            .map(|edge| {
                json!({
                    "id": edge.id,
                    "source": edge.source,
                    "target": edge.target,
                    "label": edge.label,
                    "layer": edge.layer,
                    "attrs": edge.attrs
                })
            })
            .collect();

        result["edges"]["samples"] = Value::Array(values);
    }

    if include_layers {
        let samples = Layers::find()
            .filter(LayerColumn::GraphId.eq(graph.id))
            .limit(SAMPLE_LIMIT as u64)
            .all(db)
            .await
            .map_err(|e| internal_error("Failed to load layer samples", e))?;

        let values: Vec<Value> = samples
            .into_iter()
            .map(|layer| {
                json!({
                    "id": layer.id,
                    "layer_id": layer.layer_id,
                    "name": layer.name,
                    "background_color": layer.background_color,
                    "text_color": layer.text_color,
                    "border_color": layer.border_color,
                    "comment": layer.comment,
                    "properties": layer.properties
                })
            })
            .collect();

        result["graph_layers"]["samples"] = Value::Array(values);
    }

    create_success_response(&result)
}

const SAMPLE_LIMIT: usize = 10;

fn parse_project_id(arguments: &Option<Value>) -> McpResult<i32> {
    let raw = get_required_param(arguments, "project_id")?;
    let value = raw.as_i64().ok_or_else(|| McpError::Validation {
        message: "project_id must be a number".to_string(),
    })?;

    i32::try_from(value).map_err(|_| McpError::Validation {
        message: "project_id is out of range".to_string(),
    })
}

async fn ensure_graph(project_id: i32, db: &DatabaseConnection) -> Result<graphs::Model> {
    if let Some(graph) = find_graph(project_id, db).await? {
        return Ok(graph);
    }

    let plan_service = PlanDagService::new(db.clone());
    let plan = plan_service
        .get_or_create_plan(project_id)
        .await
        .context("failed to create plan for project")?;

    let node_id = format!("imported_graph_node_{}", Uuid::new_v4().simple());
    let now = Utc::now();
    let metadata = serde_json::to_string(&json!({
        "label": format!("Imported Graph {}", project_id),
        "description": "Graph data imported via MCP CSV"
    }))?;

    let node = plan_dag_nodes::ActiveModel {
        id: Set(node_id.clone()),
        plan_id: Set(plan.id),
        node_type: Set("GraphNode".to_string()),
        position_x: Set(0.0),
        position_y: Set(0.0),
        source_position: Set(None),
        target_position: Set(None),
        metadata_json: Set(metadata),
        config_json: Set("{}".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };

    node.insert(db)
        .await
        .context("failed to create plan DAG node for graph")?;

    let mut graph_active = graphs::ActiveModel::new();
    graph_active.project_id = Set(project_id);
    graph_active.node_id = Set(node_id);
    graph_active.name = Set(format!("Imported Graph {}", project_id));
    graph_active.execution_state = Set(ExecutionState::Completed.as_str().to_string());
    graph_active.computed_date = Set(None);
    graph_active.source_hash = Set(None);
    graph_active.node_count = Set(0);
    graph_active.edge_count = Set(0);
    graph_active.error_message = Set(None);
    graph_active.metadata = Set(None);
    graph_active.last_edit_sequence = Set(0);
    graph_active.has_pending_edits = Set(false);
    graph_active.last_replay_at = Set(None);
    graph_active.created_at = Set(now);
    graph_active.updated_at = Set(now);

    let graph = graph_active
        .insert(db)
        .await
        .context("failed to insert graph record")?;

    Ok(graph)
}

async fn find_graph(project_id: i32, db: &DatabaseConnection) -> Result<Option<graphs::Model>> {
    let graph = Graphs::find()
        .filter(GraphColumn::ProjectId.eq(project_id))
        .order_by_asc(GraphColumn::Id)
        .one(db)
        .await
        .context("failed to fetch graph")?;

    Ok(graph)
}

async fn import_nodes_csv(db: &DatabaseConnection, graph_id: i32, csv_data: &str) -> Result<usize> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    let headers = reader
        .headers()
        .context("nodes CSV missing header row")?
        .clone();

    let id_idx = find_column(&headers, &["node_id", "id"]).unwrap_or(0);
    let label_idx = find_column(&headers, &["label", "name"]);
    let layer_idx = find_column(&headers, &["layer", "layer_id"]);
    let belongs_to_idx = find_column(&headers, &["belongs_to"]);
    let weight_idx = find_column(&headers, &["weight"]);
    let partition_idx = find_column(&headers, &["is_partition", "partition"]);

    let mut imported = 0usize;
    for record in reader.records() {
        let record = record.context("failed to read nodes CSV record")?;
        let node_id = record
            .get(id_idx)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow!("nodes CSV row missing node id"))?
            .to_string();

        let label = label_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let layer = layer_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let belongs_to = belongs_to_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let weight = weight_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .and_then(|v| v.parse::<f64>().ok());

        let is_partition = partition_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .map(|v| matches_ignore_case(v, "true") || v == "1")
            .unwrap_or(false);

        let mut reserved = HashSet::new();
        reserved.insert(id_idx);
        if let Some(idx) = label_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = layer_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = belongs_to_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = weight_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = partition_idx {
            reserved.insert(idx);
        }

        let attrs = collect_extra_attrs(&headers, &record, &reserved);

        upsert_node(
            db,
            graph_id,
            node_id,
            label,
            layer,
            belongs_to,
            weight,
            is_partition,
            attrs,
        )
        .await?;

        imported += 1;
    }

    Ok(imported)
}

async fn upsert_node(
    db: &DatabaseConnection,
    graph_id: i32,
    node_id: String,
    label: Option<String>,
    layer: Option<String>,
    belongs_to: Option<String>,
    weight: Option<f64>,
    is_partition: bool,
    attrs: Option<Value>,
) -> Result<()> {
    let existing = GraphNodes::find()
        .filter(GraphNodeColumn::GraphId.eq(graph_id))
        .filter(GraphNodeColumn::Id.eq(node_id.clone()))
        .one(db)
        .await?;

    if let Some(model) = existing {
        let mut active: graph_nodes::ActiveModel = model.into();
        active.label = Set(label);
        active.layer = Set(layer);
        active.belongs_to = Set(belongs_to);
        active.weight = Set(weight);
        active.is_partition = Set(is_partition);
        active.attrs = Set(attrs);
        active.update(db).await?;
    } else {
        let node = graph_nodes::ActiveModel {
            id: Set(node_id),
            graph_id: Set(graph_id),
            label: Set(label),
            layer: Set(layer),
            weight: Set(weight),
            is_partition: Set(is_partition),
            belongs_to: Set(belongs_to),
            attrs: Set(attrs),
            datasource_id: Set(None),
            comment: Set(None),
            created_at: Set(Utc::now()),
        };
        node.insert(db).await?;
    }

    Ok(())
}

async fn import_edges_csv(db: &DatabaseConnection, graph_id: i32, csv_data: &str) -> Result<usize> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(csv_data.as_bytes());

    let headers = reader
        .headers()
        .context("edges CSV missing header row")?
        .clone();

    let source_idx = find_column(&headers, &["source", "source_node_id"])
        .ok_or_else(|| anyhow!("edges CSV missing source column"))?;
    let target_idx = find_column(&headers, &["target", "target_node_id"])
        .ok_or_else(|| anyhow!("edges CSV missing target column"))?;
    let id_idx = find_column(&headers, &["edge_id", "id"]);
    let label_idx = find_column(&headers, &["label", "name"]);
    let layer_idx = find_column(&headers, &["layer", "layer_id"]);
    let weight_idx = find_column(&headers, &["weight"]);

    let mut imported = 0usize;
    for record in reader.records() {
        let record = record.context("failed to read edges CSV record")?;

        let source = record
            .get(source_idx)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow!("edge row missing source"))?
            .to_string();

        let target = record
            .get(target_idx)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow!("edge row missing target"))?
            .to_string();

        let edge_id = id_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string())
            .unwrap_or_else(|| format!("{}->{}", source, target));

        let label = label_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let layer = layer_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| v.to_string());

        let weight = weight_idx
            .and_then(|idx| record.get(idx))
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .and_then(|v| v.parse::<f64>().ok());

        let mut reserved = HashSet::new();
        reserved.insert(source_idx);
        reserved.insert(target_idx);
        if let Some(idx) = id_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = label_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = layer_idx {
            reserved.insert(idx);
        }
        if let Some(idx) = weight_idx {
            reserved.insert(idx);
        }

        let attrs = collect_extra_attrs(&headers, &record, &reserved);

        upsert_edge(
            db, graph_id, edge_id, source, target, label, layer, weight, attrs,
        )
        .await?;

        imported += 1;
    }

    Ok(imported)
}

async fn upsert_edge(
    db: &DatabaseConnection,
    graph_id: i32,
    edge_id: String,
    source: String,
    target: String,
    label: Option<String>,
    layer: Option<String>,
    weight: Option<f64>,
    attrs: Option<Value>,
) -> Result<()> {
    let existing = GraphEdges::find()
        .filter(GraphEdgeColumn::GraphId.eq(graph_id))
        .filter(GraphEdgeColumn::Id.eq(edge_id.clone()))
        .one(db)
        .await?;

    if let Some(model) = existing {
        let mut active: graph_edges::ActiveModel = model.into();
        active.source = Set(source);
        active.target = Set(target);
        active.label = Set(label);
        active.layer = Set(layer);
        active.weight = Set(weight);
        active.attrs = Set(attrs);
        active.update(db).await?;
    } else {
        let edge = graph_edges::ActiveModel {
            id: Set(edge_id),
            graph_id: Set(graph_id),
            source: Set(source),
            target: Set(target),
            label: Set(label),
            layer: Set(layer),
            weight: Set(weight),
            attrs: Set(attrs),
            datasource_id: Set(None),
            comment: Set(None),
            created_at: Set(Utc::now()),
        };
        edge.insert(db).await?;
    }

    Ok(())
}

fn collect_extra_attrs(
    headers: &csv::StringRecord,
    record: &csv::StringRecord,
    reserved_indices: &HashSet<usize>,
) -> Option<Value> {
    let mut map = Map::new();
    for (idx, header) in headers.iter().enumerate() {
        if reserved_indices.contains(&idx) {
            continue;
        }
        let value = record
            .get(idx)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        if let Some(value) = value {
            map.insert(header.to_string(), Value::String(value));
        }
    }

    if map.is_empty() {
        None
    } else {
        Some(Value::Object(map))
    }
}

async fn update_graph_counts(db: &DatabaseConnection, graph_id: i32) -> Result<()> {
    let nodes = GraphNodes::find()
        .filter(GraphNodeColumn::GraphId.eq(graph_id))
        .count(db)
        .await?;

    let edges = GraphEdges::find()
        .filter(GraphEdgeColumn::GraphId.eq(graph_id))
        .count(db)
        .await?;

    let graph = Graphs::find_by_id(graph_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("graph {} not found", graph_id))?;

    let mut active: graphs::ActiveModel = graph.into();
    active.node_count = Set(clamp_to_i32(nodes));
    active.edge_count = Set(clamp_to_i32(edges));
    active.execution_state = Set(ExecutionState::Completed.as_str().to_string());
    active.updated_at = Set(Utc::now());
    active.update(db).await?;

    Ok(())
}

fn clamp_to_i32(value: u64) -> i32 {
    value.min(i32::MAX as u64).try_into().unwrap_or(i32::MAX)
}

fn find_column(headers: &csv::StringRecord, names: &[&str]) -> Option<usize> {
    headers.iter().enumerate().find_map(|(idx, header)| {
        if names.iter().any(|name| matches_ignore_case(header, name)) {
            Some(idx)
        } else {
            None
        }
    })
}

fn matches_ignore_case(value: &str, expected: &str) -> bool {
    value.trim().eq_ignore_ascii_case(expected)
}

fn parse_export_format(format: &str) -> Result<ExportFileType, String> {
    let normalized = format.trim().to_ascii_uppercase().replace('-', "_");
    match normalized.as_str() {
        "JSON" => Ok(ExportFileType::JSON),
        "CSV" | "CSVNODES" | "CSV_NODES" => Ok(ExportFileType::CSVNodes),
        "CSV_EDGES" | "CSVEDGES" => Ok(ExportFileType::CSVEdges),
        "DOT" => Ok(ExportFileType::DOT),
        "GML" => Ok(ExportFileType::GML),
        "PLANTUML" | "PLANT_UML" => Ok(ExportFileType::PlantUML),
        "MERMAID" => Ok(ExportFileType::Mermaid),
        other => Err(format!("Unsupported export format: {}", other)),
    }
}

fn internal_error(context: &str, error: impl std::fmt::Display) -> McpError {
    McpError::Internal {
        message: format!("{context}: {error}"),
    }
}
