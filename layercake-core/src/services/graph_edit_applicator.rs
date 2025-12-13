use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait,
    QueryFilter, Set,
};
use tracing::{debug, warn};

use crate::database::entities::{
    graph_data, graph_data_edges, graph_data_nodes,
    graph_edges::{self, Entity as GraphEdges},
    graph_edits,
    graph_layers::{self, Entity as Layers},
    graph_nodes::{self, Entity as GraphNodes},
};

/// Result of applying a single edit
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyResult {
    Success { message: String },
    Skipped { reason: String },
    Error { reason: String },
}

/// Applies graph edits to database entities
pub struct GraphEditApplicator {
    db: DatabaseConnection,
}

impl GraphEditApplicator {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Apply a single graph edit to the database
    ///
    /// Returns ApplyResult indicating success, skip, or error
    pub async fn apply_edit(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let is_graph_data = graph_data::Entity::find_by_id(edit.graph_id)
            .one(&self.db)
            .await?
            .is_some();

        debug!(
            "Applying edit #{} on {}:{} ({})",
            edit.sequence_number, edit.target_type, edit.target_id, edit.operation
        );

        let result = match edit.target_type.as_str() {
            "node" => {
                if is_graph_data {
                    self.apply_node_edit_graph_data(edit).await?
                } else {
                    self.apply_node_edit(edit).await?
                }
            }
            "edge" => {
                if is_graph_data {
                    self.apply_edge_edit_graph_data(edit).await?
                } else {
                    self.apply_edge_edit(edit).await?
                }
            }
            "layer" => {
                if is_graph_data {
                    ApplyResult::Skipped {
                        reason: "Layer edits are not supported for graph_data; layers derive from node attributes"
                            .to_string(),
                    }
                } else {
                    self.apply_layer_edit(edit).await?
                }
            }
            _ => {
                let reason = format!("Unknown target type: {}", edit.target_type);
                warn!("{}", reason);
                ApplyResult::Error { reason }
            }
        };

        Ok(result)
    }

    /// Apply edit to a graph node
    async fn apply_node_edit(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        match edit.operation.as_str() {
            "create" => self.create_node(edit).await,
            "update" => self.update_node(edit).await,
            "delete" => self.delete_node(edit).await,
            _ => Ok(ApplyResult::Error {
                reason: format!("Unknown node operation: {}", edit.operation),
            }),
        }
    }

    async fn create_node(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        // Check if node already exists
        let existing = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(ApplyResult::Skipped {
                reason: format!("Node {} already exists", edit.target_id),
            });
        }

        let new_value = edit
            .new_value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Create operation missing new_value"))?;

