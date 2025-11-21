//! Data source management tools for MCP backed by the shared AppContext.

use crate::app_context::{
    AppContext, DataSetEmptyCreateRequest, DataSetExportFormat, DataSetExportRequest,
    DataSetFileCreateRequest, DataSetFileReplacement, DataSetImportFormat, DataSetImportRequest,
    DataSetUpdateRequest,
};
use crate::database::entities::common_types::{DataType as DataSetDataType, FileFormat};
use crate::mcp::tools::{create_success_response, get_required_param};
use axum_mcp::prelude::*;
use serde_json::{json, Value};
use std::collections::HashMap;

pub fn get_data_set_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_data_sets".to_string(),
            description: "List all data sources for a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "integer" }
                },
                "required": ["project_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "get_data_set".to_string(),
            description: "Fetch a single data source by ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "data_set_id": { "type": "integer" }
                },
                "required": ["data_set_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "create_data_set_from_file".to_string(),
            description: "Create a data source from base64-encoded file content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "integer" },
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "filename": { "type": "string" },
                    "file_content": { "type": "string", "description": "Base64 file content" },
                    "file_format": { "type": "string", "enum": ["csv", "tsv", "json"] },
                    "tabular_data_type": {
                        "type": "string",
                        "enum": ["nodes", "edges", "layers"],
                        "description": "Optional hint for CSV/TSV uploads"
                    },
                    "data_type": {
                        "type": "string",
                        "enum": ["nodes", "edges", "layers", "graph"],
                        "description": "Deprecated alias for tabular_data_type; kept for backwards compatibility"
                    }
                },
                "required": ["project_id", "name", "filename", "file_content", "file_format"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "create_empty_data_set".to_string(),
            description: "Create an empty data source without file content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "integer" },
                    "name": { "type": "string" },
                    "description": { "type": "string" }
                },
                "required": ["project_id", "name"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "update_data_set".to_string(),
            description: "Update data source metadata and optionally replace its file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "data_set_id": { "type": "integer" },
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "file": {
                        "type": "object",
                        "properties": {
                            "filename": { "type": "string" },
                            "file_content": { "type": "string", "description": "Base64 file content" }
                        },
                        "required": ["filename", "file_content"],
                        "additionalProperties": false
                    }
                },
                "required": ["data_set_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "delete_data_set".to_string(),
            description: "Delete a data source and related DAG nodes".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "data_set_id": { "type": "integer" }
                },
                "required": ["data_set_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "reprocess_data_set".to_string(),
            description: "Reprocess an existing data source file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "data_set_id": { "type": "integer" }
                },
                "required": ["data_set_id"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "export_data_sets".to_string(),
            description: "Export data sources to XLSX or ODS".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "integer" },
                    "data_set_ids": {
                        "type": "array",
                        "items": { "type": "integer" }
                    },
                    "format": { "type": "string", "enum": ["xlsx", "ods"] }
                },
                "required": ["project_id", "data_set_ids", "format"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
        Tool {
            name: "import_data_sets".to_string(),
            description: "Import data sources from XLSX or ODS content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project_id": { "type": "integer" },
                    "filename": { "type": "string" },
                    "file_content": { "type": "string", "description": "Base64 spreadsheet content" }
                },
                "required": ["project_id", "filename", "file_content"],
                "additionalProperties": false
            }),
            metadata: HashMap::new(),
        },
    ]
}

fn parse_file_format(value: &str) -> McpResult<FileFormat> {
    match value.to_lowercase().as_str() {
        "csv" => Ok(FileFormat::Csv),
        "tsv" => Ok(FileFormat::Tsv),
        "json" => Ok(FileFormat::Json),
        other => Err(McpError::Validation {
            message: format!(
                "Unsupported file_format '{}'. Use csv, tsv, or json.",
                other
            ),
        }),
    }
}

