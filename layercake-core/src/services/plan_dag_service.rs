use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::database::entities::{data_sets, plan_dag_edges, plan_dag_nodes, plans};
use crate::errors::{CoreError, CoreResult};
use crate::plan_dag::{PlanDagEdge, PlanDagNode, Position};
use crate::services::ValidationService;
use serde_json::Value;

/// Service layer for Plan DAG operations
/// Separates business logic from GraphQL mutation layer
#[derive(Clone)]
pub struct PlanDagService {
    db: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct PlanDagMigrationDetail {
    pub node_id: String,
    pub from_type: String,
    pub to_type: String,
    pub note: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct PlanDagMigrationOutcome {
    pub checked_nodes: usize,
    pub migrated_nodes: Vec<PlanDagMigrationDetail>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Clone)]
pub struct PlanDagNodePositionUpdate {
    pub node_id: String,
    pub position: Position,
    pub source_position: Option<String>,
    pub target_position: Option<String>,
}

impl PlanDagService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Get or create a plan for a project
    /// Auto-creates a default plan if one doesn't exist
    pub async fn get_or_create_plan(&self, project_id: i32) -> CoreResult<plans::Model> {
        match plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_asc(plans::Column::Id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
        {
            Some(plan) => {
                if Self::needs_default_plan_name(&plan, project_id) {
                    let mut active: plans::ActiveModel = plan.clone().into();
                    active.name = Set(Self::default_plan_name().to_string());
                    active.updated_at = Set(Utc::now());
                    let plan = active
                        .update(&self.db)
                        .await
                        .map_err(|e| CoreError::internal(format!("Failed to rename plan: {}", e)))?;
                    Ok(plan)
                } else {
                    Ok(plan)
                }
            }
            None => {
                let now = Utc::now();
                let new_plan = plans::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    project_id: Set(project_id),
                    name: Set(Self::default_plan_name().to_string()),
                    description: Set(None),
                    tags: Set("[]".to_string()),
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
                    .map_err(|e| CoreError::internal(format!("Failed to create plan: {}", e)))
            }
        }
    }

    fn default_plan_name() -> &'static str {
        "Main plan"
    }

    fn needs_default_plan_name(plan: &plans::Model, project_id: i32) -> bool {
        let trimmed = plan.name.trim();
        if trimmed.is_empty() {
            return true;
        }

        let legacy_name = format!("Plan for Project {}", project_id);
        trimmed.eq_ignore_ascii_case(&legacy_name)
    }

    async fn resolve_plan(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<plans::Model> {
        if let Some(plan_id) = plan_id {
            let plan = plans::Entity::find_by_id(plan_id)
                .one(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
                .ok_or_else(|| CoreError::not_found("Plan", plan_id.to_string()))?;

            if plan.project_id != project_id {
                return Err(CoreError::validation(format!(
                    "Plan {} does not belong to project {}",
                    plan_id, project_id
                )));
            }

            Ok(plan)
        } else {
            self.get_or_create_plan(project_id).await
        }
    }

    async fn fetch_current_plan_dag(
        &self,
        plan_id: i32,
    ) -> CoreResult<(Vec<PlanDagNode>, Vec<PlanDagEdge>)> {
        let dag_nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan_id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let dag_edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan_id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let nodes = dag_nodes.into_iter().map(PlanDagNode::from).collect();
        let edges = dag_edges.into_iter().map(PlanDagEdge::from).collect();

        Ok((nodes, edges))
    }

    async fn bump_plan_version(&self, plan_id: i32) -> CoreResult<i32> {
        use sea_orm::{ActiveModelTrait, Set};

        let plan = plans::Entity::find_by_id(plan_id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("Plan", plan_id.to_string()))?;

        let new_version = plan.version + 1;

        let mut plan_active: plans::ActiveModel = plan.into();
        plan_active.version = Set(new_version);
        plan_active.updated_at = Set(chrono::Utc::now());
        plan_active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update plan version: {}", e)))?;

        Ok(new_version)
    }

    /// Create a new Plan DAG node
    pub async fn create_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        node_type: String,
        position: Position,
        metadata_json: String,
        config_json: String,
    ) -> CoreResult<PlanDagNode> {
        // Validate inputs
        let validated_id = ValidationService::validate_node_id(&node_id)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_type = ValidationService::validate_plan_dag_node_type(&node_type)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let (validated_x, validated_y) =
            ValidationService::validate_plan_dag_position(position.x, position.y)
                .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_metadata = ValidationService::validate_plan_dag_metadata(&metadata_json)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_config = ValidationService::validate_plan_dag_config(&config_json)
            .map_err(|e| CoreError::validation(e.to_string()))?;

        let plan = self.resolve_plan(project_id, plan_id).await?;

        let (current_nodes, current_edges) = self
            .fetch_current_plan_dag(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to fetch current state: {}", e)))?;

        // Validate DAG limits
        ValidationService::validate_plan_dag_limits(current_nodes.len() + 1, current_edges.len())
            .map_err(|e| CoreError::validation(e.to_string()))?;

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
            metadata_json: Set(
                serde_json::to_string(&validated_metadata)
                    .map_err(|e| CoreError::validation(format!("Invalid metadata: {}", e)))?,
            ),
            config_json: Set(
                serde_json::to_string(&validated_config)
                    .map_err(|e| CoreError::validation(format!("Invalid config: {}", e)))?,
            ),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created_node = node
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to create node: {}", e)))?;

        let result_node = PlanDagNode::from(created_node);

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_node)
    }

    /// Update a Plan DAG node
    pub async fn update_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        position: Option<Position>,
        metadata_json: Option<String>,
        config_json: Option<String>,
    ) -> CoreResult<PlanDagNode> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Find the node
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDagNode", node_id.clone()))?;

