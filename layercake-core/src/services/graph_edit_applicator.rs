use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use tracing::{debug, warn};

use crate::database::entities::{
    graph_edges::{self, Entity as GraphEdges},
    graph_edits,
    graph_nodes::{self, Entity as GraphNodes},
    layers::{self, Entity as Layers},
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
        debug!(
            "Applying edit #{} on {}:{} ({})",
            edit.sequence_number, edit.target_type, edit.target_id, edit.operation
        );

        let result = match edit.target_type.as_str() {
            "node" => self.apply_node_edit(edit).await?,
            "edge" => self.apply_edge_edit(edit).await?,
            "layer" => self.apply_layer_edit(edit).await?,
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
            datasource_id: Set(None),
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
            datasource_id: Set(None),
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
            .filter(layers::Column::GraphId.eq(edit.graph_id))
            .filter(layers::Column::LayerId.eq(&edit.target_id))
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

        let color = new_value
            .get("color")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let layer = layers::ActiveModel {
            id: Set(Default::default()),
            graph_id: Set(edit.graph_id),
            layer_id: Set(edit.target_id.clone()),
            name: Set(name),
            color: Set(color),
            properties: Set(properties),
            datasource_id: Set(None),
        };

        layer.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created layer {}", edit.target_id),
        })
    }

    async fn update_layer(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let layer = Layers::find()
            .filter(layers::Column::GraphId.eq(edit.graph_id))
            .filter(layers::Column::LayerId.eq(&edit.target_id))
            .one(&self.db)
            .await?;

        let Some(layer) = layer else {
            return Ok(ApplyResult::Skipped {
                reason: format!("Layer {} not found", edit.target_id),
            });
        };

        let mut active_model: layers::ActiveModel = layer.into();

        if let Some(field_name) = &edit.field_name {
            if let Some(new_value) = &edit.new_value {
                match field_name.as_str() {
                    "name" => {
                        if let Some(name) = new_value.as_str() {
                            active_model.name = Set(name.to_string());
                        }
                    }
                    "color" => {
                        active_model.color = Set(new_value.as_str().map(|s| s.to_string()));
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
            .filter(layers::Column::GraphId.eq(edit.graph_id))
            .filter(layers::Column::LayerId.eq(&edit.target_id))
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
