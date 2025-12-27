use serde_json::{json, Value};
use super::{AppContext, GraphLayerUpdateRequest, GraphNodeUpdateRequest};
use crate::auth::Actor;
use crate::errors::{CoreError, CoreResult};
use crate::services::graph_analysis_service::GraphConnectivityReport;
use crate::services::graph_edit_service::ReplaySummary as GraphEditReplaySummary;
impl AppContext {
    // ----- Graph editing helpers ------------------------------------------
    pub async fn create_graph(
        &self,
        actor: &Actor,
        project_id: i32,
        name: String,
        node_id: Option<String>,
    ) -> CoreResult<crate::database::entities::graph_data::Model> {
        self.authorize_project_write(actor, project_id).await?;
        self.graph_service
            .create_graph(project_id, name, node_id)
            .await
    }
    pub async fn update_graph(
        &self,
        actor: &Actor,
        id: i32,
        name: Option<String>,
    ) -> CoreResult<crate::database::entities::graph_data::Model> {
        self.authorize_graph_write(actor, id).await?;
        self.graph_service
            .update_graph(id, name)
            .await
    }
    pub async fn delete_graph(&self, actor: &Actor, id: i32) -> CoreResult<()> {
        self.authorize_graph_write(actor, id).await?;
        self.graph_service
            .delete_graph(id)
            .await
    }
    pub async fn add_graph_node(
        &self,
        actor: &Actor,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        is_partition: bool,
        belongs_to: Option<String>,
        weight: Option<f64>,
        attrs: Option<Value>,
    ) -> CoreResult<crate::database::entities::graph_data_nodes::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        self.graph_service
            .add_graph_node(
                graph_id,
                node_id,
                label,
                layer,
                is_partition,
                belongs_to,
                weight,
                attrs,
            )
            .await
    }
    pub async fn delete_graph_node(
        &self,
        actor: &Actor,
        graph_id: i32,
        node_id: String,
    ) -> CoreResult<crate::database::entities::graph_data_nodes::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        self.graph_service
            .delete_graph_node(graph_id, node_id)
            .await
    }
    pub async fn create_layer(
        &self,
        actor: &Actor,
        graph_id: i32,
        layer_id: String,
        name: String,
    ) -> CoreResult<crate::database::entities::graph_layers::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        use crate::database::entities::graph_layers;
        use sea_orm::{ActiveModelTrait, Set};
        let layer = graph_layers::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_id: Set(graph_id),
            layer_id: Set(layer_id),
            name: Set(name),
            background_color: Set(None),
            text_color: Set(None),
            border_color: Set(None),
            alias: Set(None),
            comment: Set(None),
            properties: Set(None),
            dataset_id: Set(None),
        };
        layer
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to insert graph layer: {}", e)))
    }
    pub async fn add_graph_edge(
        &self,
        actor: &Actor,
        graph_id: i32,
        id: String,
        source: String,
        target: String,
        label: Option<String>,
        layer: Option<String>,
        weight: Option<f64>,
        attributes: Option<Value>,
    ) -> CoreResult<crate::database::entities::graph_data_edges::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        use crate::database::entities::graph_data_edges::ActiveModel as GraphEdgeActiveModel;
        use sea_orm::{ActiveModelTrait, ActiveValue::Set};
        let now = chrono::Utc::now();
        let edge_model = GraphEdgeActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_data_id: Set(graph_id),
            external_id: Set(id),
            source: Set(source),
            target: Set(target),
            label: Set(label),
            layer: Set(layer),
            weight: Set(weight),
            attributes: Set(attributes),
            source_dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(now),
        };
        edge_model
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to insert graph edge: {}", e)))
    }
    pub async fn delete_graph_edge(
        &self,
        actor: &Actor,
        graph_id: i32,
        edge_id: String,
    ) -> CoreResult<bool> {
        self.authorize_graph_write(actor, graph_id).await?;
        use crate::database::entities::graph_data_edges::{
            Column as EdgeColumn, Entity as GraphEdges,
        };
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let old_edge = GraphEdges::find()
            .filter(EdgeColumn::GraphDataId.eq(graph_id))
            .filter(EdgeColumn::ExternalId.eq(&edge_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph edge: {}", e)))?;
        if old_edge.is_some() {
            GraphEdges::delete_many()
                .filter(EdgeColumn::GraphDataId.eq(graph_id))
                .filter(EdgeColumn::ExternalId.eq(&edge_id))
                .exec(&self.db)
                .await
                .map_err(|e| {
                    CoreError::internal(format!("Failed to delete graph edge: {}", e))
                })?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub async fn upsert_project_layer(
        &self,
        actor: &Actor,
        project_id: i32,
        layer_id: String,
        name: String,
        background_color: String,
        text_color: String,
        border_color: String,
        alias: Option<String>,
        source_dataset_id: Option<i32>,
        enabled: bool,
    ) -> CoreResult<crate::database::entities::project_layers::Model> {
        self.authorize_project_write(actor, project_id).await?;
        self.graph_service
            .upsert_project_layer(
                project_id,
                layer_id,
                name,
                background_color,
                text_color,
                border_color,
                alias,
                source_dataset_id,
                enabled,
            )
            .await
    }
    pub async fn delete_project_layer(
        &self,
        actor: &Actor,
        project_id: i32,
        layer_id: String,
        source_dataset_id: Option<i32>,
    ) -> CoreResult<u64> {
        self.authorize_project_write(actor, project_id).await?;
        self.graph_service
            .delete_project_layer(project_id, layer_id, source_dataset_id)
            .await
    }
    pub async fn set_layer_dataset_enabled(
        &self,
        actor: &Actor,
        project_id: i32,
        dataset_id: i32,
        enabled: bool,
    ) -> CoreResult<usize> {
        self.authorize_project_write(actor, project_id).await?;
        self.graph_service
            .set_layer_dataset_enabled(project_id, dataset_id, enabled)
            .await
    }
    pub async fn reset_project_layers(
        &self,
        actor: &Actor,
        project_id: i32,
    ) -> CoreResult<()> {
        self.authorize_project_write(actor, project_id).await?;
        self.graph_service
            .reset_project_layers(project_id)
            .await
    }
    pub async fn create_layer_alias(
        &self,
        actor: &Actor,
        project_id: i32,
        alias_layer_id: String,
        target_layer_id: i32,
    ) -> CoreResult<crate::database::entities::layer_aliases::Model> {
        self.authorize_project_write(actor, project_id).await?;
        use crate::database::entities::{layer_aliases, project_layers};
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
        let target_layer = project_layers::Entity::find_by_id(target_layer_id)
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load project layer: {}", e)))?
            .ok_or_else(|| {
                CoreError::validation(format!(
                    "Target layer {} not found in project {}",
                    target_layer_id, project_id
                ))
            })?;
        let alias = layer_aliases::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            project_id: Set(project_id),
            alias_layer_id: Set(alias_layer_id),
            target_layer_id: Set(target_layer.id),
            created_at: Set(chrono::Utc::now()),
        };
        alias
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to insert layer alias: {}", e)))
    }
    pub async fn remove_layer_alias(
        &self,
        actor: &Actor,
        project_id: i32,
        alias_layer_id: String,
    ) -> CoreResult<bool> {
        self.authorize_project_write(actor, project_id).await?;
        use crate::database::entities::layer_aliases;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let result = layer_aliases::Entity::delete_many()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::AliasLayerId.eq(alias_layer_id))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete layer alias: {}", e)))?;
        Ok(result.rows_affected > 0)
    }
    pub async fn remove_layer_aliases(
        &self,
        actor: &Actor,
        project_id: i32,
        target_layer_id: i32,
    ) -> CoreResult<i32> {
        self.authorize_project_write(actor, project_id).await?;
        use crate::database::entities::layer_aliases;
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let result = layer_aliases::Entity::delete_many()
            .filter(layer_aliases::Column::ProjectId.eq(project_id))
            .filter(layer_aliases::Column::TargetLayerId.eq(target_layer_id))
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete layer aliases: {}", e)))?;
        Ok(result.rows_affected as i32)
    }
    pub async fn update_graph_node(
        &self,
        actor: &Actor,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attributes: Option<Value>,
        belongs_to: Option<String>,
    ) -> CoreResult<crate::database::entities::graph_data_nodes::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        use crate::database::entities::graph_nodes::{Column as NodeColumn, Entity as GraphNodes};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        let old_node = GraphNodes::find()
            .filter(NodeColumn::GraphId.eq(graph_id))
            .filter(NodeColumn::Id.eq(&node_id))
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load graph node {}: {}", node_id, e)))?;
        let belongs_to_param = belongs_to.as_ref().map(|value| {
            if value.is_empty() {
                None
            } else {
                Some(value.clone())
            }
        });
        let updated_node = self
            .graph_service
            .update_graph_node(
                graph_id,
                node_id.clone(),
                label.clone(),
                layer.clone(),
                attributes.clone(),
                belongs_to_param.clone(),
            )
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update graph node {}: {}", node_id, e)))?;
        if let Some(old_node) = old_node {
            if let Some(new_label) = &label {
                if old_node.label.as_ref() != Some(new_label) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("label".to_string()),
                            old_node.label.as_ref().map(|l| json!(l)),
                            Some(json!(new_label)),
                            None,
                            true,
                        )
                        .await;
                }
            }
            if let Some(new_layer) = &layer {
                let old_layer_value = old_node.layer.clone().unwrap_or_default();
                if &old_layer_value != new_layer {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("layer".to_string()),
                            if old_layer_value.is_empty() {
                                None
                            } else {
                                Some(json!(old_layer_value))
                            },
                            Some(json!(new_layer)),
                            None,
                            true,
                        )
                        .await;
                }
            }
            if let Some(new_attrs) = &attributes {
                if old_node.attrs.as_ref() != Some(new_attrs) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("attributes".to_string()),
                            old_node.attrs.clone(),
                            Some(new_attrs.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }
            if let Some(new_belongs_to) = belongs_to_param.clone() {
                if old_node.belongs_to != new_belongs_to {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            graph_id,
                            "node".to_string(),
                            node_id.clone(),
                            "update".to_string(),
                            Some("belongsTo".to_string()),
                            old_node.belongs_to.as_ref().map(|b| json!(b)),
                            new_belongs_to.as_ref().map(|b| json!(b)),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }
        Ok(updated_node)
    }
    pub async fn update_layer_properties(
        &self,
        actor: &Actor,
        layer_id: i32,
        name: Option<String>,
        alias: Option<String>,
        properties: Option<Value>,
    ) -> CoreResult<crate::database::entities::graph_layers::Model> {
        use crate::database::entities::graph_layers::Entity as Layers;
        use sea_orm::EntityTrait;
        let old_layer = Layers::find_by_id(layer_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load layer {}: {}", layer_id, e)))?;
        if let Some(layer) = &old_layer {
            self.authorize_graph_write(actor, layer.graph_id).await?;
        }
        let updated_layer = self
            .graph_service
            .update_layer_properties(layer_id, name.clone(), alias.clone(), properties.clone())
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update layer {}: {}", layer_id, e)))?;
        if let Some(old_layer) = old_layer {
            if let Some(new_name) = &name {
                if &old_layer.name != new_name {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("name".to_string()),
                            Some(json!(old_layer.name)),
                            Some(json!(new_name)),
                            None,
                            true,
                        )
                        .await;
                }
            }
            if old_layer.alias != updated_layer.alias {
                let _ = self
                    .graph_edit_service
                    .create_edit(
                        old_layer.graph_id,
                        "layer".to_string(),
                        old_layer.layer_id.clone(),
                        "update".to_string(),
                        Some("alias".to_string()),
                        old_layer.alias.as_ref().map(|value| json!(value)),
                        updated_layer.alias.as_ref().map(|value| json!(value)),
                        None,
                        true,
                    )
                    .await;
            }
            if let Some(new_properties) = &properties {
                let old_props = old_layer
                    .properties
                    .and_then(|p| serde_json::from_str::<Value>(&p).ok());
                if old_props.as_ref() != Some(new_properties) {
                    let _ = self
                        .graph_edit_service
                        .create_edit(
                            old_layer.graph_id,
                            "layer".to_string(),
                            old_layer.layer_id.clone(),
                            "update".to_string(),
                            Some("properties".to_string()),
                            old_props,
                            Some(new_properties.clone()),
                            None,
                            true,
                        )
                        .await;
                }
            }
        }
        Ok(updated_layer)
    }
    pub async fn bulk_update_graph_data(
        &self,
        actor: &Actor,
        graph_id: i32,
        node_updates: Vec<GraphNodeUpdateRequest>,
        layer_updates: Vec<GraphLayerUpdateRequest>,
    ) -> CoreResult<()> {
        self.authorize_graph_write(actor, graph_id).await?;
        for node_update in node_updates {
            self.update_graph_node(
                actor,
                graph_id,
                node_update.node_id,
                node_update.label,
                node_update.layer,
                node_update.attributes,
                node_update.belongs_to,
            )
            .await?;
        }
        for layer_update in layer_updates {
            self.update_layer_properties(
                actor,
                layer_update.id,
                layer_update.name,
                layer_update.alias,
                layer_update.properties,
            )
            .await?;
        }
        Ok(())
    }

    pub async fn update_graph_data_metadata(
        &self,
        actor: &Actor,
        id: i32,
        name: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> CoreResult<crate::database::entities::graph_data::Model> {
        self.authorize_graph_write(actor, id).await?;
        use crate::database::entities::graph_data;
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        let existing = graph_data::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to load graph data").with_source(e))?
            .ok_or_else(|| CoreError::not_found("GraphData", id.to_string()))?;

        let mut active: graph_data::ActiveModel = existing.into();
        if let Some(name) = name {
            active.name = Set(name);
        }
        if let Some(metadata) = metadata {
            active.metadata = Set(Some(metadata));
        }
        active.updated_at = Set(chrono::Utc::now());

        active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal("Failed to update graph data").with_source(e))
    }

    pub async fn replay_graph_data_edits(
        &self,
        actor: &Actor,
        graph_data_id: i32,
    ) -> CoreResult<crate::database::entities::graph_data::Model> {
        self.authorize_graph_write(actor, graph_data_id).await?;
        let service = crate::services::GraphDataService::new(self.db.clone());
        service.replay_edits(graph_data_id).await?;
        service
            .get_by_id(graph_data_id)
            .await?
            .ok_or_else(|| CoreError::not_found("GraphData", graph_data_id.to_string()))
    }

    pub async fn clear_graph_data_edits(
        &self,
        actor: &Actor,
        graph_data_id: i32,
    ) -> CoreResult<u64> {
        self.authorize_graph_write(actor, graph_data_id).await?;
        let service = crate::services::GraphDataService::new(self.db.clone());
        service.clear_edits(graph_data_id).await
    }

    pub async fn replay_graph_edits(
        &self,
        actor: &Actor,
        graph_id: i32,
    ) -> CoreResult<GraphEditReplaySummary> {
        self.authorize_graph_write(actor, graph_id).await?;
        self.graph_edit_service
            .replay_graph_edits(graph_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to replay graph edits: {}", e)))
    }

    pub async fn create_graph_edit(
        &self,
        actor: &Actor,
        graph_id: i32,
        target_type: String,
        target_id: String,
        operation: String,
        field_name: Option<String>,
        old_value: Option<Value>,
        new_value: Option<Value>,
        created_by: Option<i32>,
        applied: bool,
    ) -> CoreResult<crate::database::entities::graph_edits::Model> {
        self.authorize_graph_write(actor, graph_id).await?;
        self.graph_edit_service
            .create_edit(
                graph_id,
                target_type,
                target_id,
                operation,
                field_name,
                old_value,
                new_value,
                created_by,
                applied,
            )
            .await
    }

    pub async fn clear_graph_edits(
        &self,
        actor: &Actor,
        graph_id: i32,
    ) -> CoreResult<u64> {
        self.authorize_graph_write(actor, graph_id).await?;
        self.graph_edit_service.clear_graph_edits(graph_id).await
    }
    pub async fn analyze_graph_connectivity(
        &self,
        graph_id: i32,
    ) -> CoreResult<GraphConnectivityReport> {
        self.graph_analysis_service
            .analyze_connectivity(graph_id)
            .await
    }
    pub async fn find_graph_paths(
        &self,
        graph_id: i32,
        source_node: String,
        target_node: String,
        max_paths: usize,
    ) -> CoreResult<Vec<Vec<String>>> {
        self.graph_analysis_service
            .find_paths(graph_id, &source_node, &target_node, max_paths)
            .await
    }
}
