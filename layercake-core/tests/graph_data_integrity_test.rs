//! Integrity tests for GraphDataService added in Horizon 1 (Stabilise):
//! - `replace_contents` persists nodes and edges atomically with correct counts.
//! - `reconcile_interrupted_processing` recovers rows stuck in `Processing`.
//!
//! See plans/20260709-horizon-1-stabilise.md (Stages 2 & 5).

use layercake as layercake_core;
use layercake_core::database::entities::graph_data::GraphDataStatus;
use layercake_core::database::entities::{
    graph_data, graph_data_edges, graph_data_nodes, graph_edits, projects,
};
use layercake_core::database::migrations::Migrator;
use layercake_core::services::{
    GraphDataCreate, GraphDataEdgeInput, GraphDataNodeInput, GraphDataService, GraphEditService,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
    Set,
};
use sea_orm_migration::MigratorTrait;
use serde_json::json;

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

// --- Stage 5: sequence uniqueness (R9) and applicator ordering/idempotence ---
//
// These exercise both the direct-insert path (to prove the existing unique
// index `uq_graph_edits_graph_sequence` is enforced) and the full
// GraphEditService::create_edit path. The latter only works because the
// m20260709 migration rebuilt graph_edits to drop its dangling FK to the
// (dropped) `graphs` table — before that, any insert failed with
// "no such table: graphs".

#[allow(clippy::too_many_arguments)]
async fn insert_edit(
    db: &DatabaseConnection,
    graph_id: i32,
    sequence: i32,
    target_id: &str,
    operation: &str,
    field_name: Option<&str>,
    new_value: serde_json::Value,
) -> Result<graph_edits::Model, DbErr> {
    graph_edits::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        graph_id: Set(graph_id),
        target_type: Set("node".to_string()),
        target_id: Set(target_id.to_string()),
        operation: Set(operation.to_string()),
        field_name: Set(field_name.map(|s| s.to_string())),
        old_value: Set(None),
        new_value: Set(Some(new_value)),
        sequence_number: Set(sequence),
        applied: Set(false),
        created_at: Set(chrono::Utc::now()),
        created_by: Set(None),
    }
    .insert(db)
    .await
}

#[tokio::test]
async fn duplicate_sequence_number_is_rejected_by_unique_index() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let gd = create_graph_data(&GraphDataService::new(db.clone()), 1).await;

    insert_edit(&db, gd.id, 1, "n1", "create", None, json!({ "label": "A" }))
        .await
        .expect("first insert at sequence 1 succeeds");

    // A second edit on the same graph reusing sequence 1 must be rejected by
    // the unique (graph_id, sequence_number) index — this is the collision that
    // GraphEditService::create_edit now retries around instead of duplicating.
    let dup = insert_edit(&db, gd.id, 1, "n2", "create", None, json!({ "label": "B" })).await;
    assert!(dup.is_err(), "duplicate sequence number must be rejected");

    // The same sequence number is fine for a *different* graph.
    let gd2 = create_graph_data(&GraphDataService::new(db.clone()), 1).await;
    insert_edit(
        &db,
        gd2.id,
        1,
        "n1",
        "create",
        None,
        json!({ "label": "C" }),
    )
    .await
    .expect("sequence 1 on a different graph is allowed");
}

#[tokio::test]
async fn concurrent_create_edit_allocates_unique_sequences() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let gd = create_graph_data(&GraphDataService::new(db.clone()), 1).await;

    // Fire many create_edit calls concurrently on the same graph. Without the
    // retry-on-conflict added in Stage 5, the max+1 allocation would collide on
    // the unique index and some inserts would hard-fail.
    let mut handles = Vec::new();
    for i in 0..20 {
        let db = db.clone();
        let graph_id = gd.id;
        handles.push(tokio::spawn(async move {
            GraphEditService::new(db)
                .create_edit(
                    graph_id,
                    "node".to_string(),
                    format!("n{}", i),
                    "create".to_string(),
                    None,
                    None,
                    Some(json!({ "label": format!("Node {}", i) })),
                    None,
                    false,
                )
                .await
        }));
    }

    let mut ok = 0;
    for h in handles {
        if h.await.unwrap().is_ok() {
            ok += 1;
        }
    }
    assert_eq!(
        ok, 20,
        "all concurrent create_edit calls succeed after retry"
    );

    let edits = graph_edits::Entity::find()
        .filter(graph_edits::Column::GraphId.eq(gd.id))
        .all(&db)
        .await
        .unwrap();
    let mut seqs: Vec<i32> = edits.iter().map(|e| e.sequence_number).collect();
    seqs.sort_unstable();
    let unique = {
        let mut s = seqs.clone();
        s.dedup();
        s.len()
    };
    assert_eq!(seqs.len(), 20);
    assert_eq!(unique, 20, "no duplicate sequence numbers");
}

#[tokio::test]
async fn replay_is_ordered_and_idempotent() {
    let db = setup_test_db().await.unwrap();
    ensure_project(&db, 1).await;
    let gd_service = GraphDataService::new(db.clone());
    let gd = create_graph_data(&gd_service, 1).await;

    // Create node (seq 1) then update its label (seq 2) — order matters.
    insert_edit(
        &db,
        gd.id,
        1,
        "n1",
        "create",
        None,
        json!({ "label": "First" }),
    )
    .await
    .unwrap();
    insert_edit(
        &db,
        gd.id,
        2,
        "n1",
        "update",
        Some("label"),
        json!("Second"),
    )
    .await
    .unwrap();

    // First replay applies both edits in sequence order.
    let summary = gd_service.replay_edits(gd.id).await.unwrap();
    assert_eq!(summary.applied, 2);

    let node = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(gd.id))
        .filter(graph_data_nodes::Column::ExternalId.eq("n1"))
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        node.label.as_deref(),
        Some("Second"),
        "update applied after create (ordered)"
    );

    // Second replay is idempotent: edits are already applied, so nothing new
    // runs and the state is unchanged.
    let summary2 = gd_service.replay_edits(gd.id).await.unwrap();
    assert_eq!(summary2.applied, 0, "no unapplied edits remain");

    let count = graph_data_nodes::Entity::find()
        .filter(graph_data_nodes::Column::GraphDataId.eq(gd.id))
        .all(&db)
        .await
        .unwrap()
        .len();
    assert_eq!(count, 1, "replay did not duplicate the node");
}
