use std::path::PathBuf;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use layercake_code_analysis::analyzer::analyze_path;
use layercake_code_analysis::infra::{analyze_infra, correlate_code_infra};
use layercake_code_analysis::report::markdown::{strip_csv_blocks, MarkdownReporter};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, Set, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::code_analysis_graph::analysis_to_graph;
use crate::database::entities::code_analysis_profiles;
use crate::graph::{Edge, Graph, Layer};
use crate::infra_graph::infra_to_graph;
use crate::services::data_set_service::DataSetService;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeAnalysisProfile {
    pub id: String,
    pub project_id: i32,
    pub file_path: String,
    pub dataset_id: Option<i32>,
    pub last_run: Option<DateTime<Utc>>,
    pub report: Option<String>,
}

impl From<code_analysis_profiles::Model> for CodeAnalysisProfile {
    fn from(model: code_analysis_profiles::Model) -> Self {
        Self {
            id: model.id,
            project_id: model.project_id,
            file_path: model.file_path,
            dataset_id: model.dataset_id,
            last_run: model.last_run,
            report: model.report,
        }
    }
}

fn merge_graphs(
    mut primary: Graph,
    secondary: Graph,
    annotation: Option<String>,
    correlation: Option<&layercake_code_analysis::infra::CorrelationReport>,
) -> Graph {
    use std::collections::HashSet;

    let mut node_ids: HashSet<String> = primary.nodes.iter().map(|n| n.id.clone()).collect();
    let mut edge_ids: HashSet<String> = primary.edges.iter().map(|e| e.id.clone()).collect();
    let mut id_map = std::collections::HashMap::new();

    let mut code_label_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for node in &primary.nodes {
        code_label_map
            .entry(node.label.clone())
            .or_default()
            .push(node.id.clone());
        if let Some(comment) = &node.comment {
            code_label_map
                .entry(comment.clone())
                .or_default()
                .push(node.id.clone());
        }
        if let Some(attrs) = &node.attributes {
            let as_str = attrs.to_string();
            code_label_map
                .entry(as_str)
                .or_default()
                .push(node.id.clone());
        }
    }

    for node in &secondary.nodes {
        let mut new_id = node.id.clone();
        while node_ids.contains(&new_id) {
            new_id = format!("infra_{}", new_id);
        }
        id_map.insert(node.id.clone(), new_id.clone());
        node_ids.insert(new_id);
    }

    for mut node in secondary.nodes {
        if let Some(mapped) = id_map.get(&node.id) {
            node.id = mapped.clone();
        }
        if let Some(parent) = node.belongs_to.clone() {
            if let Some(mapped_parent) = id_map.get(&parent) {
                node.belongs_to = Some(mapped_parent.clone());
            }
        }
        primary.nodes.push(node);
    }

    for mut edge in secondary.edges {
        edge.source = id_map.get(&edge.source).cloned().unwrap_or(edge.source);
        edge.target = id_map.get(&edge.target).cloned().unwrap_or(edge.target);
        let mut new_edge_id = edge.id.clone();
        while edge_ids.contains(&new_edge_id) {
            new_edge_id = format!("infra_{}", new_edge_id);
        }
        edge.id = new_edge_id.clone();
        edge_ids.insert(new_edge_id);
        primary.edges.push(edge);
    }

    for layer in secondary.layers {
        if !primary.layers.iter().any(|l| l.id == layer.id) {
            primary.layers.push(layer);
        }
    }

    if let Some(corr) = correlation {
        if !primary.layers.iter().any(|l| l.id == "infra-code-link") {
            primary.layers.push(Layer::new(
                "infra-code-link",
                "Code ↔ Infra",
                "#e0f2fe",
                "#0ea5e9",
                "#0ea5e9",
            ));
        }

        let mut seen = HashSet::new();
        let mut next_edge_id = || loop {
            let cand = format!("edge_{}", edge_ids.len() + 1);
            if edge_ids.insert(cand.clone()) {
                break cand;
            }
        };

        for m in &corr.matches {
            let infra_id = id_map
                .get(&m.infra_node)
                .cloned()
                .unwrap_or(m.infra_node.clone());
            let code_id = code_label_map
                .get(&m.code_node)
                .and_then(|list| list.first().cloned());
            if let Some(code_id) = code_id {
                let key = (code_id.clone(), infra_id.clone());
                if seen.insert(key) {
                    primary.edges.push(Edge {
                        id: next_edge_id(),
                        source: code_id.clone(),
                        target: infra_id.clone(),
                        label: m.reason.clone(),
                        layer: "infra-code-link".to_string(),
                        weight: 1,
                        comment: None,
                        dataset: None,
                        attributes: None,
                    });
                }
            }
        }
        if !corr.warnings.is_empty() {
            let warnings = corr.warnings.join("\n");
            primary.append_annotation(format!("Infra correlation warnings:\n{warnings}"));
        }
    }

    if let Some(text) = annotation {
        primary.append_annotation(text);
    }

    primary.nodes.sort_by(|a, b| a.id.cmp(&b.id));
    primary.edges.sort_by(|a, b| a.id.cmp(&b.id));
    primary.layers.sort_by(|a, b| a.id.cmp(&b.id));

    primary
}

