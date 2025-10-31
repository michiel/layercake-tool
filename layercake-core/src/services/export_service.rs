use anyhow::Result;
use sea_orm::DatabaseConnection;

use crate::export::{to_csv_edges, to_csv_nodes, to_dot, to_gml, to_json, to_mermaid, to_plantuml};
use crate::graph::Graph;
use crate::plan::{ExportFileType, Plan, RenderConfig, RenderConfigOrientation};
pub struct ExportService {
    db: DatabaseConnection,
}

impl ExportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub fn export_to_string(&self, graph: &Graph, format: &ExportFileType) -> Result<String> {
        // Default render config
        let render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
        };

        match format {
            ExportFileType::DOT => Ok(to_dot::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::GML => Ok(to_gml::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::JSON => Ok(to_json::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::Mermaid => Ok(to_mermaid::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::PlantUML => Ok(to_plantuml::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::CSVNodes => Ok(to_csv_nodes::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            ExportFileType::CSVEdges => Ok(to_csv_edges::render(graph.clone(), render_config)
                .map_err(|e| anyhow::anyhow!("{}", e))?),
            _ => Err(anyhow::anyhow!(
                "Export format not implemented for string output"
            )),
        }
    }

    #[allow(dead_code)] // Reserved for future plan export execution
    pub async fn execute_plan_exports(
        &self,
        _project_id: i32,
        plan_yaml: &str,
    ) -> Result<Vec<String>> {
        let plan: Plan = serde_yaml::from_str(plan_yaml)?;
        // let mut graph = graph_service.build_graph_from_project(project_id).await?;
        let mut graph = Graph::default(); // Placeholder

        let mut outputs = Vec::new();

        for export_item in &plan.export.profiles {
            let graph_config = export_item.get_graph_config();

            // Apply transformations if specified
            if graph_config.invert_graph {
                graph = graph
                    .invert_graph()
                    .map_err(|e| anyhow::anyhow!("Failed to invert graph: {}", e))?;
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
}
