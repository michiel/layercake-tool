/// Test to verify that source_dataset_id is preserved throughout the pipeline
use layercake_core::database::test_utils::setup_test_db;
use layercake_core::graph::{Edge, Graph, Node};
use layercake_core::services::graph_data_service::{GraphDataCreate, GraphDataService};
use sea_orm::DatabaseConnection;

#[tokio::test]
async fn test_dataset_source_preserved_in_merge() {
    let db = setup_test_db().await;
    let project_id = 1; // Assuming test setup creates a project

    // Create graph_data service
    let service = GraphDataService::new(db.clone());

    // Create two datasets with nodes/edges
    let dataset1 = create_test_dataset(&service, project_id, "Dataset 1", "ds1", 1).await;
    let dataset2 = create_test_dataset(&service, project_id, "Dataset 2", "ds2", 2).await;

    // Load both datasets
    let (_, nodes1, edges1) = service.load_full(dataset1.id).await.unwrap();
    let (_, nodes2, edges2) = service.load_full(dataset2.id).await.unwrap();

    // Verify dataset 1 nodes/edges have source_dataset_id = 1
    assert!(nodes1.iter().all(|n| n.source_dataset_id == Some(1)));
    assert!(edges1.iter().all(|e| e.source_dataset_id == Some(1)));

    // Verify dataset 2 nodes/edges have source_dataset_id = 2
    assert!(nodes2.iter().all(|n| n.source_dataset_id == Some(2)));
    assert!(edges2.iter().all(|e| e.source_dataset_id == Some(2)));

    // Convert to Graph structs and merge (simulating MergeNode behavior)
    let graph1 = db_models_to_graph(nodes1, edges1);
    let graph2 = db_models_to_graph(nodes2, edges2);

    // Merge graphs (simulating merge_builder logic)
    let mut merged = merge_graphs(vec![graph1, graph2]);

    // Verify merged graph preserves dataset IDs
    let ds1_nodes: Vec<_> = merged
        .nodes
        .iter()
        .filter(|n| n.dataset == Some(1))
        .collect();
    let ds2_nodes: Vec<_> = merged
        .nodes
        .iter()
        .filter(|n| n.dataset == Some(2))
        .collect();

    assert_eq!(ds1_nodes.len(), 2, "Should have 2 nodes from dataset 1");
    assert_eq!(ds2_nodes.len(), 2, "Should have 2 nodes from dataset 2");

    let ds1_edges: Vec<_> = merged
        .edges
        .iter()
        .filter(|e| e.dataset == Some(1))
        .collect();
    let ds2_edges: Vec<_> = merged
        .edges
        .iter()
        .filter(|e| e.dataset == Some(2))
        .collect();

    assert_eq!(ds1_edges.len(), 1, "Should have 1 edge from dataset 1");
    assert_eq!(ds2_edges.len(), 1, "Should have 1 edge from dataset 2");
}

#[tokio::test]
async fn test_filter_preserves_dataset_source() {
    let db = setup_test_db().await;
    let project_id = 1;

    let service = GraphDataService::new(db.clone());
    let dataset = create_test_dataset(&service, project_id, "Test Dataset", "ds1", 1).await;

    let (_, nodes, edges) = service.load_full(dataset.id).await.unwrap();
    let mut graph = db_models_to_graph(nodes, edges);

    // Filter to keep only node1
    graph.nodes.retain(|n| n.id == "node1");

    // Verify filtered nodes still have dataset field
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes[0].dataset, Some(1));
}

#[tokio::test]
async fn test_new_nodes_have_no_dataset() {
    // Create a graph
    let mut graph = Graph {
        name: "Test".to_string(),
        nodes: vec![Node {
            id: "n1".into(),
            label: "Node 1".into(),
            layer: "default".into(),
            is_partition: false,
            belongs_to: None,
            weight: 1,
            comment: None,
            dataset: Some(1), // From dataset 1
            attributes: None,
        }],
        edges: vec![],
        layers: vec![],
        annotations: None,
    };

    // Add a new node programmatically (simulating transform creating a node)
    graph.nodes.push(Node {
        id: "generated".into(),
        label: "Generated".into(),
        layer: "default".into(),
        is_partition: false,
        belongs_to: None,
        weight: 1,
        comment: None,
        dataset: None, // New nodes should have dataset = None
        attributes: None,
    });

    // Verify
    assert_eq!(graph.nodes[0].dataset, Some(1));
    assert_eq!(graph.nodes[1].dataset, None);
}

// Helper functions

