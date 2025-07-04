//! Database functionality tests
//! 
//! Tests for database migrations, entity operations, and data integrity

use anyhow::Result;
use layercake::database::entities::*;
use layercake::database::setup_database;
use sea_orm::{ActiveModelTrait, EntityTrait, Set, Database, DatabaseConnection, QueryFilter, ColumnTrait};
use tempfile::NamedTempFile;
use chrono::Utc;

/// Create a test database connection with migrations
async fn setup_test_db() -> Result<(DatabaseConnection, NamedTempFile)> {
    let temp_file = NamedTempFile::new()?;
    let db_url = format!("sqlite://{}?mode=rwc", temp_file.path().display());
    
    let db = Database::connect(&db_url).await?;
    setup_database(&db).await?;
    
    Ok((db, temp_file))
}

#[tokio::test]
async fn test_database_migrations() -> Result<()> {
    let (db, _temp_file) = setup_test_db().await?;
    
    // Verify all tables exist by attempting to query them
    let projects = projects::Entity::find().all(&db).await?;
    assert_eq!(projects.len(), 0);
    
    let plans = plans::Entity::find().all(&db).await?;
    assert_eq!(plans.len(), 0);
    
    let nodes = nodes::Entity::find().all(&db).await?;
    assert_eq!(nodes.len(), 0);
    
    let edges = edges::Entity::find().all(&db).await?;
    assert_eq!(edges.len(), 0);
    
    let layers = layers::Entity::find().all(&db).await?;
    assert_eq!(layers.len(), 0);
    
    Ok(())
}

#[tokio::test]
async fn test_project_crud_operations() -> Result<()> {
    let (db, _temp_file) = setup_test_db().await?;
    
    // Create project
    let now = Utc::now();
    let new_project = projects::ActiveModel {
        name: Set("Test Project".to_string()),
        description: Set(Some("A test project".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    
    let project = new_project.insert(&db).await?;
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.description, Some("A test project".to_string()));
    
    // Read project
    let found_project = projects::Entity::find_by_id(project.id)
        .one(&db)
        .await?
        .expect("Project should exist");
    
    assert_eq!(found_project.id, project.id);
    assert_eq!(found_project.name, "Test Project");
    
    // Update project
    let mut project_update: projects::ActiveModel = found_project.into();
    project_update.name = Set("Updated Test Project".to_string());
    
    let updated_project = project_update.update(&db).await?;
    assert_eq!(updated_project.name, "Updated Test Project");
    
    // Delete project
    projects::Entity::delete_by_id(updated_project.id)
        .exec(&db)
        .await?;
    
    let deleted_project = projects::Entity::find_by_id(updated_project.id)
        .one(&db)
        .await?;
    
    assert!(deleted_project.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_plan_json_migration() -> Result<()> {
    let (db, _temp_file) = setup_test_db().await?;
    
    // Create a project first
    let now = Utc::now();
    let project = projects::ActiveModel {
        name: Set("Test Project".to_string()),
        description: Set(Some("Test project for plans".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }.insert(&db).await?;
    
    // Create a plan with JSON content
    let plan_json = serde_json::json!({
        "meta": {
            "name": "Test Plan",
            "description": "A test transformation plan"
        },
        "import": {
            "profiles": [
                {
                    "filename": "nodes.csv",
                    "filetype": "Nodes"
                }
            ]
        },
        "export": {
            "profiles": [
                {
                    "filename": "output.json",
                    "exporter": "JSON"
                }
            ]
        }
    });
    
    let new_plan = plans::ActiveModel {
        project_id: Set(project.id),
        name: Set("Test Plan".to_string()),
        plan_content: Set(serde_json::to_string_pretty(&plan_json)?),
        plan_format: Set("json".to_string()),
        plan_schema_version: Set("1.0.0".to_string()),
        dependencies: Set(None),
        status: Set("pending".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    
    let plan = new_plan.insert(&db).await?;
    
    // Test plan JSON parsing
    let parsed_json = plan.get_plan_json()?;
    assert_eq!(parsed_json["meta"]["name"], "Test Plan");
    assert_eq!(parsed_json["import"]["profiles"][0]["filename"], "nodes.csv");
    
    // Test plan validation
    assert!(plan.validate_plan_schema().is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_graph_data_relationships() -> Result<()> {
    let (db, _temp_file) = setup_test_db().await?;
    
    // Create project
    let now = Utc::now();
    let project = projects::ActiveModel {
        name: Set("Graph Test Project".to_string()),
        description: Set(Some("Testing graph relationships".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }.insert(&db).await?;
    
    // Create layers
    let layer1 = layers::ActiveModel {
        project_id: Set(project.id),
        layer_id: Set("layer1".to_string()),
        name: Set("Layer 1".to_string()),
        color: Set(Some("#FF0000".to_string())),
        properties: Set(None),
        ..Default::default()
    }.insert(&db).await?;
    
    let layer2 = layers::ActiveModel {
        project_id: Set(project.id),
        layer_id: Set("layer2".to_string()),
        name: Set("Layer 2".to_string()),
        color: Set(Some("#00FF00".to_string())),
        properties: Set(None),
        ..Default::default()
    }.insert(&db).await?;
    
    // Create nodes
    let node1 = nodes::ActiveModel {
        project_id: Set(project.id),
        node_id: Set("node1".to_string()),
        label: Set("Node 1".to_string()),
        layer_id: Set(Some("layer1".to_string())),
        properties: Set(None),
        ..Default::default()
    }.insert(&db).await?;
    
    let node2 = nodes::ActiveModel {
        project_id: Set(project.id),
        node_id: Set("node2".to_string()),
        label: Set("Node 2".to_string()),
        layer_id: Set(Some("layer2".to_string())),
        properties: Set(None),
        ..Default::default()
    }.insert(&db).await?;
    
    // Create edge
    let edge = edges::ActiveModel {
        project_id: Set(project.id),
        source_node_id: Set("node1".to_string()),
        target_node_id: Set("node2".to_string()),
        properties: Set(None),
        ..Default::default()
    }.insert(&db).await?;
    
    // Verify data integrity
    let project_nodes = nodes::Entity::find()
        .filter(nodes::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(project_nodes.len(), 2);
    
    let project_edges = edges::Entity::find()
        .filter(edges::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(project_edges.len(), 1);
    
    let project_layers = layers::Entity::find()
        .filter(layers::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(project_layers.len(), 2);
    
    // Test cascade delete (delete project should remove all related data)
    projects::Entity::delete_by_id(project.id)
        .exec(&db)
        .await?;
    
    let remaining_nodes = nodes::Entity::find()
        .filter(nodes::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(remaining_nodes.len(), 0);
    
    let remaining_edges = edges::Entity::find()
        .filter(edges::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(remaining_edges.len(), 0);
    
    let remaining_layers = layers::Entity::find()
        .filter(layers::Column::ProjectId.eq(project.id))
        .all(&db)
        .await?;
    assert_eq!(remaining_layers.len(), 0);
    
    Ok(())
}