use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{
    graph_edges, graph_layers, graph_nodes, graphs, plan_dag_nodes, plans, projects,
};
use layercake_core::database::migrations::Migrator;
use layercake_core::services::{ApplyResult, GraphEditApplicator, GraphEditService};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, Set,
};
use sea_orm_migration::MigratorTrait;

/// Create an in-memory SQLite database for testing
async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

/// Create a test graph
async fn create_test_graph(
    db: &DatabaseConnection,
    project_id: i32,
    name: &str,
) -> Result<graphs::Model, DbErr> {
    if projects::Entity::find_by_id(project_id)
        .one(db)
        .await?
        .is_none()
    {
        let mut project = projects::ActiveModel::new();
        project.id = Set(project_id);
        project.name = Set(format!("Test Project {}", project_id));
        project.description = Set(Some("Test project".to_string()));
        project.insert(db).await?;
    }

    let plan = if let Some(plan) = plans::Entity::find()
        .filter(plans::Column::ProjectId.eq(project_id))
        .one(db)
        .await?
    {
        plan
    } else {
        let plan = plans::ActiveModel {
            id: ActiveValue::NotSet,
            project_id: Set(project_id),
            name: Set(format!("Test Plan {}", uuid::Uuid::new_v4())),
            yaml_content: Set("{}".to_string()),
            dependencies: Set(None),
            status: Set("draft".to_string()),
            version: Set(1),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };
        plan.insert(db).await?
    };

    let dag_node_id = format!("test_node_{}", uuid::Uuid::new_v4());
    let mut dag_node = plan_dag_nodes::ActiveModel::new();
    dag_node.id = Set(dag_node_id.clone());
    dag_node.plan_id = Set(plan.id);
    dag_node.node_type = Set("GraphNode".to_string());
    dag_node.position_x = Set(0.0);
    dag_node.position_y = Set(0.0);
    dag_node.source_position = Set(None);
    dag_node.target_position = Set(None);
    dag_node.metadata_json = Set("{}".to_string());
    dag_node.config_json = Set("{}".to_string());
    dag_node.insert(db).await?;

    let mut graph = graphs::ActiveModel::new();
    graph.project_id = Set(project_id);
    graph.name = Set(name.to_string());
    graph.node_id = Set(dag_node_id);
    graph.insert(db).await
}

/// Create a test node
async fn create_test_node(
    db: &DatabaseConnection,
    graph_id: i32,
    id: &str,
    label: Option<&str>,
) -> Result<graph_nodes::Model, DbErr> {
    let node = graph_nodes::ActiveModel {
        id: Set(id.to_string()),
        graph_id: Set(graph_id),
        label: Set(label.map(|s| s.to_string())),
        layer: Set(None),
        is_partition: Set(false),
        belongs_to: Set(None),
        weight: Set(None),
        attrs: Set(None),
        dataset_id: Set(None),
        comment: Set(None),
        created_at: Set(Utc::now()),
    };
    node.insert(db).await
}

#[tokio::test]
async fn test_replay_node_updates() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    // Create initial nodes
    create_test_node(&db, graph.id, "node-1", Some("Original Label 1"))
        .await
        .unwrap();
    create_test_node(&db, graph.id, "node-2", Some("Original Label 2"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create edits to update labels
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Original Label 1")),
            Some(serde_json::json!("Updated Label 1")),
            None,
            false,
        )
        .await
        .unwrap();

    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-2".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Original Label 2")),
            Some(serde_json::json!("Updated Label 2")),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay edits
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.applied, 2);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.failed, 0);

    // Verify nodes were updated
    use layercake_core::database::entities::graph_nodes::{Column, Entity as GraphNodes};
    use sea_orm::{ColumnTrait, QueryFilter};

    let node1 = GraphNodes::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::Id.eq("node-1"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(node1.label, Some("Updated Label 1".to_string()));

    let node2 = GraphNodes::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::Id.eq("node-2"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(node2.label, Some("Updated Label 2".to_string()));
}

#[tokio::test]
async fn test_replay_with_missing_targets() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    // Create only one node
    create_test_node(&db, graph.id, "node-1", Some("Label 1"))
        .await
        .unwrap();

    let service = GraphEditService::new(db);

    // Create edits for both existing and non-existing nodes
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Label 1")),
            Some(serde_json::json!("Updated")),
            None,
            false,
        )
        .await
        .unwrap();

    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-missing".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Old")),
            Some(serde_json::json!("New")),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay edits
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.applied, 1);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.failed, 0);

    // Verify the skipped edit
    let skipped = summary
        .details
        .iter()
        .find(|d| d.result == "skipped")
        .unwrap();
    assert_eq!(skipped.target_id, "node-missing");
    assert!(skipped.message.contains("not found"));
}