fn parse_data_type(value: &str) -> McpResult<DataSetDataType> {
    match value.to_lowercase().as_str() {
        "nodes" => Ok(DataSetDataType::Nodes),
        "edges" => Ok(DataSetDataType::Edges),
        "layers" => Ok(DataSetDataType::Layers),
        "graph" => Ok(DataSetDataType::Graph),
        other => Err(McpError::Validation {
            message: format!(
                "Unsupported data_type '{}'. Use nodes, edges, layers, or graph.",
                other
            ),
        }),
    }
}

pub async fn list_data_sets(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let project_id = get_required_param(&arguments, "project_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })? as i32;

    let data_sets = app
        .list_data_sets(project_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to list data sources: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "count": data_sets.len(),
        "dataSets": data_sets
    }))
}

pub async fn get_data_set(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let data_set_id = get_required_param(&arguments, "data_set_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "data_set_id must be an integer".to_string(),
        })? as i32;

    let data_set = app
        .get_data_set(data_set_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to load data source: {}", e),
        })?
        .ok_or_else(|| McpError::ToolExecution {
            tool: "get_data_set".to_string(),
            message: format!("Data source {} not found", data_set_id),
        })?;

    create_success_response(&json!({ "dataSet": data_set }))
}

pub async fn create_data_set_from_file(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let args = arguments.ok_or_else(|| McpError::Validation {
        message: "Arguments are required".to_string(),
    })?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })? as i32;
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "name must be provided".to_string(),
        })?
        .to_string();
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "filename must be provided".to_string(),
        })?
        .to_string();
    let file_content = args
        .get("file_content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "file_content must be provided".to_string(),
        })?;
    let file_format = parse_file_format(
        args.get("file_format")
            .and_then(|v| v.as_str())
            .unwrap_or(""),
    )?;
    let tabular_data_type = args
        .get("tabular_data_type")
        .or_else(|| args.get("data_type"))
        .and_then(|v| v.as_str())
        .map(parse_data_type)
        .transpose()?;

    use base64::Engine;
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(file_content)
        .map_err(|e| McpError::Validation {
            message: format!("file_content must be valid base64: {}", e),
        })?;

    let summary = app
        .create_data_set_from_file(DataSetFileCreateRequest {
            project_id,
            name,
            description,
            filename,
            file_format,
            tabular_data_type,
            file_bytes,
        })
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create data source: {}", e),
        })?;

    create_success_response(&json!({
        "dataSet": summary,
        "message": "Data source created successfully"
    }))
}

pub async fn create_empty_data_set(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let args = arguments.ok_or_else(|| McpError::Validation {
        message: "Arguments are required".to_string(),
    })?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })? as i32;
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "name must be provided".to_string(),
        })?
        .to_string();
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let summary = app
        .create_empty_data_set(DataSetEmptyCreateRequest {
            project_id,
            name,
            description,
        })
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to create empty data source: {}", e),
        })?;

    create_success_response(&json!({
        "dataSet": summary,
        "message": "Empty data source created successfully"
    }))
}

pub async fn update_data_set(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let args = arguments.ok_or_else(|| McpError::Validation {
        message: "Arguments are required".to_string(),
    })?;

    let data_set_id = args
        .get("data_set_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| McpError::Validation {
            message: "data_set_id must be an integer".to_string(),
        })? as i32;

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    let new_file = if let Some(file) = args.get("file") {
        if !file.is_object() {
            return Err(McpError::Validation {
                message: "file must be an object".to_string(),
            });
        }
        let filename = file
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation {
                message: "file.filename must be provided".to_string(),
            })?;
        let content = file
            .get("file_content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation {
                message: "file.file_content must be provided".to_string(),
            })?;

        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(content)
            .map_err(|e| McpError::Validation {
                message: format!("file.file_content must be valid base64: {}", e),
            })?;

        Some(DataSetFileReplacement {
            filename: filename.to_string(),
            file_bytes: bytes,
        })
    } else {
        None
    };

    let summary = app
        .update_data_set(DataSetUpdateRequest {
            id: data_set_id,
            name,
            description,
            new_file,
        })
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update data source: {}", e),
        })?;

    create_success_response(&json!({
        "dataSet": summary,
        "message": "Data source updated successfully"
    }))
}

