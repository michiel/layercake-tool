//! Project health diagnostics (`layercake doctor`).
//!
//! Scans a project for the classes of problem that otherwise only surface as
//! silent-empty output: orphaned computed graphs, unresolvable sequence edge
//! references, plan DAG nodes pointing at missing datasets, and empty sequence
//! contexts. The logic lives in core so it can back both the CLI command and a
//! future GraphQL endpoint, and so it is unit-testable.

use anyhow::Result;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[cfg(test)]
use crate::database::entities::projects;
use crate::database::entities::{
    data_sets, graph_data, plan_dag_nodes, plans, sequence_contexts, sequences, stories,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    /// Short machine-readable category, e.g. "orphaned-computed-graph".
    pub check: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub project_id: i32,
    pub findings: Vec<Finding>,
}

impl DoctorReport {
    pub fn is_healthy(&self) -> bool {
        self.findings.is_empty()
    }
    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }
}

/// Run all diagnostics for a project.
pub async fn run_diagnostics(db: &DatabaseConnection, project_id: i32) -> Result<DoctorReport> {
    let mut findings = Vec::new();

    // Resolve the project's plan node ids (nodes are scoped by plan_id).
    let plan_ids: Vec<i32> = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(project_id))
        .all(db)
        .await?
        .into_iter()
        .map(|p| p.id)
        .collect();

    let nodes = if plan_ids.is_empty() {
        Vec::new()
    } else {
        plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.is_in(plan_ids.clone()))
            .all(db)
            .await?
    };
    let node_ids: HashSet<String> = nodes.iter().map(|n| n.id.clone()).collect();

    // Dataset PKs that exist (for the missing-dataset check).
    let dataset_ids: HashSet<i32> = data_sets::Entity::find()
        .filter(data_sets::Column::ProjectId.eq(project_id))
        .all(db)
        .await?
        .into_iter()
        .map(|d| d.id)
        .collect();

    check_nodes_reference_existing_datasets(&nodes, &dataset_ids, &mut findings);
    check_orphaned_computed_graphs(db, project_id, &node_ids, &mut findings).await?;
    check_sequence_contexts(db, project_id, &mut findings).await?;
    check_story_sequences_resolve(db, project_id, &mut findings).await?;

    Ok(DoctorReport {
        project_id,
        findings,
    })
}