#[tokio::test]
async fn test_replay_idempotence() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    create_test_node(&db, graph.id, "node-1", Some("Original"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create edit
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Original")),
            Some(serde_json::json!("Updated")),
            None,
            false,
        )
        .await
        .unwrap();

    // First replay
    let summary1 = service.replay_graph_edits(graph.id).await.unwrap();
    assert_eq!(summary1.applied, 1);

    // Second replay should have no unapplied edits
    let summary2 = service.replay_graph_edits(graph.id).await.unwrap();
    assert_eq!(summary2.total, 0);
    assert_eq!(summary2.applied, 0);

    // Verify node label is still correct
    use layercake_core::database::entities::graph_nodes::{Column, Entity as GraphNodes};
    use sea_orm::{ColumnTrait, QueryFilter};

    let node = GraphNodes::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::Id.eq("node-1"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(node.label, Some("Updated".to_string()));
}

#[tokio::test]
async fn test_replay_sequence_ordering() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    create_test_node(&db, graph.id, "node-1", Some("Start"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create sequence of edits that build on each other
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Start")),
            Some(serde_json::json!("Step 1")),
            None,
            false,
        )
        .await
        .unwrap();

    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Step 1")),
            Some(serde_json::json!("Step 2")),
            None,
            false,
        )
        .await
        .unwrap();

    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Step 2")),
            Some(serde_json::json!("Final")),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay all edits
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 3);
    assert_eq!(summary.applied, 3);

    // Verify final state
    use layercake_core::database::entities::graph_nodes::{Column, Entity as GraphNodes};
    use sea_orm::{ColumnTrait, QueryFilter};

    let node = GraphNodes::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::Id.eq("node-1"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(node.label, Some("Final".to_string()));
}

#[tokio::test]
async fn test_replay_node_create_and_edge_create() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    // Start with one node
    create_test_node(&db, graph.id, "node-1", Some("Node 1"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create edit to add second node
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-2".to_string(),
            "create".to_string(),
            None,
            None,
            Some(serde_json::json!({
                "label": "Node 2",
                "layer": "default",
                "isPartition": false,
                "attrs": {}
            })),
            None,
            false,
        )
        .await
        .unwrap();

    // Create edit to add edge between them
    service
        .create_edit(
            graph.id,
            "edge".to_string(),
            "edge-1-2".to_string(),
            "create".to_string(),
            None,
            None,
            Some(serde_json::json!({
                "source": "node-1",
                "target": "node-2",
                "label": "connects",
                "layer": "default"
            })),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay edits
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.applied, 2);
    assert_eq!(summary.skipped, 0);

    // Verify node was created
    use layercake_core::database::entities::graph_edges::{
        Column as EdgeColumn, Entity as GraphEdges,
    };
    use layercake_core::database::entities::graph_nodes::{
        Column as NodeColumn, Entity as GraphNodes,
    };
    use sea_orm::{ColumnTrait, QueryFilter};

    let node2 = GraphNodes::find()
        .filter(NodeColumn::GraphId.eq(graph.id))
        .filter(NodeColumn::Id.eq("node-2"))
        .one(&db)
        .await
        .unwrap();

    assert!(node2.is_some());
    assert_eq!(node2.unwrap().label, Some("Node 2".to_string()));

    // Verify edge was created
    let edge = GraphEdges::find()
        .filter(EdgeColumn::GraphId.eq(graph.id))
        .filter(EdgeColumn::Id.eq("edge-1-2"))
        .one(&db)
        .await
        .unwrap();

    assert!(edge.is_some());
    let edge = edge.unwrap();
    assert_eq!(edge.source, "node-1");
    assert_eq!(edge.target, "node-2");
}

