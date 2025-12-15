use chrono::Utc;
use layercake_projections::entities::{
    graph_data, graph_data_edges, graph_data_nodes, projections,
};
use layercake_projections::service::{
    ProjectionCreateInput, ProjectionService, ProjectionUpdateInput,
};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, Database, Set, Statement};
use serde_json::json;

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // Minimal schema needed for projection service tests
    let stmts = [
        r#"CREATE TABLE graph_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            source_type TEXT NOT NULL,
            dag_node_id TEXT,
            file_format TEXT,
            origin TEXT,
            filename TEXT,
            blob BLOB,
            file_size INTEGER,
            processed_at DATETIME,
            source_hash TEXT,
            computed_date DATETIME,
            last_edit_sequence INTEGER NOT NULL DEFAULT 0,
            has_pending_edits BOOLEAN NOT NULL DEFAULT 0,
            last_replay_at DATETIME,
            node_count INTEGER NOT NULL DEFAULT 0,
            edge_count INTEGER NOT NULL DEFAULT 0,
            error_message TEXT,
            metadata JSON,
            annotations JSON,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        );"#,
        r#"CREATE TABLE graph_data_nodes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            graph_data_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            label TEXT,
            layer TEXT,
            weight REAL,
            is_partition BOOLEAN NOT NULL DEFAULT 0,
            belongs_to TEXT,
            comment TEXT,
            source_dataset_id INTEGER,
            attributes JSON,
            created_at DATETIME NOT NULL,
            FOREIGN KEY(graph_data_id) REFERENCES graph_data(id)
        );"#,
        r#"CREATE TABLE graph_data_edges (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            graph_data_id INTEGER NOT NULL,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            target TEXT NOT NULL,
            label TEXT,
            layer TEXT,
            weight REAL,
            comment TEXT,
            source_dataset_id INTEGER,
            attributes JSON,
            created_at DATETIME NOT NULL,
            FOREIGN KEY(graph_data_id) REFERENCES graph_data(id)
        );"#,
        r#"CREATE TABLE projections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            graph_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            projection_type TEXT NOT NULL,
            settings_json JSON,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY(graph_id) REFERENCES graph_data(id)
        );"#,
    ];
    for sql in stmts {
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            sql.to_string(),
        ))
        .await
        .unwrap();
    }
    db
}

async fn seed_graph_data(db: &DatabaseConnection) -> i32 {
    // Create a graph_data entry
    let graph = graph_data::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        project_id: Set(1),
        name: Set("Test Graph".into()),
        source_type: Set("test".into()),
        dag_node_id: Set(None),
        file_format: Set(None),
        origin: Set(None),
        filename: Set(None),
        blob: Set(None),
        file_size: Set(None),
        processed_at: Set(Some(Utc::now())),
        source_hash: Set(None),
        computed_date: Set(None),
        last_edit_sequence: Set(0),
        has_pending_edits: Set(false),
        last_replay_at: Set(None),
        node_count: Set(0),
        edge_count: Set(0),
        error_message: Set(None),
        metadata: Set(None),
        annotations: Set(None),
        status: Set("ready".into()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    let result = graph.insert(db).await.unwrap();
    result.id
}

async fn seed_graph_with_data(db: &DatabaseConnection) -> i32 {
    let graph_id = seed_graph_data(db).await;

    // Add nodes
    let node1 = graph_data_nodes::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        graph_data_id: Set(graph_id),
        external_id: Set("n1".into()),
        label: Set(Some("Node 1".into())),
        layer: Set(Some("L1".into())),
        weight: Set(None),
        is_partition: Set(false),
        belongs_to: Set(None),
        comment: Set(None),
        source_dataset_id: Set(None),
        attributes: Set(None),
        created_at: Set(Utc::now()),
    };
    node1.insert(db).await.unwrap();

    let node2 = graph_data_nodes::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        graph_data_id: Set(graph_id),
        external_id: Set("n2".into()),
        label: Set(Some("Node 2".into())),
        layer: Set(Some("L1".into())),
        weight: Set(None),
        is_partition: Set(false),
        belongs_to: Set(None),
        comment: Set(None),
        source_dataset_id: Set(None),
        attributes: Set(None),
        created_at: Set(Utc::now()),
    };
    node2.insert(db).await.unwrap();

    // Add edge
    let edge = graph_data_edges::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        graph_data_id: Set(graph_id),
        external_id: Set("e1".into()),
        source: Set("n1".into()),
        target: Set("n2".into()),
        label: Set(None),
        layer: Set(Some("L1".into())),
        weight: Set(None),
        comment: Set(None),
        source_dataset_id: Set(None),
        attributes: Set(None),
        created_at: Set(Utc::now()),
    };
    edge.insert(db).await.unwrap();

    graph_id
}

