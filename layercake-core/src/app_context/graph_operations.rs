use serde_json::{json, Value};

use super::{AppContext, GraphLayerUpdateRequest, GraphNodeUpdateRequest};
use crate::auth::Actor;
use crate::errors::{CoreError, CoreResult};
use crate::services::graph_analysis_service::GraphConnectivityReport;
use crate::services::graph_edit_service::ReplaySummary as GraphEditReplaySummary;

impl AppContext {
    // ----- Graph editing helpers ------------------------------------------

    pub async fn update_graph_node(
        &self,
        _actor: &Actor,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attributes: Option<Value>,
        belongs_to: Option<String>,
    ) -> CoreResult<crate::database::entities::graph_data_nodes::Model> {
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
        _actor: &Actor,
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

    pub async fn replay_graph_edits(&self, graph_id: i32) -> CoreResult<GraphEditReplaySummary> {
        self.graph_edit_service
            .replay_graph_edits(graph_id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to replay graph edits: {}", e)))
    }

    pub async fn analyze_graph_connectivity(
        &self,
        graph_id: i32,
    ) -> CoreResult<GraphConnectivityReport> {
        self.graph_analysis_service
            .analyze_connectivity(graph_id)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to analyze graph connectivity: {}", e))
            })
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
            .map_err(|e| CoreError::internal(format!("Failed to find graph paths: {}", e)))
    }
}
