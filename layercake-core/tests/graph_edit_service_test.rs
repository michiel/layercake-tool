use layercake_core::database::entities::graphs;
use layercake_core::database::migrations::Migrator;
use layercake_core::services::GraphEditService;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, DbErr, Set};
use sea_orm_migration::MigratorTrait;

/// Create an in-memory SQLite database for testing
async fn setup_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // Run migrations
    Migrator::up(&db, None).await?;

    Ok(db)
}

/// Create a test graph
async fn create_test_graph(
    db: &DatabaseConnection,
    project_id: i32,
    name: &str,
) -> Result<graphs::Model, DbErr> {
    let graph = graphs::ActiveModel {
        project_id: Set(project_id),
        name: Set(name.to_string()),
        node_id: Set(format!("test_node_{}", uuid::Uuid::new_v4())),
        ..Default::default()
    };

    graph.insert(db).await
}

#[tokio::test]
async fn test_create_edit_with_sequence() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Create first edit
    let edit1 = service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Old Label")),
            Some(serde_json::json!("New Label")),
            None,
        )
        .await
        .unwrap();

    assert_eq!(edit1.sequence_number, 1);
    assert_eq!(edit1.applied, false);

    // Create second edit
    let edit2 = service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-2".to_string(),
            "update".to_string(),
            Some("layer".to_string()),
            Some(serde_json::json!("layer1")),
            Some(serde_json::json!("layer2")),
            None,
        )
        .await
        .unwrap();

    assert_eq!(edit2.sequence_number, 2);
    assert_eq!(edit2.applied, false);
}

#[tokio::test]
async fn test_get_edits_for_graph() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Create multiple edits
    for i in 1..=5 {
        service
            .create_edit(
                graph.id,
                "node".to_string(),
                format!("node-{}", i),
                "update".to_string(),
                Some("label".to_string()),
                Some(serde_json::json!(format!("Old {}", i))),
                Some(serde_json::json!(format!("New {}", i))),
                None,
            )
            .await
            .unwrap();
    }

    // Get all edits
    let edits = service.get_edits_for_graph(graph.id, false).await.unwrap();
    assert_eq!(edits.len(), 5);

    // Verify sequence order
    for (i, edit) in edits.iter().enumerate() {
        assert_eq!(edit.sequence_number, (i + 1) as i32);
    }
}

#[tokio::test]
async fn test_mark_edit_applied() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Create edit
    let edit = service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            Some("label".to_string()),
            Some(serde_json::json!("Old Label")),
            Some(serde_json::json!("New Label")),
            None,
        )
        .await
        .unwrap();

    assert_eq!(edit.applied, false);

    // Mark as applied
    service.mark_edit_applied(edit.id).await.unwrap();

    // Verify it's marked
    let edits = service.get_edits_for_graph(graph.id, false).await.unwrap();
    assert_eq!(edits[0].applied, true);

    // Verify unapplied filter works
    let unapplied = service.get_edits_for_graph(graph.id, true).await.unwrap();
    assert_eq!(unapplied.len(), 0);
}

#[tokio::test]
async fn test_clear_graph_edits() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Create multiple edits
    for i in 1..=3 {
        service
            .create_edit(
                graph.id,
                "node".to_string(),
                format!("node-{}", i),
                "update".to_string(),
                None,
                None,
                Some(serde_json::json!({"label": format!("Node {}", i)})),
                None,
            )
            .await
            .unwrap();
    }

    // Verify edits exist
    let edits = service.get_edits_for_graph(graph.id, false).await.unwrap();
    assert_eq!(edits.len(), 3);

    // Clear edits
    let deleted = service.clear_graph_edits(graph.id).await.unwrap();
    assert_eq!(deleted, 3);

    // Verify all cleared
    let edits = service.get_edits_for_graph(graph.id, false).await.unwrap();
    assert_eq!(edits.len(), 0);
}

#[tokio::test]
async fn test_edit_count() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    let service = GraphEditService::new(db);

    // Initially zero
    let count = service.get_edit_count(graph.id, false).await.unwrap();
    assert_eq!(count, 0);

    // Add edits
    for i in 1..=5 {
        service
            .create_edit(
                graph.id,
                "node".to_string(),
                format!("node-{}", i),
                "update".to_string(),
                None,
                None,
                Some(serde_json::json!(i)),
                None,
            )
            .await
            .unwrap();
    }

    // Total count
    let count = service.get_edit_count(graph.id, false).await.unwrap();
    assert_eq!(count, 5);

    // Mark some as applied
    let edits = service.get_edits_for_graph(graph.id, false).await.unwrap();
    service
        .mark_edits_applied(vec![edits[0].id, edits[1].id])
        .await
        .unwrap();

    // Unapplied count
    let unapplied_count = service.get_edit_count(graph.id, true).await.unwrap();
    assert_eq!(unapplied_count, 3);
}

#[tokio::test]
async fn test_graph_metadata_updated() {
    let db = setup_test_db().await.unwrap();
    let graph = create_test_graph(&db, 1, "Test Graph").await.unwrap();

    assert_eq!(graph.last_edit_sequence, 0);
    assert_eq!(graph.has_pending_edits, false);

    let service = GraphEditService::new(db.clone());

    // Create edit
    service
        .create_edit(
            graph.id,
            "node".to_string(),
            "node-1".to_string(),
            "update".to_string(),
            None,
            None,
            Some(serde_json::json!("test")),
            None,
        )
        .await
        .unwrap();

    // Verify graph metadata updated
    use layercake_core::database::entities::graphs::Entity as Graphs;
    use sea_orm::EntityTrait;

    let updated_graph = Graphs::find_by_id(graph.id)
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated_graph.last_edit_sequence, 1);
    assert_eq!(updated_graph.has_pending_edits, true);
}

#[tokio::test]
async fn test_multiple_graphs_independent_sequences() {
    let db = setup_test_db().await.unwrap();
    let graph1 = create_test_graph(&db, 1, "Graph 1").await.unwrap();
    let graph2 = create_test_graph(&db, 1, "Graph 2").await.unwrap();

    let service = GraphEditService::new(db);

    // Create edits for graph1
    for i in 1..=3 {
        service
            .create_edit(
                graph1.id,
                "node".to_string(),
                format!("node-{}", i),
                "update".to_string(),
                None,
                None,
                Some(serde_json::json!(i)),
                None,
            )
            .await
            .unwrap();
    }

    // Create edits for graph2
    for i in 1..=2 {
        service
            .create_edit(
                graph2.id,
                "node".to_string(),
                format!("node-{}", i),
                "update".to_string(),
                None,
                None,
                Some(serde_json::json!(i)),
                None,
            )
            .await
            .unwrap();
    }

    // Verify independent sequences
    let edits1 = service.get_edits_for_graph(graph1.id, false).await.unwrap();
    assert_eq!(edits1.len(), 3);
    assert_eq!(edits1.last().unwrap().sequence_number, 3);

    let edits2 = service.get_edits_for_graph(graph2.id, false).await.unwrap();
    assert_eq!(edits2.len(), 2);
    assert_eq!(edits2.last().unwrap().sequence_number, 2);
}
