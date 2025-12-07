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
use layercake_code_analysis::AnalysisResult;
use crate::database::entities::code_analysis_profiles;
use crate::graph::{Edge, Graph, Layer};
use crate::infra_graph::infra_to_graph;
use crate::services::data_set_service::DataSetService;

fn normalize_path(path: &str) -> String {
    path.trim().to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeAnalysisProfile {
    pub id: String,
    pub project_id: i32,
    pub file_path: String,
    pub dataset_id: Option<i32>,
    pub last_run: Option<DateTime<Utc>>,
    pub report: Option<String>,
    pub no_infra: bool,
    pub options: Option<String>,
    pub analysis_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeAnalysisOptions {
    #[serde(alias = "includeDataFlow")]
    #[serde(default = "default_true")]
    pub include_data_flow: bool,
    #[serde(alias = "includeControlFlow")]
    #[serde(default = "default_true")]
    pub include_control_flow: bool,
    #[serde(alias = "includeImports")]
    #[serde(default = "default_true")]
    pub include_imports: bool,
    #[serde(alias = "includeInfra")]
    #[serde(default = "default_true")]
    pub include_infra: bool,
    #[serde(alias = "coalesceFunctions")]
    #[serde(default)]
    pub coalesce_functions: bool,
    #[serde(alias = "excludeKnownSupportFiles")]
    #[serde(default)]
    pub exclude_known_support_files: bool,
    #[serde(alias = "excludeInferredSupport")]
    #[serde(default)]
    pub exclude_inferred_support: bool,
}

fn default_true() -> bool {
    true
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
            no_infra: model.no_infra.unwrap_or(false),
            options: model.options,
            analysis_type: model.analysis_type.unwrap_or_else(|| "code".to_string()),
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

    fn is_known_support_file(path: &str) -> bool {
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(
            filename.as_str(),
            "package-lock.json"
                | "yarn.lock"
                | "pnpm-lock.yaml"
                | "pnpm-lock.yml"
                | "package.json"
                | "pyproject.toml"
                | "requirements.txt"
                | "requirements-dev.txt"
                | "pipfile"
                | "pipfile.lock"
                | "poetry.lock"
                | "setup.py"
                | "setup.cfg"
                | "tox.ini"
                | "makefile"
                | "makefile.toml"
        )
    }

    fn is_inferred_support_path(path: &str) -> bool {
        let lowered = path.to_lowercase();
        let parts: Vec<&str> = lowered.split(&['/', '\\'][..]).collect();
        for part in &parts {
            if part.contains("test")
                || part.contains("spec")
                || part.contains("__tests__")
                || part.contains("fixture")
                || part.contains("fixtures")
                || part.contains("mocks")
                || part.contains("mock")
            {
                return true;
            }
        }
        false
    }

    fn should_exclude(path: &str, opts: &CodeAnalysisOptions) -> bool {
        (opts.exclude_known_support_files && Self::is_known_support_file(path))
            || (opts.exclude_inferred_support && Self::is_inferred_support_path(path))
    }

    fn filter_support_files(mut result: AnalysisResult, opts: &CodeAnalysisOptions) -> AnalysisResult {
        result.functions.retain(|f| !Self::should_exclude(&f.file_path, opts));
        result.imports.retain(|i| !Self::should_exclude(&i.file_path, opts));
        result.data_flows
            .retain(|f| !Self::should_exclude(&f.file_path, opts));
        result.call_edges
            .retain(|c| !Self::should_exclude(&c.file_path, opts));
        result.entry_points
            .retain(|e| !Self::should_exclude(&e.file_path, opts));
        result.env_vars
            .retain(|e| !Self::should_exclude(&e.file_path, opts));
        result
            .files
            .retain(|f| !Self::should_exclude(f, opts));
        result
            .directories
            .retain(|d| !Self::should_exclude(d, opts));
        result
    }

    async fn ensure_table(&self) -> Result<()> {
        let sql = "CREATE TABLE IF NOT EXISTS code_analysis_profiles (
            id TEXT PRIMARY KEY,
            project_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            dataset_id INTEGER,
            last_run TEXT,
            report TEXT,
            no_infra INTEGER DEFAULT 0,
            options TEXT,
            analysis_type TEXT DEFAULT 'code'
        )";
        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                sql.to_string(),
            ))
            .await?;

        // backfill column if missing
        let alter = "ALTER TABLE code_analysis_profiles ADD COLUMN no_infra INTEGER DEFAULT 0";
        let _ = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                alter.to_string(),
            ))
            .await;
        let alter_opts = "ALTER TABLE code_analysis_profiles ADD COLUMN options TEXT";
        let _ = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                alter_opts.to_string(),
            ))
            .await;
        let alter_analysis_type =
            "ALTER TABLE code_analysis_profiles ADD COLUMN analysis_type TEXT DEFAULT 'code'";
        let _ = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                alter_analysis_type.to_string(),
            ))
            .await;
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
        no_infra: bool,
        options: Option<String>,
        analysis_type: String,
    ) -> Result<CodeAnalysisProfile> {
        self.ensure_table().await?;
        let id = Uuid::new_v4().to_string();
        let active = code_analysis_profiles::ActiveModel {
            id: Set(id.clone()),
            project_id: Set(project_id),
            file_path: Set(normalize_path(&file_path)),
            dataset_id: Set(dataset_id),
            last_run: Set(None),
            report: Set(None),
            no_infra: Set(Some(no_infra)),
            options: Set(options),
            analysis_type: Set(Some(analysis_type)),
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
        no_infra: Option<bool>,
        options: Option<Option<String>>,
        analysis_type: Option<String>,
    ) -> Result<CodeAnalysisProfile> {
        self.ensure_table().await?;
        let mut model = code_analysis_profiles::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Profile not found"))?
            .into_active_model();

        if let Some(path) = file_path {
            model.file_path = Set(normalize_path(&path));
        }
        if let Some(ds) = dataset_id {
            model.dataset_id = Set(ds);
        }
        if let Some(flag) = no_infra {
            model.no_infra = Set(Some(flag));
        }
        if let Some(opts) = options {
            model.options = Set(opts);
        }
        if let Some(t) = analysis_type {
            model.analysis_type = Set(Some(t));
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

    pub async fn get(&self, id: String) -> Result<Option<CodeAnalysisProfile>> {
        let model = code_analysis_profiles::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(model.map(CodeAnalysisProfile::from))
    }

    pub async fn run(&self, id: &str) -> Result<CodeAnalysisProfile> {
        let profile = self.get_by_id(id).await?;
        let no_infra_flag = profile.no_infra.unwrap_or(false);
        let analysis_type = profile.analysis_type.clone().unwrap_or_else(|| "code".to_string());

        let reporter = MarkdownReporter::default();
        let normalized_path = normalize_path(&profile.file_path);
        let path: PathBuf = normalized_path.clone().into();
        if !path.exists() {
            return Err(anyhow!("Code analysis path does not exist: {}", normalized_path));
        }
        let path_for_task = path.clone();
        let analysis = tokio::task::spawn_blocking(move || analyze_path(&path_for_task)).await??;
        let opts: CodeAnalysisOptions = profile
            .options
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or(CodeAnalysisOptions {
                include_data_flow: true,
                include_control_flow: true,
                include_imports: true,
                include_infra: true,
                coalesce_functions: false,
                exclude_known_support_files: false,
                exclude_inferred_support: false,
            });
        let mut result = analysis.result;
        if opts.exclude_known_support_files || opts.exclude_inferred_support {
            result = Self::filter_support_files(result, &opts);
        }
        if !opts.include_data_flow {
            result.data_flows.clear();
        }
        if !opts.include_control_flow {
            result.call_edges.clear();
        }
        if !opts.include_imports {
            result.imports.clear();
        }

        let (infra_graph, correlation) = if no_infra_flag || !opts.include_infra {
            (None, None)
        } else {
            let infra = analyze_infra(&path)?;
            let corr = correlate_code_infra(&result, &infra);
            (Some(infra), Some(corr))
        };

        let report_markdown = reporter.render_with_infra(
            &result,
            &layercake_code_analysis::report::ReportMetadata::new(path, analysis.files_scanned),
            infra_graph.as_ref(),
            correlation.as_ref(),
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

        let combined_graph = if analysis_type == "solution" {
            // Build solution-level graph: coalesce to files (if requested), drop libraries, ensure single root
            let mut graph = analysis_to_graph(&result, Some(cleaned_report.clone()), opts.coalesce_functions);
            graph.nodes.retain(|n| n.layer != "library");

            // Identify or create root
            let mut root_id = None;
            for node in graph.nodes.iter_mut() {
                if node.layer == "scope" && node.belongs_to.is_none() {
                    root_id = Some(node.id.clone());
                    node.label = "Solution".to_string();
                    node.is_partition = true;
                    break;
                }
            }
            if root_id.is_none() {
                let rid = "root_solution".to_string();
                root_id = Some(rid.clone());
                graph.nodes.push(crate::graph::Node {
                    id: rid.clone(),
                    label: "Solution".to_string(),
                    layer: "scope".to_string(),
                    is_partition: true,
                    belongs_to: None,
                    weight: 1,
                    comment: None,
                    dataset: None,
                    attributes: None,
                });
            }
            let root_id = root_id.unwrap();

            for node in graph.nodes.iter_mut() {
                if node.belongs_to.is_none() {
                    node.belongs_to = Some(root_id.clone());
                }
            }

            // Drop noisy edges (imports) and any edges to root/partition scope nodes
            graph
                .edges
                .retain(|e| e.layer != "import" && e.source != root_id && e.target != root_id);
            // Remove edges that connect to scope partitions
            let scope_ids: std::collections::HashSet<String> = graph
                .nodes
                .iter()
                .filter(|n| n.layer == "scope" && n.is_partition)
                .map(|n| n.id.clone())
                .collect();
            graph
                .edges
                .retain(|e| !scope_ids.contains(&e.source) && !scope_ids.contains(&e.target));
            graph
        } else if let Some(infra_graph) = infra_graph {
            merge_graphs(
                analysis_to_graph(&result, None, opts.coalesce_functions),
                infra_to_graph(&infra_graph, None),
                Some(cleaned_report.clone()),
                correlation.as_ref(),
            )
        } else {
            analysis_to_graph(&result, Some(cleaned_report.clone()), opts.coalesce_functions)
        };
        let graph_json = serde_json::to_string(&combined_graph)?;
        let annotation_text = cleaned_report.clone();
        let ds_service = DataSetService::new(self.db.clone());
        ds_service.update_graph_data(dataset_id, graph_json).await?;
        let _ = ds_service
            .update_annotation(dataset_id, "Analysis Report".to_string(), annotation_text)
            .await;

        let mut active = profile.into_active_model();
        active.dataset_id = Set(Some(dataset_id));
        active.last_run = Set(Some(Utc::now()));
        active.report = Set(Some(cleaned_report));

        let updated = active.update(&self.db).await?;
        Ok(CodeAnalysisProfile::from(updated))
    }
}
