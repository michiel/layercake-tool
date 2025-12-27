use std::path::PathBuf;

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

use crate::code_analysis_enhanced_solution_graph::analysis_to_enhanced_solution_graph;
use crate::code_analysis_graph::analysis_to_graph;
use crate::code_analysis_solution_graph::analysis_to_solution_graph;
use crate::database::entities::code_analysis_profiles;
use crate::auth::Actor;
use crate::errors::{CoreError, CoreResult};
use crate::graph::{Edge, Graph, Layer};
use crate::infra_graph::infra_to_graph;
use crate::services::data_set_service::DataSetService;
use layercake_code_analysis::AnalysisResult;

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
    pub last_result: Option<String>,
    pub solution_options: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SolutionAnalysisOptions {
    #[serde(default = "default_true")]
    pub include_infra: bool,
    #[serde(default)]
    pub include_imports: bool,
    #[serde(default)]
    pub include_data_flow: bool,
    #[serde(default)]
    pub include_control_flow: bool,
    #[serde(default)]
    pub exclude_known_support_files: bool,
    #[serde(default)]
    pub exclude_inferred_support: bool,
    #[serde(default)]
    pub exclude_helpers: bool,
    #[serde(default)]
    pub use_enhanced_correlation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisOptions {
    pub code: Option<CodeAnalysisOptions>,
    pub solution: Option<SolutionAnalysisOptions>,
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
            last_result: model.last_result,
            solution_options: model.solution_options,
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
    let mut file_hint_map: std::collections::HashMap<String, Vec<String>> =
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
            if let Some(file) = attrs.get("file").and_then(|v| v.as_str()) {
                file_hint_map
                    .entry(file.to_string())
                    .or_default()
                    .push(node.id.clone());
            }
            if let Some(file) = attrs.get("file_path").and_then(|v| v.as_str()) {
                file_hint_map
                    .entry(file.to_string())
                    .or_default()
                    .push(node.id.clone());
            }
        }
        if let Some(comment) = &node.comment {
            file_hint_map
                .entry(comment.clone())
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
        if node.layer == "infra" {
            node.is_partition = false;
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

        let find_node_for_handler = |path_hint: &str, func_label: &str| -> Option<String> {
            let norm_path = path_hint.replace('\\', "/");
            let mut variants = vec![norm_path.clone()];
            if !norm_path.ends_with(".py") {
                variants.push(format!("{norm_path}.py"));
            }
            if !norm_path.ends_with("/__init__") {
                variants.push(format!("{norm_path}/__init__.py"));
            }

            for variant in &variants {
                if let Some(ids) = file_hint_map.get(variant) {
                    if let Some(id) = ids
                        .iter()
                        .find(|id| {
                            primary
                                .nodes
                                .iter()
                                .find(|n| &n.id == *id && n.label == func_label)
                                .is_some()
                        })
                        .cloned()
                    {
                        return Some(id);
                    }
                }
            }

            for node in &primary.nodes {
                if node.label != func_label {
                    continue;
                }
                let mut file_matches = false;
                if let Some(comment) = &node.comment {
                    let c = comment.replace('\\', "/");
                    file_matches = variants.iter().any(|v| c.ends_with(v));
                }
                if !file_matches {
                    if let Some(attrs) = &node.attributes {
                        if let Some(file) = attrs.get("file").and_then(|v| v.as_str()) {
                            let f = file.replace('\\', "/");
                            file_matches |= variants.iter().any(|v| f.ends_with(v));
                        }
                        if let Some(file) = attrs.get("file_path").and_then(|v| v.as_str()) {
                            let f = file.replace('\\', "/");
                            file_matches |= variants.iter().any(|v| f.ends_with(v));
                        }
                    }
                }
                if file_matches {
                    return Some(node.id.clone());
                }
            }
            None
        };

        for m in &corr.matches {
            let infra_id = id_map
                .get(&m.infra_node)
                .cloned()
                .unwrap_or(m.infra_node.clone());
            let code_id = if m.code_node.contains("::") {
                let parts: Vec<&str> = m.code_node.split("::").collect();
                if parts.len() >= 2 {
                    let func = parts.last().unwrap().to_string();
                    let path = parts[..parts.len() - 1].join("::");
                    find_node_for_handler(&path, &func).or_else(|| {
                        code_label_map
                            .get(&func)
                            .and_then(|list| list.first().cloned())
                    })
                } else {
                    code_label_map
                        .get(&m.code_node)
                        .and_then(|list| list.first().cloned())
                }
            } else {
                code_label_map
                    .get(&m.code_node)
                    .and_then(|list| list.first().cloned())
            };
            if let Some(code_id) = code_id {
                let key = (code_id.clone(), infra_id.clone());
                if seen.insert(key) {
                    primary.edges.push(Edge {
                        id: next_edge_id(),
                        source: code_id.clone(),
                        target: infra_id.clone(),
                        label: m.reason.clone(),
                        layer: "infra-code-link".to_string(),
                        weight: m.confidence.max(10) as i32,
                        comment: Some(format!("Confidence: {}%", m.confidence)),
                        dataset: None,
                        attributes: Some(serde_json::json!({
                            "confidence": m.confidence,
                            "reason": m.reason,
                            "edge_type": "correlation"
                        })),
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

    fn filter_support_files(
        mut result: AnalysisResult,
        opts: &CodeAnalysisOptions,
    ) -> AnalysisResult {
        result
            .functions
            .retain(|f| !Self::should_exclude(&f.file_path, opts));
        result
            .imports
            .retain(|i| !Self::should_exclude(&i.file_path, opts));
        result
            .data_flows
            .retain(|f| !Self::should_exclude(&f.file_path, opts));
        result
            .call_edges
            .retain(|c| !Self::should_exclude(&c.file_path, opts));
        result
            .entry_points
            .retain(|e| !Self::should_exclude(&e.file_path, opts));
        result
            .env_vars
            .retain(|e| !Self::should_exclude(&e.file_path, opts));
        result.files.retain(|f| !Self::should_exclude(f, opts));
        result
            .directories
            .retain(|d| !Self::should_exclude(d, opts));
        result
    }

    async fn ensure_table(&self) -> CoreResult<()> {
        let sql = "CREATE TABLE IF NOT EXISTS code_analysis_profiles (
            id TEXT PRIMARY KEY,
            project_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            dataset_id INTEGER,
            last_run TEXT,
            report TEXT,
            no_infra INTEGER DEFAULT 0,
            options TEXT,
            analysis_type TEXT DEFAULT 'code',
            last_result TEXT
        )";
        self.db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                sql.to_string(),
            ))
            .await
            .map_err(|e| CoreError::internal("Failed to ensure code analysis table").with_source(e))?;

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
        let alter_last_result = "ALTER TABLE code_analysis_profiles ADD COLUMN last_result TEXT";
        let _ = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                alter_last_result.to_string(),
            ))
            .await;
        let alter_solution_options =
            "ALTER TABLE code_analysis_profiles ADD COLUMN solution_options TEXT";
        let _ = self
            .db
            .execute(Statement::from_string(
                self.db.get_database_backend(),
                alter_solution_options.to_string(),
            ))
            .await;
        Ok(())
    }

    pub async fn list(&self, project_id: i32) -> CoreResult<Vec<CodeAnalysisProfile>> {
        self.ensure_table().await?;
        let results = code_analysis_profiles::Entity::find()
            .filter(code_analysis_profiles::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to list code analysis profiles").with_source(e))?;
        Ok(results.into_iter().map(CodeAnalysisProfile::from).collect())
    }

    pub async fn create(
        &self,
        _actor: &Actor,
        project_id: i32,
        file_path: String,
        dataset_id: Option<i32>,
        no_infra: bool,
        options: Option<String>,
        analysis_type: String,
        solution_options: Option<String>,
    ) -> CoreResult<CodeAnalysisProfile> {
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
            last_result: Set(None),
            solution_options: Set(solution_options),
        };

        code_analysis_profiles::Entity::insert(active.clone())
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to create code analysis profile").with_source(e))?;

        let model = code_analysis_profiles::Entity::find_by_id(id.clone())
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load code analysis profile").with_source(e))?
            .ok_or_else(|| CoreError::not_found("CodeAnalysisProfile", id.clone()))?;

        Ok(CodeAnalysisProfile::from(model))
    }

    pub async fn update(
        &self,
        _actor: &Actor,
        id: &str,
        file_path: Option<String>,
        dataset_id: Option<Option<i32>>,
        no_infra: Option<bool>,
        options: Option<Option<String>>,
        analysis_type: Option<String>,
        solution_options: Option<Option<String>>,
    ) -> CoreResult<CodeAnalysisProfile> {
        self.ensure_table().await?;
        let mut model = code_analysis_profiles::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load code analysis profile").with_source(e))?
            .ok_or_else(|| CoreError::not_found("CodeAnalysisProfile", id.to_string()))?
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
        if let Some(opts) = solution_options {
            model.solution_options = Set(opts);
        }

        let updated = model
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to update code analysis profile").with_source(e))?;
        Ok(CodeAnalysisProfile::from(updated))
    }

    pub async fn delete(&self, _actor: &Actor, id: &str) -> CoreResult<bool> {
        self.ensure_table().await?;
        let result = code_analysis_profiles::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to delete code analysis profile").with_source(e))?;
        Ok(result.rows_affected > 0)
    }

    async fn get_by_id(&self, id: &str) -> CoreResult<code_analysis_profiles::Model> {
        self.ensure_table().await?;
        code_analysis_profiles::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load code analysis profile").with_source(e))?
            .ok_or_else(|| CoreError::not_found("CodeAnalysisProfile", id.to_string()))
    }

    pub async fn get(&self, id: String) -> CoreResult<Option<CodeAnalysisProfile>> {
        let model = code_analysis_profiles::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load code analysis profile").with_source(e))?;
        Ok(model.map(CodeAnalysisProfile::from))
    }

    pub async fn run(&self, _actor: &Actor, id: &str) -> CoreResult<CodeAnalysisProfile> {
        let profile = self.get_by_id(id).await?;
        let no_infra_flag = profile.no_infra.unwrap_or(false);
        let analysis_type = profile
            .analysis_type
            .clone()
            .unwrap_or_else(|| "code".to_string());

        let reporter = MarkdownReporter::default();
        let normalized_path = normalize_path(&profile.file_path);
        let path: PathBuf = normalized_path.clone().into();
        if !path.exists() {
            return Err(CoreError::validation(format!(
                "Code analysis path does not exist: {}",
                normalized_path
            )));
        }
        let path_for_task = path.clone();
        let analysis = tokio::task::spawn_blocking(move || analyze_path(&path_for_task))
            .await
            .map_err(|e| CoreError::internal(format!("Code analysis task failed: {}", e)))?
            .map_err(|e| CoreError::internal(format!("Code analysis failed: {}", e)))?;
        let parsed_opts: AnalysisOptions = profile
            .options
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_default();
        let opts: CodeAnalysisOptions = parsed_opts.code.unwrap_or(CodeAnalysisOptions {
            include_data_flow: true,
            include_control_flow: true,
            include_imports: true,
            include_infra: true,
            coalesce_functions: false,
            exclude_known_support_files: false,
            exclude_inferred_support: false,
        });
        let solution_opts: SolutionAnalysisOptions = profile
            .solution_options
            .as_ref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .or(parsed_opts.solution)
            .unwrap_or_else(|| SolutionAnalysisOptions {
                include_infra: true,
                include_imports: false,
                include_data_flow: false,
                include_control_flow: false,
                exclude_known_support_files: false,
                exclude_inferred_support: false,
                exclude_helpers: false,
                use_enhanced_correlation: false,
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
            let infra = result.infra.clone().or_else(|| analyze_infra(&path).ok());
            let corr = result
                .infra_correlation
                .clone()
                .or_else(|| infra.as_ref().map(|g| correlate_code_infra(&result, g)));
            (infra, corr)
        };

        let report_markdown = reporter.render_with_infra(
            &result,
            &layercake_code_analysis::report::ReportMetadata::new(path, analysis.files_scanned),
            infra_graph.as_ref(),
            correlation.as_ref(),
        )
        .map_err(|e| {
            CoreError::internal(format!("Failed to render code analysis report: {}", e))
        })?;
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
            // Build solution-level graph without function detail
            let mut filtered_result = result.clone();
            if solution_opts.exclude_known_support_files || solution_opts.exclude_inferred_support {
                filtered_result = Self::filter_support_files(
                    filtered_result,
                    &CodeAnalysisOptions {
                        include_data_flow: true,
                        include_control_flow: true,
                        include_imports: true,
                        include_infra: true,
                        coalesce_functions: false,
                        exclude_known_support_files: solution_opts.exclude_known_support_files,
                        exclude_inferred_support: solution_opts.exclude_inferred_support,
                    },
                );
            }
            if solution_opts.exclude_helpers {
                filtered_result.exclude_functions_named(&[
                    "map",
                    "reduce",
                    "print",
                    "log",
                    "debug",
                    "console.log",
                    "console.debug",
                    "console.error",
                    "logging.info",
                    "logging.debug",
                ]);
            }
            if !solution_opts.include_imports {
                filtered_result.imports.clear();
            }
            if !solution_opts.include_data_flow {
                filtered_result.data_flows.clear();
            }
            if !solution_opts.include_control_flow {
                filtered_result.call_edges.clear();
            }
            if !solution_opts.include_infra {
                filtered_result.infra = None;
                filtered_result.infra_correlation = None;
            }
            let code_graph = if solution_opts.use_enhanced_correlation {
                analysis_to_enhanced_solution_graph(&filtered_result, Some(cleaned_report.clone()))
            } else {
                analysis_to_solution_graph(&filtered_result, Some(cleaned_report.clone()))
            };
            let mut graph = if !no_infra_flag && solution_opts.include_infra {
                if let Some(infra) = infra_graph.as_ref() {
                    merge_graphs(
                        code_graph,
                        infra_to_graph(infra, None),
                        None,
                        correlation.as_ref(),
                    )
                } else {
                    code_graph
                }
            } else {
                code_graph
            };

            // Ensure a single root partition labelled "Solution"
            let mut root_id = graph
                .nodes
                .iter()
                .find(|n| n.is_partition && n.belongs_to.is_none() && n.layer == "scope")
                .map(|n| n.id.clone())
                .or_else(|| {
                    graph
                        .nodes
                        .iter()
                        .find(|n| n.is_partition && n.belongs_to.is_none())
                        .map(|n| n.id.clone())
                });
            if let Some(rid) = root_id.clone() {
                if let Some(root) = graph.nodes.iter_mut().find(|n| n.id == rid) {
                    root.label = "Solution".to_string();
                    root.layer = "scope".to_string();
                    root.is_partition = true;
                    root.belongs_to = None;
                }
            }
            if root_id.is_none() {
                let rid = "solution_root".to_string();
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

            // Make sure every node belongs to the solution root and treat file scopes as flow nodes
            let mut codebase_ids: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            for node in graph.nodes.iter_mut() {
                if node.id != root_id && node.belongs_to.is_none() {
                    node.belongs_to = Some(root_id.clone());
                }
                if node.layer == "scope" && node.label == "Codebase" {
                    codebase_ids.insert(node.id.clone());
                }
                if node.layer == "scope" && node.id != root_id {
                    node.is_partition = false;
                }
                if node.layer == "infra" {
                    node.is_partition = false;
                    if node.belongs_to.is_none() {
                        node.belongs_to = Some(root_id.clone());
                    }
                }
            }

            // Promote top-level directories to component partitions to better represent services/functions
            let mut component_for_dir = std::collections::HashMap::new();
            let dir_stubs: Vec<(String, String)> = graph
                .nodes
                .iter()
                .filter(|n| n.layer == "scope" && !n.label.contains('/') && n.id != root_id)
                .map(|n| (n.id.clone(), n.label.clone()))
                .collect();
            for (dir_id, dir_label) in dir_stubs {
                let comp_id = format!("component_{}", dir_id);
                if graph.nodes.iter().any(|n| n.id == comp_id) {
                    continue;
                }
                component_for_dir.insert(dir_id.clone(), comp_id.clone());
                graph.nodes.push(crate::graph::Node {
                    id: comp_id.clone(),
                    label: dir_label.clone(),
                    layer: "infra".to_string(),
                    is_partition: true,
                    belongs_to: Some(root_id.clone()),
                    weight: 1,
                    comment: Some("Inferred service from directory".to_string()),
                    dataset: None,
                    attributes: None,
                });
            }

            // Re-parent scope/file nodes into inferred components where available
            for node in graph.nodes.iter_mut() {
                if node.layer == "scope" && node.belongs_to.as_deref() == Some(&root_id) {
                    if let Some(comp) = component_for_dir
                        .iter()
                        .find(|(dir_id, _)| node.id.starts_with(*dir_id))
                        .map(|(_, comp)| comp.clone())
                    {
                        node.belongs_to = Some(comp);
                    }
                }
                if let Some(parent) = node.belongs_to.clone() {
                    if codebase_ids.contains(&parent) {
                        node.belongs_to = Some(root_id.clone());
                    }
                }
            }

            // Rewire edges away from function nodes to their owning file (or root) and drop function detail
            let parent_lookup: std::collections::HashMap<String, String> = graph
                .nodes
                .iter()
                .filter_map(|n| {
                    if n.id == root_id {
                        None
                    } else {
                        n.belongs_to
                            .clone()
                            .or_else(|| Some(root_id.clone()))
                            .map(|p| (n.id.clone(), p))
                    }
                })
                .collect();
            let function_ids: std::collections::HashSet<String> = graph
                .nodes
                .iter()
                .filter(|n| n.layer == "function")
                .map(|n| n.id.clone())
                .collect();

            let mut rewired_edges = Vec::new();
            for mut edge in graph.edges.into_iter() {
                if function_ids.contains(&edge.source) {
                    if let Some(parent) = parent_lookup.get(&edge.source) {
                        edge.source = parent.clone();
                    } else {
                        continue;
                    }
                }
                if function_ids.contains(&edge.target) {
                    if let Some(parent) = parent_lookup.get(&edge.target) {
                        edge.target = parent.clone();
                    } else {
                        continue;
                    }
                }

                // Skip intra-node self loops created by collapsing
                if edge.source == edge.target {
                    continue;
                }

                rewired_edges.push(edge);
            }
            // Re-parent anything pointing at the synthetic "Codebase" node to the solution root, then drop it
            let rewired_edges: Vec<_> = rewired_edges
                .into_iter()
                .map(|mut e| {
                    if codebase_ids.contains(&e.source) {
                        e.source = root_id.clone();
                    }
                    if codebase_ids.contains(&e.target) {
                        e.target = root_id.clone();
                    }
                    e
                })
                .filter(|e| e.source != e.target)
                .collect();

            graph.edges = rewired_edges;
            graph.nodes.retain(|n| {
                n.layer != "function" && n.layer != "library" && !codebase_ids.contains(&n.id)
            });

            for node in graph.nodes.iter_mut() {
                if node.belongs_to.is_none() {
                    node.belongs_to = Some(root_id.clone());
                }
            }

            graph.append_annotation(cleaned_report.clone());
            graph
        } else if let Some(infra_graph) = infra_graph {
            merge_graphs(
                analysis_to_graph(&result, None, opts.coalesce_functions),
                infra_to_graph(&infra_graph, None),
                Some(cleaned_report.clone()),
                correlation.as_ref(),
            )
        } else {
            analysis_to_graph(
                &result,
                Some(cleaned_report.clone()),
                opts.coalesce_functions,
            )
        };
        let graph_json = serde_json::to_string(&combined_graph).map_err(|e| {
            CoreError::internal("Failed to serialize code analysis graph").with_source(e)
        })?;
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
        active.last_result = Set(serde_json::to_string(&result).ok());

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to update code analysis profile").with_source(e))?;
        Ok(CodeAnalysisProfile::from(updated))
    }
}