#[derive(Clone)]
pub struct CodeAnalysisService {
    db: DatabaseConnection,
}

impl CodeAnalysisService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn ensure_table(&self) -> Result<()> {
        let sql = "CREATE TABLE IF NOT EXISTS code_analysis_profiles (
            id TEXT PRIMARY KEY,
            project_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            dataset_id INTEGER,
            last_run TEXT,
            report TEXT
        )";
        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                sql.to_string(),
            ))
            .await?;
        Ok(())
    }

    pub async fn list(&self, project_id: i32) -> Result<Vec<CodeAnalysisProfile>> {
        self.ensure_table().await?;
        let results = code_analysis_profiles::Entity::find()
            .filter(code_analysis_profiles::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await?;
        Ok(results.into_iter().map(CodeAnalysisProfile::from).collect())
    }

    pub async fn create(
        &self,
        project_id: i32,
        file_path: String,
        dataset_id: Option<i32>,
    ) -> Result<CodeAnalysisProfile> {
        self.ensure_table().await?;
        let id = Uuid::new_v4().to_string();
        let active = code_analysis_profiles::ActiveModel {
            id: Set(id.clone()),
            project_id: Set(project_id),
            file_path: Set(file_path),
            dataset_id: Set(dataset_id),
            last_run: Set(None),
            report: Set(None),
        };

        code_analysis_profiles::Entity::insert(active.clone())
            .exec(&self.db)
            .await?;

        let model = code_analysis_profiles::Entity::find_by_id(id.clone())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Failed to find inserted item"))?;

        Ok(CodeAnalysisProfile::from(model))
    }

    pub async fn update(
        &self,
        id: &str,
        file_path: Option<String>,
        dataset_id: Option<Option<i32>>,
    ) -> Result<CodeAnalysisProfile> {
        self.ensure_table().await?;
        let mut model = code_analysis_profiles::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Profile not found"))?
            .into_active_model();

        if let Some(path) = file_path {
            model.file_path = Set(path);
        }
        if let Some(ds) = dataset_id {
            model.dataset_id = Set(ds);
        }

        let updated = model.update(&self.db).await?;
        Ok(CodeAnalysisProfile::from(updated))
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        self.ensure_table().await?;
        let result = code_analysis_profiles::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected > 0)
    }

    async fn get_by_id(&self, id: &str) -> Result<code_analysis_profiles::Model> {
        self.ensure_table().await?;
        code_analysis_profiles::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Profile not found"))
    }

    pub async fn run(&self, id: &str) -> Result<CodeAnalysisProfile> {
        let profile = self.get_by_id(id).await?;

        let reporter = MarkdownReporter::default();
        let path: PathBuf = profile.file_path.clone().into();
        let path_for_task = path.clone();
        let analysis = tokio::task::spawn_blocking(move || analyze_path(&path_for_task)).await??;
        let infra_graph = analyze_infra(&path)?;
        let correlation = correlate_code_infra(&analysis.result, &infra_graph);

        let report_markdown = reporter.render_with_infra(
            &analysis.result,
            &layercake_code_analysis::report::ReportMetadata::new(path, analysis.files_scanned),
            Some(&infra_graph),
            Some(&correlation),
        )?;
        let cleaned_report = strip_csv_blocks(&report_markdown);

        let dataset_id = match profile.dataset_id {
            Some(id) => id,
            None => {
                let ds_service = DataSetService::new(self.db.clone());
                ds_service
                    .create_empty(
                        profile.project_id,
                        "Code analysis".to_string(),
                        Some("Generated by code analysis".to_string()),
                    )
                    .await?
                    .id
            }
        };

        let combined_graph = merge_graphs(
            analysis_to_graph(&analysis.result, None),
            infra_to_graph(&infra_graph, None),
            Some(cleaned_report.clone()),
            Some(&correlation),
        );
        let graph_json = serde_json::to_string(&combined_graph)?;
        let annotation_text = cleaned_report.clone();
        let ds_service = DataSetService::new(self.db.clone());
        ds_service.update_graph_data(dataset_id, graph_json).await?;
        let _ = ds_service
            .update_annotation(dataset_id, Some(annotation_text))
            .await;

        let mut active = profile.into_active_model();
        active.dataset_id = Set(Some(dataset_id));
        active.last_run = Set(Some(Utc::now()));
        active.report = Set(Some(cleaned_report));

        let updated = active.update(&self.db).await?;
        Ok(CodeAnalysisProfile::from(updated))
    }
}
