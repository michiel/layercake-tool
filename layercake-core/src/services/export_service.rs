use sea_orm::DatabaseConnection;

use crate::errors::{CoreError, CoreResult};
use crate::export::{
    to_csv_edges, to_csv_nodes, to_dot, to_gml, to_json, to_mermaid, to_mermaid_mindmap,
    to_mermaid_treemap, to_plantuml, to_plantuml_mindmap, to_plantuml_wbs,
};
use crate::graph::Graph;
use crate::plan::{
    ExportFileType, NotePosition, Plan, RenderConfig, RenderConfigBuiltInStyle,
    RenderConfigOrientation, RenderTargetOptions,
};
pub struct ExportService {
    _db: DatabaseConnection,
}

impl ExportService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { _db: db }
    }

    pub fn export_to_string(
        &self,
        graph: &Graph,
        format: &ExportFileType,
        render_config_override: Option<RenderConfig>,
    ) -> CoreResult<String> {
        // Default render config
        let default_render_config = RenderConfig {
            contain_nodes: true,
            orientation: RenderConfigOrientation::TB,
            apply_layers: true,
            built_in_styles: RenderConfigBuiltInStyle::Light,
            target_options: RenderTargetOptions {
                graphviz: Some(crate::plan::GraphvizRenderOptions::default()),
                mermaid: None,
            },
            add_node_comments_as_notes: false,
            note_position: NotePosition::Left,
            use_node_weight: true,
            use_edge_weight: true,
            layer_source_styles: Vec::new(),
        };
        let render_config = render_config_override.unwrap_or(default_render_config);

        match format {
            ExportFileType::DOT => Ok(to_dot::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("DOT render failed: {}", e)))?),
            ExportFileType::GML => Ok(to_gml::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("GML render failed: {}", e)))?),
            ExportFileType::JSON => Ok(to_json::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("JSON render failed: {}", e)))?),
            ExportFileType::Mermaid => Ok(to_mermaid::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("Mermaid render failed: {}", e)))?),
            ExportFileType::PlantUML => Ok(to_plantuml::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("PlantUML render failed: {}", e)))?),
            ExportFileType::PlantUmlMindmap => Ok(to_plantuml_mindmap::render(
                graph,
                &render_config,
            )
            .map_err(|e| CoreError::internal(format!("PlantUML mindmap render failed: {}", e)))?),
            ExportFileType::PlantUmlWbs => Ok(to_plantuml_wbs::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("PlantUML WBS render failed: {}", e)))?),
            ExportFileType::MermaidMindmap => Ok(to_mermaid_mindmap::render(graph, &render_config)
                .map_err(|e| {
                    CoreError::internal(format!("Mermaid mindmap render failed: {}", e))
                })?),
            ExportFileType::MermaidTreemap => Ok(to_mermaid_treemap::render(graph, &render_config)
                .map_err(|e| {
                    CoreError::internal(format!("Mermaid treemap render failed: {}", e))
                })?),
            ExportFileType::CSVNodes => Ok(to_csv_nodes::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("CSV nodes render failed: {}", e)))?),
            ExportFileType::CSVEdges => Ok(to_csv_edges::render(graph, &render_config)
                .map_err(|e| CoreError::internal(format!("CSV edges render failed: {}", e)))?),
            _ => Err(CoreError::validation(
                "Export format not implemented for string output",
            )),
        }
    }

    #[allow(dead_code)] // Reserved for future plan export execution
    pub async fn execute_plan_exports(
        &self,
        _project_id: i32,
        plan_yaml: &str,
    ) -> CoreResult<Vec<String>> {
        let plan: Plan = serde_yaml::from_str(plan_yaml)
            .map_err(|e| CoreError::validation(format!("Invalid plan YAML: {}", e)))?;
        // let mut graph = graph_service.build_graph_from_project(project_id).await?;
        let mut graph = Graph::default(); // Placeholder

        let mut outputs = Vec::new();

        for export_item in &plan.export.profiles {
            let graph_config = export_item.get_graph_config();
            let render_config = export_item.get_render_config();

            // Apply transformations if specified
            if graph_config.invert_graph {
                graph = graph
                    .invert_graph()
                    .map_err(|e| CoreError::internal(format!("Failed to invert graph: {}", e)))?;
            }

            if graph_config.max_partition_width > 0 {
                let _ = graph.modify_graph_limit_partition_width(graph_config.max_partition_width);
            }

            if graph_config.max_partition_depth > 0 {
                let _ = graph.modify_graph_limit_partition_depth(graph_config.max_partition_depth);
            }

            // Generate export
            let output =
                self.export_to_string(&graph, &export_item.exporter, Some(render_config))?;
            outputs.push(output);
        }

        Ok(outputs)
    }
}