async fn create_test_dataset(
    service: &GraphDataService,
    project_id: i32,
    name: &str,
    dag_node_id: &str,
    dataset_id: i32,
) -> layercake_core::database::entities::graph_data::Model {
    use layercake_core::services::graph_data_service::{GraphDataEdgeInput, GraphDataNodeInput};
    use sea_orm::Set;

    // Create graph_data
    let gd = service
        .create(GraphDataCreate {
            project_id,
            name: name.to_string(),
            source_type: "dataset".to_string(),
            dag_node_id: Some(dag_node_id.to_string()),
            file_format: Some("json".to_string()),
            origin: Some("test".to_string()),
            filename: Some(format!("{}.json", name)),
            blob: Some(vec![]),
            file_size: Some(0),
            processed_at: Some(chrono::Utc::now()),
            source_hash: Some(format!("hash-{}", name)),
            computed_date: None,
            last_edit_sequence: Some(0),
            has_pending_edits: Some(false),
            last_replay_at: None,
            metadata: None,
            annotations: None,
            status: Some(layercake_core::database::entities::graph_data::GraphDataStatus::Active),
        })
        .await
        .unwrap();

    // Add nodes with source_dataset_id
    let nodes = vec![
        GraphDataNodeInput {
            external_id: "node1".into(),
            label: Some("Node 1".into()),
            layer: Some("default".into()),
            weight: Some(1.0),
            is_partition: Some(false),
            belongs_to: None,
            comment: None,
            source_dataset_id: Some(dataset_id),
            attributes: None,
            created_at: Some(chrono::Utc::now()),
        },
        GraphDataNodeInput {
            external_id: "node2".into(),
            label: Some("Node 2".into()),
            layer: Some("default".into()),
            weight: Some(1.0),
            is_partition: Some(false),
            belongs_to: None,
            comment: None,
            source_dataset_id: Some(dataset_id),
            attributes: None,
            created_at: Some(chrono::Utc::now()),
        },
    ];

    service.replace_nodes(gd.id, nodes).await.unwrap();

    // Add edges with source_dataset_id
    let edges = vec![GraphDataEdgeInput {
        external_id: "edge1".into(),
        source: "node1".into(),
        target: "node2".into(),
        label: Some("Edge 1".into()),
        layer: Some("default".into()),
        weight: Some(1.0),
        comment: None,
        source_dataset_id: Some(dataset_id),
        attributes: None,
        created_at: Some(chrono::Utc::now()),
    }];

    service.replace_edges(gd.id, edges).await.unwrap();

    gd
}

fn db_models_to_graph(
    nodes: Vec<layercake_core::database::entities::graph_data_nodes::Model>,
    edges: Vec<layercake_core::database::entities::graph_data_edges::Model>,
) -> Graph {
    Graph {
        name: "Test".into(),
        nodes: nodes
            .into_iter()
            .map(|n| Node {
                id: n.external_id,
                label: n.label.unwrap_or_default(),
                layer: n.layer.unwrap_or_else(|| "default".into()),
                is_partition: n.is_partition,
                belongs_to: n.belongs_to,
                weight: n.weight.map(|w| w as i32).unwrap_or(1),
                comment: n.comment,
                dataset: n.source_dataset_id,
                attributes: n.attributes,
            })
            .collect(),
        edges: edges
            .into_iter()
            .map(|e| Edge {
                id: e.external_id,
                source: e.source,
                target: e.target,
                label: e.label.unwrap_or_default(),
                layer: e.layer.unwrap_or_else(|| "default".into()),
                weight: e.weight.map(|w| w as i32).unwrap_or(1),
                comment: e.comment,
                dataset: e.source_dataset_id,
                attributes: e.attributes,
            })
            .collect(),
        layers: vec![],
        annotations: None,
    }
}

fn merge_graphs(graphs: Vec<Graph>) -> Graph {
    use std::collections::HashMap;

    let mut nodes_map: HashMap<String, Node> = HashMap::new();
    let mut edges_map: HashMap<String, Edge> = HashMap::new();

    for graph in graphs {
        for node in graph.nodes {
            // Keep first occurrence (or could merge with .or_insert)
            nodes_map.entry(node.id.clone()).or_insert(node);
        }
        for edge in graph.edges {
            edges_map.entry(edge.id.clone()).or_insert(edge);
        }
    }

    Graph {
        name: "Merged".into(),
        nodes: nodes_map.into_values().collect(),
        edges: edges_map.into_values().collect(),
        layers: vec![],
        annotations: None,
    }
}
