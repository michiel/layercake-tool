use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
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
                    let plan = active.update(&self.db).await.map_err(|e| {
                        CoreError::internal(format!("Failed to rename plan: {}", e))
                    })?;
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
            metadata_json: Set(serde_json::to_string(&validated_metadata)
                .map_err(|e| CoreError::validation(format!("Invalid metadata: {}", e)))?),
            config_json: Set(serde_json::to_string(&validated_config)
                .map_err(|e| CoreError::validation(format!("Invalid config: {}", e)))?),
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
            .map_err(|e| CoreError::internal(format!("Failed to delete connected edges: {}", e)))?;

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
        let (current_nodes, current_edges) = self
            .fetch_current_plan_dag(plan.id)
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
            metadata_json: Set(serde_json::to_string(&validated_metadata)
                .map_err(|e| CoreError::validation(format!("Invalid metadata: {}", e)))?),
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

    /// Phase 1.1: Get nodes with optional filtering
    pub async fn get_nodes_filtered(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_type: Option<String>,
        label_pattern: Option<String>,
        bounds: Option<(f64, f64, f64, f64)>,
    ) -> CoreResult<Vec<PlanDagNode>> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let mut query = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id));

        // Filter by node type if provided
        if let Some(nt) = node_type {
            query = query.filter(plan_dag_nodes::Column::NodeType.eq(nt));
        }

        // Filter by position bounds if provided
        if let Some((min_x, max_x, min_y, max_y)) = bounds {
            query = query
                .filter(
                    Condition::all()
                        .add(plan_dag_nodes::Column::PositionX.gte(min_x))
                        .add(plan_dag_nodes::Column::PositionX.lte(max_x)),
                )
                .filter(
                    Condition::all()
                        .add(plan_dag_nodes::Column::PositionY.gte(min_y))
                        .add(plan_dag_nodes::Column::PositionY.lte(max_y)),
                );
        }

        let nodes = query
            .order_by_asc(plan_dag_nodes::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let mut result: Vec<PlanDagNode> = nodes.into_iter().map(PlanDagNode::from).collect();

        // Filter by label pattern if provided
        if let Some(pattern) = label_pattern {
            let pattern_lower = pattern.to_lowercase();
            result.retain(|node| node.metadata.label.to_lowercase().contains(&pattern_lower));
        }

        Ok(result)
    }

    /// Phase 1.2: Get a single node by ID
    pub async fn get_node_by_id(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        node_id: &str,
    ) -> CoreResult<Option<PlanDagNode>> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let node = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.eq(node_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        Ok(node.map(PlanDagNode::from))
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
            .map_err(|e| CoreError::internal(format!("Failed to load plan DAG nodes: {}", e)))?;

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
                active.update(&self.db).await.map_err(|e| {
                    CoreError::internal(format!("Failed to migrate node {}: {}", node.id, e))
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
            self.bump_plan_version(plan.id).await.map_err(|e| {
                CoreError::internal(format!(
                    "Failed to increment plan version after migration: {}",
                    e
                ))
            })?;
        }

        Ok(outcome)
    }

    /// Phase 1.3: Traverse graph from a starting node
    pub async fn traverse_from_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        start_node_id: &str,
        direction: &str, // "upstream", "downstream", "both"
        max_depth: usize,
    ) -> CoreResult<(Vec<PlanDagNode>, Vec<PlanDagEdge>)> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Load all edges for traversal
        let all_edges = self.get_edges(project_id, Some(plan.id)).await?;

        // Build adjacency lists
        let mut downstream: HashMap<String, Vec<String>> = HashMap::new();
        let mut upstream: HashMap<String, Vec<String>> = HashMap::new();

        for edge in &all_edges {
            downstream
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            upstream
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
        }

        // BFS traversal
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((start_node_id.to_string(), 0));
        visited.insert(start_node_id.to_string());

        let mut found_node_ids = vec![start_node_id.to_string()];

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            let neighbors: Vec<String> = match direction {
                "downstream" => downstream.get(&node_id).cloned().unwrap_or_default(),
                "upstream" => upstream.get(&node_id).cloned().unwrap_or_default(),
                "both" => {
                    let mut both = downstream.get(&node_id).cloned().unwrap_or_default();
                    both.extend(upstream.get(&node_id).cloned().unwrap_or_default());
                    both
                }
                _ => vec![],
            };

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    found_node_ids.push(neighbor.clone());
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        // Fetch the discovered nodes
        let nodes = plan_dag_nodes::Entity::find()
            .filter(
                plan_dag_nodes::Column::PlanId
                    .eq(plan.id)
                    .and(plan_dag_nodes::Column::Id.is_in(found_node_ids.clone())),
            )
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        // Filter edges to only those connecting found nodes
        let found_set: HashSet<String> = found_node_ids.into_iter().collect();
        let relevant_edges: Vec<PlanDagEdge> = all_edges
            .into_iter()
            .filter(|e| found_set.contains(&e.source) && found_set.contains(&e.target))
            .collect();

        Ok((
            nodes.into_iter().map(PlanDagNode::from).collect(),
            relevant_edges,
        ))
    }

    /// Phase 1.3: Find shortest path between two nodes
    pub async fn find_path(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        start_node_id: &str,
        end_node_id: &str,
    ) -> CoreResult<Option<Vec<String>>> {
        use std::collections::{HashMap, VecDeque};

        let plan = self.resolve_plan(project_id, plan_id).await?;
        let all_edges = self.get_edges(project_id, Some(plan.id)).await?;

        // Build adjacency for directed graph
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &all_edges {
            adjacency
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }

        // BFS for shortest path
        let mut queue = VecDeque::new();
        let mut parent: HashMap<String, String> = HashMap::new();
        queue.push_back(start_node_id.to_string());
        parent.insert(start_node_id.to_string(), String::new());

        while let Some(node_id) = queue.pop_front() {
            if node_id == end_node_id {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = end_node_id.to_string();
                while !current.is_empty() {
                    path.push(current.clone());
                    current = parent.get(&current).cloned().unwrap_or_default();
                }
                path.reverse();
                return Ok(Some(path));
            }

            if let Some(neighbors) = adjacency.get(&node_id) {
                for neighbor in neighbors {
                    if !parent.contains_key(neighbor) {
                        parent.insert(neighbor.clone(), node_id.clone());
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }

        Ok(None)
    }

    /// Phase 2.2: Search nodes by query string
    pub async fn search_nodes(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        query: &str,
        fields: Vec<String>,
    ) -> CoreResult<Vec<PlanDagNode>> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Database error: {}", e)))?;

        let query_lower = query.to_lowercase();
        let result: Vec<PlanDagNode> = nodes
            .into_iter()
            .filter(|node| {
                let node_dag = PlanDagNode::from(node.clone());

                // Search in label
                if fields.contains(&"label".to_string()) || fields.is_empty() {
                    if node_dag.metadata.label.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }

                // Search in description
                if fields.contains(&"description".to_string()) || fields.is_empty() {
                    if let Some(desc) = &node_dag.metadata.description {
                        if desc.to_lowercase().contains(&query_lower) {
                            return true;
                        }
                    }
                }

                // Search in config
                if node_dag.config.to_lowercase().contains(&query_lower) {
                    return true;
                }

                false
            })
            .map(PlanDagNode::from)
            .collect();

        Ok(result)
    }

    /// Phase 2.2: Find nodes by edge filter
    pub async fn find_nodes_by_edge_filter(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        filter: &str,
    ) -> CoreResult<Vec<PlanDagNode>> {
        use std::collections::HashSet;

        let plan = self.resolve_plan(project_id, plan_id).await?;

        let nodes = self.get_nodes(project_id, Some(plan.id)).await?;
        let edges = self.get_edges(project_id, Some(plan.id)).await?;

        let mut outgoing: HashSet<String> = HashSet::new();
        let mut incoming: HashSet<String> = HashSet::new();

        for edge in &edges {
            outgoing.insert(edge.source.clone());
            incoming.insert(edge.target.clone());
        }

        let result: Vec<PlanDagNode> = nodes
            .into_iter()
            .filter(|node| match filter {
                "noOutgoing" => !outgoing.contains(&node.id),
                "noIncoming" => !incoming.contains(&node.id),
                "isolated" => !outgoing.contains(&node.id) && !incoming.contains(&node.id),
                _ => true,
            })
            .collect();

        Ok(result)
    }

    /// Phase 2.3: Analyze plan statistics
    pub async fn analyze_plan_stats(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<PlanStats> {
        use std::collections::{HashMap, HashSet};

        let plan = self.resolve_plan(project_id, plan_id).await?;
        let nodes = self.get_nodes(project_id, Some(plan.id)).await?;
        let edges = self.get_edges(project_id, Some(plan.id)).await?;

        let mut nodes_by_type: HashMap<String, usize> = HashMap::new();
        for node in &nodes {
            *nodes_by_type
                .entry(format!("{:?}", node.node_type))
                .or_insert(0) += 1;
        }

        // Find leaf nodes (no outgoing edges)
        let outgoing: HashSet<String> = edges.iter().map(|e| e.source.clone()).collect();
        let leaf_count = nodes.iter().filter(|n| !outgoing.contains(&n.id)).count();

        // Find isolated nodes
        let connected: HashSet<String> = edges
            .iter()
            .flat_map(|e| vec![e.source.clone(), e.target.clone()])
            .collect();
        let isolated_count = nodes.iter().filter(|n| !connected.contains(&n.id)).count();

        Ok(PlanStats {
            node_count: nodes.len(),
            edge_count: edges.len(),
            nodes_by_type,
            leaf_nodes: leaf_count,
            avg_degree: if nodes.is_empty() {
                0.0
            } else {
                (edges.len() * 2) as f64 / nodes.len() as f64
            },
            isolated_nodes: isolated_count,
        })
    }

    /// Phase 2.3: Find bottleneck nodes (high degree)
    pub async fn find_bottlenecks(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        threshold: usize,
    ) -> CoreResult<Vec<BottleneckInfo>> {
        use std::collections::HashMap;

        let plan = self.resolve_plan(project_id, plan_id).await?;
        let nodes = self.get_nodes(project_id, Some(plan.id)).await?;
        let edges = self.get_edges(project_id, Some(plan.id)).await?;

        let mut degree: HashMap<String, usize> = HashMap::new();
        for edge in &edges {
            *degree.entry(edge.source.clone()).or_insert(0) += 1;
            *degree.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let bottlenecks: Vec<BottleneckInfo> = nodes
            .into_iter()
            .filter_map(|node| {
                let deg = *degree.get(&node.id).unwrap_or(&0);
                if deg >= threshold {
                    Some(BottleneckInfo {
                        node_id: node.id.clone(),
                        label: node.metadata.label.clone(),
                        degree: deg,
                        node_type: format!("{:?}", node.node_type),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(bottlenecks)
    }

    /// Phase 2.3: Detect cycles in the graph
    pub async fn detect_cycles(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<Vec<Vec<String>>> {
        use std::collections::{HashMap, HashSet};

        let plan = self.resolve_plan(project_id, plan_id).await?;
        let edges = self.get_edges(project_id, Some(plan.id)).await?;

        // Build adjacency list
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for edge in &edges {
            adj.entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }

        // DFS-based cycle detection
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn detect_cycle_dfs(
            node: &str,
            adj: &HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
            cycles: &mut Vec<Vec<String>>,
        ) {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(neighbors) = adj.get(node) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        detect_cycle_dfs(neighbor, adj, visited, rec_stack, path, cycles);
                    } else if rec_stack.contains(neighbor) {
                        // Found a cycle
                        if let Some(pos) = path.iter().position(|n| n == neighbor) {
                            cycles.push(path[pos..].to_vec());
                        }
                    }
                }
            }

            path.pop();
            rec_stack.remove(node);
        }

        for node in adj.keys() {
            if !visited.contains(node) {
                detect_cycle_dfs(
                    node,
                    &adj,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        Ok(cycles)
    }

    /// Phase 2.5: Clone a node
    pub async fn clone_node(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
        source_node_id: &str,
        new_position: Option<Position>,
        update_label: Option<String>,
    ) -> CoreResult<PlanDagNode> {
        let plan = self.resolve_plan(project_id, plan_id).await?;

        // Fetch source node
        let source_node = self
            .get_node_by_id(project_id, Some(plan.id), source_node_id)
            .await?
            .ok_or_else(|| CoreError::not_found("Node", source_node_id.to_string()))?;

        // Generate new ID using UUID
        let uuid = uuid::Uuid::new_v4().simple().to_string();
        let short_uuid = uuid.chars().take(12).collect::<String>();
        let node_type_prefix = match source_node.node_type {
            crate::plan_dag::PlanDagNodeType::DataSet => "dataset",
            crate::plan_dag::PlanDagNodeType::Graph => "graph",
            crate::plan_dag::PlanDagNodeType::GraphArtefact => "graphartefact",
            crate::plan_dag::PlanDagNodeType::TreeArtefact => "treeartefact",
            crate::plan_dag::PlanDagNodeType::Projection => "projection",
            crate::plan_dag::PlanDagNodeType::Story => "story",
            _ => "node",
        };
        let new_id = format!("{}_{}", node_type_prefix, short_uuid);

        // Update position and label
        let position = new_position.unwrap_or(Position {
            x: source_node.position.x + 100.0,
            y: source_node.position.y + 100.0,
        });

        let mut new_metadata = source_node.metadata.clone();
        if let Some(new_label) = update_label {
            new_metadata.label = new_label;
        } else {
            new_metadata.label = format!("{} (copy)", new_metadata.label);
        }

        let metadata_json = serde_json::to_string(&new_metadata)
            .map_err(|e| CoreError::validation(format!("Invalid metadata: {}", e)))?;

        // Create new node
        self.create_node(
            project_id,
            Some(plan.id),
            new_id,
            format!("{:?}", source_node.node_type).replace("Node", "Node"), // Keep format consistent
            position,
            metadata_json,
            source_node.config.clone(),
        )
        .await
    }
}

#[derive(serde::Serialize)]
pub struct PlanStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub nodes_by_type: std::collections::HashMap<String, usize>,
    pub leaf_nodes: usize,
    pub avg_degree: f64,
    pub isolated_nodes: usize,
}

#[derive(serde::Serialize)]
pub struct BottleneckInfo {
    pub node_id: String,
    pub label: String,
    pub degree: usize,
    pub node_type: String,
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
