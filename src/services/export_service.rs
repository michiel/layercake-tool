use anyhow::Result;
use sea_orm::DatabaseConnection;

use crate::export::{to_dot, to_gml, to_json, to_mermaid, to_plantuml, to_csv_nodes, to_csv_edges};
use crate::graph::Graph;
use crate::plan::legacy_plan::{Plan, ExportFileType, RenderConfig, RenderConfigOrientation};
use crate::services::GraphService;

pub struct ExportService {
    db: DatabaseConnection,
}

impl ExportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn export_graph(&self, project_id: i32, format: &str) -> Result<String> {
        let graph_service = GraphService::new(self.db.clone());
        let graph = graph_service.build_graph_from_project(project_id).await?;

        let export_format = match format.to_lowercase().as_str() {
            "dot" => ExportFileType::DOT,
            "gml" => ExportFileType::GML,
            "json" => ExportFileType::JSON,
            "mermaid" => ExportFileType::Mermaid,
            "plantuml" | "puml" => ExportFileType::PlantUML,
            "csv-nodes" => ExportFileType::CSVNodes,
            "csv-edges" => ExportFileType::CSVEdges,
            _ => return Err(anyhow::anyhow!("Unsupported export format: {}", format)),
        };

        self.export_to_string(&graph, &export_format)
    }

    fn export_to_string(&self, graph: &Graph, format: &ExportFileType) -> Result<String> {
        // Default render config
        let render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
        };

        match format {
            ExportFileType::DOT => {
                Ok(to_dot::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::GML => {
                Ok(to_gml::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::JSON => {
                Ok(to_json::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::Mermaid => {
                Ok(to_mermaid::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::PlantUML => {
                Ok(to_plantuml::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::CSVNodes => {
                Ok(to_csv_nodes::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            ExportFileType::CSVEdges => {
                Ok(to_csv_edges::render(graph.clone(), render_config).map_err(|e| anyhow::anyhow!("{}", e))?)
            }
            _ => Err(anyhow::anyhow!("Export format not implemented for string output")),
        }
    }

    pub async fn execute_plan_exports(&self, project_id: i32, plan_yaml: &str) -> Result<Vec<String>> {
        let plan: Plan = serde_yaml::from_str(plan_yaml)?;
        let graph_service = GraphService::new(self.db.clone());
        let mut graph = graph_service.build_graph_from_project(project_id).await?;

        let mut outputs = Vec::new();

        for export_item in &plan.export.profiles {
            let graph_config = export_item.get_graph_config();
            
            // Apply transformations if specified
            if graph_config.invert_graph {
                graph = graph.invert_graph();
            }

            if graph_config.max_partition_width > 0 {
                let _ = graph.modify_graph_limit_partition_width(graph_config.max_partition_width);
            }

            if graph_config.max_partition_depth > 0 {
                let _ = graph.modify_graph_limit_partition_depth(graph_config.max_partition_depth);
            }

            // Generate export
            let output = self.export_to_string(&graph, &export_item.exporter)?;
            outputs.push(output);
        }

        Ok(outputs)
    }

    /// Export graph as JSON to file
    pub async fn export_as_json(&self, graph: &Graph, output_path: &str) -> Result<()> {
        let render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
        };
        
        let content = to_json::render(graph.clone(), render_config)
            .map_err(|e| anyhow::anyhow!("JSON export failed: {}", e))?;
        
        tokio::fs::write(output_path, content).await
            .map_err(|e| anyhow::anyhow!("Failed to write JSON file: {}", e))?;
        
        Ok(())
    }

    /// Export graph as CSV to file
    pub async fn export_as_csv(&self, graph: &Graph, output_path: &str) -> Result<()> {
        let render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
        };
        
        // Export nodes as CSV (default CSV export)
        let content = to_csv_nodes::render(graph.clone(), render_config)
            .map_err(|e| anyhow::anyhow!("CSV export failed: {}", e))?;
        
        tokio::fs::write(output_path, content).await
            .map_err(|e| anyhow::anyhow!("Failed to write CSV file: {}", e))?;
        
        Ok(())
    }

    /// Export graph as DOT to file
    pub async fn export_as_dot(&self, graph: &Graph, output_path: &str) -> Result<()> {
        let render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
        };
        
        let content = to_dot::render(graph.clone(), render_config)
            .map_err(|e| anyhow::anyhow!("DOT export failed: {}", e))?;
        
        tokio::fs::write(output_path, content).await
            .map_err(|e| anyhow::anyhow!("Failed to write DOT file: {}", e))?;
        
        Ok(())
    }
}