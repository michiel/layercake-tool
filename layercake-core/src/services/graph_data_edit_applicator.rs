use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use tracing::{debug, warn};

use crate::database::entities::{
    graph_data_edges::{self, Entity as GraphDataEdges},
    graph_data_nodes::{self, Entity as GraphDataNodes},
    graph_edits,
};

/// Result of applying a single edit
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyResult {
    Success { message: String },
    Skipped { reason: String },
    Error { reason: String },
}

/// Applies graph edits to graph_data entities (unified model)
///
/// This is the Phase 3 replacement for GraphEditApplicator that targets the
/// unified graph_data tables instead of the legacy graphs/graph_nodes/graph_edges.
pub struct GraphDataEditApplicator {
    db: DatabaseConnection,
}

impl GraphDataEditApplicator {
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
            "layer" => {
                // Layer edits are skipped for graph_data - Phase 5 will remove layer storage
                ApplyResult::Skipped {
                    reason: "Layer edits not supported for graph_data (use project palette instead)".to_string(),
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

    /// Apply edit to a graph_data node
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
        let existing = GraphDataNodes::find()
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

        let weight = new_value
            .get("weight")
            .and_then(|v| v.as_f64());

        let is_partition = new_value
            .get("isPartition")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let belongs_to = new_value
            .get("belongsTo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let attributes = new_value.get("attributes").cloned();

        let comment = new_value
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let node = graph_data_nodes::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_data_id: Set(edit.graph_id),
            external_id: Set(edit.target_id.clone()),
            label: Set(label),
            layer: Set(layer),
            weight: Set(weight),
            is_partition: Set(is_partition),
            belongs_to: Set(belongs_to),
            comment: Set(comment),
            source_dataset_id: Set(None), // Edits don't track source
            attributes: Set(attributes),
            created_at: Set(chrono::Utc::now()),
        };

        node.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created node {}", edit.target_id),
        })
    }

    async fn update_node(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let node = GraphDataNodes::find()
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
                    "weight" => {
                        active_model.weight = Set(new_value.as_f64());
                    }
                    "isPartition" => {
                        if let Some(is_partition) = new_value.as_bool() {
                            active_model.is_partition = Set(is_partition);
                        }
                    }
                    "belongsTo" => {
                        active_model.belongs_to = Set(new_value.as_str().map(|s| s.to_string()));
                    }
                    "attributes" => {
                        active_model.attributes = Set(Some(new_value.clone()));
                    }
                    "comment" => {
                        active_model.comment = Set(new_value.as_str().map(|s| s.to_string()));
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
        let result = GraphDataNodes::delete_many()
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

    /// Apply edit to a graph_data edge
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
        let existing = GraphDataEdges::find()
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
            .ok_or_else(|| anyhow::anyhow!("Edge create missing source"))?
            .to_string();

        let target = new_value
            .get("target")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Edge create missing target"))?
            .to_string();

        // Verify source and target nodes exist
        let source_exists = GraphDataNodes::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_nodes::Column::ExternalId.eq(&source))
            .count(&self.db)
            .await?
            > 0;

        let target_exists = GraphDataNodes::find()
            .filter(graph_data_nodes::Column::GraphDataId.eq(edit.graph_id))
            .filter(graph_data_nodes::Column::ExternalId.eq(&target))
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

        let weight = new_value
            .get("weight")
            .and_then(|v| v.as_f64());

        let attributes = new_value.get("attributes").cloned();

        let comment = new_value
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let edge = graph_data_edges::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_data_id: Set(edit.graph_id),
            external_id: Set(edit.target_id.clone()),
            source: Set(source),
            target: Set(target),
            label: Set(label),
            layer: Set(layer),
            weight: Set(weight),
            comment: Set(comment),
            source_dataset_id: Set(None), // Edits don't track source
            attributes: Set(attributes),
            created_at: Set(chrono::Utc::now()),
        };

        edge.insert(&self.db).await?;

        Ok(ApplyResult::Success {
            message: format!("Created edge {}", edit.target_id),
        })
    }

    async fn update_edge(&self, edit: &graph_edits::Model) -> Result<ApplyResult> {
        let edge = GraphDataEdges::find()
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
                    "weight" => {
                        active_model.weight = Set(new_value.as_f64());
                    }
                    "attributes" => {
                        active_model.attributes = Set(Some(new_value.clone()));
                    }
                    "comment" => {
                        active_model.comment = Set(new_value.as_str().map(|s| s.to_string()));
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
        let result = GraphDataEdges::delete_many()
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
}
