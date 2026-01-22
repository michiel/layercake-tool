use super::AppContext;
use crate::auth::Actor;
use crate::errors::{CoreError, CoreResult};
use crate::export::sequence_renderer::SequenceRenderConfigResolved;
use crate::export::{to_mermaid_sequence, to_plantuml_sequence};
use crate::plan::{ExportFileType, RenderConfig};
use crate::plan_dag::config::SequenceArtefactRenderTarget;
use crate::sequence_context::{apply_render_config, build_story_context};

fn apply_preview_limit(content: String, format: ExportFileType, max_rows: Option<usize>) -> String {
    match (format, max_rows) {
        (
            ExportFileType::CSVNodes | ExportFileType::CSVEdges | ExportFileType::CSVMatrix,
            Some(limit),
        ) => {
            let mut limited_lines = Vec::new();

            for (index, line) in content.lines().enumerate() {
                if index == 0 || index <= limit {
                    limited_lines.push(line.to_string());
                } else {
                    break;
                }
            }

            limited_lines.join("\n")
        }
        _ => content,
    }
}

impl AppContext {
    pub async fn preview_graph_export(
        &self,
        _actor: &Actor,
        graph_id: i32,
        format: ExportFileType,
        render_config: Option<RenderConfig>,
        max_rows: Option<usize>,
    ) -> CoreResult<String> {
        let graph = self
            .graph_service
            .build_graph_from_dag_graph(graph_id)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load graph {}: {}", graph_id, e))
            })?;

        let content = self
            .export_service
            .export_to_string(&graph, &format, render_config)
            .map_err(|e| CoreError::internal(format!("Failed to render graph export: {}", e)))?;

        Ok(apply_preview_limit(content, format, max_rows))
    }

    #[allow(dead_code)]
    pub async fn preview_sequence_export(
        &self,
        _actor: &Actor,
        project_id: i32,
        story_id: i32,
        render_target: SequenceArtefactRenderTarget,
        render_config: SequenceRenderConfigResolved,
    ) -> CoreResult<String> {
        let base_context = build_story_context(&self.db, project_id, story_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to build story context: {}", e)))?;

        let context = apply_render_config(&base_context, render_config);

        let rendered = match render_target {
            SequenceArtefactRenderTarget::MermaidSequence => to_mermaid_sequence::render(&context)
                .map_err(|e| {
                    CoreError::internal(format!("Failed to render Mermaid sequence: {}", e))
                })?,
            SequenceArtefactRenderTarget::PlantUmlSequence => {
                to_plantuml_sequence::render(&context).map_err(|e| {
                    CoreError::internal(format!("Failed to render PlantUML sequence: {}", e))
                })?
            }
        };

        Ok(rendered)
    }
}
