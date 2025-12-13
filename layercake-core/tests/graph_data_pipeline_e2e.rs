use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{graph_data, plan_dag_nodes, project_layers, projects, projections};
use layercake_core::database::migrations::Migrator;
use layercake_core::pipeline::DagExecutor;
use layercake_core::services::{GraphDataEdgeInput, GraphDataNodeInput, GraphDataService};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, Database, Set};
use sea_orm_migration::MigratorTrait;
use serde_json::json;

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn seed_project_and_palette(db: &DatabaseConnection) -> i32 {
    // Create project
    let project = projects::ActiveModel {
        id: Set(1),
        name: Set("GraphData Pipeline Project".into()),
        description: Set(None),
        tags: Set("[]".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    project.insert(db).await.unwrap();

    // Palette layer
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

async fn create_graph_data_dataset(
    service: &GraphDataService,
    project_id: i32,
    name: &str,
    dag_node_id: &str,
) -> graph_data::Model {
    let dataset = service
        .create(layercake_core::services::GraphDataCreate {
            project_id,
            name: name.to_string(),
            source_type: "dataset".to_string(),
            dag_node_id: Some(dag_node_id.to_string()),
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

    service
        .replace_nodes(
            dataset.id,
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
                    weight: Some(1.0),
                    is_partition: Some(false),
                    belongs_to: None,
                    comment: None,
                    source_dataset_id: Some(dataset.id),
                    attributes: None,
                    created_at: None,
                },
            ],
        )
        .await
        .unwrap();

    service
        .replace_edges(
            dataset.id,
            vec![GraphDataEdgeInput {
                external_id: format!("{}-e1", name),
                source: format!("{}-n1", name),
                target: format!("{}-n2", name),
                label: Some(format!("{} Edge", name)),
                layer: Some("L1".to_string()),
                weight: Some(1.0),
                comment: None,
                source_dataset_id: Some(dataset.id),
                attributes: None,
                created_at: None,
            }],
        )
        .await
        .unwrap();

    dataset
}

#[tokio::test]
async fn graph_data_pipeline_end_to_end() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Seed two dataset graph_data records
    let ds1 = create_graph_data_dataset(&service, project_id, "DS1", "ds1-node").await;
    let ds2 = create_graph_data_dataset(&service, project_id, "DS2", "ds2-node").await;

    // Plan DAG nodes covering merge -> transform -> filter -> projection
    let nodes = vec![
        plan_dag_nodes::Model {
            id: "graph-a".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 0.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Graph A"}).to_string(),
            config_json: json!({"graphDataIds": [ds1.id]}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "graph-b".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 0.0,
            position_y: 50.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Graph B"}).to_string(),
            config_json: json!({"graphDataIds": [ds2.id]}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "merge-1".to_string(),
            plan_id: 1,
            node_type: "MergeNode".to_string(),
            position_x: 150.0,
            position_y: 25.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Merge"}).to_string(),
            config_json: json!({"mergeStrategy": "Union", "conflictResolution": "PreferLast"}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "transform-1".to_string(),
            plan_id: 1,
            node_type: "TransformNode".to_string(),
            position_x: 300.0,
            position_y: 25.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Transform"}).to_string(),
            config_json: json!({"transforms": [{"kind": "AggregateEdges", "params": {}}]}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "filter-1".to_string(),
            plan_id: 1,
            node_type: "FilterNode".to_string(),
            position_x: 450.0,
            position_y: 25.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Filter"}).to_string(),
            config_json: json!({
                "query": {
                    "targets": ["nodes"],
                    "mode": "include",
                    "linkPruningMode": "retainEdges",
                    "ruleGroup": { "combinator": "and", "rules": [] },
                    "fieldMetadataVersion": "v1"
                }
            })
            .to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "projection-1".to_string(),
            plan_id: 1,
            node_type: "ProjectionNode".to_string(),
            position_x: 600.0,
            position_y: 25.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Projection"}).to_string(),
            config_json: json!({"projectionType": "force3d"}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
    ];

    let edges = vec![
        ("graph-a".to_string(), "merge-1".to_string()),
        ("graph-b".to_string(), "merge-1".to_string()),
        ("merge-1".to_string(), "transform-1".to_string()),
        ("transform-1".to_string(), "filter-1".to_string()),
        ("filter-1".to_string(), "projection-1".to_string()),
    ];

    let executor = DagExecutor::new(db.clone());
    executor
        .execute_dag(project_id, 1, &nodes, &edges)
        .await
        .expect("pipeline execution should succeed");

    // Verify merge, transform, and filter nodes produced graph_data entries
    for node_id in ["merge-1", "transform-1", "filter-1"] {
        let gd = service
            .get_by_dag_node(node_id)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| panic!("graph_data for {} should exist", node_id));
        assert_eq!(gd.source_type, "computed");
        assert_eq!(gd.node_count, 4, "node_count should propagate");
        assert_eq!(gd.edge_count, 2, "edge_count should propagate");
    }

    // Projection should be linked to filter output
    let projection = projections::Entity::find()
        .filter(projections::Column::ProjectId.eq(project_id))
        .one(&db)
        .await
        .expect("query projections")
        .expect("projection should exist");
    assert_eq!(projection.graph_id, service.get_by_dag_node("filter-1").await.unwrap().unwrap().id);
}

