use sea_orm::*;
use anyhow::Result;
use chrono::Utc;
use tracing::{info, warn};
use std::path::PathBuf;
use std::collections::HashMap;
use serde_yaml;
use serde_json;

use crate::database::entities::{
    projects, plans, nodes, edges, layers
};

pub async fn create_example_project(db: &DatabaseConnection) -> Result<()> {
    info!("Starting to create all sample projects...");
    
    // Define all sample projects
    let sample_projects = vec![
        ("sample/attack_tree", "Attack Tree", "Security threat modeling with attack vectors and defense strategies"),
        ("sample/distributed-monolith", "Distributed Monolith", "A comprehensive example showcasing distributed monolith architecture with microservices"),
        ("sample/kvm_control_flow", "KVM Control Flow", "Kernel-based Virtual Machine control flow analysis and visualization"),
        ("sample/layercake-overview", "Layercake Overview", "Overview of the Layercake project structure and components"),
        ("sample/ref", "Reference Model", "Reference implementation with comprehensive export examples"),
    ];

    for (sample_path, project_name, description) in sample_projects {
        match create_sample_project(db, sample_path, project_name, description).await {
            Ok(_) => info!("Successfully created sample project: {}", project_name),
            Err(e) => warn!("Failed to create sample project {}: {}", project_name, e),
        }
    }
    
    info!("Completed sample project creation");
    Ok(())
}

async fn create_sample_project(
    db: &DatabaseConnection, 
    sample_path: &str, 
    project_name: &str, 
    description: &str
) -> Result<()> {
    // Check if project already exists
    let existing_project = projects::Entity::find()
        .filter(projects::Column::Name.eq(project_name))
        .one(db)
        .await?;
    
    if existing_project.is_some() {
        info!("Sample project '{}' already exists, skipping", project_name);
        return Ok(());
    }

    info!("Creating sample project: {}", project_name);
    
    // Create the project
    let now = Utc::now();
    let project = projects::ActiveModel {
        name: Set(project_name.to_string()),
        description: Set(Some(description.to_string())),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let project_result = projects::Entity::insert(project).exec(db).await?;
    let project_id = project_result.last_insert_id;
    
    info!("Created project '{}' with ID: {}", project_name, project_id);

    // Load CSV data from sample directory
    let sample_dir = PathBuf::from(sample_path);
    
    // Create layers from CSV
    if let Err(e) = create_layers_from_csv(db, project_id, &sample_dir).await {
        warn!("Failed to create layers for {}: {}", project_name, e);
    }
    
    // Create nodes from CSV
    if let Err(e) = create_nodes_from_csv(db, project_id, &sample_dir).await {
        warn!("Failed to create nodes for {}: {}", project_name, e);
    }
    
    // Create edges from CSV
    if let Err(e) = create_edges_from_csv(db, project_id, &sample_dir).await {
        warn!("Failed to create edges for {}: {}", project_name, e);
    }
    
    // Create plan from YAML
    if let Err(e) = create_plan_from_yaml(db, project_id, &sample_dir, project_name).await {
        warn!("Failed to create plan for {}: {}", project_name, e);
    }
    
    Ok(())
}

async fn create_layers_from_csv(
    db: &DatabaseConnection, 
    project_id: i32, 
    sample_dir: &PathBuf
) -> Result<()> {
    let layers_file = sample_dir.join("layers.csv");
    
    if !layers_file.exists() {
        return Err(anyhow::anyhow!("layers.csv not found in {}", sample_dir.display()));
    }

    let content = std::fs::read_to_string(&layers_file)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    
    let mut layer_models = Vec::new();
    for result in reader.records() {
        let record = result?;
        
        // Parse CSV: layer,label,background,border,text
        if record.len() >= 3 {
            let layer_id = record.get(0).unwrap_or("").to_string();
            let name = record.get(1).unwrap_or(&layer_id).to_string();
            let background = record.get(2).unwrap_or("ffffff").to_string();
            
            if !layer_id.is_empty() {
                let properties = serde_json::json!({
                    "background": background,
                    "border": record.get(3).unwrap_or("dddddd"),
                    "text": record.get(4).unwrap_or("000000")
                }).to_string();
                
                layer_models.push(layers::ActiveModel {
                    project_id: Set(project_id),
                    layer_id: Set(layer_id),
                    name: Set(name),
                    color: Set(Some(format!("#{}", background))),
                    properties: Set(Some(properties)),
                    ..Default::default()
                });
            }
        }
    }

    if !layer_models.is_empty() {
        let count = layer_models.len();
        layers::Entity::insert_many(layer_models).exec(db).await?;
        info!("Created {} layers from CSV", count);
    }
    
    Ok(())
}

async fn create_nodes_from_csv(
    db: &DatabaseConnection, 
    project_id: i32, 
    sample_dir: &PathBuf
) -> Result<()> {
    let nodes_file = sample_dir.join("nodes.csv");
    
    if !nodes_file.exists() {
        return Err(anyhow::anyhow!("nodes.csv not found in {}", sample_dir.display()));
    }

    let content = std::fs::read_to_string(&nodes_file)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    
    let mut node_models = Vec::new();
    for result in reader.records() {
        let record = result?;
        
        // Parse CSV: id,label,layer,is_partition,belongs_to,weight,comment
        if record.len() >= 2 {
            let node_id = record.get(0).unwrap_or("").to_string();
            let label = record.get(1).unwrap_or(&node_id).to_string();
            let layer_id = record.get(2).map(|s| if s.is_empty() { None } else { Some(s.to_string()) }).unwrap_or(None);
            
            if !node_id.is_empty() {
                let mut properties = HashMap::new();
                
                // Add additional properties from CSV columns
                if let Some(is_partition) = record.get(3) {
                    if !is_partition.is_empty() {
                        properties.insert("is_partition".to_string(), serde_json::Value::String(is_partition.to_string()));
                    }
                }
                if let Some(belongs_to) = record.get(4) {
                    if !belongs_to.is_empty() {
                        properties.insert("belongs_to".to_string(), serde_json::Value::String(belongs_to.to_string()));
                    }
                }
                if let Some(weight) = record.get(5) {
                    if !weight.is_empty() {
                        if let Ok(w) = weight.parse::<f64>() {
                            properties.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(w).unwrap_or_else(|| serde_json::Number::from(1))));
                        }
                    }
                }
                if let Some(comment) = record.get(6) {
                    if !comment.is_empty() {
                        properties.insert("comment".to_string(), serde_json::Value::String(comment.to_string()));
                    }
                }
                
                let properties_json = if properties.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&properties)?)
                };
                
                node_models.push(nodes::ActiveModel {
                    project_id: Set(project_id),
                    node_id: Set(node_id),
                    label: Set(label),
                    layer_id: Set(layer_id),
                    properties: Set(properties_json),
                    ..Default::default()
                });
            }
        }
    }

    if !node_models.is_empty() {
        let count = node_models.len();
        nodes::Entity::insert_many(node_models).exec(db).await?;
        info!("Created {} nodes from CSV", count);
    }
    
    Ok(())
}

