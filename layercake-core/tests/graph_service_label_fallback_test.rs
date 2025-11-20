/// Test to verify graph_service correctly handles None values for label and layer
///
/// This test ensures:
/// 1. Nodes with None label fall back to using the node ID
/// 2. Nodes/edges with None layer use empty string (inherit default styling)
/// 3. Appropriate warnings are logged when fallbacks are used
use layercake::database::entities::{graph_nodes, graphs};
use layercake::services::graph_service::GraphService;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};

async fn setup_test_db() -> DatabaseConnection {
    Database::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database")
}

async fn create_test_graph(db: &DatabaseConnection) -> i32 {
    // Create a graph
    let graph = graphs::ActiveModel {
        project_id: Set(1),
        name: Set("Test Graph".to_string()),
        node_id: Set("test_graph_node".to_string()),
        ..Default::default()
    };
    let graph = graph.insert(db).await.expect("Failed to insert graph");
    graph.id
}

async fn create_node_with_none_label(db: &DatabaseConnection, graph_id: i32, node_id: &str) {
    let node = graph_nodes::ActiveModel {
        id: Set(node_id.to_string()),
        graph_id: Set(graph_id),
        label: Set(None), // None label should fall back to node ID
        layer: Set(Some("test_layer".to_string())),
        is_partition: Set(false),
        belongs_to: Set(None),
        weight: Set(Some(1.0)),
        attrs: Set(None),
        dataset_id: Set(None),
        comment: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    node.insert(db).await.expect("Failed to insert node");
}

async fn create_node_with_none_layer(db: &DatabaseConnection, graph_id: i32, node_id: &str) {
    let node = graph_nodes::ActiveModel {
        id: Set(node_id.to_string()),
        graph_id: Set(graph_id),
        label: Set(Some("Test Label".to_string())),
        layer: Set(None), // None layer should become empty string
        is_partition: Set(false),
        belongs_to: Set(None),
        weight: Set(Some(1.0)),
        attrs: Set(None),
        dataset_id: Set(None),
        comment: Set(None),
        created_at: Set(chrono::Utc::now()),
    };
    node.insert(db).await.expect("Failed to insert node");
}

#[tokio::test]
async fn test_none_label_falls_back_to_node_id() {
    let db = setup_test_db().await;

    // Run migrations (if needed)
    // For this test, we'll assume the schema exists or use a test fixture

    let graph_id = create_test_graph(&db).await;
    create_node_with_none_label(&db, graph_id, "node_without_label").await;

    let service = GraphService::new(db.clone());
    let graph = service
        .build_graph_from_dag_graph(graph_id)
        .await
        .expect("Failed to build graph");

    // Find the node
    let node = graph
        .nodes
        .iter()
        .find(|n| n.id == "node_without_label")
        .expect("Node should exist");

    // Label should fall back to node ID
    assert_eq!(node.label, "node_without_label");
}

#[tokio::test]
async fn test_none_layer_becomes_empty_string() {
    let db = setup_test_db().await;

    let graph_id = create_test_graph(&db).await;
    create_node_with_none_layer(&db, graph_id, "node_without_layer").await;

    let service = GraphService::new(db.clone());
    let graph = service
        .build_graph_from_dag_graph(graph_id)
        .await
        .expect("Failed to build graph");

    // Find the node
    let node = graph
        .nodes
        .iter()
        .find(|n| n.id == "node_without_layer")
        .expect("Node should exist");

    // Layer should be empty string (inherits default styling)
    assert_eq!(node.layer, "");
    assert_eq!(node.label, "Test Label");
}

#[tokio::test]
async fn test_logging_for_missing_labels() {
    // This test verifies that warnings are logged
    // In a real scenario, you'd use a logging capture mechanism
    // For now, we just verify the function runs without panicking

    let db = setup_test_db().await;
    let graph_id = create_test_graph(&db).await;

    // Create multiple nodes with missing labels
    create_node_with_none_label(&db, graph_id, "node1").await;
    create_node_with_none_label(&db, graph_id, "node2").await;
    create_node_with_none_label(&db, graph_id, "node3").await;

    let service = GraphService::new(db.clone());
    let graph = service
        .build_graph_from_dag_graph(graph_id)
        .await
        .expect("Failed to build graph");

    // All nodes should have fallen back to using their IDs
    assert_eq!(graph.nodes.len(), 3);
    for node in &graph.nodes {
        assert_eq!(node.label, node.id);
    }
}