#[tokio::test]
async fn test_create_projection() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    let input = ProjectionCreateInput {
        project_id: 1,
        graph_id,
        name: "Test Projection".into(),
        projection_type: "force3d".into(),
        settings_json: Some(json!({"foo": "bar"})),
    };

    let projection = service.create(input).await.unwrap();

    assert_eq!(projection.name, "Test Projection");
    assert_eq!(projection.projection_type, "force3d");
    assert_eq!(projection.graph_id, graph_id);
    assert_eq!(projection.project_id, 1);
    assert!(projection.settings_json.is_some());
}

#[tokio::test]
async fn test_get_projection() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    let input = ProjectionCreateInput {
        project_id: 1,
        graph_id,
        name: "Test Projection".into(),
        projection_type: "force3d".into(),
        settings_json: None,
    };

    let created = service.create(input).await.unwrap();
    let fetched = service.get(created.id).await.unwrap();

    assert!(fetched.is_some());
    let projection = fetched.unwrap();
    assert_eq!(projection.id, created.id);
    assert_eq!(projection.name, "Test Projection");
}

#[tokio::test]
async fn test_list_by_project() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    // Create two projections for project 1
    service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "Projection 1".into(),
            projection_type: "force3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "Projection 2".into(),
            projection_type: "layer3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    let projections = service.list_by_project(1).await.unwrap();

    assert_eq!(projections.len(), 2);
    assert!(projections.iter().any(|p| p.name == "Projection 1"));
    assert!(projections.iter().any(|p| p.name == "Projection 2"));
}

#[tokio::test]
async fn test_update_projection() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    let created = service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "Original Name".into(),
            projection_type: "force3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    let updated = service
        .update(
            created.id,
            ProjectionUpdateInput {
                name: Some("Updated Name".into()),
                projection_type: Some("layer3d".into()),
                settings_json: Some(Some(json!({"updated": true}))),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.projection_type, "layer3d");
    assert!(updated.settings_json.is_some());
}

#[tokio::test]
async fn test_delete_projection() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    let created = service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "To Delete".into(),
            projection_type: "force3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    let deleted = service.delete(created.id).await.unwrap();
    assert_eq!(deleted, 1);

    let fetched = service.get(created.id).await.unwrap();
    assert!(fetched.is_none());
}

#[tokio::test]
async fn test_load_graph() {
    let db = setup_db().await;
    let graph_id = seed_graph_with_data(&db).await;
    let service = ProjectionService::new(db);

    let created = service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "Graph Loader".into(),
            projection_type: "force3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    let graph = service.load_graph(created.id).await.unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);

    assert!(graph.nodes.iter().any(|n| n.id == "n1"));
    assert!(graph.nodes.iter().any(|n| n.id == "n2"));
    assert_eq!(graph.edges[0].source, "n1");
    assert_eq!(graph.edges[0].target, "n2");
}

#[tokio::test]
async fn test_save_and_get_state() {
    let db = setup_db().await;
    let graph_id = seed_graph_data(&db).await;
    let service = ProjectionService::new(db);

    let created = service
        .create(ProjectionCreateInput {
            project_id: 1,
            graph_id,
            name: "State Test".into(),
            projection_type: "force3d".into(),
            settings_json: None,
        })
        .await
        .unwrap();

    let state = json!({"camera": {"x": 0, "y": 0, "z": 100}});
    service.save_state(created.id, state.clone()).await.unwrap();

    let fetched_state = service.get_state(created.id).await;
    assert!(fetched_state.is_some());
    assert_eq!(fetched_state.unwrap(), state);
}
