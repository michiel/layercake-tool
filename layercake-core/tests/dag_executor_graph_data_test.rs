use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{graph_data, plan_dag_nodes, project_layers, projects};
use layercake_core::database::migrations::Migrator;
use layercake_core::pipeline::DagExecutor;
use layercake_core::services::{
    GraphDataCreate, GraphDataEdgeInput, GraphDataNodeInput, GraphDataService,
};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, Database, Set};
use sea_orm_migration::MigratorTrait;
use serde_json::json;

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();

    // Disable foreign key checks since we're testing graph_data path
    use sea_orm::Statement;
    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "PRAGMA foreign_keys = OFF;".to_string(),
    ))
    .await
    .unwrap();

    db
}

async fn seed_project_and_palette(db: &DatabaseConnection) -> i32 {
    // Create project
    let project = projects::ActiveModel {
        id: Set(1),
        name: Set("DAG Test Project".into()),
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

async fn create_test_dataset(
    service: &GraphDataService,
    project_id: i32,
    name: &str,
) -> graph_data::Model {
    create_dataset_with_edges(service, project_id, name, vec![], vec![]).await
}

async fn create_dataset_with_edges(
    service: &GraphDataService,
    project_id: i32,
    name: &str,
    nodes: Vec<GraphDataNodeInput>,
    edges: Vec<layercake_core::services::GraphDataEdgeInput>,
) -> graph_data::Model {
    let dataset = service
        .create(GraphDataCreate {
            project_id,
            name: name.to_string(),
            source_type: "dataset".to_string(),
            dag_node_id: None,
            file_format: Some("json".to_string()),
            origin: None,
            filename: None,
            blob: None,
            file_size: None,
            processed_at: Some(Utc::now()),
            source_hash: Some(format!("hash-{}", name)),
            computed_date: None,
            last_edit_sequence: None,
            has_pending_edits: None,
            last_replay_at: None,
            metadata: None,
            annotations: None,
            status: Some(graph_data::GraphDataStatus::Active),
        })
        .await
        .unwrap();

    // Add some nodes to the dataset
    service
        .replace_nodes(
            dataset.id,
            if nodes.is_empty() {
                vec![
                    GraphDataNodeInput {
                        external_id: format!("{}-n1", name),
                        label: Some(format!("{} Node 1", name)),
                        layer: Some("L1".to_string()),
                        weight: Some(1.0),
                        is_partition: Some(false),
                        belongs_to: None,
                        comment: None,
                        source_dataset_id: Some(dataset.id),
                        attributes: None,
                        created_at: None,
                    },
                    GraphDataNodeInput {
                        external_id: format!("{}-n2", name),
                        label: Some(format!("{} Node 2", name)),
                        layer: Some("L1".to_string()),
                        weight: Some(2.0),
                        is_partition: Some(false),
                        belongs_to: None,
                        comment: None,
                        source_dataset_id: Some(dataset.id),
                        attributes: None,
                        created_at: None,
                    },
                ]
            } else {
                nodes
            },
        )
        .await
        .unwrap();

    if !edges.is_empty() {
        service
            .replace_edges(
                dataset.id,
                edges
                    .into_iter()
                    .map(|mut e| {
                        let mut edge = e;
                        edge.source_dataset_id = Some(dataset.id);
                        edge
                    })
                    .collect(),
            )
            .await
            .unwrap();
    }

    dataset
}

#[tokio::test]
async fn test_dag_executor_simple_graph_build() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Create a source dataset (via graph_data)
    let dataset = create_test_dataset(&service, project_id, "Source").await;

    // Create DAG with just GraphNode referencing the dataset
    // (No DataSetNode needed - that's for legacy data_sets table)
    let nodes = vec![plan_dag_nodes::Model {
        id: "graph-node".to_string(),
        plan_id: 1,
        node_type: "GraphNode".to_string(),
        position_x: 0.0,
        position_y: 0.0,
        source_position: None,
        target_position: None,
        metadata_json: json!({"label": "Graph Node"}).to_string(),
        config_json: json!({"graphDataIds": [dataset.id]}).to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let edges: Vec<(String, String)> = vec![];

    // Execute DAG
    let executor = DagExecutor::new(db.clone());
    let result = executor.execute_dag(project_id, 1, &nodes, &edges).await;

    assert!(result.is_ok(), "DAG execution should succeed");

    // Verify graph was created via GraphDataBuilder
    let computed_graphs = service.list_computed(project_id).await.unwrap();
    assert_eq!(
        computed_graphs.len(),
        1,
        "Should have created one computed graph"
    );

    let graph = &computed_graphs[0];
    assert_eq!(graph.dag_node_id, Some("graph-node".to_string()));
    assert_eq!(graph.source_type, "computed");
    assert_eq!(
        graph.node_count, 2,
        "Should have merged 2 nodes from dataset"
    );

    // Verify nodes were copied
    let nodes = service.load_nodes(graph.id).await.unwrap();
    assert_eq!(nodes.len(), 2);
}

#[tokio::test]
async fn test_dag_executor_graph_chaining() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Create two source datasets
    let dataset1 = create_test_dataset(&service, project_id, "DS1").await;
    let dataset2 = create_test_dataset(&service, project_id, "DS2").await;

    // Create DAG: Graph1 (merges DS1 + DS2) -> Graph2 (chains from Graph1)
    let nodes = vec![
        plan_dag_nodes::Model {
            id: "graph1-node".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 0.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Merged Graph"}).to_string(),
            config_json: json!({"graphDataIds": [dataset1.id, dataset2.id]}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "graph2-node".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 100.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Chained Graph"}).to_string(),
            // Will be updated after graph1 is built
            config_json: json!({"graphDataIds": []}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let edges = vec![("graph1-node".to_string(), "graph2-node".to_string())];

    // Execute DAG for graph1 only
    let executor = DagExecutor::new(db.clone());
    executor
        .execute_dag(project_id, 1, &nodes[0..1], &[])
        .await
        .unwrap();

    // Get graph1's ID to use in graph2
    let graph1 = service.get_by_dag_node("graph1-node").await.unwrap();
    assert!(graph1.is_some(), "Graph1 should be created");
    let graph1 = graph1.unwrap();
    assert_eq!(
        graph1.node_count, 4,
        "Graph1 should merge 4 nodes (2 from each dataset)"
    );

    // Update graph2 config to chain from graph1
    let mut nodes_with_chain = nodes.clone();
    nodes_with_chain[1].config_json = json!({"graphDataIds": [graph1.id]}).to_string();

    // Execute complete DAG including graph2
    executor
        .execute_dag(project_id, 1, &nodes_with_chain, &edges)
        .await
        .unwrap();

    // Verify both graphs exist
    let computed_graphs = service.list_computed(project_id).await.unwrap();
    assert_eq!(computed_graphs.len(), 2, "Should have two computed graphs");

    // Verify graph2 was built from graph1
    let graph2 = service.get_by_dag_node("graph2-node").await.unwrap();
    assert!(graph2.is_some(), "Graph2 should be created");
    let graph2 = graph2.unwrap();

    // Graph2 should have the same nodes as graph1 (4 total: 2 from DS1 + 2 from DS2)
    assert_eq!(
        graph2.node_count, graph1.node_count,
        "Graph2 should inherit graph1's merged nodes"
    );
    assert_eq!(graph2.node_count, 4, "Should have 4 nodes total");
}

#[tokio::test]
async fn test_merge_preserves_edges_and_partition_flags() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    let dataset_a = create_dataset_with_edges(
        &service,
        project_id,
        "A",
        vec![
            GraphDataNodeInput {
                external_id: "A-root".into(),
                label: Some("Root".into()),
                layer: Some("L1".into()),
                weight: Some(1.0),
                is_partition: Some(true),
                belongs_to: None,
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            },
            GraphDataNodeInput {
                external_id: "A-child".into(),
                label: Some("Child".into()),
                layer: Some("L1".into()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: Some("A-root".into()),
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            },
        ],
        (0..3)
            .map(|i| GraphDataEdgeInput {
                external_id: format!("A-e{}", i),
                source: "A-root".into(),
                target: "A-child".into(),
                label: Some(format!("A edge {}", i)),
                layer: Some("L1".into()),
                weight: Some(1.0),
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            })
            .collect(),
    )
    .await;

    let dataset_b = create_dataset_with_edges(
        &service,
        project_id,
        "B",
        vec![
            GraphDataNodeInput {
                external_id: "B-n1".into(),
                label: Some("B1".into()),
                layer: Some("L1".into()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            },
            GraphDataNodeInput {
                external_id: "B-n2".into(),
                label: Some("B2".into()),
                layer: Some("L1".into()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            },
            GraphDataNodeInput {
                external_id: "B-n3".into(),
                label: Some("B3".into()),
                layer: Some("L1".into()),
                weight: Some(1.0),
                is_partition: Some(false),
                belongs_to: None,
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            },
        ],
        (0..12)
            .map(|i| GraphDataEdgeInput {
                external_id: format!("B-e{}", i),
                source: if i % 2 == 0 {
                    "B-n1".into()
                } else {
                    "B-n2".into()
                },
                target: "B-n3".into(),
                label: Some(format!("B edge {}", i)),
                layer: Some("L1".into()),
                weight: Some(1.0),
                comment: None,
                source_dataset_id: None,
                attributes: None,
                created_at: None,
            })
            .collect(),
    )
    .await;

    let nodes = vec![plan_dag_nodes::Model {
        id: "merge-node".to_string(),
        plan_id: 1,
        node_type: "GraphNode".to_string(),
        position_x: 0.0,
        position_y: 0.0,
        source_position: None,
        target_position: None,
        metadata_json: json!({"label": "Merged Graph"}).to_string(),
        config_json: json!({"graphDataIds": [dataset_a.id, dataset_b.id]}).to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];
    let edges: Vec<(String, String)> = vec![];

    let executor = DagExecutor::new(db.clone());
    executor
        .execute_dag(project_id, 1, &nodes, &edges)
        .await
        .expect("DAG should succeed");

    let merged = service
        .get_by_dag_node("merge-node")
        .await
        .expect("query merged graph")
        .expect("merged graph missing");
    assert_eq!(merged.edge_count, 15, "expected edges from both datasets");

    let merged_nodes = service.load_nodes(merged.id).await.unwrap();
    let merged_edges = service.load_edges(merged.id).await.unwrap();
    assert_eq!(
        merged_nodes.len(),
        5,
        "should merge all nodes from both datasets"
    );
    assert_eq!(merged_edges.len(), 15, "edges should be preserved");

    let root = merged_nodes
        .iter()
        .find(|n| n.external_id == "A-root")
        .expect("root node missing");
    assert!(root.is_partition, "partition flag should be preserved");

    let child = merged_nodes
        .iter()
        .find(|n| n.external_id == "A-child")
        .expect("child node missing");
    assert_eq!(
        child.belongs_to.as_deref(),
        Some("A-root"),
        "belongs_to should survive merges"
    );

    assert!(
        merged_edges.iter().any(|e| e.external_id == "B-e11"),
        "edge ids should remain stable through merge"
    );
}

#[tokio::test]
async fn test_dag_executor_change_detection() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Create a source dataset
    let dataset = create_test_dataset(&service, project_id, "Source").await;

    // Create DAG node
    let nodes = vec![plan_dag_nodes::Model {
        id: "graph-node".to_string(),
        plan_id: 1,
        node_type: "GraphNode".to_string(),
        position_x: 0.0,
        position_y: 0.0,
        source_position: None,
        target_position: None,
        metadata_json: json!({"label": "Graph"}).to_string(),
        config_json: json!({"graphDataIds": [dataset.id]}).to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }];

    let edges: Vec<(String, String)> = vec![];

    // Execute DAG first time
    let executor = DagExecutor::new(db.clone());
    executor
        .execute_dag(project_id, 1, &nodes, &edges)
        .await
        .unwrap();

    let graph1 = service
        .get_by_dag_node("graph-node")
        .await
        .unwrap()
        .unwrap();
    let hash1 = graph1.source_hash.clone();
    let updated1 = graph1.updated_at;

    // Execute DAG again with same inputs
    // Give it a tiny delay to ensure updated_at would change if rebuild happened
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    executor
        .execute_dag(project_id, 1, &nodes, &edges)
        .await
        .unwrap();

    let graph2 = service
        .get_by_dag_node("graph-node")
        .await
        .unwrap()
        .unwrap();
    let hash2 = graph2.source_hash.clone();
    let updated2 = graph2.updated_at;

    // Should reuse existing graph (same ID, hash unchanged, no rebuild)
    assert_eq!(graph1.id, graph2.id, "Should reuse same graph_data record");
    assert_eq!(hash1, hash2, "Source hash should be unchanged");
    assert_eq!(
        updated1.timestamp_millis(),
        updated2.timestamp_millis(),
        "Graph should not be rebuilt if inputs unchanged"
    );
}

#[tokio::test]
async fn test_dag_executor_affected_nodes() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Create datasets
    let dataset1 = create_test_dataset(&service, project_id, "DS1").await;
    let dataset2 = create_test_dataset(&service, project_id, "DS2").await;

    // Create DAG: Graph1 (merges DS1 + DS2) -> Graph2 (chains from Graph1)
    let nodes = vec![
        plan_dag_nodes::Model {
            id: "graph1-node".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 0.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Graph 1"}).to_string(),
            config_json: json!({"graphDataIds": [dataset1.id, dataset2.id]}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "graph2-node".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 100.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Graph 2"}).to_string(),
            config_json: json!({"graphDataIds": []}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let edges = vec![("graph1-node".to_string(), "graph2-node".to_string())];

    // Build graph1 first
    let executor = DagExecutor::new(db.clone());
    executor
        .execute_dag(project_id, 1, &nodes[0..1], &[])
        .await
        .unwrap();

    let graph1 = service
        .get_by_dag_node("graph1-node")
        .await
        .unwrap()
        .unwrap();

    // Update graph2 to depend on graph1
    let mut updated_nodes = nodes.clone();
    updated_nodes[1].config_json = json!({"graphDataIds": [graph1.id]}).to_string();

    // Execute full DAG
    executor
        .execute_dag(project_id, 1, &updated_nodes, &edges)
        .await
        .unwrap();

    // Both graphs should exist
    assert!(service
        .get_by_dag_node("graph1-node")
        .await
        .unwrap()
        .is_some());
    assert!(service
        .get_by_dag_node("graph2-node")
        .await
        .unwrap()
        .is_some());

    // Now execute only affected nodes downstream of graph1
    // This should rebuild graph2 (graph1 itself won't rebuild unless inputs changed)
    let result = executor
        .execute_affected_nodes(project_id, 1, "graph1-node", &updated_nodes, &edges)
        .await;

    assert!(result.is_ok(), "Affected nodes execution should succeed");

    // Verify graphs still exist and were processed
    let final_graph1 = service
        .get_by_dag_node("graph1-node")
        .await
        .unwrap()
        .unwrap();
    let final_graph2 = service
        .get_by_dag_node("graph2-node")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(final_graph1.dag_node_id, Some("graph1-node".to_string()));
    assert_eq!(final_graph2.dag_node_id, Some("graph2-node".to_string()));
}
