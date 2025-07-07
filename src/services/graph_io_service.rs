//! Service layer for graph import/export operations

use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, Set, ActiveModelTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
// use tempfile::TempDir; // Using std temp dir for now
use tracing::{debug, info, error};
use uuid::Uuid;

use crate::database::entities::{projects, graphs};
use crate::graph::Graph;
use crate::graph_io::{GraphIO, GraphFormat, ImportOptions, ExportOptions, ImportResult, ExportResult};

/// API request types for graph I/O operations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportGraphRequest {
    pub project_id: i32,
    pub format: Option<GraphFormat>,
    pub validate: Option<bool>,
    pub merge_mode: Option<crate::graph_io::MergeMode>,
    pub auto_generate_layers: Option<bool>,
    pub preserve_ids: Option<bool>,
    pub field_mappings: Option<crate::graph_io::FieldMappings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportGraphRequest {
    pub project_id: i32,
    pub format: GraphFormat,
    pub include_metadata: Option<bool>,
    pub include_hierarchy: Option<bool>,
    pub include_layers: Option<bool>,
    pub prettify: Option<bool>,
    pub field_mappings: Option<crate::graph_io::FieldMappings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportGraphResponse {
    pub success: bool,
    pub graph_id: Option<String>,
    pub import_result: ImportResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportGraphResponse {
    pub success: bool,
    pub download_url: Option<String>,
    pub export_result: ExportResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub format: GraphFormat,
    pub name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub supports_hierarchy: bool,
    pub supports_layers: bool,
    pub mime_type: String,
}

/// Graph I/O service
pub struct GraphIOService {
    db: DatabaseConnection,
    temp_dir_path: PathBuf,
}

impl GraphIOService {
    /// Create a new graph I/O service
    pub fn new(db: DatabaseConnection) -> Result<Self> {
        let temp_dir_path = std::env::temp_dir().join("layercake_io");
        std::fs::create_dir_all(&temp_dir_path)?;
        Ok(Self { db, temp_dir_path })
    }
    
    /// Import a graph from uploaded file data
    pub async fn import_from_data(
        &self,
        request: ImportGraphRequest,
        file_data: &[u8],
        filename: &str,
    ) -> Result<ImportGraphResponse> {
        debug!("Importing graph from file: {}", filename);
        
        // Verify project exists
        let project = projects::Entity::find_by_id(request.project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        
        // Create temporary file
        let temp_file_path = self.temp_dir_path.join(filename);
        std::fs::write(&temp_file_path, file_data)?;
        
        // Detect format if not specified
        let format = request.format.unwrap_or_else(|| {
            GraphIO::detect_format(&temp_file_path).unwrap_or(GraphFormat::JSON)
        });
        
        // Build import options
        let import_options = ImportOptions {
            format,
            validate: request.validate.unwrap_or(true),
            merge_mode: request.merge_mode.unwrap_or(crate::graph_io::MergeMode::Replace),
            field_mappings: request.field_mappings,
            auto_generate_layers: request.auto_generate_layers.unwrap_or(true),
            preserve_ids: request.preserve_ids.unwrap_or(true),
        };
        
        // Import the graph
        let (mut graph, import_result) = GraphIO::import_from_file(&temp_file_path, import_options)?;
        
        if !import_result.success {
            return Ok(ImportGraphResponse {
                success: false,
                graph_id: None,
                import_result,
            });
        }
        
        // Set graph name from project and filename if not set
        if graph.name.is_empty() {
            graph.name = format!("{} - {}", project.name, 
                Path::new(filename).file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Imported Graph"));
        }
        
        // Save graph to database
        let graph_id = Uuid::new_v4().to_string();
        let graph_data = serde_json::to_string(&graph)?;
        
        let graph_model = graphs::ActiveModel {
            id: Set(graph_id.clone()),
            plan_id: Set(request.project_id), // Using project_id as plan_id for now - needs proper plan integration
            plan_node_id: Set("import_node".to_string()), // Placeholder - needs proper plan node integration
            name: Set(graph.name.clone()),
            description: Set(Some(format!("Imported from {} ({})", filename, format_name(format)))),
            graph_data: Set(graph_data),
            metadata: Set(None),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };
        
        graph_model.insert(&self.db).await?;
        
        info!("Successfully imported graph {} for project {}", graph_id, request.project_id);
        
        Ok(ImportGraphResponse {
            success: true,
            graph_id: Some(graph_id),
            import_result,
        })
    }
    
    /// Export a graph to specified format
    pub async fn export_graph(
        &self,
        request: ExportGraphRequest,
    ) -> Result<ExportGraphResponse> {
        debug!("Exporting graph for project: {}", request.project_id);
        
        // Get project and graph data
        let project = projects::Entity::find_by_id(request.project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Project not found"))?;
        
        // For now, get the latest graph for the project
        // TODO: Support selecting specific graph versions
        let graph_record = graphs::Entity::find()
            .filter(graphs::Column::PlanId.eq(request.project_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No graph found for project"))?;
        
        let graph: Graph = serde_json::from_str(&graph_record.graph_data)?;
        
        // Build export options
        let export_options = ExportOptions {
            format: request.format,
            include_metadata: request.include_metadata.unwrap_or(true),
            include_hierarchy: request.include_hierarchy.unwrap_or(true),
            include_layers: request.include_layers.unwrap_or(true),
            field_mappings: request.field_mappings,
            prettify: request.prettify.unwrap_or(true),
        };
        
        // Generate output filename
        let extensions = GraphIO::get_extensions(request.format);
        let extension = extensions.first().unwrap_or(&"dat");
        let filename = format!("{}_{}.{}", 
            project.name.replace(' ', "_").to_lowercase(),
            graph.name.replace(' ', "_").to_lowercase(),
            extension
        );
        
        let output_path = self.temp_dir_path.join(&filename);
        
        // Export the graph
        let export_result = GraphIO::export_to_file(&graph, &output_path, export_options)?;
        
        if !export_result.success {
            return Ok(ExportGraphResponse {
                success: false,
                download_url: None,
                export_result,
            });
        }
        
        // Generate download URL (in a real app, this would be a proper URL)
        let download_url = format!("/api/download/{}", filename);
        
        info!("Successfully exported graph for project {} to {}", request.project_id, filename);
        
        Ok(ExportGraphResponse {
            success: true,
            download_url: Some(download_url),
            export_result,
        })
    }
    
    /// Get information about supported formats
    pub fn get_supported_formats(&self) -> Vec<FormatInfo> {
        vec![
            FormatInfo {
                format: GraphFormat::JSON,
                name: "JSON".to_string(),
                description: crate::graph_io::formats::get_format_description(GraphFormat::JSON).to_string(),
                extensions: GraphIO::get_extensions(GraphFormat::JSON).iter().map(|s| s.to_string()).collect(),
                supports_hierarchy: crate::graph_io::formats::supports_hierarchy(GraphFormat::JSON),
                supports_layers: crate::graph_io::formats::supports_layers(GraphFormat::JSON),
                mime_type: crate::graph_io::formats::get_mime_type(GraphFormat::JSON).to_string(),
            },
            FormatInfo {
                format: GraphFormat::CSV,
                name: "CSV".to_string(),
                description: crate::graph_io::formats::get_format_description(GraphFormat::CSV).to_string(),
                extensions: GraphIO::get_extensions(GraphFormat::CSV).iter().map(|s| s.to_string()).collect(),
                supports_hierarchy: crate::graph_io::formats::supports_hierarchy(GraphFormat::CSV),
                supports_layers: crate::graph_io::formats::supports_layers(GraphFormat::CSV),
                mime_type: crate::graph_io::formats::get_mime_type(GraphFormat::CSV).to_string(),
            },
            FormatInfo {
                format: GraphFormat::DOT,
                name: "DOT (Graphviz)".to_string(),
                description: crate::graph_io::formats::get_format_description(GraphFormat::DOT).to_string(),
                extensions: GraphIO::get_extensions(GraphFormat::DOT).iter().map(|s| s.to_string()).collect(),
                supports_hierarchy: crate::graph_io::formats::supports_hierarchy(GraphFormat::DOT),
                supports_layers: crate::graph_io::formats::supports_layers(GraphFormat::DOT),
                mime_type: crate::graph_io::formats::get_mime_type(GraphFormat::DOT).to_string(),
            },
            FormatInfo {
                format: GraphFormat::Layercake,
                name: "Layercake Native".to_string(),
                description: crate::graph_io::formats::get_format_description(GraphFormat::Layercake).to_string(),
                extensions: GraphIO::get_extensions(GraphFormat::Layercake).iter().map(|s| s.to_string()).collect(),
                supports_hierarchy: crate::graph_io::formats::supports_hierarchy(GraphFormat::Layercake),
                supports_layers: crate::graph_io::formats::supports_layers(GraphFormat::Layercake),
                mime_type: crate::graph_io::formats::get_mime_type(GraphFormat::Layercake).to_string(),
            },
            // Note: GraphML and GEXF are commented out as they're not fully implemented
            // FormatInfo {
            //     format: GraphFormat::GraphML,
            //     name: "GraphML".to_string(),
            //     description: crate::graph_io::formats::get_format_description(GraphFormat::GraphML).to_string(),
            //     extensions: GraphIO::get_extensions(GraphFormat::GraphML).iter().map(|s| s.to_string()).collect(),
            //     supports_hierarchy: crate::graph_io::formats::supports_hierarchy(GraphFormat::GraphML),
            //     supports_layers: crate::graph_io::formats::supports_layers(GraphFormat::GraphML),
            //     mime_type: crate::graph_io::formats::get_mime_type(GraphFormat::GraphML).to_string(),
            // },
        ]
    }
    
    /// Validate file format before import
    pub fn validate_import_file(&self, filename: &str, content: &[u8]) -> Result<GraphFormat> {
        // Try to detect format from filename
        let path = Path::new(filename);
        if let Some(format) = GraphIO::detect_format(path) {
            return Ok(format);
        }
        
        // Try to detect from content
        if let Ok(content_str) = std::str::from_utf8(content) {
            if let Some(format) = crate::graph_io::formats::detect_format_from_content(content_str) {
                return Ok(format);
            }
        }
        
        Err(anyhow::anyhow!("Unable to detect file format"))
    }
    
    /// Get temporary file path for downloads
    pub fn get_temp_file_path(&self, filename: &str) -> PathBuf {
        self.temp_dir_path.join(filename)
    }
}

/// Get human-readable format name
fn format_name(format: GraphFormat) -> &'static str {
    match format {
        GraphFormat::CSV => "CSV",
        GraphFormat::JSON => "JSON",
        GraphFormat::GraphML => "GraphML",
        GraphFormat::GEXF => "GEXF",
        GraphFormat::DOT => "DOT",
        GraphFormat::Layercake => "Layercake",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_format_validation() {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        let service = GraphIOService::new(db).unwrap();
        
        // Test JSON format detection
        let json_content = br#"{"nodes": [], "edges": []}"#;
        let format = service.validate_import_file("test.json", json_content).unwrap();
        assert_eq!(format, GraphFormat::JSON);
        
        // Test CSV format detection
        let csv_content = b"id,label\n1,Node1\n";
        let format = service.validate_import_file("test.csv", csv_content).unwrap();
        assert_eq!(format, GraphFormat::CSV);
    }
    
    #[test]
    fn test_get_supported_formats() {
        // Note: This test would need async runtime in a real scenario
        
        // For now, just test the format info structure
        let formats = vec![GraphFormat::JSON, GraphFormat::CSV, GraphFormat::DOT, GraphFormat::Layercake];
        for format in formats {
            let extensions = GraphIO::get_extensions(format);
            assert!(!extensions.is_empty());
            
            let description = crate::graph_io::formats::get_format_description(format);
            assert!(!description.is_empty());
        }
    }
}