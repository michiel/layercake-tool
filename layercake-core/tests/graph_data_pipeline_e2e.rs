use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{
    graph_data, plan_dag_nodes, project_layers, projections, projects,
};
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
        import_export_path: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    project.insert(db).await.unwrap();

    // Plan (id 1) so plan_dag_nodes referencing plan_id = 1 satisfy their FK.
    use layercake_core::database::entities::plans;
    plans::ActiveModel {
        id: Set(1),
        project_id: Set(1),
        name: Set("Pipeline Plan".into()),
        description: Set(None),
        tags: Set("[]".into()),
        yaml_content: Set("{}".into()),
        dependencies: Set(None),
        status: Set("draft".to_string()),
        version: Set(1),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .unwrap();

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

/// Create a real `data_sets` row (with graph_json) that a DataSetNode can
/// reference — this is how source data actually enters the DAG. Returns its id.
async fn create_data_set(db: &DatabaseConnection, project_id: i32, name: &str) -> i32 {
    use layercake_core::database::entities::data_sets;
    let graph_json = json!({
        "nodes": [
            {"id": format!("{}-n1", name), "label": format!("{} Node 1", name), "layer": "L1", "weight": 1},
            {"id": format!("{}-n2", name), "label": format!("{} Node 2", name), "layer": "L1", "weight": 1},
        ],
        "edges": [
            {"id": format!("{}-e1", name), "source": format!("{}-n1", name), "target": format!("{}-n2", name), "label": "", "layer": "L1", "weight": 1},
        ],
        "layers": [],
    })
    .to_string();

    let ds = data_sets::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        project_id: Set(project_id),
        name: Set(name.to_string()),
        description: Set(None),
        file_format: Set("json".to_string()),
        data_type: Set("graph".to_string()),
        origin: Set("manual_edit".to_string()),
        filename: Set(format!("{}.json", name)),
        blob: Set(Vec::new()),
        graph_json: Set(graph_json),
        status: Set("active".to_string()),
        error_message: Set(None),
        file_size: Set(0),
        processed_at: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        annotations: Set(None),
    }
    .insert(db)
    .await
    .unwrap();
    ds.id
}

#[allow(dead_code)]
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

// Full pipeline: DataSetNode -> GraphNode -> Merge -> Transform -> Filter ->
// Projection. Source data enters the DAG via DataSetNodes referencing data_sets
// rows (GraphNodes get their input from DAG edges, not config — see the
// GraphNodeConfig "Removed: graphId" note in the frontend types).
#[tokio::test]
async fn graph_data_pipeline_end_to_end() {
    let db = setup_db().await;
    let project_id = seed_project_and_palette(&db).await;
    let service = GraphDataService::new(db.clone());

    // Seed two real datasets that DataSetNodes will reference.
    let ds1 = create_data_set(&db, project_id, "DS1").await;
    let ds2 = create_data_set(&db, project_id, "DS2").await;

    // Plan DAG nodes covering dataset -> graph -> merge -> transform -> filter -> projection
    let nodes = vec![
        plan_dag_nodes::Model {
            id: "dataset-a".to_string(),
            plan_id: 1,
            node_type: "DataSetNode".to_string(),
            position_x: -150.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Dataset A"}).to_string(),
            config_json: json!({"dataSetId": ds1}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "dataset-b".to_string(),
            plan_id: 1,
            node_type: "DataSetNode".to_string(),
            position_x: -150.0,
            position_y: 50.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Dataset B"}).to_string(),
            config_json: json!({"dataSetId": ds2}).to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        },
        plan_dag_nodes::Model {
            id: "graph-a".to_string(),
            plan_id: 1,
            node_type: "GraphNode".to_string(),
            position_x: 0.0,
            position_y: 0.0,
            source_position: None,
            target_position: None,
            metadata_json: json!({"label": "Graph A"}).to_string(),
            config_json: json!({}).to_string(),
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
            config_json: json!({}).to_string(),
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
            config_json: json!({"mergeStrategy": "Union", "conflictResolution": "PreferLast"})
                .to_string(),
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
            config_json: json!({"transforms": [{"kind": "AggregateEdges", "params": {}}]})
                .to_string(),
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
        ("dataset-a".to_string(), "graph-a".to_string()),
        ("dataset-b".to_string(), "graph-b".to_string()),
        ("graph-a".to_string(), "merge-1".to_string()),
        ("graph-b".to_string(), "merge-1".to_string()),
        ("merge-1".to_string(), "transform-1".to_string()),
        ("transform-1".to_string(), "filter-1".to_string()),
        ("filter-1".to_string(), "projection-1".to_string()),
    ];

    // Persist the plan DAG nodes so nodes that write back to their own config
    // (e.g. ProjectionNode storing the created projectionId) have a real row to
    // update. execute_dag takes the node models by value but expects them to
    // exist in the database.
    for node in &nodes {
        let active: plan_dag_nodes::ActiveModel = node.clone().into();
        active.insert(&db).await.unwrap();
    }

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
    assert_eq!(
        projection.graph_id,
        service
            .get_by_dag_node("filter-1")
            .await
            .unwrap()
            .unwrap()
            .id
    );
}
