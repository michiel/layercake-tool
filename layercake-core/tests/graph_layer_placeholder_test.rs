use anyhow::Result;
use chrono::Utc;
use layercake::database::entities::{
    graph_edges, graph_nodes, graphs, plan_dag_nodes, plans, projects,
};
use layercake::services::graph_service::GraphService;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};

#[tokio::test]
async fn build_graph_includes_placeholder_layers() -> Result<()> {
    let db = setup_in_memory_db().await?;

    let project = insert_project(&db, "Placeholder Project").await?;
    let graph = insert_graph(&db, project.id, "Placeholder Graph").await?;
    insert_node(
        &db,
        graph.id,
        "orphan_node",
        Some("node missing layer"),
        Some("ghost_layer"),
    )
    .await?;
    insert_edge(
        &db,
        graph.id,
        "ghost_edge",
        "orphan_node",
        "orphan_node",
        Some("ghost_layer"),
    )
    .await?;

    let service = GraphService::new(db.clone());
    let built = service.build_graph_from_dag_graph(graph.id).await?;

    assert!(
        built.layers.iter().any(|layer| layer.id == "ghost_layer"),
        "graph layers should include placeholder for ghost_layer"
    );
    let placeholder = built
        .layers
        .iter()
        .find(|layer| layer.id == "ghost_layer")
        .expect("placeholder layer missing");
    assert_eq!(placeholder.label, "ghost_layer");
    assert_eq!(placeholder.background_color.to_lowercase(), "f7f7f8");
    assert_eq!(placeholder.text_color.to_lowercase(), "0f172a");
    assert_eq!(placeholder.border_color.to_lowercase(), "1f2933");

    Ok(())
}

async fn setup_in_memory_db() -> Result<DatabaseConnection> {
    let db = Database::connect("sqlite::memory:").await?;
    use sea_orm_migration::MigratorTrait;
    layercake::database::migrations::Migrator::up(&db, None).await?;
    Ok(db)
}

async fn insert_project(db: &DatabaseConnection, name: &str) -> Result<projects::Model> {
    let mut model = projects::ActiveModel::new();
    model.name = Set(name.to_string());
    model.description = Set(Some("placeholder project".to_string()));
    Ok(model.insert(db).await?)
}

async fn insert_graph(
    db: &DatabaseConnection,
    project_id: i32,
    name: &str,
) -> Result<graphs::Model> {
    let plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set(format!("Plan for {}", name)),
        description: Set(Some("placeholder plan".to_string())),
        tags: Set("[]".to_string()),
        yaml_content: Set("{}".to_string()),
        dependencies: Set(None),
        status: Set("draft".to_string()),
        version: Set(1),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let node_id = format!("graphnode_{}", name.replace(' ', "_"));
    plan_dag_nodes::ActiveModel {
        id: Set(node_id.clone()),
        plan_id: Set(plan.id),
        node_type: Set("GraphArtefactNode".to_string()),
        position_x: Set(0.0),
        position_y: Set(0.0),
        source_position: Set(None),
        target_position: Set(None),
        metadata_json: Set("{}".to_string()),
        config_json: Set("{}".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..plan_dag_nodes::ActiveModel::new()
    }
    .insert(db)
    .await?;

    let mut model = graphs::ActiveModel::new();
    model.project_id = Set(project_id);
    model.name = Set(name.to_string());
    model.node_id = Set(node_id);
    Ok(model.insert(db).await?)
}

async fn insert_node(
    db: &DatabaseConnection,
    graph_id: i32,
    node_id: &str,
    label: Option<&str>,
    layer: Option<&str>,
) -> Result<graph_nodes::Model> {
    let mut model = graph_nodes::ActiveModel {
        ..Default::default()
    };
    model.graph_id = Set(graph_id);
    model.id = Set(node_id.to_string());
    model.label = Set(label.map(|v| v.to_string()));
    model.layer = Set(layer.map(|v| v.to_string()));
    model.is_partition = Set(false);
    model.created_at = Set(Utc::now());
    Ok(model.insert(db).await?)
}

async fn insert_edge(
    db: &DatabaseConnection,
    graph_id: i32,
    edge_id: &str,
    source: &str,
    target: &str,
    layer: Option<&str>,
) -> Result<graph_edges::Model> {
    let mut model = graph_edges::ActiveModel {
        ..Default::default()
    };
    model.graph_id = Set(graph_id);
    model.id = Set(edge_id.to_string());
    model.source = Set(source.to_string());
    model.target = Set(target.to_string());
    model.layer = Set(layer.map(|v| v.to_string()));
    model.created_at = Set(Utc::now());
    Ok(model.insert(db).await?)
}
