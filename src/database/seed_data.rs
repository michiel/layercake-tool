use sea_orm::*;
use anyhow::Result;
use chrono::Utc;
use tracing::info;

use crate::database::entities::{
    projects, plans, nodes, edges, layers
};

pub async fn create_example_project(db: &DatabaseConnection) -> Result<()> {
    // First check if example project already exists
    let existing_project = projects::Entity::find()
        .filter(projects::Column::Name.eq("Distributed Monolith Example"))
        .one(db)
        .await?;
    
    if existing_project.is_some() {
        info!("Example project already exists, skipping seed data creation");
        return Ok(());
    }

    info!("Creating example project: Distributed Monolith");
    
    // Create the project
    let now = Utc::now();
    let project = projects::ActiveModel {
        name: Set("Distributed Monolith Example".to_string()),
        description: Set(Some("A comprehensive example showcasing a distributed monolith architecture with microservices, databases, and API gateways. This project demonstrates the complexity and interconnections in modern cloud-native applications.".to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let project_result = projects::Entity::insert(project).exec(db).await?;
    let project_id = project_result.last_insert_id;
    
    info!("Created project with ID: {}", project_id);

    // Create layers
    create_example_layers(db, project_id).await?;
    
    // Create nodes
    create_example_nodes(db, project_id).await?;
    
    // Create edges
    create_example_edges(db, project_id).await?;
    
    // Create a sample plan
    create_example_plan(db, project_id).await?;
    
    info!("Successfully created all example data for project {}", project_id);
    Ok(())
}

async fn create_example_layers(db: &DatabaseConnection, project_id: i32) -> Result<()> {
    info!("Creating example layers...");
    
    let layers_data = vec![
        ("database", "Database", "#ffcccc"),
        ("table", "Table", "#ffebcc"),
        ("lambda", "Lambda", "#ccffcc"),
        ("container", "Container", "#ccccff"),
        ("stored_proc", "Stored Procedure", "#ffccff"),
        ("api_gateway", "API Gateway", "#ffff99"),
        ("s3", "S3 Storage", "#ccffff"),
        ("root", "Root", "#ffffff"),
    ];

    let mut layer_models = Vec::new();
    for (layer_id, name, color) in layers_data {
        layer_models.push(layers::ActiveModel {
            project_id: Set(project_id),
            layer_id: Set(layer_id.to_string()),
            name: Set(name.to_string()),
            color: Set(Some(color.to_string())),
            properties: Set(Some(r#"{"description": "Auto-generated layer from seed data"}"#.to_string())),
            ..Default::default()
        });
    }

    layers::Entity::insert_many(layer_models).exec(db).await?;
    info!("Created {} layers", 8);
    Ok(())
}

async fn create_example_nodes(db: &DatabaseConnection, project_id: i32) -> Result<()> {
    info!("Creating example nodes...");
    
    let nodes_data = vec![
        // Root and main components
        ("project", "Project", Some("root")),
        ("database", "Database", Some("root")),
        ("lambda", "Lambda", Some("root")),
        ("container", "Container", Some("root")),
        ("api_gateway", "Api Gateway", Some("root")),
        ("s3", "S3 Storage", Some("root")),
        
        // Database instances
        ("mysql", "MySQL Database", Some("database")),
        ("postgres", "PostgreSQL Database", Some("database")),
        
        // API Gateway instance
        ("api_gateway_instance", "Main API Gateway", Some("api_gateway")),
        
        // Sample Lambda functions
        ("lambda_1", "User Authentication Lambda", Some("lambda")),
        ("lambda_2", "Order Processing Lambda", Some("lambda")),
        ("lambda_3", "Inventory Management Lambda", Some("lambda")),
        ("lambda_4", "Payment Processing Lambda", Some("lambda")),
        ("lambda_5", "Notification Service Lambda", Some("lambda")),
        
        // Container services
        ("container_1", "Web Frontend Service", Some("container")),
        ("container_2", "Analytics Service", Some("container")),
        ("container_3", "Reporting Service", Some("container")),
        
        // Database tables
        ("mysql_table_1", "users_table", Some("table")),
        ("mysql_table_2", "orders_table", Some("table")),
        ("mysql_table_3", "products_table", Some("table")),
        ("mysql_table_4", "inventory_table", Some("table")),
        ("mysql_table_5", "payments_table", Some("table")),
        ("postgres_table_1", "analytics_events", Some("table")),
        ("postgres_table_2", "user_sessions", Some("table")),
        ("postgres_table_3", "system_logs", Some("table")),
        
        // S3 buckets
        ("s3_bucket_1", "Static Assets Bucket", Some("s3")),
        ("s3_bucket_2", "User Uploads Bucket", Some("s3")),
        ("s3_bucket_3", "Backup Storage Bucket", Some("s3")),
    ];

    let mut node_models = Vec::new();
    let nodes_count = nodes_data.len();
    for (node_id, label, layer_id) in nodes_data {
        let properties = match layer_id.as_deref() {
            Some("lambda") => r#"{"runtime": "nodejs18.x", "memory": 512, "timeout": 30}"#,
            Some("container") => r#"{"image": "nginx:latest", "replicas": 2, "cpu": "500m", "memory": "1Gi"}"#,
            Some("table") => r#"{"engine": "InnoDB", "charset": "utf8mb4", "rows": 10000}"#,
            Some("s3") => r#"{"region": "us-east-1", "versioning": true, "encryption": "AES256"}"#,
            _ => r#"{"description": "Auto-generated node from seed data"}"#,
        };
        
        node_models.push(nodes::ActiveModel {
            project_id: Set(project_id),
            node_id: Set(node_id.to_string()),
            label: Set(label.to_string()),
            layer_id: Set(layer_id.map(|s| s.to_string())),
            properties: Set(Some(properties.to_string())),
            ..Default::default()
        });
    }

    nodes::Entity::insert_many(node_models).exec(db).await?;
    info!("Created {} nodes", nodes_count);
    Ok(())
}

async fn create_example_edges(db: &DatabaseConnection, project_id: i32) -> Result<()> {
    info!("Creating example edges...");
    
    let edges_data = vec![
        // API Gateway to services
        ("api_gateway_instance", "lambda_1", r#"{"route": "/auth", "method": "POST"}"#),
        ("api_gateway_instance", "lambda_2", r#"{"route": "/orders", "method": "POST"}"#),
        ("api_gateway_instance", "lambda_3", r#"{"route": "/inventory", "method": "GET"}"#),
        ("api_gateway_instance", "lambda_4", r#"{"route": "/payments", "method": "POST"}"#),
        ("api_gateway_instance", "lambda_5", r#"{"route": "/notifications", "method": "POST"}"#),
        ("api_gateway_instance", "container_1", r#"{"route": "/app/*", "method": "GET"}"#),
        
        // Lambda to database connections
        ("lambda_1", "mysql_table_1", r#"{"operation": "read_write", "connection_pool": 5}"#),
        ("lambda_1", "postgres_table_2", r#"{"operation": "write", "connection_pool": 2}"#),
        ("lambda_2", "mysql_table_2", r#"{"operation": "read_write", "connection_pool": 10}"#),
        ("lambda_2", "mysql_table_3", r#"{"operation": "read", "connection_pool": 3}"#),
        ("lambda_2", "mysql_table_4", r#"{"operation": "read_write", "connection_pool": 5}"#),
        ("lambda_3", "mysql_table_4", r#"{"operation": "read_write", "connection_pool": 8}"#),
        ("lambda_3", "mysql_table_3", r#"{"operation": "read", "connection_pool": 5}"#),
        ("lambda_4", "mysql_table_5", r#"{"operation": "write", "connection_pool": 3}"#),
        ("lambda_4", "mysql_table_2", r#"{"operation": "read", "connection_pool": 2}"#),
        ("lambda_5", "postgres_table_1", r#"{"operation": "write", "connection_pool": 2}"#),
        
        // Container to database connections
        ("container_1", "mysql_table_1", r#"{"operation": "read", "connection_pool": 3}"#),
        ("container_1", "s3_bucket_1", r#"{"operation": "read", "access_pattern": "static_assets"}"#),
        ("container_2", "postgres_table_1", r#"{"operation": "read", "connection_pool": 5}"#),
        ("container_2", "postgres_table_3", r#"{"operation": "read_write", "connection_pool": 3}"#),
        ("container_3", "mysql_table_2", r#"{"operation": "read", "connection_pool": 2}"#),
        ("container_3", "mysql_table_5", r#"{"operation": "read", "connection_pool": 2}"#),
        
        // Service to S3 connections
        ("lambda_1", "s3_bucket_2", r#"{"operation": "write", "access_pattern": "user_uploads"}"#),
        ("lambda_2", "s3_bucket_3", r#"{"operation": "write", "access_pattern": "order_backups"}"#),
        ("container_2", "s3_bucket_3", r#"{"operation": "read_write", "access_pattern": "analytics_data"}"#),
        
        // Inter-service communications
        ("lambda_2", "lambda_3", r#"{"protocol": "HTTP", "async": true, "purpose": "inventory_check"}"#),
        ("lambda_2", "lambda_4", r#"{"protocol": "HTTP", "async": false, "purpose": "payment_processing"}"#),
        ("lambda_4", "lambda_5", r#"{"protocol": "SNS", "async": true, "purpose": "payment_notification"}"#),
    ];

    let mut edge_models = Vec::new();
    let edges_count = edges_data.len();
    for (source, target, properties) in edges_data {
        edge_models.push(edges::ActiveModel {
            project_id: Set(project_id),
            source_node_id: Set(source.to_string()),
            target_node_id: Set(target.to_string()),
            properties: Set(Some(properties.to_string())),
            ..Default::default()
        });
    }

    edges::Entity::insert_many(edge_models).exec(db).await?;
    info!("Created {} edges", edges_count);
    Ok(())
}

async fn create_example_plan(db: &DatabaseConnection, project_id: i32) -> Result<()> {
    info!("Creating example plan...");
    
    let plan_content = serde_json::json!({
        "meta": {
            "name": "Distributed Monolith Analysis",
            "version": "1.0.0",
            "description": "Comprehensive analysis and visualization of the distributed monolith architecture"
        },
        "import": {
            "profiles": [
                {
                    "filename": "nodes.csv",
                    "filetype": "Nodes",
                    "description": "Import all service nodes, database tables, and infrastructure components"
                },
                {
                    "filename": "edges.csv", 
                    "filetype": "Edges",
                    "description": "Import all connections and dependencies between components"
                },
                {
                    "filename": "layers.csv",
                    "filetype": "Layers", 
                    "description": "Import layer definitions for visual organization"
                }
            ]
        },
        "export": {
            "profiles": [
                {
                    "filename": "out/distributed-monolith.gml",
                    "exporter": "GML",
                    "description": "Graph Modeling Language export for Gephi and other graph tools"
                },
                {
                    "filename": "out/distributed-monolith.dot",
                    "exporter": "DOT",
                    "description": "Graphviz DOT format for network diagrams",
                    "render_config": {
                        "contain_nodes": false
                    }
                },
                {
                    "filename": "out/service-dependencies.puml",
                    "exporter": "PlantUML",
                    "description": "PlantUML diagram showing service dependencies"
                },
                {
                    "filename": "out/connectivity-matrix.csv",
                    "exporter": "CSVMatrix",
                    "description": "Adjacency matrix showing all connections"
                }
            ]
        },
        "analysis": {
            "connectivity": {
                "enabled": true,
                "min_connections": 1,
                "highlight_hubs": true
            },
            "layers": {
                "analyze_layer_separation": true,
                "detect_layer_violations": true
            },
            "performance": {
                "identify_bottlenecks": true,
                "connection_pool_analysis": true
            }
        }
    });

    let now = Utc::now();
    let plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set("Distributed Monolith Analysis Plan".to_string()),
        plan_content: Set(serde_json::to_string_pretty(&plan_content)?),
        plan_format: Set("json".to_string()),
        plan_schema_version: Set("1.0.0".to_string()),
        dependencies: Set(None),
        status: Set("pending".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let plan_result = plans::Entity::insert(plan).exec(db).await?;
    let plan_id = plan_result.last_insert_id;
    
    info!("Created plan with ID: {}", plan_id);
    Ok(())
}