        let label = new_value
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let layer = new_value
            .get("layer")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let is_partition = new_value
            .get("isPartition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let belongs_to = new_value
            .get("belongsTo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let attrs = new_value.get("attrs").cloned();

        let node = graph_nodes::ActiveModel {
            id: Set(edit.target_id.clone()),
            graph_id: Set(edit.graph_id),
            label: Set(label),
            layer: Set(layer),
            is_partition: Set(is_partition),
            belongs_to: Set(belongs_to),
            weight: Set(None),
            attrs: Set(attrs),
            dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(chrono::Utc::now()),
        };

        node.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created node {}", edit.target_id),
        })
    }

    async fn update_node(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let node = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(node) = node else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Node {} not found", edit.target_id),
            });
        };

        let mut active_model: graph_nodes::ActiveModel = node.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "label" => {
                        active_model.label = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "layer" => {
                        active_model.layer = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "isPartition" => {
                        if let Some(is_partition) = new_value.as_bool() {
                            active_model.is_partition = Set(is_partition);
                        }
                    }
                    "belongsTo" => {
                        active_model.belongs_to = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "attrs" => {
                        active_model.attrs = Set(Some(new_value.clone()));
                    }
                    _ => {
                        return Ok(ApplyResult::Skipped {
                            reason: format!("Unknown field: {}", field_name),
                        });
                    }
                }
            }
        }

        active_model.update(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!(
                "Updated node {} field {:?}",
                edit.target_id, edit.field_name
            ),
        })
    }

    async fn delete_node(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let result = GraphNodes::delete_many()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&edit.target_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            Ok(ApplyResult::Skipped {
                reason: format!("Node {} not found", edit.target_id),
            })
        } else {
            Ok(ApplyResult::Success {
                message: format!("Deleted node {}", edit.target_id),
            })
        }
    }

    /// Apply edit to graph_data node
    async fn apply_node_edit_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        match edit.operation.as_str() {
            "create" => self.create_node_graph_data(edit).await,
            "update" => self.update_node_graph_data(edit).await,
            "delete" => self.delete_node_graph_data(edit).await,
            _ => Ok(ApplyResult::Error {
                reason: format!("Unknown node operation: {}", edit.operation),
            }),
        }
    }

    async fn create_node_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let existing = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_nodes::Column::ExternalId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(ApplyResult::Skipped {
                reason: format!("Node {} already exists", edit.target_id),
            });
        }

        let new_value = edit
            .new_value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Create operation missing new_value"))?;

        let label = new_value
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let layer = new_value
            .get("layer")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let is_partition = new_value
            .get("isPartition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let belongs_to = new_value
            .get("belongsTo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let attrs = new_value.get("attrs").cloned().or_else(|| new_value.get("attributes").cloned());

        let node = graph_data_nodes::ActiveModel {
            id: ActiveValue::NotSet,
            graph_data_id: Set(edit.graph_id),
            external_id: Set(edit.target_id.clone()),
            label: Set(label),
            layer: Set(layer),
            weight: Set(None),
            is_partition: Set(is_partition),
            belongs_to: Set(belongs_to),
            comment: Set(None),
            source_dataset_id: Set(None),
            attributes: Set(attrs),
            created_at: Set(chrono::Utc::now()),
        };

        node.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created node {}", edit.target_id),
        })
    }

    async fn update_node_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let node = graph_data_nodes::Entity::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_nodes::Column::ExternalId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(node) = node else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Node {} not found", edit.target_id),
            });
        };

        let mut active_model: graph_data_nodes::ActiveModel = node.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "label" => {
                        active_model.label = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "layer" => {
                        active_model.layer = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "isPartition" => {
                        if let Some(is_partition) = new_value.as_bool() {
                            active_model.is_partition = Set(is_partition);
                        }
                    }
                    "belongsTo" => {
                        active_model.belongs_to = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "attrs" | "attributes" => {
                        active_model.attributes = Set(Some(new_value.clone()));
                    }
                    _ => {
                        return Ok(ApplyResult::Skipped {
                            reason: format!("Unknown field: {}", field_name),
                        });
                    }
                }
            }
        }

        active_model.update(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!(
                "Updated node {} field {:?}",
                edit.target_id, edit.field_name
            ),
        })
    }

    async fn delete_node_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let result = graph_data_nodes::Entity::delete_many()
            .filter(graph_data_nodes::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_nodes::Column::ExternalId.eq(&edit.target_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            Ok(ApplyResult::Skipped {
                reason: format!("Node {} not found", edit.target_id),
            })
        } else {
            Ok(ApplyResult::Success {
                message: format!("Deleted node {}", edit.target_id),
            })
        }
    }

    /// Apply edit to a graph edge
    async fn apply_edge_edit(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        match edit.operation.as_str() {
            "create" => self.create_edge(edit).await,
            "update" => self.update_edge(edit).await,
            "delete" => self.delete_edge(edit).await,
            _ => Ok(ApplyResult::Error {
                reason: format!("Unknown edge operation: {}", edit.operation),
            }),
        }
    }

    async fn create_edge(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        // Check if edge already exists
        let existing = GraphEdges::find()
            .filter(graph_edges::Column::GraphId.eq(edit.graph_id))
            .filter(graph_edges::Column::Id.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(ApplyResult::Skipped {
                reason: format!("Edge {} already exists", edit.target_id),
            });
        }

        let new_value = edit
            .new_value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Create operation missing new_value"))?;

        let source = new_value
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge create missing source"))?
            .to_string();

        let target = new_value
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge create missing target"))?
            .to_string();

        // Verify source and target nodes exist
        let source_exists = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&source))
            .count(&self.db)
            .await?
            > 0;

        let target_exists = GraphNodes::find()
            .filter(graph_nodes::Column::GraphId.eq(edit.graph_id))
            .filter(graph_nodes::Column::Id.eq(&target))
            .count(&self.db)
            .await?
            > 0;

        if !source_exists || !target_exists {
            return Ok(ApplyResult::Skipped {
                reason: format!(
                    "Source or target node not found for edge {}",
                    edit.target_id
                ),
            });
        }

        let label = new_value
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let layer = new_value
            .get("layer")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let attrs = new_value.get("attrs").cloned();

        let edge = graph_edges::ActiveModel {
            id: Set(edit.target_id.clone()),
            graph_id: Set(edit.graph_id),
            source: Set(source),
            target: Set(target),
            label: Set(label),
            layer: Set(layer),
            weight: Set(None),
            attrs: Set(attrs),
            dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(chrono::Utc::now()),
        };

        edge.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created edge {}", edit.target_id),
        })
    }

    async fn update_edge(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let edge = GraphEdges::find()
            .filter(graph_edges::Column::GraphId.eq(edit.graph_id))
            .filter(graph_edges::Column::Id.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(edge) = edge else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Edge {} not found", edit.target_id),
            });
        };

        let mut active_model: graph_edges::ActiveModel = edge.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "label" => {
                        active_model.label = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "layer" => {
                        active_model.layer = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "attrs" => {
                        active_model.attrs = Set(Some(new_value.clone()));
                    }
                    _ => {
                        return Ok(ApplyResult::Skipped {
                            reason: format!("Unknown field: {}", field_name),
                        });
                    }
                }
            }
        }

        active_model.update(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!(
                "Updated edge {} field {:?}",
                edit.target_id, edit.field_name
            ),
        })
    }

    async fn delete_edge(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let result = GraphEdges::delete_many()
            .filter(graph_edges::Column::GraphId.eq(edit.graph_id))
            .filter(graph_edges::Column::Id.eq(&edit.target_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            Ok(ApplyResult::Skipped {
                reason: format!("Edge {} not found", edit.target_id),
            })
        } else {
            Ok(ApplyResult::Success {
                message: format!("Deleted edge {}", edit.target_id),
            })
        }
    }

    async fn apply_edge_edit_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        match edit.operation.as_str() {
            "create" => self.create_edge_graph_data(edit).await,
            "update" => self.update_edge_graph_data(edit).await,
            "delete" => self.delete_edge_graph_data(edit).await,
            _ => Ok(ApplyResult::Error {
                reason: format!("Unknown edge operation: {}", edit.operation),
            }),
        }
    }

    async fn create_edge_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let existing = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_edges::Column::ExternalId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(ApplyResult::Skipped {
                reason: format!("Edge {} already exists", edit.target_id),
            });
        }

        let new_value = edit
            .new_value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Create operation missing new_value"))?;

        let source = new_value
            .get("source")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge source missing"))?
            .to_string();
        let target = new_value
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge target missing"))?
            .to_string();
        let label = new_value
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let layer = new_value
            .get("layer")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let attrs = new_value.get("attrs").cloned().or_else(|| new_value.get("attributes").cloned());

        let edge = graph_data_edges::ActiveModel {
            id: ActiveValue::NotSet,
            graph_data_id: Set(edit.graph_id),
            external_id: Set(edit.target_id.clone()),
            source: Set(source),
            target: Set(target),
            label: Set(label),
            layer: Set(layer),
            weight: Set(None),
            comment: Set(None),
            source_dataset_id: Set(None),
            attributes: Set(attrs),
            created_at: Set(chrono::Utc::now()),
        };

        edge.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created edge {}", edit.target_id),
        })
    }

    async fn update_edge_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let edge = graph_data_edges::Entity::find()
            .filter(graph_data_edges::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_edges::Column::ExternalId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(edge) = edge else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Edge {} not found", edit.target_id),
            });
        };

        let mut active_model: graph_data_edges::ActiveModel = edge.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "label" => {
                        active_model.label = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "layer" => {
                        active_model.layer = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "attrs" | "attributes" => {
                        active_model.attributes = Set(Some(new_value.clone()));
                    }
                    _ => {
                        return Ok(ApplyResult::Skipped {
                            reason: format!("Unknown field: {}", field_name),
                        });
                    }
                }
            }
        }

        active_model.update(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!(
                "Updated edge {} field {:?}",
                edit.target_id, edit.field_name
            ),
        })
    }

    async fn delete_edge_graph_data(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let result = graph_data_edges::Entity::delete_many()
            .filter(graph_data_edges::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_edges::Column::ExternalId.eq(&edit.target_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            Ok(ApplyResult::Skipped {
                reason: format!("Edge {} not found", edit.target_id),
            })
        } else {
            Ok(ApplyResult::Success {
                message: format!("Deleted edge {}", edit.target_id),
            })
        }
    }

    /// Apply edit to a layer
    async fn apply_layer_edit(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        match edit.operation.as_str() {
            "create" => self.create_layer(edit).await,
            "update" => self.update_layer(edit).await,
            "delete" => self.delete_layer(edit).await,
            _ => Ok(ApplyResult::Error {
                reason: format!("Unknown layer operation: {}", edit.operation),
            }),
        }
    }

    async fn create_layer(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        // Check if layer already exists
        let existing = Layers::find()
            .filter(graph_layers::Column::GraphId.eq(edit.graph_id))
            .filter(graph_layers::Column::LayerId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Ok(ApplyResult::Skipped {
                reason: format!("Layer {} already exists", edit.target_id),
            });
        }

        let new_value = edit
            .new_value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Create operation missing new_value"))?;

        let name = new_value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&edit.target_id)
            .to_string();

        let properties = new_value
            .get("properties")
            .map(serde_json::to_string)
            .transpose()?;

        let background_color = new_value
            .get("background_color")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let text_color = new_value
            .get("text_color")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let border_color = new_value
            .get("border_color")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let comment = new_value
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let alias = new_value
            .get("alias")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let layer = graph_layers::ActiveModel {
            id: Set(Default::default()),
            graph_id: Set(edit.graph_id),
            layer_id: Set(edit.target_id.clone()),
            name: Set(name),
            background_color: Set(background_color),
            text_color: Set(text_color),
            border_color: Set(border_color),
            alias: Set(alias),
            comment: Set(comment),
            properties: Set(properties),
            dataset_id: Set(None),
        };

        layer.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created layer {}", edit.target_id),
        })
    }

    async fn update_layer(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let layer = Layers::find()
            .filter(graph_layers::Column::GraphId.eq(edit.graph_id))
            .filter(graph_layers::Column::LayerId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(layer) = layer else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Layer {} not found", edit.target_id),
            });
        };

        let mut active_model: graph_layers::ActiveModel = layer.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "name" => {
                        if let Some(name) = new_value.as_str() {
                            active_model.name = Set(name.to_string());
                        }
                    }
                    "background_color" => {
                        active_model.background_color =
                            Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "text_color" => {
                        active_model.text_color = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "border_color" => {
                        active_model.border_color = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "comment" => {
                        active_model.comment = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "alias" => {
                        let alias_value = new_value
                            .as_str()
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        active_model.alias = Set(alias_value);
                    }
                    "properties" => {
                        let properties_string = serde_json::to_string(new_value)?;
                        active_model.properties = Set(Some(properties_string));
                    }
                    _ => {
                        return Ok(ApplyResult::Skipped {
                            reason: format!("Unknown field: {}", field_name),
                        });
                    }
                }
            }
        }

        active_model.update(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!(
                "Updated layer {} field {:?}",
                edit.target_id, edit.field_name
            ),
        })
    }

    async fn delete_layer(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let result = Layers::delete_many()
            .filter(graph_layers::Column::GraphId.eq(edit.graph_id))
            .filter(graph_layers::Column::LayerId.eq(&edit.target_id))
            .exec(&self.db)
            .await?;

        if result.rows_affected == 0 {
            Ok(ApplyResult::Skipped {
                reason: format!("Layer {} not found", edit.target_id),
            })
        } else {
            Ok(ApplyResult::Success {
                message: format!("Deleted layer {}", edit.target_id),
            })
        }
    }
}