/// Plan DAG nodes whose `config.dataSetId` points at a dataset that no longer
/// exists in this project.
fn check_nodes_reference_existing_datasets(
    nodes: &[plan_dag_nodes::Model],
    dataset_ids: &HashSet<i32>,
    findings: &mut Vec<Finding>,
) {
    for node in nodes {
        let config: serde_json::Value = match serde_json::from_str(&node.config_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let linked = config
            .get("dataSetId")
            .or_else(|| config.get("datasetId"))
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        if let Some(id) = linked {
            if !dataset_ids.contains(&id) {
                findings.push(Finding {
                    severity: Severity::Error,
                    check: "node-missing-dataset".into(),
                    message: format!(
                        "plan node '{}' ({}) references dataset {} which does not exist",
                        node.id, node.node_type, id
                    ),
                });
            }
        }
    }
}

/// Computed `graph_data` rows whose originating DAG node is gone.
async fn check_orphaned_computed_graphs(
    db: &DatabaseConnection,
    project_id: i32,
    node_ids: &HashSet<String>,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let graphs = graph_data::Entity::find()
        .filter(graph_data::Column::ProjectId.eq(project_id))
        .all(db)
        .await?;
    for g in graphs {
        if g.source_type == "computed" {
            if let Some(dag_node_id) = &g.dag_node_id {
                if !node_ids.contains(dag_node_id) {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        check: "orphaned-computed-graph".into(),
                        message: format!(
                            "computed graph '{}' (id {}) originates from DAG node '{}' which no longer exists",
                            g.name, g.id, dag_node_id
                        ),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Persisted sequence contexts that are empty (no participants), which render
/// as blank diagrams.
async fn check_sequence_contexts(
    db: &DatabaseConnection,
    project_id: i32,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let contexts = sequence_contexts::Entity::find()
        .filter(sequence_contexts::Column::ProjectId.eq(project_id))
        .all(db)
        .await?;
    for ctx in contexts {
        let parsed: serde_json::Value = match serde_json::from_str(&ctx.context_json) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let participant_count = parsed
            .get("participants")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if participant_count == 0 {
            findings.push(Finding {
                severity: Severity::Warning,
                check: "empty-sequence-context".into(),
                message: format!(
                    "sequence context for node '{}' (story {}) has no participants — its diagram will be empty",
                    ctx.node_id, ctx.story_id
                ),
            });
        }
        // Surface any warnings recorded when the context was built.
        if let Some(warns) = parsed.get("warnings").and_then(|v| v.as_array()) {
            for w in warns {
                if let Some(text) = w.as_str() {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        check: "sequence-context-warning".into(),
                        message: format!("node '{}': {}", ctx.node_id, text),
                    });
                }
            }
        }
    }
    Ok(())
}

/// Stories whose sequences reference edges that don't resolve against the
/// enabled datasets — the classic "green but empty" author error.
async fn check_story_sequences_resolve(
    db: &DatabaseConnection,
    project_id: i32,
    findings: &mut Vec<Finding>,
) -> Result<()> {
    let story_models = stories::Entity::find()
        .filter(stories::Column::ProjectId.eq(project_id))
        .all(db)
        .await?;
    for story in story_models {
        // Reuse the real context builder so this check matches actual rendering.
        match crate::sequence_context::build_story_context(db, project_id, story.id).await {
            Ok(ctx) => {
                for w in ctx.warnings {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        check: "story-sequence-unresolved".into(),
                        message: format!("story '{}' (id {}): {}", story.name, story.id, w),
                    });
                }
                let has_sequences = sequences::Entity::find()
                    .filter(sequences::Column::StoryId.eq(story.id))
                    .one(db)
                    .await?
                    .is_some();
                if has_sequences && ctx.participants.is_empty() {
                    findings.push(Finding {
                        severity: Severity::Error,
                        check: "story-empty".into(),
                        message: format!(
                            "story '{}' (id {}) has sequences but resolves to zero participants",
                            story.name, story.id
                        ),
                    });
                }
            }
            Err(e) => findings.push(Finding {
                severity: Severity::Error,
                check: "story-build-failed".into(),
                message: format!("story '{}' (id {}): {}", story.name, story.id, e),
            }),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::test_utils::setup_test_db;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};

    async fn seed_base(db: &DatabaseConnection) {
        projects::ActiveModel {
            id: Set(1),
            name: Set("P".into()),
            description: Set(None),
            tags: Set("[]".into()),
            import_export_path: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await
        .unwrap();

        data_sets::ActiveModel {
            id: Set(10),
            project_id: Set(1),
            name: Set("ds".into()),
            description: Set(None),
            file_format: Set("json".into()),
            data_type: Set("graph".into()),
            origin: Set("manual_edit".into()),
            filename: Set("ds.json".into()),
            blob: Set(Vec::new()),
            graph_json: Set(
                r#"{"nodes":[{"id":"n1","label":"A","layer":"l","weight":1}],"edges":[],"layers":[]}"#
                    .into(),
            ),
            status: Set("active".into()),
            error_message: Set(None),
            file_size: Set(0),
            processed_at: Set(None),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            annotations: Set(None),
        }
        .insert(db)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn healthy_empty_project_has_no_findings() {
        let db = setup_test_db().await;
        seed_base(&db).await;
        let report = run_diagnostics(&db, 1).await.unwrap();
        assert!(report.is_healthy(), "{:?}", report.findings);
    }

    #[tokio::test]
    async fn flags_story_with_unresolvable_edge() {
        let db = setup_test_db().await;
        seed_base(&db).await;

        stories::ActiveModel {
            id: Set(1),
            project_id: Set(1),
            name: Set("Story1".into()),
            description: Set(None),
            tags: Set("[]".into()),
            enabled_dataset_ids: Set("[10]".into()),
            enabled_graph_ids: Set("[]".into()),
            layer_config: Set("[]".into()),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        }
        .insert(&db)
        .await
        .unwrap();

        sequences::ActiveModel {
            id: Set(1),
            story_id: Set(1),
            name: Set("seq".into()),
            description: Set(None),
            enabled_dataset_ids: Set("[10]".into()),
            edge_order: Set(r#"[{"datasetId":10,"edgeId":"nope"}]"#.into()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&db)
        .await
        .unwrap();

        let report = run_diagnostics(&db, 1).await.unwrap();
        assert!(!report.is_healthy());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.check == "story-sequence-unresolved" && f.message.contains("nope")),
            "expected unresolved-edge finding: {:?}",
            report.findings
        );
        assert!(
            report.findings.iter().any(|f| f.check == "story-empty"),
            "expected empty-story finding: {:?}",
            report.findings
        );
        assert!(report.error_count() >= 1);
    }
}
