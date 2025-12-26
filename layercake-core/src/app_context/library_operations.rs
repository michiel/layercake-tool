use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Cursor, Read, Seek, Write};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use zip::{result::ZipError, write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use super::PlanDagSnapshot;
use super::{summarize_graph_counts, AppContext, ProjectArchiveFile, ProjectSummary};
use crate::database::entities::common_types::FileFormat as DataSetFileFormat;
use crate::database::entities::{
    data_sets, layer_aliases, library_items, plan_dag_edges, plan_dag_nodes, plans, project_layers,
    projects, sequences, stories,
};
use crate::services::data_set_service::DataSetService;
use crate::services::library_item_service::{
    LibraryItemService, ITEM_TYPE_PROJECT, ITEM_TYPE_PROJECT_TEMPLATE,
};
use crate::errors::{CoreError, CoreResult};
use crate::services::plan_service::PlanCreateRequest;
use layercake_genai::entities::{
    file_tags as kb_file_tags, files as kb_files, kb_documents as kb_docs, tags as kb_tags,
    vector_index_state as kb_vector_state,
};

// Private helper structs for archive/template operations
#[derive(Clone)]
struct ExportAsset {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlanIndexEntry {
    original_id: i32,
    filename: String,
    name: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PlansIndex {
    plans: Vec<PlanIndexEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedPlanMetadata {
    original_id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    yaml_content: String,
    dependencies: Option<Vec<i32>>,
    status: String,
    version: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedPlanFile {
    metadata: ExportedPlanMetadata,
    dag: PlanDagSnapshot,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedSequence {
    original_id: i32,
    name: String,
    description: Option<String>,
    enabled_dataset_ids: Vec<i32>,
    edge_order: Vec<crate::sequence_types::SequenceEdgeRef>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedStory {
    original_id: i32,
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    enabled_dataset_ids: Vec<i32>,
    layer_config: Value,
    sequences: Vec<ExportedSequence>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct StoriesExport {
    stories: Vec<ExportedStory>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedLayer {
    original_id: i32,
    layer_id: String,
    name: String,
    background_color: String,
    text_color: String,
    border_color: String,
    alias: Option<String>,
    source_dataset_id: Option<i32>,
    enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportedLayerAlias {
    alias_layer_id: String,
    target_layer_original_id: i32,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PaletteExport {
    layers: Vec<ExportedLayer>,
    aliases: Vec<ExportedLayerAlias>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseFileEntry {
    id: String,
    filename: String,
    media_type: String,
    size_bytes: i64,
    checksum: String,
    created_at: DateTime<Utc>,
    indexed: bool,
    blob_path: String,
    tag_ids: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseTagEntry {
    id: String,
    name: String,
    scope: String,
    color: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseFileTagEntry {
    file_id: String,
    tag_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseDocumentEntry {
    id: String,
    file_id: Option<String>,
    chunk_id: String,
    media_type: String,
    chunk_text: String,
    metadata: Option<Value>,
    embedding_model: Option<String>,
    embedding: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseVectorStateEntry {
    id: String,
    status: String,
    last_indexed_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    config: Option<Value>,
    embedding_provider: Option<String>,
    embedding_model: Option<String>,
    updated_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct KnowledgeBaseIndex {
    files: Vec<KnowledgeBaseFileEntry>,
    tags: Vec<KnowledgeBaseTagEntry>,
    file_tags: Vec<KnowledgeBaseFileTagEntry>,
    documents: Vec<KnowledgeBaseDocumentEntry>,
    vector_states: Vec<KnowledgeBaseVectorStateEntry>,
}

fn deserialize_i32_from_string_or_number<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Unexpected};
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(n) => n
            .as_i64()
            .and_then(|i| i32::try_from(i).ok())
            .ok_or_else(|| {
                de::Error::invalid_value(
                    Unexpected::Signed(n.as_i64().unwrap_or(0)),
                    &"a valid i32",
                )
            }),
        Value::String(s) => s.parse::<i32>().map_err(|_| {
            de::Error::invalid_value(Unexpected::Str(&s), &"a string containing a valid i32")
        }),
        _ => Err(de::Error::invalid_type(
            match value {
                Value::Bool(_) => Unexpected::Bool(false),
                Value::Null => Unexpected::Unit,
                _ => Unexpected::Other("unexpected type"),
            },
            &"a number or string representing an i32",
        )),
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetBundleDescriptor {
    #[serde(deserialize_with = "deserialize_i32_from_string_or_number")]
    pub original_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub filename: String,
    pub file_format: String,
    pub node_count: Option<usize>,
    pub edge_count: Option<usize>,
    pub layer_count: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DatasetBundleIndex {
    pub datasets: Vec<DatasetBundleDescriptor>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectBundleManifest {
    pub manifest_version: String,
    pub bundle_type: String,
    pub created_with: String,
    pub project_format_version: u32,
    pub generated_at: DateTime<Utc>,
    pub source_project_id: i32,
    pub plan_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectRecord {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

fn has_hidden_component(path: &Path) -> bool {
    path.components().any(|component| match component {
        Component::Normal(value) => value.to_string_lossy().starts_with('.'),
        Component::ParentDir | Component::RootDir | Component::Prefix(_) => true,
        Component::CurDir => false,
    })
}

fn sanitize_relative_path(path: &str) -> Result<PathBuf> {
    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(anyhow!("Invalid path component in archive entry: {}", path));
    }
    Ok(candidate.components().collect())
}

fn node_type_storage_name(
    node_type: &crate::plan_dag::PlanDagNodeType,
) -> &'static str {
    use crate::plan_dag::PlanDagNodeType;
    match node_type {
        PlanDagNodeType::DataSet => "DataSetNode",
        PlanDagNodeType::Graph => "GraphNode",
        PlanDagNodeType::Transform => "TransformNode",
        PlanDagNodeType::Filter => "FilterNode",
        PlanDagNodeType::Merge => "MergeNode",
        PlanDagNodeType::GraphArtefact => "GraphArtefactNode",
        PlanDagNodeType::TreeArtefact => "TreeArtefactNode",
        PlanDagNodeType::Projection => "ProjectionNode",
        PlanDagNodeType::Story => "StoryNode",
        PlanDagNodeType::SequenceArtefact => "SequenceArtefactNode",
    }
}

fn collect_paths_for_zip(bytes: &[u8], target_dir: &Path) -> Result<HashSet<PathBuf>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| anyhow!("Failed to read archive: {}", e))?;
    let mut written_paths = HashSet::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| anyhow!("Failed to read archive entry: {}", e))?;
        if entry.is_dir() {
            continue;
        }

        let rel_path = sanitize_relative_path(entry.name())?;
        if has_hidden_component(&rel_path) {
            continue;
        }

        let out_path = target_dir.join(&rel_path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create directory {:?}: {}", parent, e))?;
        }

        let mut outfile = fs::File::create(&out_path)
            .map_err(|e| anyhow!("Failed to create file {:?}: {}", out_path, e))?;
        std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| anyhow!("Failed to write {:?}: {}", out_path, e))?;

        written_paths.insert(rel_path);
    }

    Ok(written_paths)
}

fn remove_stale_exports(root: &Path, expected: &HashSet<PathBuf>) -> Result<()> {
    let mut allowed_heads: HashSet<String> = HashSet::new();
    for path in expected {
        if let Some(Component::Normal(head)) = path.components().next() {
            allowed_heads.insert(head.to_string_lossy().to_string());
        }
    }

    fn walk(
        root: &Path,
        current: &Path,
        expected: &HashSet<PathBuf>,
        allowed_heads: &HashSet<String>,
    ) -> Result<bool> {
        let mut is_empty = true;
        for entry in fs::read_dir(current)
            .map_err(|e| anyhow!("Failed to read directory {:?}: {}", current, e))?
        {
            let entry = entry.map_err(|e| anyhow!("Failed to read dir entry: {}", e))?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|e| anyhow!("Failed to compute relative path: {}", e))?;
            if has_hidden_component(rel) {
                is_empty = false;
                continue;
            }

            if path.is_dir() {
                let child_empty = walk(root, &path, expected, allowed_heads)?;
                if child_empty {
                    fs::remove_dir(&path)
                        .map_err(|e| anyhow!("Failed to remove directory {:?}: {}", path, e))?;
                } else {
                    is_empty = false;
                }
                continue;
            }

            let head_ok = rel
                .components()
                .next()
                .and_then(|c| match c {
                    Component::Normal(val) => Some(val.to_string_lossy().to_string()),
                    _ => None,
                })
                .map(|head| allowed_heads.contains(&head))
                .unwrap_or(false);

            if !expected.contains(rel) && head_ok {
                fs::remove_file(&path)
                    .map_err(|e| anyhow!("Failed to remove file {:?}: {}", path, e))?;
            } else {
                is_empty = false;
            }
        }

        Ok(is_empty)
    }

    let _ = walk(root, root, expected, &allowed_heads)?;
    Ok(())
}

fn write_archive_to_directory(bytes: &[u8], target_dir: &Path) -> Result<()> {
    let expected = collect_paths_for_zip(bytes, target_dir)?;
    remove_stale_exports(target_dir, &expected)?;
    Ok(())
}

fn archive_directory(source_dir: &Path) -> Result<Vec<u8>> {
    if !source_dir.exists() {
        return Err(anyhow!(
            "Import source directory {:?} does not exist",
            source_dir
        ));
    }
    let mut files: Vec<PathBuf> = Vec::new();

    fn collect(dir: &Path, root: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
        for entry in
            fs::read_dir(dir).map_err(|e| anyhow!("Failed to read directory {:?}: {}", dir, e))?
        {
            let entry = entry.map_err(|e| anyhow!("Failed to read dir entry: {}", e))?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|e| anyhow!("Failed to compute relative path: {}", e))?;
            if has_hidden_component(rel) {
                continue;
            }
            if path.is_dir() {
                collect(&path, root, acc)?;
            } else {
                acc.push(rel.to_path_buf());
            }
        }
        Ok(())
    }

    collect(source_dir, source_dir, &mut files)?;
    files.sort();

    let mut cursor = Cursor::new(Vec::new());
    {
        let mut writer = ZipWriter::new(&mut cursor);
        let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

        for rel in files {
            let full_path = source_dir.join(&rel);
            let mut file = fs::File::open(&full_path)
                .map_err(|e| anyhow!("Failed to open {:?}: {}", full_path, e))?;
            let rel_string = rel.to_string_lossy().replace('\\', "/");
            writer
                .start_file(rel_string, options)
                .map_err(|e| anyhow!("Failed to start archive entry {:?}: {}", rel, e))?;
            std::io::copy(&mut file, &mut writer)
                .map_err(|e| anyhow!("Failed to write {:?}: {}", rel, e))?;
        }

        writer
            .finish()
            .map_err(|e| anyhow!("Failed to finalize archive: {}", e))?;
    }
    Ok(cursor.into_inner())
}

impl AppContext {
    pub async fn export_project_as_template(
        &self,
        project_id: i32,
    ) -> CoreResult<library_items::Model> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project {}: {}", project_id, e))
            })?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to load plan for project {}: {}",
                    project_id, e
                ))
            })?
            .ok_or_else(|| CoreError::not_found("Plan", project_id.to_string()))?;

        let snapshot = self
            .load_plan_dag(project_id, None)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load plan DAG: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDag", project_id.to_string()))?;

        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load data sets for template: {}", e))
            })?;

        let (dataset_records, dataset_graphs) = analyze_data_sets(&data_sets)
            .map_err(|e| CoreError::internal(format!("Failed to analyze data sets: {}", e)))?;
        let dataset_index = DatasetBundleIndex {
            datasets: dataset_records.clone(),
        };

        let manifest = ProjectBundleManifest {
            manifest_version: "1.0".to_string(),
            bundle_type: ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
            created_with: format!("layercake-{}", env!("CARGO_PKG_VERSION")),
            project_format_version: 1,
            generated_at: chrono::Utc::now(),
            source_project_id: project.id,
            plan_name: plan.name.clone(),
        };

        let project_record = ProjectRecord {
            name: project.name.clone(),
            description: project.description.clone(),
            tags: serde_json::from_str(&project.tags).unwrap_or_default(),
        };

        let dag_bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| CoreError::internal(format!("Failed to encode DAG snapshot: {}", e)))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| CoreError::internal(format!("Failed to encode template manifest: {}", e)))?;
        let project_bytes = serde_json::to_vec_pretty(&project_record)
            .map_err(|e| CoreError::internal(format!("Failed to encode project metadata: {}", e)))?;
        let dataset_index_bytes = serde_json::to_vec_pretty(&dataset_index)
            .map_err(|e| CoreError::internal(format!("Failed to encode dataset index: {}", e)))?;
        let metadata_bytes = serde_json::to_vec_pretty(&json!({
            "layercakeProjectFormatVersion": 1
        }))
        .map_err(|e| CoreError::internal(format!("Failed to encode metadata.json: {}", e)))?;

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            write_bundle_common_files(
                &mut zip,
                &manifest_bytes,
                &metadata_bytes,
                &project_bytes,
                &dag_bytes,
                &dataset_index_bytes,
            )
            .map_err(|e| CoreError::internal(format!("Failed to write bundle files: {}", e)))?;
            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

            for descriptor in &dataset_records {
                if let Some(graph_json) = dataset_graphs.get(&descriptor.original_id) {
                    let path = format!("datasets/{}", descriptor.filename);
                    zip.start_file(path, options)
                        .map_err(|e| {
                            CoreError::internal(format!("Failed to add dataset file: {}", e))
                        })?;
                    zip.write_all(graph_json.as_bytes())
                        .map_err(|e| {
                            CoreError::internal(format!("Failed to write dataset file: {}", e))
                        })?;
                }
            }

            zip.finish()
                .map_err(|e| {
                    CoreError::internal(format!("Failed to finalize template archive: {}", e))
                })?;
        }

        let zip_bytes = cursor.into_inner();
        let service = LibraryItemService::new(self.db.clone());
        let tags = serde_json::from_str(&project.tags).unwrap_or_default();

        let metadata = json!({
            "projectId": project.id,
            "planId": plan.id,
            "nodeCount": snapshot.nodes.len(),
            "edgeCount": snapshot.edges.len(),
            "datasetCount": dataset_records.len(),
            "manifestVersion": manifest.manifest_version,
            "projectFormatVersion": manifest.project_format_version
        });

        let item = service
            .create_binary_item(
                ITEM_TYPE_PROJECT_TEMPLATE.to_string(),
                format!("{} Template", project.name),
                project.description.clone(),
                tags,
                metadata,
                Some("application/zip".to_string()),
                zip_bytes,
            )
            .await
            .map_err(|e| CoreError::internal(format!("Failed to persist template: {}", e)))?;

        Ok(item)
    }

    pub async fn export_project_archive(
        &self,
        project_id: i32,
        include_knowledge_base: bool,
    ) -> CoreResult<ProjectArchiveFile> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project {}: {}", project_id, e))
            })?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to load plan for project {}: {}",
                    project_id, e
                ))
            })?
            .ok_or_else(|| CoreError::not_found("Plan", project_id.to_string()))?;

        let snapshot = self
            .load_plan_dag(project_id, None)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load plan DAG: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDag", project_id.to_string()))?;

        let data_sets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load data sets for export: {}", e))
            })?;

        let (dataset_records, dataset_graphs) = analyze_data_sets(&data_sets)
            .map_err(|e| CoreError::internal(format!("Failed to analyze data sets: {}", e)))?;
        let dataset_index = DatasetBundleIndex {
            datasets: dataset_records.clone(),
        };

        let manifest = ProjectBundleManifest {
            manifest_version: "1.0".to_string(),
            bundle_type: "project_archive".to_string(),
            created_with: format!("layercake-{}", env!("CARGO_PKG_VERSION")),
            project_format_version: 1,
            generated_at: chrono::Utc::now(),
            source_project_id: project.id,
            plan_name: plan.name.clone(),
        };

        let project_record = ProjectRecord {
            name: project.name.clone(),
            description: project.description.clone(),
            tags: serde_json::from_str(&project.tags).unwrap_or_default(),
        };

        let dag_bytes = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| CoreError::internal(format!("Failed to encode DAG snapshot: {}", e)))?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| {
                CoreError::internal(format!("Failed to encode project export manifest: {}", e))
            })?;
        let project_bytes = serde_json::to_vec_pretty(&project_record)
            .map_err(|e| CoreError::internal(format!("Failed to encode project metadata: {}", e)))?;
        let dataset_index_bytes = serde_json::to_vec_pretty(&dataset_index)
            .map_err(|e| CoreError::internal(format!("Failed to encode dataset index: {}", e)))?;
        let metadata_bytes = serde_json::to_vec_pretty(&json!({
            "layercakeProjectFormatVersion": 1
        }))
        .map_err(|e| CoreError::internal(format!("Failed to encode metadata.json: {}", e)))?;

        let mut additional_assets = self
            .collect_plan_assets(project_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to collect plan assets: {}", e)))?;
        if let Some(asset) = self
            .collect_stories_asset(project_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to collect stories asset: {}", e)))?
        {
            additional_assets.push(asset);
        }
        if let Some(asset) = self
            .collect_palette_asset(project_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to collect palette asset: {}", e)))?
        {
            additional_assets.push(asset);
        }
        if include_knowledge_base {
            if let Some((index_asset, mut file_assets)) =
                self.collect_knowledge_base_assets(project_id)
                    .await
                    .map_err(|e| {
                        CoreError::internal(format!("Failed to collect knowledge base assets: {}", e))
                    })?
            {
                additional_assets.push(index_asset);
                additional_assets.append(&mut file_assets);
            }
        }

        let mut cursor = Cursor::new(Vec::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            write_bundle_common_files(
                &mut zip,
                &manifest_bytes,
                &metadata_bytes,
                &project_bytes,
                &dag_bytes,
                &dataset_index_bytes,
            )
            .map_err(|e| CoreError::internal(format!("Failed to write bundle files: {}", e)))?;

            let options = FileOptions::default().compression_method(CompressionMethod::Deflated);
            for descriptor in &dataset_records {
                if let Some(graph_json) = dataset_graphs.get(&descriptor.original_id) {
                    let path = format!("datasets/{}", descriptor.filename);
                    zip.start_file(path, options)
                        .map_err(|e| {
                            CoreError::internal(format!("Failed to add dataset file: {}", e))
                        })?;
                    zip.write_all(graph_json.as_bytes())
                        .map_err(|e| {
                            CoreError::internal(format!("Failed to write dataset file: {}", e))
                        })?;
                }
            }

            for asset in additional_assets {
                zip.start_file(asset.path.clone(), options)
                    .map_err(|e| {
                        CoreError::internal(format!("Failed to add {}: {}", asset.path, e))
                    })?;
                zip.write_all(&asset.bytes)
                    .map_err(|e| {
                        CoreError::internal(format!("Failed to write {}: {}", asset.path, e))
                    })?;
            }

            zip.finish()
                .map_err(|e| {
                    CoreError::internal(format!("Failed to finalize project archive: {}", e))
                })?;
        }

        let filename = format!(
            "{}-project-export.zip",
            sanitize_dataset_filename(&project.name)
        );

        Ok(ProjectArchiveFile {
            filename,
            bytes: cursor.into_inner(),
        })
    }

    pub async fn export_project_to_directory(
        &self,
        project_id: i32,
        target_path: &Path,
        include_knowledge_base: bool,
        keep_connection: bool,
    ) -> CoreResult<()> {
        if !target_path.is_absolute() {
            return Err(CoreError::validation(format!(
                "Export target must be an absolute path, got {:?}",
                target_path
            )));
        }

        fs::create_dir_all(target_path).map_err(|e| {
            CoreError::internal(format!(
                "Failed to ensure export directory {:?} exists: {}",
                target_path, e
            ))
        })?;

        let archive = self
            .export_project_archive(project_id, include_knowledge_base)
            .await?;
        write_archive_to_directory(&archive.bytes, target_path)
            .map_err(|e| CoreError::internal(format!("Failed to write archive: {}", e)))?;

        if keep_connection {
            let path_string = target_path
                .to_str()
                .ok_or_else(|| anyhow!("Failed to serialize export path"))?
                .to_string();
            let update =
                super::ProjectUpdate::new(None, None, false, None, Some(Some(path_string)));
            let _ = self
                .update_project(&crate::auth::SystemActor::internal(), project_id, update)
                .await?;
        }

        Ok(())
    }

    pub async fn import_project_archive(
        &self,
        archive_bytes: Vec<u8>,
        project_name: Option<String>,
    ) -> CoreResult<ProjectSummary> {
        self.import_project_archive_internal(archive_bytes, project_name, None, None)
            .await
    }

    async fn import_project_archive_internal(
        &self,
        archive_bytes: Vec<u8>,
        project_name: Option<String>,
        target_project_id: Option<i32>,
        keep_connection_path: Option<String>,
    ) -> CoreResult<ProjectSummary> {
        let mut archive = ZipArchive::new(Cursor::new(archive_bytes))
            .map_err(|e| CoreError::internal(format!("Failed to read project archive: {}", e)))?;

        let manifest: ProjectBundleManifest =
            read_template_json(&mut archive, "manifest.json")
                .map_err(|e| CoreError::internal(format!("Failed to read manifest.json: {}", e)))?;
        let project_record: ProjectRecord = read_template_json(&mut archive, "project.json")
            .map_err(|e| CoreError::internal(format!("Failed to read project.json: {}", e)))?;
        let dag_snapshot: PlanDagSnapshot = read_template_json(&mut archive, "dag.json")
            .map_err(|e| CoreError::internal(format!("Failed to read dag.json: {}", e)))?;
        let dataset_index: DatasetBundleIndex =
            read_template_json(&mut archive, "datasets/index.json")
                .map_err(|e| CoreError::internal(format!("Failed to read datasets index: {}", e)))?;
        let plans_index: Option<PlansIndex> =
            try_read_template_json(&mut archive, "plans/index.json")
                .map_err(|e| CoreError::internal(format!("Failed to read plans index: {}", e)))?;
        let stories_export: Option<StoriesExport> =
            try_read_template_json(&mut archive, "stories/stories.json").map_err(|e| {
                CoreError::internal(format!("Failed to read stories export: {}", e))
            })?;
        let palette_export: Option<PaletteExport> =
            try_read_template_json(&mut archive, "layers/palette.json").map_err(|e| {
                CoreError::internal(format!("Failed to read palette export: {}", e))
            })?;
        let knowledge_base_index: Option<KnowledgeBaseIndex> =
            try_read_template_json(&mut archive, "kb/index.json")
                .map_err(|e| CoreError::internal(format!("Failed to read KB index: {}", e)))?;
        let mut kb_file_blobs: HashMap<String, Vec<u8>> = HashMap::new();
        if let Some(index) = &knowledge_base_index {
            for file_entry in &index.files {
                let bytes = read_zip_file_bytes(&mut archive, &file_entry.blob_path).map_err(
                    |e| CoreError::internal(format!("Failed to read KB blob: {}", e)),
                )?;
                kb_file_blobs.insert(file_entry.id.clone(), bytes);
            }
        }

        let tags = project_record.tags.clone();
        let desired_name = project_name.unwrap_or(project_record.name.clone());
        let now = Utc::now();

        let project_model = projects::ActiveModel {
            id: target_project_id.map(Set).unwrap_or(NotSet),
            name: Set(desired_name.clone()),
            description: Set(project_record.description.clone()),
            tags: Set(serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())),
            import_export_path: Set(keep_connection_path.clone()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let project = project_model
            .insert(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to create project from archive: {}", e))
            })?;

        let dataset_service = DataSetService::new(self.db.clone());
        let mut id_map = HashMap::new();

        for descriptor in &dataset_index.datasets {
            let dataset_path = format!("datasets/{}", descriptor.filename);
            let file_bytes = {
                let mut dataset_file = archive
                    .by_name(&dataset_path)
                    .map_err(|e| {
                        CoreError::internal(format!(
                            "Missing dataset file {}: {}",
                            descriptor.filename, e
                        ))
                    })?;
                let mut bytes = Vec::new();
                dataset_file.read_to_end(&mut bytes).map_err(|e| {
                    CoreError::internal(format!(
                        "Failed to read dataset {}: {}",
                        descriptor.filename, e
                    ))
                })?;
                bytes
            };
            let file_format = DataSetFileFormat::from_str(&descriptor.file_format)
                .unwrap_or(DataSetFileFormat::Json);

            let dataset = dataset_service
                .create_from_file(
                    project.id,
                    descriptor.name.clone(),
                    descriptor.description.clone(),
                    descriptor.filename.clone(),
                    file_format,
                    file_bytes,
                    None,
                )
                .await
                .map_err(|e| {
                    CoreError::internal(format!(
                        "Failed to import dataset {}: {}",
                        descriptor.name, e
                    ))
                })?;

            id_map.insert(descriptor.original_id, dataset.id);
        }

        if let Some(palette) = palette_export {
            self.import_palette_from_export(project.id, palette, &id_map)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to import palette: {}", e)))?;
        }

        if let Some(index) = plans_index {
            self.import_plans_from_export(project.id, index, &mut archive, &id_map)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to import plans: {}", e)))?;
        } else {
            let plan = self
                .create_plan(&crate::auth::SystemActor::internal(), PlanCreateRequest {
                    project_id: project.id,
                    name: if manifest.plan_name.trim().is_empty() {
                        format!("{} Plan", desired_name)
                    } else {
                        manifest.plan_name.clone()
                    },
                    description: None,
                    tags: Some(vec![]),
                    yaml_content: "".to_string(),
                    dependencies: None,
                    status: Some("draft".to_string()),
                })
                .await?;

            insert_plan_dag_from_snapshot(&self.db, plan.id, &dag_snapshot, &id_map)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to recreate plan DAG: {}", e))
                })?;
        }

        if let Some(story_bundle) = stories_export {
            self.import_stories_from_export(project.id, story_bundle, &id_map)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to import stories: {}", e)))?;
        }

        if let Some(kb_index) = knowledge_base_index {
            self.import_knowledge_base_from_export(project.id, kb_index, kb_file_blobs)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to import knowledge base: {}", e)))?;
        }

        Ok(ProjectSummary::from(project))
    }

    pub async fn import_project_from_directory(
        &self,
        source_path: &Path,
        project_name: Option<String>,
        keep_connection: bool,
    ) -> CoreResult<ProjectSummary> {
        if !source_path.is_absolute() {
            return Err(CoreError::validation(format!(
                "Import source must be an absolute path, got {:?}",
                source_path
            )));
        }

        let archive_bytes = archive_directory(source_path)
            .map_err(|e| CoreError::internal(format!("Failed to archive directory: {}", e)))?;
        let path_string = if keep_connection {
            Some(
                source_path
                    .to_str()
                    .ok_or_else(|| {
                        CoreError::internal("Failed to serialize import path".to_string())
                    })?
                    .to_string(),
            )
        } else {
            None
        };

        self.import_project_archive_internal(archive_bytes, project_name, None, path_string)
            .await
    }

    pub async fn reimport_project_from_connection(
        &self,
        project_id: i32,
    ) -> CoreResult<ProjectSummary> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project {}: {}", project_id, e))
            })?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let path = project
            .import_export_path
            .clone()
            .ok_or_else(|| {
                CoreError::validation(format!(
                    "Project {} has no connected import/export path",
                    project_id
                ))
            })?;

        // Remove the existing project and re-import using the same ID
        self.delete_project(&crate::auth::SystemActor::internal(), project_id)
            .await?;

        let archive_bytes = archive_directory(Path::new(&path))
            .map_err(|e| CoreError::internal(format!("Failed to archive directory: {}", e)))?;
        self.import_project_archive_internal(
            archive_bytes,
            Some(project.name),
            Some(project_id),
            Some(path),
        )
        .await
    }

    pub async fn reexport_project_to_connection(
        &self,
        project_id: i32,
        include_knowledge_base: bool,
    ) -> CoreResult<()> {
        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project {}: {}", project_id, e))
            })?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let path = project
            .import_export_path
            .clone()
            .ok_or_else(|| {
                CoreError::validation(format!(
                    "Project {} has no connected import/export path",
                    project_id
                ))
            })?;

        self.export_project_to_directory(
            project_id,
            Path::new(&path),
            include_knowledge_base,
            false,
        )
        .await
    }

    pub async fn create_project_from_library(
        &self,
        library_item_id: i32,
        project_name: Option<String>,
    ) -> CoreResult<ProjectSummary> {
        let service = LibraryItemService::new(self.db.clone());
        let item = service
            .get(library_item_id)
            .await
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to load library item {}: {}",
                    library_item_id, e
                ))
            })?
            .ok_or_else(|| CoreError::not_found("LibraryItem", library_item_id.to_string()))?;

        if item.item_type != ITEM_TYPE_PROJECT && item.item_type != ITEM_TYPE_PROJECT_TEMPLATE {
            return Err(CoreError::validation(format!(
                "Library item {} is type {}, expected project or project_template",
                library_item_id, item.item_type
            )));
        }

        let mut archive = ZipArchive::new(Cursor::new(item.content_blob.clone())).map_err(|e| {
            CoreError::internal(format!(
                "Failed to read template archive for library item {}: {}",
                library_item_id, e
            ))
        })?;

        let manifest: ProjectBundleManifest = read_template_json(&mut archive, "manifest.json")
            .map_err(|e| CoreError::internal(format!("Failed to read manifest.json: {}", e)))?;
        let project_record: ProjectRecord = read_template_json(&mut archive, "project.json")
            .map_err(|e| CoreError::internal(format!("Failed to read project.json: {}", e)))?;
        let dag_snapshot: PlanDagSnapshot = read_template_json(&mut archive, "dag.json")
            .map_err(|e| CoreError::internal(format!("Failed to read dag.json: {}", e)))?;
        let dataset_index: DatasetBundleIndex =
            read_template_json(&mut archive, "datasets/index.json").map_err(|e| {
                CoreError::internal(format!("Failed to read datasets index: {}", e))
            })?;

        let tags = project_record.tags.clone();
        let desired_name = project_name.unwrap_or(project_record.name.clone());
        let now = Utc::now();

        let project = projects::ActiveModel {
            name: Set(desired_name.clone()),
            description: Set(project_record.description.clone()),
            tags: Set(serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| {
            CoreError::internal(format!("Failed to create project from template: {}", e))
        })?;

        let plan = self
            .create_plan(&crate::auth::SystemActor::internal(), PlanCreateRequest {
                project_id: project.id,
                name: if manifest.plan_name.trim().is_empty() {
                    format!("{} Plan", desired_name)
                } else {
                    manifest.plan_name.clone()
                },
                description: None,
                tags: Some(vec![]),
                yaml_content: "".to_string(),
                dependencies: None,
                status: Some("draft".to_string()),
            })
            .await?;

        let dataset_service = DataSetService::new(self.db.clone());
        let mut id_map = HashMap::new();

        let is_template = item.item_type == ITEM_TYPE_PROJECT_TEMPLATE;

        for descriptor in &dataset_index.datasets {
            let dataset = if is_template {
                // Templates should not carry data rows forward; create empty datasets using the schema metadata.
                dataset_service
                    .create_empty(
                        project.id,
                        descriptor.name.clone(),
                        descriptor.description.clone(),
                    )
                    .await
            } else {
                let dataset_path = format!("datasets/{}", descriptor.filename);
                let file_bytes = {
                    let mut dataset_file = archive.by_name(&dataset_path).map_err(|e| {
                        CoreError::internal(format!(
                            "Missing dataset file {}: {}",
                            descriptor.filename, e
                        ))
                    })?;
                    let mut bytes = Vec::new();
                    dataset_file.read_to_end(&mut bytes).map_err(|e| {
                        CoreError::internal(format!(
                            "Failed to read dataset {}: {}",
                            descriptor.filename, e
                        ))
                    })?;
                    bytes
                };
                let file_format = DataSetFileFormat::from_str(&descriptor.file_format)
                    .unwrap_or(DataSetFileFormat::Json);

                dataset_service
                    .create_from_file(
                        project.id,
                        descriptor.name.clone(),
                        descriptor.description.clone(),
                        descriptor.filename.clone(),
                        file_format,
                        file_bytes,
                        None,
                    )
                    .await
            }
            .map_err(|e| {
                CoreError::internal(format!(
                    "Failed to import dataset {}: {}",
                    descriptor.name, e
                ))
            })?;

            id_map.insert(descriptor.original_id, dataset.id);
        }

        insert_plan_dag_from_snapshot(&self.db, plan.id, &dag_snapshot, &id_map)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to recreate plan DAG: {}", e)))?;

        Ok(ProjectSummary::from(project))
    }

    async fn collect_plan_assets(&self, project_id: i32) -> Result<Vec<ExportAsset>> {
        let plans = self
            .plan_service
            .list_plans(project_id)
            .await
            .map_err(|e| anyhow!("Failed to list plans: {}", e))?;
        if plans.is_empty() {
            return Ok(Vec::new());
        }

        let mut assets = Vec::new();
        let mut index = PlansIndex::default();

        for plan in plans {
            if let Some(snapshot) = self
                .load_plan_dag(project_id, Some(plan.id))
                .await
                .map_err(|e| anyhow!("Failed to load plan DAG: {}", e))?
            {
                let filename = format!("plan_{}.json", plan.id);
                let metadata = ExportedPlanMetadata {
                    original_id: plan.id,
                    name: plan.name.clone(),
                    description: plan.description.clone(),
                    tags: serde_json::from_str(&plan.tags).unwrap_or_default(),
                    yaml_content: plan.yaml_content.clone(),
                    dependencies: plan
                        .dependencies
                        .as_ref()
                        .and_then(|value| serde_json::from_str::<Vec<i32>>(value).ok()),
                    status: plan.status.clone(),
                    version: plan.version,
                    created_at: plan.created_at,
                    updated_at: plan.updated_at,
                };
                let payload = ExportedPlanFile {
                    metadata,
                    dag: snapshot,
                };
                let bytes = serde_json::to_vec_pretty(&payload)
                    .map_err(|e| anyhow!("Failed to encode plan {}: {}", plan.id, e))?;
                assets.push(ExportAsset {
                    path: format!("plans/{}", filename),
                    bytes,
                });
                index.plans.push(PlanIndexEntry {
                    original_id: plan.id,
                    filename,
                    name: plan.name,
                });
            }
        }

        if !index.plans.is_empty() {
            let bytes = serde_json::to_vec_pretty(&index)
                .map_err(|e| anyhow!("Failed to encode plan index: {}", e))?;
            assets.push(ExportAsset {
                path: "plans/index.json".into(),
                bytes,
            });
        }

        Ok(assets)
    }

    async fn collect_stories_asset(&self, project_id: i32) -> Result<Option<ExportAsset>> {
        let stories = stories::Entity::find()
            .filter(stories::Column::ProjectId.eq(project_id))
            .order_by_asc(stories::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load stories for export: {}", e))?;

        if stories.is_empty() {
            return Ok(None);
        }

        let story_ids: Vec<i32> = stories.iter().map(|s| s.id).collect();
        let sequence_rows = if story_ids.is_empty() {
            Vec::new()
        } else {
            sequences::Entity::find()
                .filter(sequences::Column::StoryId.is_in(story_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load sequences for export: {}", e))?
        };

        let mut sequence_map: HashMap<i32, Vec<sequences::Model>> = HashMap::new();
        for sequence in sequence_rows {
            sequence_map
                .entry(sequence.story_id)
                .or_default()
                .push(sequence);
        }

        let mut payload = StoriesExport::default();
        for story in stories {
            let tags = serde_json::from_str(&story.tags).unwrap_or_default();
            let enabled_dataset_ids =
                serde_json::from_str(&story.enabled_dataset_ids).unwrap_or_default();
            let layer_config = serde_json::from_str(&story.layer_config).unwrap_or(Value::Null);

            let sequences = sequence_map
                .remove(&story.id)
                .unwrap_or_default()
                .into_iter()
                .map(|sequence| {
                    let enabled_sequence_dataset_ids =
                        serde_json::from_str(&sequence.enabled_dataset_ids).unwrap_or_default();
                    let edge_order = serde_json::from_str(&sequence.edge_order).unwrap_or_default();

                    ExportedSequence {
                        original_id: sequence.id,
                        name: sequence.name,
                        description: sequence.description,
                        enabled_dataset_ids: enabled_sequence_dataset_ids,
                        edge_order,
                        created_at: sequence.created_at,
                        updated_at: sequence.updated_at,
                    }
                })
                .collect();

            payload.stories.push(ExportedStory {
                original_id: story.id,
                name: story.name,
                description: story.description,
                tags,
                enabled_dataset_ids,
                layer_config,
                sequences,
                created_at: story.created_at,
                updated_at: story.updated_at,
            });
        }

        let bytes = serde_json::to_vec_pretty(&payload)
            .map_err(|e| anyhow!("Failed to encode stories export: {}", e))?;
        Ok(Some(ExportAsset {
            path: "stories/stories.json".into(),
            bytes,
        }))
    }

    async fn collect_palette_asset(&self, project_id: i32) -> Result<Option<ExportAsset>> {
        let layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .order_by_asc(project_layers::Column::Id)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load project layers: {}", e))?;

        if layers.is_empty() {
            return Ok(None);
        }

        let aliases = layer_aliases::Entity::find()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load layer aliases: {}", e))?;

        let export = PaletteExport {
            layers: layers
                .into_iter()
                .map(|layer| ExportedLayer {
                    original_id: layer.id,
                    layer_id: layer.layer_id,
                    name: layer.name,
                    background_color: layer.background_color,
                    text_color: layer.text_color,
                    border_color: layer.border_color,
                    alias: layer.alias,
                    source_dataset_id: layer.source_dataset_id,
                    enabled: layer.enabled,
                    created_at: layer.created_at,
                    updated_at: layer.updated_at,
                })
                .collect(),
            aliases: aliases
                .into_iter()
                .map(|alias| ExportedLayerAlias {
                    alias_layer_id: alias.alias_layer_id,
                    target_layer_original_id: alias.target_layer_id,
                })
                .collect(),
        };

        let bytes = serde_json::to_vec_pretty(&export)
            .map_err(|e| anyhow!("Failed to encode layer palette export: {}", e))?;
        Ok(Some(ExportAsset {
            path: "layers/palette.json".into(),
            bytes,
        }))
    }

    async fn collect_knowledge_base_assets(
        &self,
        project_id: i32,
    ) -> Result<Option<(ExportAsset, Vec<ExportAsset>)>> {
        let files = kb_files::Entity::find()
            .filter(kb_files::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base files: {}", e))?;

        if files.is_empty() {
            return Ok(None);
        }

        let file_ids: Vec<Uuid> = files.iter().map(|file| file.id).collect();
        let file_tags = if file_ids.is_empty() {
            Vec::new()
        } else {
            kb_file_tags::Entity::find()
                .filter(kb_file_tags::Column::FileId.is_in(file_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load file tags: {}", e))?
        };

        let tag_ids: Vec<Uuid> = file_tags.iter().map(|row| row.tag_id).collect();
        let tags = if tag_ids.is_empty() {
            Vec::new()
        } else {
            kb_tags::Entity::find()
                .filter(kb_tags::Column::Id.is_in(tag_ids.clone()))
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to load tags: {}", e))?
        };

        let documents = kb_docs::Entity::find()
            .filter(kb_docs::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base documents: {}", e))?;

        let vector_states = kb_vector_state::Entity::find()
            .filter(kb_vector_state::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load knowledge base vector state: {}", e))?;

        let mut tag_lookup = HashMap::new();
        for tag in tags {
            tag_lookup.insert(
                tag.id,
                KnowledgeBaseTagEntry {
                    id: tag.id.to_string(),
                    name: tag.name,
                    scope: tag.scope,
                    color: tag.color,
                    created_at: tag.created_at,
                },
            );
        }

        let mut file_tag_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        let mut file_tag_entries = Vec::new();
        for link in file_tags {
            file_tag_map
                .entry(link.file_id)
                .or_default()
                .push(link.tag_id.to_string());
            file_tag_entries.push(KnowledgeBaseFileTagEntry {
                file_id: link.file_id.to_string(),
                tag_id: link.tag_id.to_string(),
            });
        }

        let mut index = KnowledgeBaseIndex::default();
        index.tags = tag_lookup.values().cloned().collect();
        index.file_tags = file_tag_entries;

        let mut binary_assets = Vec::new();
        for file in files {
            let sanitized_name = sanitize_dataset_filename(&file.filename);
            let blob_path = format!("kb/files/{}/{}", file.id, sanitized_name);
            binary_assets.push(ExportAsset {
                path: blob_path.clone(),
                bytes: file.blob.clone(),
            });

            index.files.push(KnowledgeBaseFileEntry {
                id: file.id.to_string(),
                filename: file.filename,
                media_type: file.media_type,
                size_bytes: file.size_bytes,
                checksum: file.checksum,
                created_at: file.created_at,
                indexed: file.indexed,
                blob_path,
                tag_ids: file_tag_map.remove(&file.id).unwrap_or_default(),
            });
        }

        index.documents = documents
            .into_iter()
            .map(|doc| KnowledgeBaseDocumentEntry {
                id: doc.id.to_string(),
                file_id: doc.file_id.map(|id| id.to_string()),
                chunk_id: doc.chunk_id,
                media_type: doc.media_type,
                chunk_text: doc.chunk_text,
                metadata: doc.metadata,
                embedding_model: doc.embedding_model,
                embedding: doc
                    .embedding
                    .map(|bytes| general_purpose::STANDARD.encode(bytes)),
                created_at: doc.created_at,
            })
            .collect();

        index.vector_states = vector_states
            .into_iter()
            .map(|state| KnowledgeBaseVectorStateEntry {
                id: state.id.to_string(),
                status: state.status,
                last_indexed_at: state.last_indexed_at,
                last_error: state.last_error,
                config: state.config,
                embedding_provider: state.embedding_provider,
                embedding_model: state.embedding_model,
                updated_at: state.updated_at,
            })
            .collect();

        let index_bytes = serde_json::to_vec_pretty(&index)
            .map_err(|e| anyhow!("Failed to encode knowledge base index: {}", e))?;
        let index_asset = ExportAsset {
            path: "kb/index.json".into(),
            bytes: index_bytes,
        };

        Ok(Some((index_asset, binary_assets)))
    }

    async fn import_plans_from_export(
        &self,
        project_id: i32,
        plans_index: PlansIndex,
        archive: &mut ZipArchive<Cursor<Vec<u8>>>,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if plans_index.plans.is_empty() {
            return Ok(());
        }

        for entry in plans_index.plans {
            let path = format!("plans/{}", entry.filename);
            let plan_file: ExportedPlanFile = read_template_json(archive, &path)?;
            let plan = self
                .create_plan(&crate::auth::SystemActor::internal(), PlanCreateRequest {
                    project_id,
                    name: plan_file.metadata.name.clone(),
                    description: plan_file.metadata.description.clone(),
                    tags: Some(plan_file.metadata.tags.clone()),
                    yaml_content: plan_file.metadata.yaml_content.clone(),
                    dependencies: plan_file.metadata.dependencies.clone(),
                    status: Some(plan_file.metadata.status.clone()),
                })
                .await?;

            insert_plan_dag_from_snapshot(&self.db, plan.id, &plan_file.dag, dataset_map)
                .await
                .map_err(|e| anyhow!("Failed to recreate plan {} DAG: {}", plan.id, e))?;
        }

        Ok(())
    }

    async fn import_stories_from_export(
        &self,
        project_id: i32,
        export: StoriesExport,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if export.stories.is_empty() {
            return Ok(());
        }

        for story in export.stories {
            let mapped_dataset_ids: Vec<i32> = story
                .enabled_dataset_ids
                .into_iter()
                .filter_map(|id| dataset_map.get(&id).copied())
                .collect();

            let story_model = stories::ActiveModel {
                id: NotSet,
                project_id: Set(project_id),
                name: Set(story.name),
                description: Set(story.description),
                tags: Set(serde_json::to_string(&story.tags).unwrap_or_else(|_| "[]".into())),
                enabled_dataset_ids: Set(
                    serde_json::to_string(&mapped_dataset_ids).unwrap_or_else(|_| "[]".into())
                ),
                layer_config: Set(
                    serde_json::to_string(&story.layer_config).unwrap_or_else(|_| "{}".into())
                ),
                created_at: Set(story.created_at),
                updated_at: Set(story.updated_at),
            }
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to insert story: {}", e))?;

            for sequence in story.sequences {
                let mapped_sequence_ids: Vec<i32> = sequence
                    .enabled_dataset_ids
                    .into_iter()
                    .filter_map(|id| dataset_map.get(&id).copied())
                    .collect();
                let mapped_edge_order: Vec<_> = sequence
                    .edge_order
                    .into_iter()
                    .filter_map(|mut ref_entry| {
                        dataset_map
                            .get(&ref_entry.dataset_id)
                            .copied()
                            .map(|new_id| {
                                ref_entry.dataset_id = new_id;
                                ref_entry
                            })
                    })
                    .collect();

                let sequence_model =
                    sequences::ActiveModel {
                        id: NotSet,
                        story_id: Set(story_model.id),
                        name: Set(sequence.name),
                        description: Set(sequence.description),
                        enabled_dataset_ids: Set(serde_json::to_string(&mapped_sequence_ids)
                            .unwrap_or_else(|_| "[]".into())),
                        edge_order: Set(serde_json::to_string(&mapped_edge_order)
                            .unwrap_or_else(|_| "[]".into())),
                        created_at: Set(sequence.created_at),
                        updated_at: Set(sequence.updated_at),
                    };

                sequence_model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert sequence: {}", e))?;
            }
        }

        Ok(())
    }

    async fn import_palette_from_export(
        &self,
        project_id: i32,
        export: PaletteExport,
        dataset_map: &HashMap<i32, i32>,
    ) -> Result<()> {
        if export.layers.is_empty() {
            return Ok(());
        }

        let mut layer_id_map = HashMap::new();
        for layer in export.layers {
            let inserted = project_layers::ActiveModel {
                id: NotSet,
                project_id: Set(project_id),
                layer_id: Set(layer.layer_id),
                name: Set(layer.name),
                background_color: Set(layer.background_color),
                text_color: Set(layer.text_color),
                border_color: Set(layer.border_color),
                alias: Set(layer.alias),
                source_dataset_id: Set(layer
                    .source_dataset_id
                    .and_then(|id| dataset_map.get(&id).copied())),
                enabled: Set(layer.enabled),
                created_at: Set(layer.created_at),
                updated_at: Set(layer.updated_at),
            }
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to insert project layer: {}", e))?;

            layer_id_map.insert(layer.original_id, inserted.id);
        }

        for alias in export.aliases {
            if let Some(target_id) = layer_id_map.get(&alias.target_layer_original_id) {
                let alias_model = layer_aliases::ActiveModel {
                    id: NotSet,
                    project_id: Set(project_id),
                    alias_layer_id: Set(alias.alias_layer_id),
                    target_layer_id: Set(*target_id),
                    created_at: Set(Utc::now()),
                };
                alias_model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert layer alias: {}", e))?;
            }
        }

        Ok(())
    }

    async fn import_knowledge_base_from_export(
        &self,
        project_id: i32,
        index: KnowledgeBaseIndex,
        blobs: HashMap<String, Vec<u8>>,
    ) -> Result<()> {
        if index.files.is_empty() {
            return Ok(());
        }

        let mut tag_id_map = HashMap::new();
        for tag in index.tags {
            let tag_id = Uuid::parse_str(&tag.id)
                .map_err(|e| anyhow!("Invalid tag id {}: {}", tag.id, e))?;
            tag_id_map.insert(tag.id.clone(), tag_id);
            if kb_tags::Entity::find_by_id(tag_id)
                .one(&self.db)
                .await?
                .is_none()
            {
                let model = kb_tags::ActiveModel {
                    id: Set(tag_id),
                    name: Set(tag.name),
                    scope: Set(tag.scope),
                    color: Set(tag.color),
                    created_at: Set(tag.created_at),
                };
                model
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to insert knowledge base tag: {}", e))?;
            }
        }

        let mut file_id_map = HashMap::new();
        for file in index.files {
            let file_id = Uuid::parse_str(&file.id)
                .map_err(|e| anyhow!("Invalid file id {}: {}", file.id, e))?;
            let blob = blobs
                .get(&file.id)
                .cloned()
                .ok_or_else(|| anyhow!("Missing blob for knowledge base file {}", file.id))?;
            let model = kb_files::ActiveModel {
                id: Set(file_id),
                project_id: Set(project_id),
                filename: Set(file.filename),
                media_type: Set(file.media_type),
                size_bytes: Set(file.size_bytes),
                blob: Set(blob),
                checksum: Set(file.checksum),
                created_by: Set(None),
                created_at: Set(file.created_at),
                indexed: Set(file.indexed),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert knowledge base file: {}", e))?;
            file_id_map.insert(file.id, file_id);
        }

        for link in index.file_tags {
            let file_id = file_id_map
                .get(&link.file_id)
                .copied()
                .ok_or_else(|| anyhow!("Unknown file id {} in file_tags", link.file_id))?;
            let tag_id = tag_id_map
                .get(&link.tag_id)
                .copied()
                .ok_or_else(|| anyhow!("Unknown tag id {} in file_tags", link.tag_id))?;

            let model = kb_file_tags::ActiveModel {
                id: Set(Uuid::new_v4()),
                file_id: Set(file_id),
                tag_id: Set(tag_id),
                created_at: Set(Utc::now()),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert file tag mapping: {}", e))?;
        }

        for document in index.documents {
            let doc_id = Uuid::parse_str(&document.id)
                .map_err(|e| anyhow!("Invalid document id {}: {}", document.id, e))?;
            let file_id = match document.file_id {
                Some(ref id) => Some(
                    file_id_map
                        .get(id)
                        .copied()
                        .ok_or_else(|| anyhow!("Unknown file id {} in documents", id))?,
                ),
                None => None,
            };
            let embedding = match document.embedding {
                Some(ref encoded) => {
                    Some(general_purpose::STANDARD.decode(encoded).map_err(|e| {
                        anyhow!("Failed to decode embedding for {}: {}", document.id, e)
                    })?)
                }
                None => None,
            };

            let model = kb_docs::ActiveModel {
                id: Set(doc_id),
                project_id: Set(project_id),
                file_id: Set(file_id),
                chunk_id: Set(document.chunk_id),
                media_type: Set(document.media_type),
                chunk_text: Set(document.chunk_text),
                metadata: Set(document.metadata),
                embedding_model: Set(document.embedding_model),
                embedding: Set(embedding),
                created_at: Set(document.created_at),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert knowledge base document: {}", e))?;
        }

        for state in index.vector_states {
            let state_id = Uuid::parse_str(&state.id)
                .map_err(|e| anyhow!("Invalid vector state id {}: {}", state.id, e))?;
            let model = kb_vector_state::ActiveModel {
                id: Set(state_id),
                project_id: Set(project_id),
                status: Set(state.status),
                last_indexed_at: Set(state.last_indexed_at),
                last_error: Set(state.last_error),
                config: Set(state.config),
                updated_at: Set(state.updated_at),
                embedding_provider: Set(state.embedding_provider),
                embedding_model: Set(state.embedding_model),
            };
            model
                .insert(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to insert vector index state: {}", e))?;
        }

        Ok(())
    }
}

// ----- Standalone helper functions -----

fn write_bundle_common_files<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    manifest_bytes: &[u8],
    metadata_bytes: &[u8],
    project_bytes: &[u8],
    dag_bytes: &[u8],
    dataset_index_bytes: &[u8],
) -> Result<()> {
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("manifest.json", options)
        .map_err(|e| anyhow!("Failed to add manifest.json: {}", e))?;
    zip.write_all(manifest_bytes)
        .map_err(|e| anyhow!("Failed to write manifest.json: {}", e))?;

    zip.start_file("metadata.json", options)
        .map_err(|e| anyhow!("Failed to add metadata.json: {}", e))?;
    zip.write_all(metadata_bytes)
        .map_err(|e| anyhow!("Failed to write metadata.json: {}", e))?;

    zip.start_file("project.json", options)
        .map_err(|e| anyhow!("Failed to add project.json: {}", e))?;
    zip.write_all(project_bytes)
        .map_err(|e| anyhow!("Failed to write project.json: {}", e))?;

    zip.start_file("dag.json", options)
        .map_err(|e| anyhow!("Failed to add dag.json: {}", e))?;
    zip.write_all(dag_bytes)
        .map_err(|e| anyhow!("Failed to write dag.json: {}", e))?;

    zip.start_file("datasets/index.json", options)
        .map_err(|e| anyhow!("Failed to add datasets/index.json: {}", e))?;
    zip.write_all(dataset_index_bytes)
        .map_err(|e| anyhow!("Failed to write datasets/index.json: {}", e))?;

    Ok(())
}

fn sanitize_dataset_filename(name: &str) -> String {
    let filtered: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    let trimmed = filtered.trim_matches('_');
    if trimmed.is_empty() {
        "dataset".to_string()
    } else {
        trimmed.to_string()
    }
}

fn read_template_json<T: DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    path: &str,
) -> Result<T> {
    let mut file = archive
        .by_name(path)
        .map_err(|e| anyhow!("Template archive missing {}: {}", path, e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
    serde_json::from_slice(&buffer).map_err(|e| anyhow!("Failed to parse {}: {}", path, e))
}

fn try_read_template_json<T: DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<Vec<u8>>>,
    path: &str,
) -> Result<Option<T>> {
    match archive.by_name(path) {
        Ok(mut file) => {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
            let parsed = serde_json::from_slice(&buffer)
                .map_err(|e| anyhow!("Failed to parse {}: {}", path, e))?;
            Ok(Some(parsed))
        }
        Err(ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(anyhow!("Template archive missing {}: {}", path, e)),
    }
}

fn read_zip_file_bytes(archive: &mut ZipArchive<Cursor<Vec<u8>>>, path: &str) -> Result<Vec<u8>> {
    let mut file = archive
        .by_name(path)
        .map_err(|e| anyhow!("Archive missing {}: {}", path, e))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Failed to read {}: {}", path, e))?;
    Ok(buffer)
}

fn analyze_data_sets(
    data_sets: &[data_sets::Model],
) -> Result<(Vec<DatasetBundleDescriptor>, HashMap<i32, String>)> {
    let mut descriptors = Vec::new();
    let mut tables = HashMap::new();

    for data_set in data_sets {
        let (node_count, edge_count, layer_count) = summarize_graph_counts(&data_set.graph_json);
        let descriptor = DatasetBundleDescriptor {
            original_id: data_set.id,
            name: data_set.name.clone(),
            description: data_set.description.clone(),
            filename: format!(
                "{}_{}.json",
                sanitize_dataset_filename(&data_set.name),
                data_set.id
            ),
            file_format: "json".to_string(),
            node_count,
            edge_count,
            layer_count,
        };
        tables.insert(data_set.id, data_set.graph_json.clone());
        descriptors.push(descriptor);
    }

    Ok((descriptors, tables))
}

async fn insert_plan_dag_from_snapshot(
    db: &DatabaseConnection,
    plan_id: i32,
    snapshot: &PlanDagSnapshot,
    dataset_id_map: &HashMap<i32, i32>,
) -> Result<()> {
    let now = Utc::now();

    // Remap node and edge IDs to avoid collisions with existing records
    let mut node_id_map: HashMap<String, String> = HashMap::new();
    let mut edge_id_map: HashMap<String, String> = HashMap::new();

    let mut allocate_node_id = |old_id: &str| -> String {
        node_id_map
            .entry(old_id.to_string())
            .or_insert_with(|| format!("node_{}", Uuid::new_v4().simple()))
            .clone()
    };

    let mut allocate_edge_id = |old_id: &str| -> String {
        edge_id_map
            .entry(old_id.to_string())
            .or_insert_with(|| format!("edge_{}", Uuid::new_v4().simple()))
            .clone()
    };

    for node in &snapshot.nodes {
        let mut config_value: Value = serde_json::from_str(&node.config)
            .map_err(|e| anyhow!("Invalid node config JSON: {}", e))?;

        if let Some(old_id) = config_value
            .get("dataSetId")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32)
        {
            if let Some(new_id) = dataset_id_map.get(&old_id) {
                if let Some(obj) = config_value.as_object_mut() {
                    obj.insert("dataSetId".to_string(), json!(new_id));
                }
            }
        }

        let metadata_json = serde_json::to_string(&node.metadata)
            .map_err(|e| anyhow!("Failed to encode node metadata: {}", e))?;
        let config_json = serde_json::to_string(&config_value)
            .map_err(|e| anyhow!("Failed to encode node config: {}", e))?;

        let new_id = allocate_node_id(&node.id);

        plan_dag_nodes::ActiveModel {
            id: Set(new_id.clone()),
            plan_id: Set(plan_id),
            node_type: Set(node_type_storage_name(&node.node_type).to_string()),
            position_x: Set(node.position.x),
            position_y: Set(node.position.y),
            source_position: Set(node.source_position.clone()),
            target_position: Set(node.target_position.clone()),
            metadata_json: Set(metadata_json),
            config_json: Set(config_json),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("Failed to insert plan node {}: {}", new_id, e))?;
    }

    for edge in &snapshot.edges {
        let metadata_json = serde_json::to_string(&edge.metadata)
            .map_err(|e| anyhow!("Failed to encode edge metadata: {}", e))?;

        let new_id = allocate_edge_id(&edge.id);
        let source = node_id_map
            .get(&edge.source)
            .cloned()
            .unwrap_or_else(|| edge.source.clone());
        let target = node_id_map
            .get(&edge.target)
            .cloned()
            .unwrap_or_else(|| edge.target.clone());

        plan_dag_edges::ActiveModel {
            id: Set(new_id.clone()),
            plan_id: Set(plan_id),
            source_node_id: Set(source),
            target_node_id: Set(target),
            metadata_json: Set(metadata_json),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("Failed to insert plan edge {}: {}", new_id, e))?;
    }

    Ok(())
}
