use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

use chrono::Utc;
use csv::ReaderBuilder;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::database::entities::common_types::{DataType, FileFormat};
use crate::database::entities::{data_sets, library_items, projects};
use crate::errors::{CoreError, CoreResult};
use crate::services::source_processing;

const REPO_LIBRARY_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/library");
const REPO_PROMPTS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/prompts");

pub const ITEM_TYPE_DATASET: &str = "dataset";
pub const ITEM_TYPE_PROJECT: &str = "project";
pub const ITEM_TYPE_PROJECT_TEMPLATE: &str = "project_template";
pub const ITEM_TYPE_PROMPT: &str = "prompt";

#[derive(Debug, Default, Clone)]
pub struct LibraryItemFilter {
    pub item_types: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatasetMetadata {
    #[serde(rename = "format")]
    pub format: String,
    #[serde(rename = "dataType")]
    pub data_type: String,
    pub filename: String,
    #[serde(rename = "rowCount")]
    pub row_count: Option<usize>,
    #[serde(rename = "columnCount")]
    pub column_count: Option<usize>,
    pub headers: Option<Vec<String>>,
}

pub struct LibraryItemService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct SeedLibraryResult {
    pub total_remote_files: usize,
    pub created_count: usize,
    pub skipped_count: usize,
    pub failed_files: Vec<String>,
}

impl LibraryItemService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self, filter: LibraryItemFilter) -> CoreResult<Vec<library_items::Model>> {
        let mut query =
            library_items::Entity::find().order_by_desc(library_items::Column::UpdatedAt);

        if let Some(types) = filter.item_types.filter(|v| !v.is_empty()) {
            query = query.filter(library_items::Column::ItemType.is_in(types));
        }

        if let Some(search) = filter.search.filter(|s| !s.trim().is_empty()) {
            let pattern = format!("%{}%", search.trim());
            query = query.filter(
                sea_orm::Condition::any()
                    .add(library_items::Column::Name.like(pattern.clone()))
                    .add(library_items::Column::Description.like(pattern)),
            );
        }

        if let Some(tags) = filter.tags.filter(|v| !v.is_empty()) {
            let mut condition = sea_orm::Condition::any();
            for tag in tags {
                let token = format!("%\"{}\"%", tag);
                condition = condition.add(library_items::Column::Tags.like(token));
            }
            query = query.filter(condition);
        }