async fn create_edges_from_csv(
    db: &DatabaseConnection, 
    project_id: i32, 
    sample_dir: &PathBuf
) -> Result<()> {
    // Try both 'edges.csv' and 'links.csv'
    let edges_file = if sample_dir.join("edges.csv").exists() {
        sample_dir.join("edges.csv")
    } else if sample_dir.join("links.csv").exists() {
        sample_dir.join("links.csv")
    } else {
        return Err(anyhow::anyhow!("Neither edges.csv nor links.csv found in {}", sample_dir.display()));
    };

    let content = std::fs::read_to_string(&edges_file)?;
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    
    let mut edge_models = Vec::new();
    for result in reader.records() {
        let record = result?;
        
        // Parse CSV: id,source,target,label,layer,weight,comment
        if record.len() >= 3 {
            let source = record.get(1).unwrap_or("").to_string();
            let target = record.get(2).unwrap_or("").to_string();
            
            if !source.is_empty() && !target.is_empty() {
                let mut properties = HashMap::new();
                
                // Add properties from CSV columns
                if let Some(edge_id) = record.get(0) {
                    if !edge_id.is_empty() {
                        properties.insert("edge_id".to_string(), serde_json::Value::String(edge_id.to_string()));
                    }
                }
                if let Some(label) = record.get(3) {
                    if !label.is_empty() {
                        properties.insert("label".to_string(), serde_json::Value::String(label.to_string()));
                    }
                }
                if let Some(layer) = record.get(4) {
                    if !layer.is_empty() {
                        properties.insert("layer".to_string(), serde_json::Value::String(layer.to_string()));
                    }
                }
                if let Some(weight) = record.get(5) {
                    if !weight.is_empty() {
                        if let Ok(w) = weight.parse::<f64>() {
                            properties.insert("weight".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(w).unwrap_or_else(|| serde_json::Number::from(1))));
                        }
                    }
                }
                if let Some(comment) = record.get(6) {
                    if !comment.is_empty() {
                        properties.insert("comment".to_string(), serde_json::Value::String(comment.to_string()));
                    }
                }
                
                let properties_json = if properties.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&properties)?)
                };
                
                edge_models.push(edges::ActiveModel {
                    project_id: Set(project_id),
                    source_node_id: Set(source),
                    target_node_id: Set(target),
                    properties: Set(properties_json),
                    ..Default::default()
                });
            }
        }
    }

    if !edge_models.is_empty() {
        let count = edge_models.len();
        edges::Entity::insert_many(edge_models).exec(db).await?;
        info!("Created {} edges from CSV", count);
    }
    
    Ok(())
}

async fn create_plan_from_yaml(
    db: &DatabaseConnection, 
    project_id: i32, 
    sample_dir: &PathBuf,
    project_name: &str
) -> Result<()> {
    let plan_file = sample_dir.join("plan.yaml");
    
    if !plan_file.exists() {
        return Err(anyhow::anyhow!("plan.yaml not found in {}", sample_dir.display()));
    }

    let yaml_content = std::fs::read_to_string(&plan_file)?;
    
    // Parse YAML and convert to JSON
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_content)?;
    let json_value: serde_json::Value = serde_yaml::from_value(yaml_value)?;
    let json_content = serde_json::to_string_pretty(&json_value)?;
    
    let plan_name = if let Some(meta) = json_value.get("meta") {
        meta.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or(&format!("{} Plan", project_name))
            .to_string()
    } else {
        format!("{} Plan", project_name)
    };

    let now = Utc::now();
    let plan = plans::ActiveModel {
        project_id: Set(project_id),
        name: Set(plan_name.clone()),
        plan_content: Set(json_content),
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
    
    info!("Created plan '{}' with ID: {}", plan_name, plan_id);
    Ok(())
}