        let node_type = node.node_type.clone();
        let mut metadata_json_current = node.metadata_json.clone();

        let mut node_active: plan_dag_nodes::ActiveModel = node.into();

        // Update position if provided
        if let Some(pos) = position {
            node_active.position_x = Set(pos.x);
            node_active.position_y = Set(pos.y);
        }

        // Update metadata if provided
        if let Some(metadata) = &metadata_json {
            node_active.metadata_json = Set(metadata.clone());
            metadata_json_current = metadata.clone();
        }

        // Update config if provided
        if let Some(config) = &config_json {
            node_active.config_json = Set(config.clone());

            if node_type == "DataSetNode" {
                if let Ok(config_value) = serde_json::from_str::<Value>(config) {
                    if let Some(data_set_id) =
                        config_value.get("dataSetId").and_then(|v| v.as_i64())
                    {
                        if let Some(data_set) = data_sets::Entity::find_by_id(data_set_id as i32)
                            .one(&self.db)
                            .await
                            .map_err(|e| {
                                CoreError::internal(format!(
                                    "Failed to load data source {}: {}",
                                    data_set_id, e
                                ))
                            })?
                        {
                            let mut metadata_obj =
                                serde_json::from_str::<Value>(&metadata_json_current)
                                    .ok()
                                    .and_then(|value| value.as_object().cloned())
                                    .unwrap_or_default();

                            let needs_update = match metadata_obj.get("label") {
                                Some(Value::String(current_label))
                                    if current_label == &data_set.name =>
                                {
                                    false
                                }
                                _ => true,
                            };

                            if needs_update {
                                metadata_obj.insert(
                                    "label".to_string(),
                                    Value::String(data_set.name.clone()),
                                );
                                let metadata_value = Value::Object(metadata_obj);
                                let metadata_json = metadata_value.to_string();
                                node_active.metadata_json = Set(metadata_json.clone());

                            }
                        }
                    }
                }
            }
        }

        node_active.updated_at = Set(Utc::now());
        let updated_node = node_active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update node: {}", e)))?;

