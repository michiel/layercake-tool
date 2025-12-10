use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{
    graph_data, project_layers, projects,
};
use layercake_core::database::migrations::Migrator;
use layercake_core::pipeline::GraphDataBuilder;
use layercake_core::services::{GraphDataCreate, GraphDataEdgeInput, GraphDataNodeInput, GraphDataService, LayerPaletteService};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, Database, Set};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn seed_project_and_palette(db: &DatabaseConnection) -> i32 {
    // Create project
    let project = projects::ActiveModel {
        id: Set(1),
        name: Set("Test Project".into()),
        description: Set(None),
        tags: Set("[]".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    project.insert(db).await.unwrap();

    // Create layer palette
    let layer = project_layers::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        project_id: Set(1),
        layer_id: Set("L1".into()),
        name: Set("Layer 1".into()),
        background_color: Set("#fff".into()),
        text_color: Set("#000".into()),
        border_color: Set("#000".into()),
        alias: Set(None),
        source_dataset_id: Set(None),
        enabled: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    layer.insert(db).await.unwrap();

    1
}

async fn create_test_graph_data(
    service: &GraphDataService,
    project_id: i32,
    name: &str,
    source_type: &str,
) -> graph_data::Model {
    service
        .create(GraphDataCreate {
            project_id,
            name: name.to_string(),
            source_type: source_type.to_string(),
            dag_node_id: None,
            file_format: None,
            origin: None,
            filename: None,
            blob: None,
            file_size: None,
            processed_at: Some(Utc::now()),
            source_hash: None,
            computed_date: None,
            last_edit_sequence: None,
            has_pending_edits: None,
            last_replay_at: None,
            metadata: None,
            annotations: None,
            status: Some(graph_data::GraphDataStatus::Active),
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn test_graph_data_builder_merge_upstream() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;

    let service = Arc::new(GraphDataService::new(db.clone()));
    let palette_service = Arc::new(LayerPaletteService::new(db.clone()));

    // Create two datasets with nodes
    let ds1 = create_test_graph_data(&service, project_id, "Dataset 1", "dataset").await;
    service
        .replace_nodes(
            ds1.id,
            vec![GraphDataNodeInput {
                external_id: "n1".to_string(),
                label: Some("Node 1".to_string()),
                layer: Some("L1".to_string()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: Some(ds1.id),
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    let ds2 = create_test_graph_data(&service, project_id, "Dataset 2", "dataset").await;
    service
        .replace_nodes(
            ds2.id,
            vec![GraphDataNodeInput {
                external_id: "n2".to_string(),
                label: Some("Node 2".to_string()),
                layer: Some("L1".to_string()),
                weight: Some(2.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: Some(ds2.id),
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    // Build a graph that merges the two datasets
    let builder = GraphDataBuilder::new(service.clone(), palette_service);
    let result = builder
        .build_graph(
            project_id,
            "test-dag-node".to_string(),
            "Merged Graph".to_string(),
            vec![ds1.id, ds2.id],
        )
        .await;

    assert!(result.is_ok(), "Graph build should succeed");
    let graph = result.unwrap();

    // Verify the merged graph
    assert_eq!(graph.source_type, "computed");
    assert_eq!(graph.node_count, 2); // Should have both nodes

    // Load nodes to verify
    let nodes = service.load_nodes(graph.id).await.unwrap();
    assert_eq!(nodes.len(), 2);

    let external_ids: Vec<String> = nodes.iter().map(|n| n.external_id.clone()).collect();
    assert!(external_ids.contains(&"n1".to_string()));
    assert!(external_ids.contains(&"n2".to_string()));
}

#[tokio::test]
async fn test_graph_data_builder_layer_validation() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;

    let service = Arc::new(GraphDataService::new(db.clone()));
    let palette_service = Arc::new(LayerPaletteService::new(db.clone()));

    // Create dataset with invalid layer
    let ds1 = create_test_graph_data(&service, project_id, "Dataset 1", "dataset").await;
    service
        .replace_nodes(
            ds1.id,
            vec![GraphDataNodeInput {
                external_id: "n1".to_string(),
                label: Some("Node 1".to_string()),
                layer: Some("INVALID_LAYER".to_string()), // Not in palette
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: Some(ds1.id),
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    // Build should fail due to missing layer
    let builder = GraphDataBuilder::new(service, palette_service);
    let result = builder
        .build_graph(
            project_id,
            "test-dag-node".to_string(),
            "Graph with invalid layer".to_string(),
            vec![ds1.id],
        )
        .await;

    assert!(result.is_err(), "Graph build should fail for missing layer");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Missing layers"));
}

#[tokio::test]
async fn test_graph_data_builder_change_detection() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;

    let service = Arc::new(GraphDataService::new(db.clone()));
    let palette_service = Arc::new(LayerPaletteService::new(db.clone()));

    // Create dataset
    let ds1 = create_test_graph_data(&service, project_id, "Dataset 1", "dataset").await;
    service
        .replace_nodes(
            ds1.id,
            vec![GraphDataNodeInput {
                external_id: "n1".to_string(),
                label: Some("Node 1".to_string()),
                layer: Some("L1".to_string()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: Some(ds1.id),
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    let builder = GraphDataBuilder::new(service.clone(), palette_service);

    // Build graph first time
    let graph1 = builder
        .build_graph(
            project_id,
            "test-dag-node".to_string(),
            "Test Graph".to_string(),
            vec![ds1.id],
        )
        .await
        .unwrap();

    let hash1 = graph1.source_hash.clone();
    assert!(hash1.is_some(), "First build should have hash");

    // Build again with same inputs - should reuse existing graph
    let graph2 = builder
        .build_graph(
            project_id,
            "test-dag-node".to_string(),
            "Test Graph".to_string(),
            vec![ds1.id],
        )
        .await
        .unwrap();

    // Should be the same graph (reused)
    assert_eq!(graph1.id, graph2.id, "Should reuse existing graph");
    assert_eq!(
        graph1.source_hash, graph2.source_hash,
        "Hash should be unchanged"
    );
}

#[tokio::test]
async fn test_graph_data_convenience_methods() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;

    let service = GraphDataService::new(db.clone());

    // Test create_computed
    let computed = service
        .create_computed(project_id, "dag-node-1".to_string(), "Computed Graph".to_string())
        .await
        .unwrap();

    assert_eq!(computed.source_type, "computed");
    assert_eq!(computed.dag_node_id, Some("dag-node-1".to_string()));
    assert_eq!(computed.status, graph_data::GraphDataStatus::Processing.as_str());

    // Test create_from_json
    let dataset = service
        .create_from_json(project_id, "Dataset from JSON".to_string(), None)
        .await
        .unwrap();

    assert_eq!(dataset.source_type, "dataset");
    assert_eq!(dataset.file_format, Some("json".to_string()));
    assert_eq!(dataset.status, graph_data::GraphDataStatus::Active.as_str());

    // Test list_datasets and list_computed
    let datasets = service.list_datasets(project_id).await.unwrap();
    assert_eq!(datasets.len(), 1);

    let computed_graphs = service.list_computed(project_id).await.unwrap();
    assert_eq!(computed_graphs.len(), 1);

    // Test get_by_dag_node
    let found = service.get_by_dag_node("dag-node-1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, computed.id);

    // Test status transitions
    service.mark_processing(computed.id).await.unwrap();
    let processing = service.get_by_id(computed.id).await.unwrap().unwrap();
    assert_eq!(processing.status, graph_data::GraphDataStatus::Processing.as_str());

    service
        .mark_complete(computed.id, "test-hash".to_string())
        .await
        .unwrap();
    let active = service.get_by_id(computed.id).await.unwrap().unwrap();
    assert_eq!(active.status, graph_data::GraphDataStatus::Active.as_str());
    assert_eq!(active.source_hash, Some("test-hash".to_string()));

    service
        .mark_error(computed.id, "Test error".to_string())
        .await
        .unwrap();
    let error = service.get_by_id(computed.id).await.unwrap().unwrap();
    assert_eq!(error.status, graph_data::GraphDataStatus::Error.as_str());
    assert_eq!(error.error_message, Some("Test error".to_string()));
}

#[tokio::test]
async fn test_graph_data_lazy_loading() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;

    let service = GraphDataService::new(db);

    // Create graph with nodes and edges
    let graph = service
        .create_from_json(project_id, "Test Graph".to_string(), None)
        .await
        .unwrap();

    service
        .replace_nodes(
            graph.id,
            vec![
                GraphDataNodeInput {
                    external_id: "n1".to_string(),
                    label: Some("Node 1".to_string()),
                    layer: Some("L1".to_string()),
                    weight: Some(1.0),
                    is_partition: Some(false),
                    belongs_to: None,
                    comment: None,
                    source_dataset_id: None,
                    attributes: None,
                    created_at: None,
                },
                GraphDataNodeInput {
                    external_id: "n2".to_string(),
                    label: Some("Node 2".to_string()),
                    layer: Some("L1".to_string()),
                    weight: Some(2.0),
                    is_partition: Some(false),
                    belongs_to: None,
                    comment: None,
                    source_dataset_id: None,
                    attributes: None,
                    created_at: None,
                },
            ],
        )
        .await
        .unwrap();

    service
        .replace_edges(
            graph.id,
            vec![GraphDataEdgeInput {
                external_id: "e1".to_string(),
                source: "n1".to_string(),
                target: "n2".to_string(),
                label: Some("Edge 1".to_string()),
                layer: Some("L1".to_string()),
                weight: Some(1.5),
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    // Test load_nodes
    let nodes = service.load_nodes(graph.id).await.unwrap();
    assert_eq!(nodes.len(), 2);

    // Test load_edges
    let edges = service.load_edges(graph.id).await.unwrap();
    assert_eq!(edges.len(), 1);

    // Test load_full
    let (full_graph, full_nodes, full_edges) = service.load_full(graph.id).await.unwrap();
    assert_eq!(full_graph.id, graph.id);
    assert_eq!(full_nodes.len(), 2);
    assert_eq!(full_edges.len(), 1);
    assert_eq!(full_graph.node_count, 2);
    assert_eq!(full_graph.edge_count, 1);
}
