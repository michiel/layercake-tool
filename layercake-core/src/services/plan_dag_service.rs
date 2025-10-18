use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans};
use crate::graphql::mutations::plan_dag_delta;
use crate::graphql::types::{PlanDagEdge, PlanDagNode, Position};
use crate::services::ValidationService;

/// Service layer for Plan DAG operations
/// Separates business logic from GraphQL mutation layer
#[derive(Clone)]
pub struct PlanDagService {
    db: DatabaseConnection,
}

impl PlanDagService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get or create a plan for a project
    /// Auto-creates a default plan if one doesn't exist
    pub async fn get_or_create_plan(&self, project_id: i32) -> Result<plans::Model> {
        match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
        {
            Some(plan) => Ok(plan),
            None => {
                let now = Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(project_id),
                    name: Set(format!("Plan for Project {}", project_id)),
                    yaml_content: Set("".to_string()),
                    dependencies: Set(None),
                    status: Set("draft".to_string()),
                    version: Set(1),
                    created_at: Set(now),
                    updated_at: Set(now),
                };
                new_plan
                    .insert(&self.db)
                    .await
                    .map_err(|e| anyhow!("Failed to create plan: {}", e))
            }
        }
    }

    /// Create a new Plan DAG node
    pub async fn create_node(
        &self,
        project_id: i32,
        node_id: String,
        node_type: String,
        position: Position,
        metadata_json: String,
        config_json: String,
    ) -> Result<PlanDagNode> {
        // Validate inputs
        let validated_id = ValidationService::validate_node_id(&node_id)?;
        let validated_type = ValidationService::validate_plan_dag_node_type(&node_type)?;
        let (validated_x, validated_y) =
            ValidationService::validate_plan_dag_position(position.x, position.y)?;
        let validated_metadata = ValidationService::validate_plan_dag_metadata(&metadata_json)?;
        let validated_config = ValidationService::validate_plan_dag_config(&config_json)?;

        let plan = self.get_or_create_plan(project_id).await?;

        // Fetch current state for delta generation and validation
        let (current_nodes, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to fetch current state: {}", e))?;

        // Validate DAG limits
        ValidationService::validate_plan_dag_limits(current_nodes.len() + 1, current_edges.len())?;

        // Create the node with validated values
        let now = Utc::now();
        let node = plan_dag_nodes::ActiveModel {
            id: Set(validated_id),
            plan_id: Set(plan.id),
            node_type: Set(validated_type),
            position_x: Set(validated_x),
            position_y: Set(validated_y),
            source_position: Set(None),
            target_position: Set(None),
            metadata_json: Set(serde_json::to_string(&validated_metadata)?),
            config_json: Set(serde_json::to_string(&validated_config)?),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created_node = node
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create node: {}", e))?;

        let result_node = PlanDagNode::from(created_node);

        // Generate delta and broadcast
        let index = current_nodes.len();
        let patch_op = plan_dag_delta::generate_node_add_patch(&result_node, index);

        let new_version = plan_dag_delta::increment_plan_version(&self.db, plan.id)
            .await
            .map_err(|e| anyhow!("Failed to increment version: {}", e))?;

        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, vec![patch_op])
            .await
            .ok(); // Non-fatal if broadcast fails

        Ok(result_node)
    }

    /// Update a Plan DAG node
    pub async fn update_node(
        &self,
        project_id: i32,
        node_id: String,
        position: Option<Position>,
        metadata_json: Option<String>,
        config_json: Option<String>,
    ) -> Result<PlanDagNode> {
        let plan = self.get_or_create_plan(project_id).await?;

        // Fetch current state for delta generation
        let (current_nodes, _) = plan_dag_delta::fetch_current_plan_dag(&self.db, plan.id)
            .await
            .map_err(|e| anyhow!("Failed to fetch current state: {}", e))?;

        // Find the node
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Node not found"))?;

        let mut node_active: plan_dag_nodes::ActiveModel = node.into();
        let mut patch_ops = Vec::new();

        // Update position if provided
        if let Some(pos) = position {
            node_active.position_x = Set(pos.x);
            node_active.position_y = Set(pos.y);

            patch_ops.extend(plan_dag_delta::generate_node_position_patch(
                &node_id,
                pos.x,
                pos.y,
                &current_nodes,
            ));
        }

        // Update metadata if provided
        if let Some(metadata) = &metadata_json {
            node_active.metadata_json = Set(metadata.clone());

            if let Some(patch) = plan_dag_delta::generate_node_update_patch(
                &node_id,
                "metadata",
                serde_json::from_str(metadata).unwrap_or(serde_json::Value::Null),
                &current_nodes,
            ) {
                patch_ops.push(patch);
            }
        }

        // Update config if provided
        if let Some(config) = &config_json {
            node_active.config_json = Set(config.clone());

            if let Some(patch) = plan_dag_delta::generate_node_update_patch(
                &node_id,
                "config",
                serde_json::from_str(config).unwrap_or(serde_json::Value::Null),
                &current_nodes,
            ) {
                patch_ops.push(patch);
            }
        }

        node_active.updated_at = Set(Utc::now());
        let updated_node = node_active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to update node: {}", e))?;

        let result_node = PlanDagNode::from(updated_node);

        // Increment plan version and broadcast delta
        if !patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to increment version: {}", e))?;

            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                .await
                .ok(); // Non-fatal if broadcast fails
        }

        Ok(result_node)
    }

    /// Delete a Plan DAG node and its connected edges
    pub async fn delete_node(&self, project_id: i32, node_id: String) -> Result<PlanDagNode> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (current_nodes, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to fetch current state: {}", e))?;

        // Find the node to delete
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Node not found"))?;

        let result_node = PlanDagNode::from(node);

        // Generate delta for node deletion
        let mut patch_ops = Vec::new();
        if let Some(patch) = plan_dag_delta::generate_node_delete_patch(&node_id, &current_nodes) {
            patch_ops.push(patch);
        }

        // Find and delete connected edges, generating deltas for each
        let connected_edges: Vec<&PlanDagEdge> = current_edges
            .iter()
            .filter(|e| e.source == node_id || e.target == node_id)
            .collect();

        for edge in &connected_edges {
            if let Some(patch) =
                plan_dag_delta::generate_edge_delete_patch(&edge.id, &current_edges)
            {
                patch_ops.push(patch);
            }
        }

        // Delete edges connected to this node first
        plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId.eq(plan.id).and(
                    plan_dag_edges::Column::SourceNodeId
                        .eq(&node_id)
                        .or(plan_dag_edges::Column::TargetNodeId.eq(&node_id)),
                ),
            )
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete connected edges: {}", e))?;

        // Delete the node
        plan_dag_nodes::Entity::delete_many()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete node: {}", e))?;

        // Increment plan version and broadcast delta
        if !patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to increment version: {}", e))?;

            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                .await
                .ok(); // Non-fatal if broadcast fails
        }

        Ok(result_node)
    }

    /// Move a Plan DAG node to a new position
    pub async fn move_node(
        &self,
        project_id: i32,
        node_id: String,
        position: Position,
    ) -> Result<PlanDagNode> {
        self.update_node(project_id, node_id, Some(position), None, None)
            .await
    }

    /// Create a new Plan DAG edge
    pub async fn create_edge(
        &self,
        project_id: i32,
        edge_id: String,
        source_node_id: String,
        target_node_id: String,
        metadata_json: String,
    ) -> Result<PlanDagEdge> {
        // Validate inputs
        let validated_edge_id = ValidationService::validate_node_id(&edge_id)?;
        let validated_source = ValidationService::validate_node_id(&source_node_id)?;
        let validated_target = ValidationService::validate_node_id(&target_node_id)?;
        let validated_metadata = ValidationService::validate_plan_dag_metadata(&metadata_json)?;

        // Validate no self-loop
        ValidationService::validate_edge_no_self_loop(&validated_source, &validated_target)?;

        let plan = self.get_or_create_plan(project_id).await?;

        // Fetch current state for delta generation and validation
        let (current_nodes, current_edges) =
            plan_dag_delta::fetch_current_plan_dag(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to fetch current state: {}", e))?;

        // Validate DAG limits
        ValidationService::validate_plan_dag_limits(current_nodes.len(), current_edges.len() + 1)?;

        // Create the edge with validated values
        let now = Utc::now();
        let edge = plan_dag_edges::ActiveModel {
            id: Set(validated_edge_id),
            plan_id: Set(plan.id),
            source_node_id: Set(validated_source),
            target_node_id: Set(validated_target),
            // Removed source_handle and target_handle for floating edges
            metadata_json: Set(serde_json::to_string(&validated_metadata)?),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created_edge = edge
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create edge: {}", e))?;

        let result_edge = PlanDagEdge::from(created_edge);

        // Generate delta and broadcast
        let index = current_edges.len();
        let patch_op = plan_dag_delta::generate_edge_add_patch(&result_edge, index);

        let new_version = plan_dag_delta::increment_plan_version(&self.db, plan.id)
            .await
            .map_err(|e| anyhow!("Failed to increment version: {}", e))?;

        let user_id = "demo_user".to_string(); // TODO: Get from auth context
        plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, vec![patch_op])
            .await
            .ok(); // Non-fatal if broadcast fails

        Ok(result_edge)
    }

    /// Delete a Plan DAG edge
    pub async fn delete_edge(&self, project_id: i32, edge_id: String) -> Result<PlanDagEdge> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Plan not found for project"))?;

        // Fetch current state for delta generation
        let (_, current_edges) = plan_dag_delta::fetch_current_plan_dag(&self.db, plan.id)
            .await
            .map_err(|e| anyhow!("Failed to fetch current state: {}", e))?;

        // Find the edge to delete
        let edge = plan_dag_edges::Entity::find()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Edge not found"))?;

        let result_edge = PlanDagEdge::from(edge);

        // Generate delta for edge deletion
        let mut patch_ops = Vec::new();
        if let Some(patch) = plan_dag_delta::generate_edge_delete_patch(&edge_id, &current_edges) {
            patch_ops.push(patch);
        }

        // Delete the edge
        plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to delete edge: {}", e))?;

        // Increment plan version and broadcast delta
        if !patch_ops.is_empty() {
            let new_version = plan_dag_delta::increment_plan_version(&self.db, plan.id)
                .await
                .map_err(|e| anyhow!("Failed to increment version: {}", e))?;

            let user_id = "demo_user".to_string(); // TODO: Get from auth context
            plan_dag_delta::publish_plan_dag_delta(project_id, new_version, user_id, patch_ops)
                .await
                .ok(); // Non-fatal if broadcast fails
        }

        Ok(result_edge)
    }

    /// Get all nodes for a project's plan
    pub async fn get_nodes(&self, project_id: i32) -> Result<Vec<PlanDagNode>> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Plan not found for project"))?;

        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .order_by_asc(plan_dag_nodes::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(nodes.into_iter().map(PlanDagNode::from).collect())
    }

    /// Get all edges for a project's plan
    pub async fn get_edges(&self, project_id: i32) -> Result<Vec<PlanDagEdge>> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?
            .ok_or_else(|| anyhow!("Plan not found for project"))?;

        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .order_by_asc(plan_dag_edges::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Database error: {}", e))?;

        Ok(edges.into_iter().map(PlanDagEdge::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These would be integration tests requiring a test database
    // For now, just testing the service creation

    #[test]
    fn test_service_creation() {
        // This would require a real database connection for proper testing
        // We'll add integration tests when we have a test database setup
    }
}
