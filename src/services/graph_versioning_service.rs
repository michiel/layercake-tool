use std::collections::HashMap;
use anyhow::{Result, Context};
use chrono::Utc;
use sea_orm::*;
use tracing::{info, error, debug};
use serde_json::json;

use crate::database::entities::{
    graph_snapshots, graph_versions, snapshot_data, nodes, edges, layers,
    graph_versions::{ChangeType, EntityType},
};

/// Service for managing graph versioning and snapshots
#[derive(Clone)]
pub struct GraphVersioningService {
    db: DatabaseConnection,
}

impl GraphVersioningService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a new graph snapshot
    pub async fn create_snapshot(
        &self,
        project_id: i32,
        name: String,
        description: Option<String>,
        is_automatic: bool,
        created_by: Option<String>,
    ) -> Result<graph_snapshots::Model> {
        let txn = self.db.begin().await?;

        // Get next version number
        let last_version = graph_snapshots::Entity::find()
            .filter(graph_snapshots::Column::ProjectId.eq(project_id))
            .order_by_desc(graph_snapshots::Column::Version)
            .one(&txn)
            .await?
            .map(|s| s.version)
            .unwrap_or(0);

        // Count current entities
        let node_count = nodes::Entity::find()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .count(&txn)
            .await? as i32;

        let edge_count = edges::Entity::find()
            .filter(edges::Column::ProjectId.eq(project_id))
            .count(&txn)
            .await? as i32;

        let layer_count = layers::Entity::find()
            .filter(layers::Column::ProjectId.eq(project_id))
            .count(&txn)
            .await? as i32;

        // Create snapshot record
        let snapshot = graph_snapshots::ActiveModel {
            project_id: Set(project_id),
            name: Set(name),
            description: Set(description),
            version: Set(last_version + 1),
            is_automatic: Set(is_automatic),
            created_at: Set(Utc::now()),
            created_by: Set(created_by),
            node_count: Set(node_count),
            edge_count: Set(edge_count),
            layer_count: Set(layer_count),
            ..Default::default()
        };

        let snapshot = graph_snapshots::Entity::insert(snapshot)
            .exec_with_returning(&txn)
            .await?;

        // Store snapshot data
        self.store_snapshot_data(&txn, &snapshot).await?;

        txn.commit().await?;

        info!(
            "Created snapshot {} for project {} with {} entities",
            snapshot.name,
            project_id,
            snapshot.total_entities()
        );

        Ok(snapshot)
    }

    /// Store current graph data for a snapshot
    async fn store_snapshot_data(
        &self,
        txn: &DatabaseTransaction,
        snapshot: &graph_snapshots::Model,
    ) -> Result<()> {
        let now = Utc::now();
        let mut snapshot_data_vec = Vec::new();

        // Store nodes
        let nodes = nodes::Entity::find()
            .filter(nodes::Column::ProjectId.eq(snapshot.project_id))
            .all(txn)
            .await?;

        for node in nodes {
            let data = snapshot_data::ActiveModel {
                snapshot_id: Set(snapshot.id),
                entity_type: Set(EntityType::Node.into()),
                entity_id: Set(node.node_id.clone()),
                entity_data: Set(json!({
                    "id": node.id,
                    "project_id": node.project_id,
                    "node_id": node.node_id,
                    "label": node.label,
                    "layer_id": node.layer_id,
                    "properties": node.properties
                })),
                created_at: Set(now),
                ..Default::default()
            };
            snapshot_data_vec.push(data);
        }

        // Store edges
        let edges = edges::Entity::find()
            .filter(edges::Column::ProjectId.eq(snapshot.project_id))
            .all(txn)
            .await?;

        for edge in edges {
            let data = snapshot_data::ActiveModel {
                snapshot_id: Set(snapshot.id),
                entity_type: Set(EntityType::Edge.into()),
                entity_id: Set(format!("{}-{}", edge.source_node_id, edge.target_node_id)),
                entity_data: Set(json!({
                    "id": edge.id,
                    "project_id": edge.project_id,
                    "source_node_id": edge.source_node_id,
                    "target_node_id": edge.target_node_id,
                    "properties": edge.properties
                })),
                created_at: Set(now),
                ..Default::default()
            };
            snapshot_data_vec.push(data);
        }

        // Store layers
        let layers = layers::Entity::find()
            .filter(layers::Column::ProjectId.eq(snapshot.project_id))
            .all(txn)
            .await?;

        for layer in layers {
            let data = snapshot_data::ActiveModel {
                snapshot_id: Set(snapshot.id),
                entity_type: Set(EntityType::Layer.into()),
                entity_id: Set(layer.layer_id.clone()),
                entity_data: Set(json!({
                    "id": layer.id,
                    "project_id": layer.project_id,
                    "layer_id": layer.layer_id,
                    "name": layer.name,
                    "color": layer.color,
                    "properties": layer.properties
                })),
                created_at: Set(now),
                ..Default::default()
            };
            snapshot_data_vec.push(data);
        }

        // Insert all snapshot data
        if !snapshot_data_vec.is_empty() {
            snapshot_data::Entity::insert_many(snapshot_data_vec)
                .exec(txn)
                .await?;
        }

        debug!(
            "Stored {} entities in snapshot {}",
            snapshot.total_entities(),
            snapshot.id
        );

        Ok(())
    }

    /// Record a change in the graph
    pub async fn record_change(
        &self,
        project_id: i32,
        change_type: ChangeType,
        entity_type: EntityType,
        entity_id: String,
        old_data: Option<serde_json::Value>,
        new_data: Option<serde_json::Value>,
        changed_by: Option<String>,
        description: Option<String>,
    ) -> Result<graph_versions::Model> {
        let version = graph_versions::ActiveModel {
            project_id: Set(project_id),
            change_type: Set(change_type.into()),
            entity_type: Set(entity_type.into()),
            entity_id: Set(entity_id),
            old_data: Set(old_data.map(|v| v.into())),
            new_data: Set(new_data.map(|v| v.into())),
            changed_at: Set(Utc::now()),
            changed_by: Set(changed_by),
            change_description: Set(description),
            ..Default::default()
        };

        let version = graph_versions::Entity::insert(version)
            .exec_with_returning(&self.db)
            .await?;

        debug!(
            "Recorded {} change for {} {}",
            version.change_type, version.entity_type, version.entity_id
        );

        Ok(version)
    }

    /// Get all snapshots for a project
    pub async fn list_snapshots(
        &self,
        project_id: i32,
    ) -> Result<Vec<graph_snapshots::Model>> {
        graph_snapshots::Entity::find()
            .filter(graph_snapshots::Column::ProjectId.eq(project_id))
            .order_by_desc(graph_snapshots::Column::CreatedAt)
            .all(&self.db)
            .await
            .context("Failed to fetch snapshots")
    }

    /// Get a specific snapshot
    pub async fn get_snapshot(
        &self,
        snapshot_id: i32,
    ) -> Result<Option<graph_snapshots::Model>> {
        graph_snapshots::Entity::find_by_id(snapshot_id)
            .one(&self.db)
            .await
            .context("Failed to fetch snapshot")
    }

    /// Get changes for a project
    pub async fn get_changes(
        &self,
        project_id: i32,
        limit: Option<u64>,
    ) -> Result<Vec<graph_versions::Model>> {
        let mut query = graph_versions::Entity::find()
            .filter(graph_versions::Column::ProjectId.eq(project_id))
            .order_by_desc(graph_versions::Column::ChangedAt);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }

        query
            .all(&self.db)
            .await
            .context("Failed to fetch changes")
    }

    /// Restore from a snapshot
    pub async fn restore_from_snapshot(
        &self,
        project_id: i32,
        snapshot_id: i32,
        restored_by: Option<String>,
    ) -> Result<()> {
        let txn = self.db.begin().await?;

        // Verify snapshot exists and belongs to project
        let snapshot = graph_snapshots::Entity::find_by_id(snapshot_id)
            .one(&txn)
            .await?
            .context("Snapshot not found")?;

        if snapshot.project_id != project_id {
            return Err(anyhow::anyhow!("Snapshot does not belong to project"));
        }

        // Clear current data
        nodes::Entity::delete_many()
            .filter(nodes::Column::ProjectId.eq(project_id))
            .exec(&txn)
            .await?;

        edges::Entity::delete_many()
            .filter(edges::Column::ProjectId.eq(project_id))
            .exec(&txn)
            .await?;

        layers::Entity::delete_many()
            .filter(layers::Column::ProjectId.eq(project_id))
            .exec(&txn)
            .await?;

        // Restore data from snapshot
        let snapshot_data = snapshot_data::Entity::find()
            .filter(snapshot_data::Column::SnapshotId.eq(snapshot_id))
            .all(&txn)
            .await?;

        for data in snapshot_data {
            match data.get_entity_type() {
                EntityType::Node => {
                    if let Ok(node_data) = serde_json::from_value::<nodes::Model>(data.entity_data.clone()) {
                        let node = nodes::ActiveModel {
                            project_id: Set(project_id),
                            node_id: Set(node_data.node_id),
                            label: Set(node_data.label),
                            layer_id: Set(node_data.layer_id),
                            properties: Set(node_data.properties),
                            ..Default::default()
                        };
                        nodes::Entity::insert(node).exec(&txn).await?;
                    }
                }
                EntityType::Edge => {
                    if let Ok(edge_data) = serde_json::from_value::<edges::Model>(data.entity_data.clone()) {
                        let edge = edges::ActiveModel {
                            project_id: Set(project_id),
                            source_node_id: Set(edge_data.source_node_id),
                            target_node_id: Set(edge_data.target_node_id),
                            properties: Set(edge_data.properties),
                            ..Default::default()
                        };
                        edges::Entity::insert(edge).exec(&txn).await?;
                    }
                }
                EntityType::Layer => {
                    if let Ok(layer_data) = serde_json::from_value::<layers::Model>(data.entity_data.clone()) {
                        let layer = layers::ActiveModel {
                            project_id: Set(project_id),
                            layer_id: Set(layer_data.layer_id),
                            name: Set(layer_data.name),
                            color: Set(layer_data.color),
                            properties: Set(layer_data.properties),
                            ..Default::default()
                        };
                        layers::Entity::insert(layer).exec(&txn).await?;
                    }
                }
            }
        }

        // Record the restore operation
        let restore_version = graph_versions::ActiveModel {
            project_id: Set(project_id),
            snapshot_id: Set(Some(snapshot_id)),
            change_type: Set(ChangeType::Restore.into()),
            entity_type: Set("project".to_string()),
            entity_id: Set(project_id.to_string()),
            changed_at: Set(Utc::now()),
            changed_by: Set(restored_by),
            change_description: Set(Some(format!(
                "Restored from snapshot '{}' (v{})",
                snapshot.name, snapshot.version
            ))),
            ..Default::default()
        };

        graph_versions::Entity::insert(restore_version)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!(
            "Restored project {} from snapshot '{}' (v{})",
            project_id, snapshot.name, snapshot.version
        );

        Ok(())
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: i32) -> Result<bool> {
        let txn = self.db.begin().await?;

        // Check if snapshot exists
        let snapshot = graph_snapshots::Entity::find_by_id(snapshot_id)
            .one(&txn)
            .await?;

        if snapshot.is_none() {
            return Ok(false);
        }

        // Delete snapshot data first (due to foreign key constraints)
        snapshot_data::Entity::delete_many()
            .filter(snapshot_data::Column::SnapshotId.eq(snapshot_id))
            .exec(&txn)
            .await?;

        // Delete the snapshot
        graph_snapshots::Entity::delete_by_id(snapshot_id)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        info!("Deleted snapshot {}", snapshot_id);
        Ok(true)
    }

    /// Get snapshot data for a specific entity type
    pub async fn get_snapshot_data(
        &self,
        snapshot_id: i32,
        entity_type: Option<EntityType>,
    ) -> Result<Vec<snapshot_data::Model>> {
        let mut query = snapshot_data::Entity::find()
            .filter(snapshot_data::Column::SnapshotId.eq(snapshot_id));

        if let Some(entity_type) = entity_type {
            query = query.filter(snapshot_data::Column::EntityType.eq(String::from(entity_type)));
        }

        query
            .order_by_asc(snapshot_data::Column::EntityType)
            .order_by_asc(snapshot_data::Column::EntityId)
            .all(&self.db)
            .await
            .context("Failed to fetch snapshot data")
    }

    /// Create automatic snapshot if needed
    pub async fn create_auto_snapshot_if_needed(
        &self,
        project_id: i32,
        threshold_changes: u32,
    ) -> Result<Option<graph_snapshots::Model>> {
        // Count recent changes since last snapshot
        let last_snapshot = graph_snapshots::Entity::find()
            .filter(graph_snapshots::Column::ProjectId.eq(project_id))
            .order_by_desc(graph_snapshots::Column::CreatedAt)
            .one(&self.db)
            .await?;

        let changes_since = if let Some(last_snapshot) = last_snapshot {
            graph_versions::Entity::find()
                .filter(graph_versions::Column::ProjectId.eq(project_id))
                .filter(graph_versions::Column::ChangedAt.gt(last_snapshot.created_at))
                .count(&self.db)
                .await?
        } else {
            // No snapshots yet, count all changes
            graph_versions::Entity::find()
                .filter(graph_versions::Column::ProjectId.eq(project_id))
                .count(&self.db)
                .await?
        };

        if changes_since >= threshold_changes as u64 {
            let snapshot = self
                .create_snapshot(
                    project_id,
                    format!("Auto-snapshot at {} changes", changes_since),
                    Some(format!("Automatic snapshot after {} changes", changes_since)),
                    true,
                    Some("system".to_string()),
                )
                .await?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }
}
