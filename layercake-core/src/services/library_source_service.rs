use anyhow::{anyhow, Result};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::database::entities::data_sources::{self, DataType, FileFormat};
use crate::database::entities::{library_sources, projects};
use crate::services::source_processing;

/// Service layer for managing reusable datasource definitions stored in the library
#[derive(Clone)]
pub struct LibrarySourceService {
    db: DatabaseConnection,
}

impl LibrarySourceService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list(&self) -> Result<Vec<library_sources::Model>> {
        let items = library_sources::Entity::find().all(&self.db).await?;
        Ok(items)
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Option<library_sources::Model>> {
        let item = library_sources::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        Ok(item)
    }

    pub async fn create_from_file(
        &self,
        name: String,
        description: Option<String>,
        filename: String,
        file_format: FileFormat,
        data_type: DataType,
        file_data: Vec<u8>,
    ) -> Result<library_sources::Model> {
        if !data_type.is_compatible_with_format(&file_format) {
            return Err(anyhow!(
                "Invalid combination: {} format cannot contain {} data",
                file_format.as_str(),
                data_type.as_str()
            ));
        }

        let detected_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;
        if detected_format != file_format {
            return Err(anyhow!(
                "File extension doesn't match declared format. Expected {}, got {}",
                file_format.as_str(),
                detected_format.as_str()
            ));
        }

        let active_model = library_sources::ActiveModel {
            name: Set(name),
            description: Set(description),
            file_format: Set(file_format.as_str().to_string()),
            data_type: Set(data_type.as_str().to_string()),
            filename: Set(filename),
            blob: Set(file_data.clone()),
            file_size: Set(file_data.len() as i64),
            ..library_sources::ActiveModel::new()
        };

        let model = active_model.insert(&self.db).await?;

        let processed =
            match source_processing::process_file(&file_format, &data_type, &file_data).await {
                Ok(graph_json) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.graph_json = Set(graph_json);
                    active.status = Set("active".to_string());
                    active.processed_at = Set(Some(chrono::Utc::now()));
                    active.updated_at = Set(chrono::Utc::now());
                    active.update(&self.db).await?
                }
                Err(err) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.status = Set("error".to_string());
                    active.error_message = Set(Some(err.to_string()));
                    active.updated_at = Set(chrono::Utc::now());
                    let _ = active.update(&self.db).await?;
                    return Err(err);
                }
            };

        Ok(processed)
    }

    pub async fn update(
        &self,
        id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<library_sources::Model> {
        let existing = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Library source not found"))?;

        let mut active: library_sources::ActiveModel = existing.into();
        if let Some(name) = name {
            active.name = Set(name);
        }
        if let Some(description) = description {
            active.description = Set(Some(description));
        }
        active.updated_at = Set(chrono::Utc::now());

        let updated = active.update(&self.db).await?;
        Ok(updated)
    }

    pub async fn update_file(
        &self,
        id: i32,
        filename: String,
        file_data: Vec<u8>,
    ) -> Result<library_sources::Model> {
        let existing = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Library source not found"))?;

        let file_format = FileFormat::from_extension(&filename)
            .ok_or_else(|| anyhow!("Unsupported file extension: {}", filename))?;
        let data_type = existing
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type on existing record"))?;

        if !data_type.is_compatible_with_format(&file_format) {
            return Err(anyhow!(
                "Invalid combination: {} format cannot contain {} data",
                file_format.as_str(),
                data_type.as_str()
            ));
        }

        let mut active: library_sources::ActiveModel = existing.into();
        active.filename = Set(filename);
        active.blob = Set(file_data.clone());
        active.file_size = Set(file_data.len() as i64);
        active.file_format = Set(file_format.as_str().to_string());
        active.status = Set("processing".to_string());
        active.error_message = Set(None);
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(&self.db).await?;

        let processed =
            match source_processing::process_file(&file_format, &data_type, &file_data).await {
                Ok(graph_json) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.graph_json = Set(graph_json);
                    active.status = Set("active".to_string());
                    active.processed_at = Set(Some(chrono::Utc::now()));
                    active.updated_at = Set(chrono::Utc::now());
                    active.update(&self.db).await?
                }
                Err(err) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.status = Set("error".to_string());
                    active.error_message = Set(Some(err.to_string()));
                    active.updated_at = Set(chrono::Utc::now());
                    let _ = active.update(&self.db).await?;
                    return Err(err);
                }
            };

        Ok(processed)
    }

    pub async fn delete(&self, id: i32) -> Result<()> {
        library_sources::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn reprocess(&self, id: i32) -> Result<library_sources::Model> {
        let existing = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("Library source not found"))?;

        let file_format = existing
            .get_file_format()
            .ok_or_else(|| anyhow!("Invalid file format"))?;
        let data_type = existing
            .get_data_type()
            .ok_or_else(|| anyhow!("Invalid data type"))?;

        let mut active: library_sources::ActiveModel = existing.into();
        active.status = Set("processing".to_string());
        active.error_message = Set(None);
        active.updated_at = Set(chrono::Utc::now());

        let model = active.update(&self.db).await?;

        let processed =
            match source_processing::process_file(&file_format, &data_type, &model.blob).await {
                Ok(graph_json) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.graph_json = Set(graph_json);
                    active.status = Set("active".to_string());
                    active.processed_at = Set(Some(chrono::Utc::now()));
                    active.updated_at = Set(chrono::Utc::now());
                    active.update(&self.db).await?
                }
                Err(err) => {
                    let mut active: library_sources::ActiveModel = model.into();
                    active.status = Set("error".to_string());
                    active.error_message = Set(Some(err.to_string()));
                    active.updated_at = Set(chrono::Utc::now());
                    let _ = active.update(&self.db).await?;
                    return Err(err);
                }
            };

        Ok(processed)
    }

    pub async fn import_into_project(
        &self,
        project_id: i32,
        library_source_id: i32,
    ) -> Result<data_sources::Model> {
        projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        let source = self
            .get_by_id(library_source_id)
            .await?
            .ok_or_else(|| anyhow!("Library source not found"))?;

        let model = data_sources::ActiveModel {
            project_id: Set(project_id),
            name: Set(source.name.clone()),
            description: Set(source.description.clone()),
            file_format: Set(source.file_format.clone()),
            data_type: Set(source.data_type.clone()),
            filename: Set(source.filename.clone()),
            blob: Set(source.blob.clone()),
            graph_json: Set(source.graph_json.clone()),
            status: Set(source.status.clone()),
            error_message: Set(source.error_message.clone()),
            file_size: Set(source.file_size),
            processed_at: Set(source.processed_at),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..data_sources::ActiveModel::new()
        };

        let inserted = model.insert(&self.db).await?;
        Ok(inserted)
    }

    pub async fn import_many_into_project(
        &self,
        project_id: i32,
        library_source_ids: &[i32],
    ) -> Result<Vec<data_sources::Model>> {
        let mut imported = Vec::new();
        for id in library_source_ids {
            let model = self.import_into_project(project_id, *id).await?;
            imported.push(model);
        }
        Ok(imported)
    }

    pub async fn find_by_ids(&self, ids: &[i32]) -> Result<Vec<library_sources::Model>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let items = library_sources::Entity::find()
            .filter(library_sources::Column::Id.is_in(ids.to_vec()))
            .all(&self.db)
            .await?;

        Ok(items)
    }
}
