use crate::database::entities::project_layers;
use crate::errors::{CoreError, CoreResult};
use chrono::Utc;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, QueryFilter};
use std::collections::HashSet;

pub struct LayerPaletteService {
    db: DatabaseConnection,
}

impl LayerPaletteService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_project_palette(
        &self,
        project_id: i32,
    ) -> CoreResult<Vec<project_layers::Model>> {
        project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project palette: {}", e))
            })
    }

    pub async fn get_layers_by_ids(
        &self,
        project_id: i32,
        layer_ids: HashSet<String>,
    ) -> CoreResult<Vec<project_layers::Model>> {
        if layer_ids.is_empty() {
            return Ok(vec![]);
        }

        project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .filter(project_layers::Column::LayerId.is_in(layer_ids))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load project layers: {}", e))
            })
    }

    pub async fn add_layer(
        &self,
        project_id: i32,
        layer: NewLayer,
    ) -> CoreResult<project_layers::Model> {
        let now = Utc::now();
        let active = project_layers::ActiveModel {
            project_id: Set(project_id),
            layer_id: Set(layer.layer_id),
            name: Set(layer.name),
            background_color: Set(layer.background_color.unwrap_or_else(|| "#FFFFFF".into())),
            text_color: Set(layer.text_color.unwrap_or_else(|| "#000000".into())),
            border_color: Set(layer.border_color.unwrap_or_else(|| "#000000".into())),
            alias: Set(layer.alias),
            source_dataset_id: Set(layer.source_dataset_id),
            enabled: Set(layer.enabled.unwrap_or(true)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        project_layers::Entity::insert(active)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to insert project layer: {}", e)))
    }

    pub async fn validate_layer_references(
        &self,
        project_id: i32,
        layer_ids: &HashSet<String>,
    ) -> CoreResult<LayerValidationResult> {
        let existing = self
            .get_project_palette(project_id)
            .await?
            .into_iter()
            .map(|layer| layer.layer_id)
            .collect::<HashSet<_>>();

        let missing = layer_ids
            .iter()
            .filter(|id| !existing.contains(*id))
            .cloned()
            .collect::<Vec<_>>();

        let orphaned = existing
            .iter()
            .filter(|id| !layer_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();

        Ok(LayerValidationResult {
            missing_layers: missing.clone(),
            orphaned_layers: orphaned,
            is_valid: missing.is_empty(),
        })
    }
}

pub struct NewLayer {
    pub layer_id: String,
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub alias: Option<String>,
    pub source_dataset_id: Option<i32>,
    pub enabled: Option<bool>,
}

pub struct LayerValidationResult {
    pub missing_layers: Vec<String>,
    pub orphaned_layers: Vec<String>,
    pub is_valid: bool,
}
