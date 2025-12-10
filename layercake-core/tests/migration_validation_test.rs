use chrono::Utc;
use layercake as layercake_core;
use layercake_core::database::entities::{
    data_sets, dataset_graph_edges, dataset_graph_nodes, graph_edges, graph_nodes, graphs,
    project_layers, projects,
};
use layercake_core::database::migrations::{
    m20251210_000004_migrate_existing_graph_data, Migrator,
};
use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue, Database, QueryOrder, Set};
use sea_orm_migration::{MigrationTrait, MigratorTrait, SchemaManager};

async fn setup_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::up(&db, None).await.unwrap();
    db
}

async fn seed_old_graphs(db: &DatabaseConnection) {
    // Minimal project
    let project = projects::ActiveModel {
        id: Set(1),
        name: Set("Test Project".into()),
        description: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    project.insert(db).await.unwrap();

    // Dataset with nodes/edges
    let dataset = data_sets::ActiveModel {
        id: Set(1),
        project_id: Set(1),
        name: Set("Dataset".into()),
        file_format: Set("csv".into()),
        blob: Set(None),
        graph_json: Set("{}".into()),
        origin: Set(None),
        filename: Set(None),
        status: Set("active".into()),
        error_message: Set(None),
        annotations: Set("[]".into()),
        metadata: Set(serde_json::json!({})),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        indexed: Set(false),
        rag_metadata: Set(None),
        file_size: Set(None),
        processed_at: Set(None),
    };
    dataset.insert(db).await.unwrap();

    let node = dataset_graph_nodes::ActiveModel {
        id: Set("n1".into()),
        dataset_id: Set(1),
        label: Set(Some("A".into())),
        layer: Set(Some("L".into())),
        weight: Set(Some(1)),
        is_partition: Set(false),
        belongs_to: Set(None),
        comment: Set(None),
        attributes: Set(None),
        created_at: Set(Utc::now()),
    };
    node.insert(db).await.unwrap();

    let edge = dataset_graph_edges::ActiveModel {
        id: Set("e1".into()),
        dataset_id: Set(1),
        source: Set("n1".into()),
        target: Set("n1".into()),
        label: Set(Some("E".into())),
        layer: Set(Some("L".into())),
        weight: Set(Some(2)),
        comment: Set(None),
        attributes: Set(None),
        created_at: Set(Utc::now()),
    };
    edge.insert(db).await.unwrap();

    // Computed graph with nodes/edges and a layer to offset
    let graph = graphs::ActiveModel {
        id: Set(2),
        project_id: Set(1),
        node_id: Set("node-graph".into()),
        name: Set("Graph".into()),
        execution_state: Set("Completed".into()),
        computed_date: Set(Some(Utc::now())),
        source_hash: Set(Some("hash".into())),
        node_count: Set(1),
        edge_count: Set(1),
        error_message: Set(None),
        metadata: Set(None),
        annotations: Set(Some("[]".into())),
        last_edit_sequence: Set(0),
        has_pending_edits: Set(false),
        last_replay_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    graph.insert(db).await.unwrap();

    let g_node = graph_nodes::ActiveModel {
        id: Set("gn1".into()),
        graph_id: Set(2),
        label: Set(Some("GN".into())),
        layer: Set(Some("L".into())),
        is_partition: Set(false),
        belongs_to: Set(None),
        weight: Set(Some(1.5)),
        attrs: Set(None),
        dataset_id: Set(None),
        comment: Set(None),
        created_at: Set(Utc::now()),
    };
    g_node.insert(db).await.unwrap();

    let g_edge = graph_edges::ActiveModel {
        id: Set("ge1".into()),
        graph_id: Set(2),
        source: Set("gn1".into()),
        target: Set("gn1".into()),
        label: Set(Some("GE".into())),
        layer: Set(Some("L".into())),
        weight: Set(Some(2.5)),
        attrs: Set(None),
        dataset_id: Set(None),
        comment: Set(None),
        created_at: Set(Utc::now()),
    };
    g_edge.insert(db).await.unwrap();

    // Palette entry to satisfy layer validation
    let layer = project_layers::ActiveModel {
        id: ActiveValue::NotSet,
        project_id: Set(1),
        layer_id: Set("L".into()),
        name: Set("Layer".into()),
        background_color: Set("#fff".into()),
        text_color: Set("#000".into()),
        border_color: Set("#000".into()),
        alias: Set(None),
        source_dataset_id: Set(Some(1)),
        enabled: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    layer.insert(db).await.unwrap();
}

#[tokio::test]
async fn test_graph_data_migration_fk_validation() {
    let db = setup_db().await;
    seed_old_graphs(&db).await;

    // Clear any previous copy to force rerun of migration logic
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "DELETE FROM graph_data_edges; DELETE FROM graph_data_nodes; DELETE FROM graph_data; DELETE FROM graph_data_migration_validation;",
    ))
    .await
    .unwrap();

    // Rerun migration copy step
    let manager = SchemaManager::new(&db);
    m20251210_000004_migrate_existing_graph_data::Migration
        .up(&manager)
        .await
        .unwrap();

    // Validate FK checks recorded
    let rows = sea_orm::QuerySelect::query(&graph_data_migration_validation::Entity)
        .order_by_asc(graph_data_migration_validation::Column::CheckName)
        .all(&db)
        .await
        .unwrap();

    let missing_edges = rows
        .iter()
        .find(|r| r.check_name == "edges_missing_graph_data")
        .unwrap();
    assert_eq!(missing_edges.new_count.unwrap_or(0), 0);

    let missing_nodes = rows
        .iter()
        .find(|r| r.check_name == "nodes_missing_graph_data")
        .unwrap();
    assert_eq!(missing_nodes.new_count.unwrap_or(0), 0);

    let missing_edits = rows
        .iter()
        .find(|r| r.check_name == "graph_edits_missing_graph_data");
    if let Some(edit_row) = missing_edits {
        assert_eq!(edit_row.new_count.unwrap_or(0), 0);
    }

    // Ensure counts migrated
    let datasets = sea_orm::QuerySelect::query(&layercake_core::database::entities::graph_data::Entity)
        .filter(layercake_core::database::entities::graph_data::Column::SourceType.eq("dataset"))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(datasets.len(), 1);

    let computed = sea_orm::QuerySelect::query(&layercake_core::database::entities::graph_data::Entity)
        .filter(layercake_core::database::entities::graph_data::Column::SourceType.eq("computed"))
        .all(&db)
        .await
        .unwrap();
    assert_eq!(computed.len(), 1);
}

mod graph_data_migration_validation {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "graph_data_migration_validation")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub check_name: String,
        pub old_count: Option<i64>,
        pub new_count: Option<i64>,
        pub delta: Option<i64>,
        pub created_at: ChronoDateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
