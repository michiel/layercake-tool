//! Integrity tests for GraphDataService added in Horizon 1 (Stabilise):
//! - `replace_contents` persists nodes and edges atomically with correct counts.
//! - `reconcile_interrupted_processing` recovers rows stuck in `Processing`.
//!
//! See plans/20260709-horizon-1-stabilise.md (Stages 2 & 5).

use layercake as layercake_core;
use layercake_core::database::entities::graph_data::GraphDataStatus;
use layercake_core::database::entities::{
    graph_data, graph_data_edges, graph_data_nodes, projects,
};
use layercake_core::database::migrations::Migrator;
use layercake_core::services::{
    GraphDataCreate, GraphDataEdgeInput, GraphDataNodeInput, GraphDataService,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set,
};
use sea_orm_migration::MigratorTrait;

async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

async fn ensure_project(db: &DatabaseConnection, project_id: i32) {
    if projects::Entity::find_by_id(project_id)
        .one(db)
        .await
        .unwrap()
        .is_none()
    {
        let mut project = projects::ActiveModel::new();
        project.id = Set(project_id);
        project.name = Set(format!("Test Project {}", project_id));
        project.description = Set(Some("Test project".to_string()));
        project.insert(db).await.unwrap();
    }
}

async fn create_graph_data(service: &GraphDataService, project_id: i32) -> graph_data::Model {
    service
        .create(GraphDataCreate {
            project_id,
            name: "integrity-graph".to_string(),
            source_type: "computed".to_string(),
            dag_node_id: None,
            file_format: None,
            origin: None,
            filename: None,
            blob: None,
            file_size: None,
            processed_at: None,
            source_hash: None,
            computed_date: None,
            last_edit_sequence: None,
            has_pending_edits: None,
            last_replay_at: None,
            metadata: None,
            annotations: None,
            status: Some(GraphDataStatus::Active),
        })
        .await
        .unwrap()
}

fn node(external_id: &str) -> GraphDataNodeInput {
    GraphDataNodeInput {
        external_id: external_id.to_string(),
        label: Some(external_id.to_string()),
        layer: None,
        weight: None,
        is_partition: None,
        belongs_to: None,
        comment: None,
        source_dataset_id: None,
        attributes: None,
        created_at: None,
    }
}

fn edge(external_id: &str, source: &str, target: &str) -> GraphDataEdgeInput {
    GraphDataEdgeInput {
        external_id: external_id.to_string(),
        source: source.to_string(),
        target: target.to_string(),
        label: None,
        layer: None,
        weight: None,
        comment: None,
        source_dataset_id: None,
        attributes: None,
        created_at: None,
    }
}

#[tokio::test]
async fn replace_contents_persists_nodes_and_edges_with_correct_counts() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let service = GraphDataService::new(db.clone());
    let gd = create_graph_data(&service, 1).await;

    service
        .replace_contents(
            gd.id,
            vec![node("a"), node("b"), node("c")],
            vec![edge("e1", "a", "b"), edge("e2", "b", "c")],
        )
        .await
        .unwrap();

    // Rows persisted
    let nodes = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(gd.id))
        .all(&db)
        .await
        .unwrap();
    let edges = graph_data_edges::Entity::find()
        .filter(graph_data_edges::Column::GraphDataId.eq(gd.id))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 3, "all nodes persisted");
    assert_eq!(edges.len(), 2, "all edges persisted");

    // Counts on the graph_data row reflect both nodes AND edges (regression:
    // previously replace_nodes then replace_edges could leave edge_count wrong
    // if the second call failed).
    let reloaded = graph_data::Entity::find_by_id(gd.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.node_count, 3);
    assert_eq!(reloaded.edge_count, 2);
}

#[tokio::test]
async fn replace_contents_replaces_previous_contents() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let service = GraphDataService::new(db.clone());
    let gd = create_graph_data(&service, 1).await;

    service
        .replace_contents(
            gd.id,
            vec![node("a"), node("b")],
            vec![edge("e1", "a", "b")],
        )
        .await
        .unwrap();

    // Replace with a smaller set — old rows must be gone.
    service
        .replace_contents(gd.id, vec![node("x")], vec![])
        .await
        .unwrap();

    let nodes = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(gd.id))
        .all(&db)
        .await
        .unwrap();
    let edges = graph_data_edges::Entity::find()
        .filter(graph_data_edges::Column::GraphDataId.eq(gd.id))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].external_id, "x");
    assert_eq!(edges.len(), 0);

    let reloaded = graph_data::Entity::find_by_id(gd.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.node_count, 1);
    assert_eq!(reloaded.edge_count, 0);
}

#[tokio::test]
async fn reconcile_moves_stuck_processing_rows_to_error() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let service = GraphDataService::new(db.clone());

    // One row stuck Processing (simulating an interrupted execution), one Active.
    let stuck = create_graph_data(&service, 1).await;
    service.mark_processing(stuck.id).await.unwrap();
    let healthy = create_graph_data(&service, 1).await;

    let reconciled = service.reconcile_interrupted_processing().await.unwrap();
    assert_eq!(reconciled, 1, "exactly the stuck row is reconciled");

    let stuck_after = graph_data::Entity::find_by_id(stuck.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let error_status: String = GraphDataStatus::Error.into();
    assert_eq!(stuck_after.status, error_status);
    assert!(stuck_after.error_message.is_some());

    // The healthy row is untouched.
    let healthy_after = graph_data::Entity::find_by_id(healthy.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let active_status: String = GraphDataStatus::Active.into();
    assert_eq!(healthy_after.status, active_status);

    // Idempotent: running again reconciles nothing.
    assert_eq!(service.reconcile_interrupted_processing().await.unwrap(), 0);
}
