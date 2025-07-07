use sea_orm_migration::prelude::*;
use sea_orm::Statement;
use chrono::Utc;
use uuid::Uuid;
use serde_json::{json, Value};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // First, get all existing plans
        let plans_query = "SELECT id, project_id, name FROM plans";
        let plans_result = db.query_all(Statement::from_string(
            manager.get_database_backend(),
            plans_query.to_string()
        )).await?;
        
        for plan_row in plans_result {
            let plan_id: i32 = plan_row.try_get("", "id")?;
            let project_id: i32 = plan_row.try_get("", "project_id")?;
            let plan_name: String = plan_row.try_get("", "name")?;
            
            // Check if there are any nodes/edges/layers for this project
            let nodes_count_query = format!("SELECT COUNT(*) as count FROM nodes WHERE project_id = {}", project_id);
            let nodes_count_result = db.query_one(Statement::from_string(
                manager.get_database_backend(),
                nodes_count_query
            )).await?;
            
            if let Some(count_row) = nodes_count_result {
                let count: i64 = count_row.try_get("", "count")?;
                
                if count > 0 {
                    // Create default plan nodes for this plan
                    let import_node_id = Uuid::new_v4().to_string();
                    let export_node_id = Uuid::new_v4().to_string();
                    let graph_id = Uuid::new_v4().to_string();
                    let now = Utc::now().to_rfc3339();
                    
                    // Create import plan node
                    let import_config = json!({
                        "source": "migrated_from_project_data",
                        "project_id": project_id
                    });
                    
                    let insert_import_node = format!(
                        "INSERT INTO plan_nodes (id, plan_id, node_type, name, description, configuration, graph_id, position_x, position_y, created_at, updated_at) VALUES ('{}', {}, '{}', '{}', '{}', '{}', '{}', {}, {}, '{}', '{}')",
                        import_node_id,
                        plan_id,
                        "import",
                        format!("Import {}", plan_name),
                        "Migrated import node from existing project data",
                        import_config.to_string(),
                        graph_id,
                        100.0,
                        100.0,
                        now,
                        now
                    );
                    
                    db.execute(Statement::from_string(
                        manager.get_database_backend(),
                        insert_import_node
                    )).await?;
                    
                    // Create export plan node
                    let export_config = json!({
                        "format": "json",
                        "target": "default_export"
                    });
                    
                    let insert_export_node = format!(
                        "INSERT INTO plan_nodes (id, plan_id, node_type, name, description, configuration, graph_id, position_x, position_y, created_at, updated_at) VALUES ('{}', {}, '{}', '{}', '{}', '{}', NULL, {}, {}, '{}', '{}')",
                        export_node_id,
                        plan_id,
                        "export",
                        format!("Export {}", plan_name),
                        "Migrated export node",
                        export_config.to_string(),
                        300.0,
                        100.0,
                        now,
                        now
                    );
                    
                    db.execute(Statement::from_string(
                        manager.get_database_backend(),
                        insert_export_node
                    )).await?;
                    
                    // Get all nodes, edges, and layers for this project
                    let nodes_query = format!("SELECT * FROM nodes WHERE project_id = {}", project_id);
                    let edges_query = format!("SELECT * FROM edges WHERE project_id = {}", project_id);
                    let layers_query = format!("SELECT * FROM layers WHERE project_id = {}", project_id);
                    
                    let nodes_result = db.query_all(Statement::from_string(
                        manager.get_database_backend(),
                        nodes_query
                    )).await?;
                    
                    let edges_result = db.query_all(Statement::from_string(
                        manager.get_database_backend(),
                        edges_query
                    )).await?;
                    
                    let layers_result = db.query_all(Statement::from_string(
                        manager.get_database_backend(),
                        layers_query
                    )).await?;
                    
                    // Convert nodes to JSON array
                    let mut nodes_json = Vec::new();
                    for node_row in nodes_result {
                        let node_id: String = node_row.try_get("", "node_id")?;
                        let label: String = node_row.try_get("", "label")?;
                        let layer_id: Option<String> = node_row.try_get("", "layer_id").ok();
                        let properties: Option<String> = node_row.try_get("", "properties").ok();
                        
                        let properties_json: Value = if let Some(props_str) = properties {
                            serde_json::from_str(&props_str).unwrap_or(json!({}))
                        } else {
                            json!({})
                        };
                        
                        nodes_json.push(json!({
                            "id": node_id,
                            "label": label,
                            "layer": layer_id,
                            "properties": properties_json
                        }));
                    }
                    
                    // Convert edges to JSON array
                    let mut edges_json = Vec::new();
                    for edge_row in edges_result {
                        let source: String = edge_row.try_get("", "source_node_id")?;
                        let target: String = edge_row.try_get("", "target_node_id")?;
                        let properties: Option<String> = edge_row.try_get("", "properties").ok();
                        
                        let properties_json: Value = if let Some(props_str) = properties {
                            serde_json::from_str(&props_str).unwrap_or(json!({}))
                        } else {
                            json!({})
                        };
                        
                        edges_json.push(json!({
                            "source": source,
                            "target": target,
                            "properties": properties_json
                        }));
                    }
                    
                    // Convert layers to JSON array
                    let mut layers_json = Vec::new();
                    for layer_row in layers_result {
                        let layer_id: String = layer_row.try_get("", "layer_id")?;
                        let name: String = layer_row.try_get("", "name")?;
                        let color: Option<String> = layer_row.try_get("", "color").ok();
                        let properties: Option<String> = layer_row.try_get("", "properties").ok();
                        
                        let properties_json: Value = if let Some(props_str) = properties {
                            serde_json::from_str(&props_str).unwrap_or(json!({}))
                        } else {
                            json!({})
                        };
                        
                        layers_json.push(json!({
                            "id": layer_id,
                            "name": name,
                            "color": color,
                            "properties": properties_json
                        }));
                    }
                    
                    // Create graph data JSON
                    let graph_data = json!({
                        "nodes": nodes_json,
                        "edges": edges_json,
                        "layers": layers_json
                    });
                    
                    let metadata = json!({
                        "migrated_from_project": project_id,
                        "migration_timestamp": now,
                        "nodes_count": nodes_json.len(),
                        "edges_count": edges_json.len(),
                        "layers_count": layers_json.len()
                    });
                    
                    // Insert graph record
                    let insert_graph = format!(
                        "INSERT INTO graphs (id, plan_id, plan_node_id, name, description, graph_data, metadata, created_at, updated_at) VALUES ('{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}')",
                        graph_id,
                        plan_id,
                        import_node_id,
                        format!("Migrated Graph for {}", plan_name),
                        format!("Graph data migrated from project {} nodes/edges/layers", project_id),
                        graph_data.to_string().replace("'", "''"), // Escape single quotes
                        metadata.to_string().replace("'", "''"),   // Escape single quotes
                        now,
                        now
                    );
                    
                    db.execute(Statement::from_string(
                        manager.get_database_backend(),
                        insert_graph
                    )).await?;
                }
            }
        }
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        
        // Remove migrated data - delete graphs that were created by migration
        let delete_migrated_graphs = "DELETE FROM graphs WHERE metadata LIKE '%migrated_from_project%'";
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            delete_migrated_graphs.to_string()
        )).await?;
        
        // Remove migrated plan nodes
        let delete_migrated_plan_nodes = "DELETE FROM plan_nodes WHERE description LIKE '%Migrated%'";
        db.execute(Statement::from_string(
            manager.get_database_backend(),
            delete_migrated_plan_nodes.to_string()
        )).await?;
        
        Ok(())
    }
}