        let result_node = PlanDagNode::from(updated_node);

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_node)
    }

    /// Delete a Plan DAG node and its connected edges
    pub async fn delete_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
    ) -> CoreResult<PlanDagNode> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Find the node to delete
        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDagNode", node_id.clone()))?;

        let result_node = PlanDagNode::from(node);

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
            .map_err(|e| {
                CoreError::internal(format!("Failed to delete connected edges: {}", e))
            })?;

        // Delete the node
        plan_dag_nodes::Entity::delete_many()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(&node_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete node: {}", e)))?;

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_node)
    }

    /// Move a Plan DAG node to a new position
    pub async fn move_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: String,
        position: Position,
    ) -> CoreResult<PlanDagNode> {
        self.update_node(project_id, plan_id, node_id, Some(position), None, None)
            .await
    }

    /// Create a new Plan DAG edge
    pub async fn create_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
        source_node_id: String,
        target_node_id: String,
        metadata_json: String,
    ) -> CoreResult<PlanDagEdge> {
        // Validate inputs
        let validated_edge_id = ValidationService::validate_node_id(&edge_id)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_source = ValidationService::validate_node_id(&source_node_id)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_target = ValidationService::validate_node_id(&target_node_id)
            .map_err(|e| CoreError::validation(e.to_string()))?;
        let validated_metadata = ValidationService::validate_plan_dag_metadata(&metadata_json)
            .map_err(|e| CoreError::validation(e.to_string()))?;

        // Validate no self-loop
        ValidationService::validate_edge_no_self_loop(&validated_source, &validated_target)
            .map_err(|e| CoreError::validation(e.to_string()))?;

        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Fetch current state for delta generation and validation
        let (current_nodes, current_edges) =
            self.fetch_current_plan_dag(plan.id)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to fetch current state: {}", e)))?;

        // Validate DAG limits
        ValidationService::validate_plan_dag_limits(current_nodes.len(), current_edges.len() + 1)
            .map_err(|e| CoreError::validation(e.to_string()))?;

        // Create the edge with validated values
        let now = Utc::now();
        let edge = plan_dag_edges::ActiveModel {
            id: Set(validated_edge_id),
            plan_id: Set(plan.id),
            source_node_id: Set(validated_source),
            target_node_id: Set(validated_target),
            // Removed source_handle and target_handle for floating edges
            metadata_json: Set(
                serde_json::to_string(&validated_metadata)
                    .map_err(|e| CoreError::validation(format!("Invalid metadata: {}", e)))?,
            ),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created_edge = edge
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to create edge: {}", e)))?;

        let result_edge = PlanDagEdge::from(created_edge);

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_edge)
    }

    /// Delete a Plan DAG edge
    pub async fn delete_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
    ) -> CoreResult<PlanDagEdge> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Find the edge to delete
        let edge = plan_dag_edges::Entity::find()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDagEdge", edge_id.clone()))?;

        let result_edge = PlanDagEdge::from(edge);

        // Delete the edge
        plan_dag_edges::Entity::delete_many()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete edge: {}", e)))?;

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_edge)
    }

    /// Get all nodes for a project's plan
    pub async fn get_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<Vec<PlanDagNode>> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .order_by_asc(plan_dag_nodes::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        Ok(nodes.into_iter().map(PlanDagNode::from).collect())
    }

    /// Get all edges for a project's plan
    pub async fn get_edges(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<Vec<PlanDagEdge>> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .order_by_asc(plan_dag_edges::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        Ok(edges.into_iter().map(PlanDagEdge::from).collect())
    }

    /// Update metadata for a Plan DAG edge
    pub async fn update_edge(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        edge_id: String,
        metadata_json: Option<String>,
    ) -> CoreResult<PlanDagEdge> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let edge = plan_dag_edges::Entity::find()
            .filter(
                plan_dag_edges::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_edges::Column::Id.eq(&edge_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| CoreError::not_found("PlanDagEdge", edge_id.clone()))?;

        let mut edge_active: plan_dag_edges::ActiveModel = edge.into();
        if let Some(metadata) = metadata_json {
            edge_active.metadata_json = Set(metadata.clone());
        }

        edge_active.updated_at = Set(Utc::now());
        let updated_edge = edge_active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update edge: {}", e)))?;

        let result_edge = PlanDagEdge::from(updated_edge);

        let _ = self
            .bump_plan_version(plan.id)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;

        Ok(result_edge)
    }

    /// Batch move nodes with delta publication
    pub async fn batch_move_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_positions: Vec<PlanDagNodePositionUpdate>,
    ) -> CoreResult<Vec<PlanDagNode>> {
        if node_positions.is_empty() {
            return Ok(Vec::new());
        }

        let plan = self.resolve_plan(project_id, plan_id).await?;

        let mut updated_nodes = Vec::new();
        for node_pos in node_positions {
            let node = plan_dag_nodes::Entity::find()
                .filter(
                    plan_dag_nodes::Column::PlanId
                        .eq(plan.id)
                        .and(plan_dag_nodes::Column::Id.eq(&node_pos.node_id)),
                )
                .one(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

            if let Some(node) = node {
                let mut node_active: plan_dag_nodes::ActiveModel = node.into();
                node_active.position_x = Set(node_pos.position.x);
                node_active.position_y = Set(node_pos.position.y);
                node_active.source_position = Set(node_pos.source_position.clone());
                node_active.target_position = Set(node_pos.target_position.clone());
                node_active.updated_at = Set(Utc::now());

                let updated_node = node_active
                    .update(&self.db)
                    .await
                    .map_err(|e| CoreError::internal(format!("Failed to update node: {}", e)))?;

                updated_nodes.push(PlanDagNode::from(updated_node));
            }
        }

        if !updated_nodes.is_empty() {
            let _ = self
                .bump_plan_version(plan.id)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to increment version: {}", e)))?;
        }

        Ok(updated_nodes)
    }

    /// Validate and migrate legacy plan DAG node types for a project.
    /// Currently normalises legacy OutputNode/artefact aliases to GraphArtefactNode and tree artefact aliases.
    pub async fn validate_and_migrate_legacy_nodes(
        &self,
        project_id: i32,
    ) -> CoreResult<PlanDagMigrationOutcome> {
        let plan = self.get_or_create_plan(project_id).await?;

        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::internal(format!("Failed to load plan DAG nodes: {}", e))
            })?;

        let mut outcome = PlanDagMigrationOutcome {
            checked_nodes: nodes.len(),
            ..Default::default()
        };

        let mut updated_any = false;

        for node in nodes {
            let original_type = node.node_type.clone();
            let (normalized_type, note) = normalize_legacy_node_type(&original_type);
            let validated_type =
                match ValidationService::validate_plan_dag_node_type(&normalized_type) {
                    Ok(valid) => valid,
                    Err(err) => {
                        outcome.errors.push(format!(
                            "Node {} has invalid type '{}': {}",
                            node.id, original_type, err
                        ));
                        continue;
                    }
                };

            if validated_type != original_type {
                let mut active: plan_dag_nodes::ActiveModel = node.clone().into();
                active.node_type = Set(validated_type.clone());
                active.updated_at = Set(Utc::now());
                active
                    .update(&self.db)
                    .await
                    .map_err(|e| {
                        CoreError::internal(format!(
                            "Failed to migrate node {}: {}",
                            node.id, e
                        ))
                    })?;

                outcome.migrated_nodes.push(PlanDagMigrationDetail {
                    node_id: node.id.clone(),
                    from_type: original_type,
                    to_type: validated_type.clone(),
                    note: note.or_else(|| Some("Normalized legacy node type".to_string())),
                });
                updated_any = true;
            }
        }

        if updated_any {
            self.bump_plan_version(plan.id)
                .await
                .map_err(|e| {
                    CoreError::internal(format!(
                        "Failed to increment plan version after migration: {}",
                        e
                    ))
                })?;
        }

        Ok(outcome)
    }
}

fn normalize_legacy_node_type(node_type: &str) -> (String, Option<String>) {
    match node_type {
        "OutputNode" | "Output" => (
            "GraphArtefactNode".to_string(),
            Some("Renamed legacy Output node to GraphArtefactNode".to_string()),
        ),
        "GraphArtefactNode" => ("GraphArtefactNode".to_string(), None),
        "GraphArtefact" | "GraphArtifact" | "GraphArtifactNode" => (
            "GraphArtefactNode".to_string(),
            Some("Normalized artefact node naming".to_string()),
        ),
        "TreeArtefactNode" => ("TreeArtefactNode".to_string(), None),
        "TreeArtefact" | "TreeArtifact" => (
            "TreeArtefactNode".to_string(),
            Some("Normalized tree artefact node naming".to_string()),
        ),
        other => (other.to_string(), None),
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    // Note: These would be integration tests requiring a test database
    // For now, just testing the service creation

    #[test]
    fn test_service_creation() {
        // This would require a real database connection for proper testing
        // We'll add integration tests when we have a test database setup
    }
}