        let items = query
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to list library items").with_source(e))?;
        Ok(items)
    }

    pub async fn get(&self, id: i32) -> CoreResult<Option<library_items::Model>> {
        let item = library_items::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load library item").with_source(e))?;
        Ok(item)
    }

    pub async fn update_fields(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> CoreResult<library_items::Model> {
        let model = library_items::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load library item").with_source(e))?
            .ok_or_else(|| CoreError::not_found("LibraryItem", id.to_string()))?;

        let mut active: library_items::ActiveModel = model.into();

        if let Some(n) = name {
            active.name = Set(n);
        }
        if description.is_some() {
            active.description = Set(description);
        }
        if let Some(tags_vec) = tags {
            let tags_json = serde_json::to_string(&tags_vec).unwrap_or_else(|_| "[]".to_string());
            active.tags = Set(tags_json);
        }
        active.updated_at = Set(Utc::now());

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to update library item").with_source(e))?;
        Ok(updated)
    }

    pub async fn create_dataset_item(
        &self,
        name: String,
        description: Option<String>,
        tags: Vec<String>,
        file_name: String,
        file_format: FileFormat,
        tabular_data_type: Option<DataType>,
        content_type: Option<String>,
        bytes: Vec<u8>,
    ) -> CoreResult<library_items::Model> {
        let resolved_data_type =
            Self::resolve_dataset_data_type(&file_format, &bytes, &file_name, tabular_data_type)?;
        self.validate_dataset_format(&file_format, &resolved_data_type)?;

        let metadata =
            self.build_dataset_metadata(&file_format, &resolved_data_type, &file_name, &bytes)?;
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let now = Utc::now();

        let model =
            library_items::ActiveModel {
                item_type: Set(ITEM_TYPE_DATASET.to_string()),
                name: Set(name),
                description: Set(description),
                tags: Set(tags_json),
                metadata: Set(serde_json::to_string(&metadata).map_err(|e| {
                    CoreError::internal("Failed to encode dataset metadata").with_source(e)
                })?),
                content_blob: Set(bytes.clone()),
                content_size: Set(Some(bytes.len() as i64)),
                content_type: Set(
                    content_type.or_else(|| Some(self.detect_content_type(&file_format)))
                ),
                created_at: Set(now),
                updated_at: Set(now),
                ..library_items::ActiveModel::new()
            }
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to create dataset item").with_source(e))?;

        Ok(model)
    }

    pub async fn create_binary_item(
        &self,
        item_type: String,
        name: String,
        description: Option<String>,
        tags: Vec<String>,
        metadata: Value,
        content_type: Option<String>,
        bytes: Vec<u8>,
    ) -> CoreResult<library_items::Model> {
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        let now = Utc::now();
        let model = library_items::ActiveModel {
            item_type: Set(item_type),
            name: Set(name),
            description: Set(description),
            tags: Set(tags_json),
            metadata: Set(metadata.to_string()),
            content_blob: Set(bytes.clone()),
            content_size: Set(Some(bytes.len() as i64)),
            content_type: Set(content_type),
            created_at: Set(now),
            updated_at: Set(now),
            ..library_items::ActiveModel::new()
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to create library item").with_source(e))?;

        Ok(model)
    }

    pub async fn seed_from_repository(&self) -> CoreResult<SeedLibraryResult> {
        let mut result = SeedLibraryResult {
            total_remote_files: 0,
            created_count: 0,
            skipped_count: 0,
            failed_files: Vec::new(),
        };

        // Seed datasets from library directory
        let library_dir = PathBuf::from(REPO_LIBRARY_PATH);
        if library_dir.is_dir() {
            let files: Vec<PathBuf> = fs::read_dir(&library_dir)
                .map_err(|e| {
                    CoreError::internal(format!(
                        "Failed to read {}: {}",
                        library_dir.display(),
                        e
                    ))
                })?
                .filter_map(|entry| entry.ok().map(|e| e.path()).filter(|path| path.is_file()))
                .collect();

            let mut existing_filenames: HashSet<String> = library_items::Entity::find()
                .filter(library_items::Column::ItemType.eq(ITEM_TYPE_DATASET))
                .all(&self.db)
                .await
                .map_err(|e| CoreError::internal("Failed to load library items").with_source(e))?
                .into_iter()
                .filter_map(|model| {
                    serde_json::from_str::<DatasetMetadata>(&model.metadata)
                        .ok()
                        .map(|meta| meta.filename)
                })
                .collect();

            result.total_remote_files += files.len();

            for path in files {
                let filename = path
                    .file_name()
                    .and_then(|os| os.to_str())
                    .ok_or_else(|| {
                        CoreError::validation(format!("Invalid filename in {}", path.display()))
                    })?
                    .to_string();

                if existing_filenames.contains(&filename) {
                    result.skipped_count += 1;
                    continue;
                }

                match self.seed_local_file(&path, &filename).await {
                    Ok(_) => {
                        result.created_count += 1;
                        existing_filenames.insert(filename);
                    }
                    Err(err) => {
                        result.failed_files.push(format!("{}: {}", filename, err));
                    }
                }
            }
        }

        // Seed prompts from prompts directory
        let prompts_dir = PathBuf::from(REPO_PROMPTS_PATH);
        if prompts_dir.is_dir() {
            let prompt_files: Vec<PathBuf> = fs::read_dir(&prompts_dir)
                .map_err(|e| {
                    CoreError::internal(format!(
                        "Failed to read {}: {}",
                        prompts_dir.display(),
                        e
                    ))
                })?
                .filter_map(|entry| entry.ok().map(|e| e.path()).filter(|path| path.is_file()))
                .filter(|path| {
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "md" || ext == "txt")
                        .unwrap_or(false)
                })
                .collect();

            let existing_prompt_names: HashSet<String> = library_items::Entity::find()
                .filter(library_items::Column::ItemType.eq(ITEM_TYPE_PROMPT))
                .all(&self.db)
                .await
                .map_err(|e| CoreError::internal("Failed to load prompt items").with_source(e))?
                .into_iter()
                .map(|model| model.name)
                .collect();

            result.total_remote_files += prompt_files.len();

            for path in prompt_files {
                let filename = path
                    .file_name()
                    .and_then(|os| os.to_str())
                    .ok_or_else(|| {
                        CoreError::validation(format!("Invalid filename in {}", path.display()))
                    })?
                    .to_string();

                let name = derive_name(&filename);
                if existing_prompt_names.contains(&name) {
                    result.skipped_count += 1;
                    continue;
                }

                match self.seed_prompt_file(&path, &filename).await {
                    Ok(_) => {
                        result.created_count += 1;
                    }
                    Err(err) => {
                        result.failed_files.push(format!("{}: {}", filename, err));
                    }
                }
            }
        }

        Ok(result)
    }

    async fn seed_local_file(&self, path: &PathBuf, filename: &str) -> CoreResult<()> {
        let file_bytes = fs::read(path).map_err(|e| {
            CoreError::internal(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let file_format = FileFormat::from_extension(filename)
            .ok_or_else(|| {
                CoreError::validation(format!("Unsupported file extension for {}", filename))
            })?;

        let data_type = infer_data_type(filename, &file_format, &file_bytes)?;
        let name = derive_name(filename);
        let description = Some("Seeded from resources/library".to_string());

        self.create_dataset_item(
            name,
            description,
            vec![],
            filename.to_string(),
            file_format,
            Some(data_type),
            None,
            file_bytes,
        )
        .await?;

        Ok(())
    }

    async fn seed_prompt_file(&self, path: &PathBuf, filename: &str) -> CoreResult<()> {
        let file_bytes = fs::read(path).map_err(|e| {
            CoreError::internal(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let name = derive_name(filename);
        let description = Some("Seeded from resources/prompts".to_string());

        let is_markdown = filename.ends_with(".md");
        let content_type = if is_markdown {
            "text/markdown"
        } else {
            "text/plain"
        };

        let metadata = serde_json::json!({
            "filename": filename,
            "format": if is_markdown { "markdown" } else { "text" }
        });

        self.create_binary_item(
            ITEM_TYPE_PROMPT.to_string(),
            name,
            description,
            vec!["prompt".to_string()],
            metadata,
            Some(content_type.to_string()),
            file_bytes,
        )
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: i32) -> CoreResult<()> {
        library_items::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to delete library item").with_source(e))?;
        Ok(())
    }

    pub async fn import_dataset_into_project(
        &self,
        project_id: i32,
        library_item_id: i32,
    ) -> CoreResult<data_sets::Model> {
        projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load project").with_source(e))?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let item = self
            .get(library_item_id)
            .await?
            .ok_or_else(|| CoreError::not_found("LibraryItem", library_item_id.to_string()))?;

        if item.item_type != ITEM_TYPE_DATASET {
            return Err(CoreError::validation(format!(
                "Library item {} is of type {}, expected dataset",
                library_item_id, item.item_type
            )));
        }

        let metadata = serde_json::from_str::<DatasetMetadata>(&item.metadata).unwrap_or_default();
        let file_format = metadata.format.parse::<FileFormat>().unwrap_or_else(|_| {
            FileFormat::from_extension(&metadata.filename).unwrap_or(FileFormat::Csv)
        });

        // Always try to infer the data type from the actual file content first (most accurate)
        let data_type = match infer_data_type(&metadata.filename, &file_format, &item.content_blob)
        {
            Ok(inferred) if inferred.is_compatible_with_format(&file_format) => inferred,
            _ => {
                // Fall back to metadata if inference fails
                metadata
                    .data_type
                    .parse::<DataType>()
                    .ok()
                    .filter(|dt| dt.is_compatible_with_format(&file_format))
                    .unwrap_or(DataType::Nodes) // Last resort: safe default
            }
        };

        let filename = if metadata.filename.is_empty() {
            format!("{}.{}", item.name, file_format.as_ref())
        } else {
            metadata.filename.clone()
        };

        let processed =
            source_processing::process_file(&file_format, &data_type, &item.content_blob)
                .await
                .map_err(|e| {
                    CoreError::internal("Failed to process dataset file").with_source(e)
                })?;

        let now = Utc::now();
        let dataset = data_sets::ActiveModel {
            project_id: Set(project_id),
            name: Set(item.name.clone()),
            description: Set(item.description.clone()),
            file_format: Set(file_format.as_ref().to_string()),
            data_type: Set(data_type.as_ref().to_string()),
            filename: Set(filename),
            blob: Set(item.content_blob.clone()),
            graph_json: Set(processed),
            status: Set("active".to_string()),
            error_message: Set(None),
            file_size: Set(item.content_size.unwrap_or(item.content_blob.len() as i64)),
            processed_at: Set(Some(now)),
            created_at: Set(now),
            updated_at: Set(now),
            annotations: Set(Some("[]".to_string())),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(|e| CoreError::internal("Failed to import dataset").with_source(e))?;

        Ok(dataset)
    }

    pub async fn import_many_datasets(
        &self,
        project_id: i32,
        ids: &[i32],
    ) -> CoreResult<Vec<data_sets::Model>> {
        let mut imported = Vec::new();
        for id in ids {
            imported.push(self.import_dataset_into_project(project_id, *id).await?);
        }
        Ok(imported)
    }

    fn validate_dataset_format(
        &self,
        file_format: &FileFormat,
        data_type: &DataType,
    ) -> CoreResult<()> {
        if !data_type.is_compatible_with_format(file_format) {
            return Err(CoreError::validation(format!(
                "Invalid combination: {} format cannot contain {} data",
                file_format.as_ref(),
                data_type.as_ref()
            )));
        }
        Ok(())
    }

    fn detect_content_type(&self, format: &FileFormat) -> String {
        match format {
            FileFormat::Csv => "text/csv",
            FileFormat::Tsv => "text/tab-separated-values",
            FileFormat::Json => "application/json",
            FileFormat::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            FileFormat::Ods => "application/vnd.oasis.opendocument.spreadsheet",
            FileFormat::Pdf => "application/pdf",
            FileFormat::Xml => "application/xml",
        }
        .to_string()
    }

    fn build_dataset_metadata(
        &self,
        file_format: &FileFormat,
        data_type: &DataType,
        file_name: &str,
        bytes: &[u8],
    ) -> CoreResult<DatasetMetadata> {
        let mut metadata = DatasetMetadata {
            format: file_format.as_ref().to_string(),
            data_type: data_type.as_ref().to_string(),
            filename: file_name.to_string(),
            ..Default::default()
        };

        if matches!(file_format, FileFormat::Csv | FileFormat::Tsv) {
            let delimiter = file_format
                .get_delimiter()
                .ok_or_else(|| {
                    CoreError::validation(format!(
                        "Unable to determine delimiter for {:?}",
                        file_format
                    ))
                })?;
            let mut reader = ReaderBuilder::new()
                .has_headers(true)
                .delimiter(delimiter)
                .from_reader(Cursor::new(bytes));

            if let Ok(headers) = reader.headers() {
                metadata.headers = Some(headers.iter().map(|h| h.to_string()).collect());
                metadata.column_count = Some(headers.len());
            }

            let mut row_count = 0usize;
            for result in reader.records() {
                if result.is_ok() {
                    row_count += 1;
                }

                if row_count >= 100_000 {
                    break;
                }
            }
            metadata.row_count = Some(row_count);
        }

        Ok(metadata)
    }

    fn resolve_dataset_data_type(
        file_format: &FileFormat,
        file_bytes: &[u8],
        file_name: &str,
        manual_hint: Option<DataType>,
    ) -> CoreResult<DataType> {
        match file_format {
            FileFormat::Csv | FileFormat::Tsv => {
                if let Some(hint) = manual_hint {
                    if matches!(hint, DataType::Nodes | DataType::Edges | DataType::Layers) {
                        return Ok(hint);
                    } else {
                        return Err(CoreError::validation(
                            "CSV/TSV uploads only support Nodes, Edges, or Layers data types",
                        ));
                    }
                }

                infer_data_type(file_name, file_format, file_bytes)
            }
            FileFormat::Json => Ok(DataType::Graph),
            _ => infer_data_type(file_name, file_format, file_bytes),
        }
    }
}

impl TryFrom<&library_items::Model> for DatasetMetadata {
    type Error = CoreError;

    fn try_from(model: &library_items::Model) -> std::result::Result<Self, Self::Error> {
        let meta = serde_json::from_str::<DatasetMetadata>(&model.metadata).map_err(|e| {
            CoreError::internal("Failed to decode library item metadata").with_source(e)
        })?;
        Ok(meta)
    }
}

pub fn infer_data_type(
    filename: &str,
    file_format: &FileFormat,
    file_data: &[u8],
) -> CoreResult<DataType> {
    if let Some(dtype) = infer_data_type_from_filename(filename) {
        if dtype.is_compatible_with_format(file_format) {
            return Ok(dtype);
        }
    }

    match file_format {
        FileFormat::Json => Ok(DataType::Graph),
        FileFormat::Csv | FileFormat::Tsv => infer_data_type_from_headers(file_format, file_data),
        FileFormat::Xlsx | FileFormat::Ods | FileFormat::Pdf | FileFormat::Xml => Err(
            CoreError::validation(format!(
                "File format {:?} is not supported for data type inference",
                file_format
            )),
        ),
    }
}

fn infer_data_type_from_headers(
    file_format: &FileFormat,
    file_data: &[u8],
) -> CoreResult<DataType> {
    let delimiter = file_format
        .get_delimiter()
        .ok_or_else(|| {
            CoreError::validation(format!(
                "Unable to determine delimiter for {:?}",
                file_format
            ))
        })?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .from_reader(Cursor::new(file_data));

    let headers = reader
        .headers()
        .map_err(|e| CoreError::validation(format!("Failed to read headers: {}", e)))?;

    let header_set: HashSet<String> = headers.iter().map(|h| h.trim().to_lowercase()).collect();

    for candidate in [DataType::Nodes, DataType::Edges, DataType::Layers] {
        if candidate
            .get_expected_headers()
            .iter()
            .all(|required| header_set.contains(&required.to_lowercase()))
        {
            return Ok(candidate);
        }
    }

    Err(CoreError::validation(format!(
        "Could not infer data type from headers {:?}",
        headers
    )))
}

fn infer_data_type_from_filename(filename: &str) -> Option<DataType> {
    let normalized = filename
        .to_lowercase()
        .replace(['(', ')', '-', '_', '.', ','], " ");
    let tokens: Vec<&str> = normalized.split_whitespace().collect();

    if tokens
        .iter()
        .any(|token| *token == "nodes" || *token == "node")
    {
        return Some(DataType::Nodes);
    }

    if tokens.iter().any(|token| {
        *token == "edges"
            || *token == "edge"
            || *token == "links"
            || *token == "link"
            || *token == "relationships"
            || *token == "relationship"
    }) {
        return Some(DataType::Edges);
    }

    if tokens
        .iter()
        .any(|token| *token == "layers" || *token == "layer")
    {
        return Some(DataType::Layers);
    }

    if tokens
        .iter()
        .any(|token| *token == "graph" || *token == "graphs")
    {
        return Some(DataType::Graph);
    }

    None
}

pub fn derive_name(filename: &str) -> String {
    let without_extension = filename
        .rsplit_once('.')
        .map(|(name, _)| name)
        .unwrap_or(filename);

    without_extension.trim().to_string()
}
