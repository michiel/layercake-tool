use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans};

#[derive(Clone)]
pub struct PlanService {
    db: DatabaseConnection,
}

#[derive(Clone)]
pub struct PlanCreateRequest {
    pub project_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub yaml_content: String,
    pub dependencies: Option<Vec<i32>>,
    pub status: Option<String>,
}

#[derive(Clone)]
pub struct PlanUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub yaml_content: Option<String>,
    pub dependencies: Option<Vec<i32>>,
    pub dependencies_is_set: bool,
    pub status: Option<String>,
}

impl PlanService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn serialize_dependencies(dependencies: Option<Vec<i32>>) -> Result<Option<String>> {
        match dependencies {
            Some(values) => {
                Ok(Some(serde_json::to_string(&values).map_err(|e| {
                    anyhow!("Invalid plan dependencies: {}", e)
                })?))
            }
            None => Ok(None),
        }
    }

    fn serialize_tags(tags: Option<Vec<String>>) -> String {
        let values = tags.unwrap_or_default();
        serde_json::to_string(&values).unwrap_or_else(|_| "[]".to_string())
    }

    fn normalize_description(description: Option<String>) -> Option<String> {
        description.and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
    }

    fn generate_node_id(node_type: &str) -> String {
        let prefix = match node_type {
            "DataSetNode" => "dataset",
            "GraphNode" => "graph",
            "TransformNode" => "transform",
            "FilterNode" => "filter",
            "MergeNode" => "merge",
            "GraphArtefactNode" => "graphartefact",
            "TreeArtefactNode" => "treeartefact",
            _ => "node",
        };
        let uuid = Uuid::new_v4().simple().to_string();
        let short_uuid: String = uuid.chars().take(12).collect();
        format!("{}_{}", prefix, short_uuid)
    }

    fn generate_edge_id() -> String {
        let uuid = Uuid::new_v4().simple().to_string();
        let short_uuid: String = uuid.chars().take(12).collect();
        format!("edge_{}", short_uuid)
    }

    pub async fn list_plans(&self, project_id: i32) -> Result<Vec<plans::Model>> {
        let plans = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to list plans for project {}: {}", project_id, e))?;

        Ok(plans)
    }

    pub async fn get_plan(&self, id: i32) -> Result<Option<plans::Model>> {
        let plan = plans::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?;

        Ok(plan)
    }

    pub async fn get_default_plan(&self, project_id: i32) -> Result<Option<plans::Model>> {
        let plan = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .order_by_desc(plans::Column::UpdatedAt)
            .order_by_desc(plans::Column::Id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

        Ok(plan)
    }

    pub async fn create_plan(&self, request: PlanCreateRequest) -> Result<plans::Model> {
        let dependencies_json = Self::serialize_dependencies(request.dependencies)?;
        let tags_json = Self::serialize_tags(request.tags);
        let description = Self::normalize_description(request.description);
        let now = Utc::now();

        let plan = plans::ActiveModel {
            id: NotSet,
            project_id: Set(request.project_id),
            name: Set(request.name),
            description: Set(description),
            tags: Set(tags_json),
            yaml_content: Set(request.yaml_content),
            dependencies: Set(dependencies_json),
            status: Set(request.status.unwrap_or_else(|| "pending".to_string())),
            version: Set(1),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let created = plan
            .insert(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to create plan: {}", e))?;

        Ok(created)
    }

    pub async fn update_plan(&self, id: i32, request: PlanUpdateRequest) -> Result<plans::Model> {
        let plan = plans::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Plan {} not found", id))?;

        let mut active: plans::ActiveModel = plan.into();

        if let Some(name) = request.name {
            active.name = Set(name);
        }

        if let Some(description) = request.description {
            active.description = Set(Self::normalize_description(Some(description)));
        }

        if let Some(tags) = request.tags {
            active.tags = Set(Self::serialize_tags(Some(tags)));
        }

        if let Some(yaml_content) = request.yaml_content {
            active.yaml_content = Set(yaml_content);
        }

        if request.dependencies_is_set {
            active.dependencies = Set(Self::serialize_dependencies(request.dependencies)?);
        }

        if let Some(status) = request.status {
            active.status = Set(status);
        }

        active.updated_at = Set(Utc::now());

        let updated = active
            .update(&self.db)
            .await
            .map_err(|e| anyhow!("Failed to update plan {}: {}", id, e))?;

        Ok(updated)
    }

    pub async fn delete_plan(&self, id: i32) -> Result<()> {
        let txn = self.db.begin().await?;

        let plan = plans::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Plan {} not found", id))?;

        plan_dag_nodes::Entity::delete_many()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .exec(&txn)
            .await
            .map_err(|e| anyhow!("Failed to delete plan nodes for plan {}: {}", id, e))?;

        plan_dag_edges::Entity::delete_many()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .exec(&txn)
            .await
            .map_err(|e| anyhow!("Failed to delete plan edges for plan {}: {}", id, e))?;

        plans::Entity::delete_by_id(plan.id)
            .exec(&txn)
            .await
            .map_err(|e| anyhow!("Failed to delete plan {}: {}", id, e))?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn duplicate_plan(&self, id: i32, new_name: String) -> Result<plans::Model> {
        let txn = self.db.begin().await?;

        let plan = plans::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?
            .ok_or_else(|| anyhow!("Plan {} not found", id))?;

        let now = Utc::now();

        let duplicated_plan = plans::ActiveModel {
            id: NotSet,
            project_id: Set(plan.project_id),
            name: Set(new_name),
            description: Set(plan.description.clone()),
            tags: Set(plan.tags.clone()),
            yaml_content: Set(plan.yaml_content.clone()),
            dependencies: Set(plan.dependencies.clone()),
            status: Set(plan.status.clone()),
            version: Set(1),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(&txn)
        .await
        .map_err(|e| anyhow!("Failed to duplicate plan {}: {}", id, e))?;

        let nodes = plan_dag_nodes::Entity::find()
            .filter(plan_dag_nodes::Column::PlanId.eq(plan.id))
            .all(&txn)
            .await
            .map_err(|e| anyhow!("Failed to load plan nodes for duplication: {}", e))?;

        let mut id_map = std::collections::HashMap::new();

        for node in nodes {
            let new_id = Self::generate_node_id(&node.node_type);
            id_map.insert(node.id.clone(), new_id.clone());

            let duplicate_node = plan_dag_nodes::ActiveModel {
                id: Set(new_id),
                plan_id: Set(duplicated_plan.id),
                node_type: Set(node.node_type.clone()),
                position_x: Set(node.position_x),
                position_y: Set(node.position_y),
                source_position: Set(node.source_position.clone()),
                target_position: Set(node.target_position.clone()),
                metadata_json: Set(node.metadata_json.clone()),
                config_json: Set(node.config_json.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            duplicate_node
                .insert(&txn)
                .await
                .map_err(|e| anyhow!("Failed to copy node {}: {}", node.id, e))?;
        }

        let edges = plan_dag_edges::Entity::find()
            .filter(plan_dag_edges::Column::PlanId.eq(plan.id))
            .all(&txn)
            .await
            .map_err(|e| anyhow!("Failed to load plan edges for duplication: {}", e))?;

        for edge in edges {
            let new_edge = plan_dag_edges::ActiveModel {
                id: Set(Self::generate_edge_id()),
                plan_id: Set(duplicated_plan.id),
                source_node_id: Set(id_map
                    .get(&edge.source_node_id)
                    .cloned()
                    .unwrap_or_else(|| edge.source_node_id.clone())),
                target_node_id: Set(id_map
                    .get(&edge.target_node_id)
                    .cloned()
                    .unwrap_or_else(|| edge.target_node_id.clone())),
                metadata_json: Set(edge.metadata_json.clone()),
                created_at: Set(now),
                updated_at: Set(now),
            };

            new_edge
                .insert(&txn)
                .await
                .map_err(|e| anyhow!("Failed to copy edge {}: {}", edge.id, e))?;
        }

        txn.commit().await?;

        Ok(duplicated_plan)
    }

    pub async fn ensure_default_plan(&self, project_id: i32) -> Result<plans::Model> {
        if let Some(plan) = self.get_default_plan(project_id).await? {
            return Ok(plan);
        }

        self.create_plan(PlanCreateRequest {
            project_id,
            name: "Main Plan".to_string(),
            description: Some("Default plan".to_string()),
            tags: Some(vec![]),
            yaml_content: "".to_string(),
            dependencies: None,
            status: Some("draft".to_string()),
        })
        .await
    }
}