#[tokio::test]
async fn test_replay_edge_without_source_target() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Try to create edge without source/target nodes existing
    service
        .create_edit(
            graph.id,
            "edge".to_string(),
            "edge-orphan".to_string(),
            "create".to_string(),
            None,
            None,
            Some(serde_json::json!({
                "source": "nonexistent-1",
                "target": "nonexistent-2",
                "label": "orphan"
            })),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay should skip this edit
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 1);
    assert_eq!(summary.applied, 0);
    assert_eq!(summary.skipped, 1);

    let skipped = &summary.details[0];
    assert!(skipped.message.contains("not found"));
}

#[tokio::test]
async fn test_replay_layer_operations() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db.clone());

    // Create layer
    service
        .create_edit(
            graph.id,
            "layer".to_string(),
            "layer-new".to_string(),
            "create".to_string(),
            None,
            None,
            Some(serde_json::json!({
                "name": "New Layer",
                "color": "FF0000",
                "properties": {
                    "background_color": "ffffff",
                    "border_color": "000000"
                }
            })),
            None,
            false,
        )
        .await
        .unwrap();

    // Update layer properties
    service
        .create_edit(
            graph.id,
            "layer".to_string(),
            "layer-new".to_string(),
            "update".to_string(),
            Some("properties".to_string()),
            Some(serde_json::json!({"background_color": "ffffff"})),
            Some(serde_json::json!({"background_color": "ff0000"})),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay
    let summary = service.replay_graph_edits(graph.id).await.unwrap();

    assert_eq!(summary.total, 2);
    assert_eq!(summary.applied, 2);

    // Verify layer exists with updated properties
    use layercake_core::database::entities::graph_layers::{Column, Entity as Layers};
    use sea_orm::{ColumnTrait, QueryFilter};

    let layer = Layers::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::LayerId.eq("layer-new"))
        .one(&db)
        .await
        .unwrap();

    assert!(layer.is_some());
    let layer = layer.unwrap();
    assert_eq!(layer.name, "New Layer");

    let props: serde_json::Value = serde_json::from_str(&layer.properties.unwrap()).unwrap();
    assert_eq!(props["background_color"], "ff0000");
}

#[tokio::test]
async fn test_applicator_node_delete() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    create_test_node(&db, graph.id, "node-to-delete", Some("Delete Me"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create delete edit
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-to-delete".to_string(),
            "delete".to_string(),
            None,
            Some(serde_json::json!({"label": "Delete Me"})),
            None,
            None,
            false,
        )
        .await
        .unwrap();

    // Apply the edit
    let applicator = GraphEditApplicator::new(db.clone());
    let edits = service.get_edits_for_graph(graph.id, true).await.unwrap();
    let result = applicator.apply_edit(&edits[0]).await.unwrap();

    assert!(matches!(result, ApplyResult::Success { .. }));

    // Verify node was deleted
    use layercake_core::database::entities::graph_nodes::{Column, Entity as GraphNodes};
    use sea_orm::{ColumnTrait, QueryFilter};

    let node = GraphNodes::find()
        .filter(Column::GraphId.eq(graph.id))
        .filter(Column::Id.eq("node-to-delete"))
        .one(&db)
        .await
        .unwrap();

    assert!(node.is_none());
}

#[tokio::test]
async fn test_graph_metadata_after_replay() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    create_test_node(&db, graph.id, "node-1", Some("Test"))
        .await
        .unwrap();

    let service = GraphEditService::new(db.clone());

    // Create edit
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Test")),
            Some(serde_json::json!("Updated")),
            None,
            false,
        )
        .await
        .unwrap();

    // Replay
    service.replay_graph_edits(graph.id).await.unwrap();

    // Check graph metadata
    use layercake_core::database::entities::graphs::Entity as Graphs;

    let updated_graph = Graphs::find_by_id(graph.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert!(updated_graph.last_replay_at.is_some());
    assert_eq!(updated_graph.last_edit_sequence, 1);
    assert_eq!(updated_graph.has_pending_edits, false);
}
