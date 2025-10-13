use crate::database::entities::{plan_dag_edges, plan_dag_nodes, plans};
use crate::graphql::types::{PatchOp, PatchOperation, PlanDagDeltaEvent, PlanDagEdge, PlanDagNode};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

/// Helper function to generate JSON Patch for node addition
pub fn generate_node_add_patch(node: &PlanDagNode, index: usize) -> PatchOperation {
    let node_json = serde_json::to_value(node).unwrap_or(serde_json::Value::Null);

    PatchOperation {
        op: PatchOp::Add,
        path: format!("/nodes/{}", index),
        value: Some(node_json),
        from: None,
    }
}

/// Helper function to generate JSON Patch for node update
pub fn generate_node_update_patch(
    node_id: &str,
    field: &str,
    value: serde_json::Value,
    nodes: &[PlanDagNode],
) -> Option<PatchOperation> {
    // Find the index of the node
    let index = nodes.iter().position(|n| n.id == node_id)?;

    Some(PatchOperation {
        op: PatchOp::Replace,
        path: format!("/nodes/{}/{}", index, field),
        value: Some(value),
        from: None,
    })
}

/// Helper function to generate JSON Patch for node position update
pub fn generate_node_position_patch(
    node_id: &str,
    x: f64,
    y: f64,
    nodes: &[PlanDagNode],
) -> Vec<PatchOperation> {
    if let Some(index) = nodes.iter().position(|n| n.id == node_id) {
        vec![
            PatchOperation {
                op: PatchOp::Replace,
                path: format!("/nodes/{}/position/x", index),
                value: Some(serde_json::json!(x)),
                from: None,
            },
            PatchOperation {
                op: PatchOp::Replace,
                path: format!("/nodes/{}/position/y", index),
                value: Some(serde_json::json!(y)),
                from: None,
            },
        ]
    } else {
        vec![]
    }
}

/// Helper function to generate JSON Patch for node deletion
pub fn generate_node_delete_patch(node_id: &str, nodes: &[PlanDagNode]) -> Option<PatchOperation> {
    let index = nodes.iter().position(|n| n.id == node_id)?;

    Some(PatchOperation {
        op: PatchOp::Remove,
        path: format!("/nodes/{}", index),
        value: None,
        from: None,
    })
}

/// Helper function to generate JSON Patch for edge addition
pub fn generate_edge_add_patch(edge: &PlanDagEdge, index: usize) -> PatchOperation {
    let edge_json = serde_json::to_value(edge).unwrap_or(serde_json::Value::Null);

    PatchOperation {
        op: PatchOp::Add,
        path: format!("/edges/{}", index),
        value: Some(edge_json),
        from: None,
    }
}

/// Helper function to generate JSON Patch for edge deletion
pub fn generate_edge_delete_patch(edge_id: &str, edges: &[PlanDagEdge]) -> Option<PatchOperation> {
    let index = edges.iter().position(|e| e.id == edge_id)?;

    Some(PatchOperation {
        op: PatchOp::Remove,
        path: format!("/edges/{}", index),
        value: None,
        from: None,
    })
}

/// Fetch current Plan DAG state for diff generation
pub async fn fetch_current_plan_dag(
    db: &DatabaseConnection,
    plan_id: i32,
) -> Result<(Vec<PlanDagNode>, Vec<PlanDagEdge>), sea_orm::DbErr> {
    let dag_nodes = plan_dag_nodes::Entity::find()
        .filter(plan_dag_nodes::Column::PlanId.eq(plan_id))
        .all(db)
        .await?;

    let dag_edges = plan_dag_edges::Entity::find()
        .filter(plan_dag_edges::Column::PlanId.eq(plan_id))
        .all(db)
        .await?;

    let nodes: Vec<PlanDagNode> = dag_nodes.into_iter().map(PlanDagNode::from).collect();
    let edges: Vec<PlanDagEdge> = dag_edges.into_iter().map(PlanDagEdge::from).collect();

    Ok((nodes, edges))
}

/// Publish a delta event after a mutation
pub async fn publish_plan_dag_delta(
    project_id: i32,
    version: i32,
    user_id: String,
    operations: Vec<PatchOperation>,
) -> Result<(), String> {
    let event = PlanDagDeltaEvent {
        project_id,
        version,
        user_id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        operations,
    };

    crate::graphql::subscriptions::publish_delta_event(event).await
}

/// Increment plan version and return new version
pub async fn increment_plan_version(
    db: &DatabaseConnection,
    plan_id: i32,
) -> Result<i32, sea_orm::DbErr> {
    use sea_orm::{ActiveModelTrait, Set};

    let plan = plans::Entity::find_by_id(plan_id)
        .one(db)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Plan not found".to_string()))?;

    let new_version = plan.version + 1;

    let mut plan_active: plans::ActiveModel = plan.into();
    plan_active.version = Set(new_version);
    plan_active.updated_at = Set(chrono::Utc::now());
    plan_active.update(db).await?;

    Ok(new_version)
}