pub async fn delete_data_set(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let data_set_id = get_required_param(&arguments, "data_set_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "data_set_id must be an integer".to_string(),
        })? as i32;

    app.delete_data_set(data_set_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to delete data source: {}", e),
        })?;

    create_success_response(&json!({
        "dataSetId": data_set_id,
        "message": "Data source deleted successfully"
    }))
}

pub async fn reprocess_data_set(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let data_set_id = get_required_param(&arguments, "data_set_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "data_set_id must be an integer".to_string(),
        })? as i32;

    let summary = app
        .reprocess_data_set(data_set_id)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to reprocess data source: {}", e),
        })?;

    create_success_response(&json!({
        "dataSet": summary,
        "message": "Data source reprocessed successfully"
    }))
}

pub async fn export_data_sets(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let args = arguments.ok_or_else(|| McpError::Validation {
        message: "Arguments are required".to_string(),
    })?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })? as i32;
    let ids = args
        .get("data_set_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| McpError::Validation {
            message: "data_set_ids must be an array of integers".to_string(),
        })?
        .iter()
        .map(|value| {
            value.as_i64().ok_or_else(|| McpError::Validation {
                message: "data_set_ids must contain integers".to_string(),
            })
        })
        .collect::<Result<Vec<_>, McpError>>()?
        .into_iter()
        .map(|v| v as i32)
        .collect::<Vec<_>>();

    let format =
        args.get("format")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::Validation {
                message: "format must be provided".to_string(),
            })?;

    let format = match format.to_lowercase().as_str() {
        "xlsx" => DataSetExportFormat::Xlsx,
        "ods" => DataSetExportFormat::Ods,
        other => {
            return Err(McpError::Validation {
                message: format!("Unsupported export format '{}'", other),
            })
        }
    };

    let exported = app
        .export_data_sets(DataSetExportRequest {
            project_id,
            data_set_ids: ids,
            format,
        })
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to export data sources: {}", e),
        })?;

    use base64::{engine::general_purpose, Engine as _};
    let encoded = general_purpose::STANDARD.encode(&exported.data);

    create_success_response(&json!({
        "projectId": project_id,
        "filename": exported.filename,
        "format": exported.format.extension(),
        "fileContent": encoded
    }))
}

pub async fn import_data_sets(
    arguments: Option<Value>,
    app: &AppContext,
) -> McpResult<ToolsCallResult> {
    let args = arguments.ok_or_else(|| McpError::Validation {
        message: "Arguments are required".to_string(),
    })?;

    let project_id = args
        .get("project_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| McpError::Validation {
            message: "project_id must be an integer".to_string(),
        })? as i32;
    let filename = args
        .get("filename")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "filename must be provided".to_string(),
        })?;
    let file_content = args
        .get("file_content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::Validation {
            message: "file_content must be provided".to_string(),
        })?;

    use base64::Engine;
    let file_bytes = base64::engine::general_purpose::STANDARD
        .decode(file_content)
        .map_err(|e| McpError::Validation {
            message: format!("file_content must be valid base64: {}", e),
        })?;

    let format =
        DataSetImportFormat::from_filename(filename).ok_or_else(|| McpError::Validation {
            message: "Only .xlsx and .ods filenames are supported for import".to_string(),
        })?;

    let outcome = app
        .import_data_sets(DataSetImportRequest {
            project_id,
            format,
            file_bytes,
        })
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to import data sources: {}", e),
        })?;

    create_success_response(&json!({
        "projectId": project_id,
        "dataSets": outcome.data_sets,
        "createdCount": outcome.created_count,
        "updatedCount": outcome.updated_count
    }))
}
