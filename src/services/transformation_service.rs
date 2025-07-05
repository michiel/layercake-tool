//! Service layer for graph transformations

use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, Set, ActiveModelTrait, QueryFilter, QueryOrder, ColumnTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, error};
use uuid::Uuid;

use crate::database::entities::{graphs, transformation_pipelines, transformation_rules};
use crate::graph::Graph;
use crate::transformations::{
    TransformationEngine, 
    TransformationPipeline, 
    TransformationRule,
    TransformationType,
    TransformationResult,
    TransformationStatistics,
};

/// API request/response types for transformations

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePipelineRequest {
    pub name: String,
    pub description: Option<String>,
    pub validation_enabled: Option<bool>,
    pub rollback_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePipelineRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub validation_enabled: Option<bool>,
    pub rollback_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub operation: TransformationType,
    pub enabled: Option<bool>,
    pub conditions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub operation: Option<TransformationType>,
    pub enabled: Option<bool>,
    pub conditions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePipelineRequest {
    pub graph_id: String,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRuleRequest {
    pub graph_id: String,
    pub rule_id: String,
    pub dry_run: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutionResult {
    pub success: bool,
    pub results: Vec<RuleExecutionResult>,
    pub total_statistics: TransformationStatistics,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleExecutionResult {
    pub rule_id: String,
    pub rule_name: String,
    pub success: bool,
    pub statistics: TransformationStatistics,
    pub error: Option<String>,
}

/// Transformation service for managing pipelines and executing transformations
pub struct TransformationService {
    db: DatabaseConnection,
    engine: TransformationEngine,
}

impl TransformationService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db,
            engine: TransformationEngine::new(),
        }
    }
    
    /// Create a new transformation pipeline
    pub async fn create_pipeline(&self, req: CreatePipelineRequest) -> Result<transformation_pipelines::Model> {
        debug!("Creating transformation pipeline: {}", req.name);
        
        let pipeline = TransformationPipeline {
            id: Uuid::new_v4().to_string(),
            name: req.name.clone(),
            description: req.description.clone(),
            rules: Vec::new(),
            validation_enabled: req.validation_enabled.unwrap_or(true),
            rollback_enabled: req.rollback_enabled.unwrap_or(true),
        };
        
        let pipeline_data = serde_json::to_string(&pipeline)?;
        
        let active_model = transformation_pipelines::ActiveModel {
            id: Set(pipeline.id.clone()),
            name: Set(req.name),
            description: Set(req.description),
            pipeline_data: Set(pipeline_data),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };
        
        let result = active_model.insert(&self.db).await?;
        info!("Created transformation pipeline: {}", result.id);
        Ok(result)
    }
    
    /// Get all transformation pipelines
    pub async fn list_pipelines(&self) -> Result<Vec<transformation_pipelines::Model>> {
        debug!("Listing transformation pipelines");
        
        let pipelines = transformation_pipelines::Entity::find()
            .all(&self.db)
            .await?;
        
        debug!("Found {} transformation pipelines", pipelines.len());
        Ok(pipelines)
    }
    
    /// Get a transformation pipeline by ID
    pub async fn get_pipeline(&self, id: &str) -> Result<Option<transformation_pipelines::Model>> {
        debug!("Getting transformation pipeline: {}", id);
        
        let pipeline = transformation_pipelines::Entity::find_by_id(id)
            .one(&self.db)
            .await?;
        
        Ok(pipeline)
    }
    
    /// Update a transformation pipeline
    pub async fn update_pipeline(&self, id: &str, req: UpdatePipelineRequest) -> Result<transformation_pipelines::Model> {
        debug!("Updating transformation pipeline: {}", id);
        
        let pipeline = transformation_pipelines::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Pipeline not found"))?;
        
        let mut active_model = transformation_pipelines::ActiveModel {
            id: Set(id.to_string()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        
        if let Some(name) = req.name {
            active_model.name = Set(name);
        }
        
        if let Some(description) = req.description {
            active_model.description = Set(Some(description));
        }
        
        // Update pipeline data if validation/rollback settings changed
        if req.validation_enabled.is_some() || req.rollback_enabled.is_some() {
            let mut pipeline_obj: TransformationPipeline = serde_json::from_str(&pipeline.pipeline_data)?;
            
            if let Some(validation) = req.validation_enabled {
                pipeline_obj.validation_enabled = validation;
            }
            
            if let Some(rollback) = req.rollback_enabled {
                pipeline_obj.rollback_enabled = rollback;
            }
            
            active_model.pipeline_data = Set(serde_json::to_string(&pipeline_obj)?);
        }
        
        let result = active_model.update(&self.db).await?;
        info!("Updated transformation pipeline: {}", result.id);
        Ok(result)
    }
    
    /// Delete a transformation pipeline
    pub async fn delete_pipeline(&self, id: &str) -> Result<()> {
        debug!("Deleting transformation pipeline: {}", id);
        
        // Delete associated rules first
        transformation_rules::Entity::delete_many()
            .filter(transformation_rules::Column::PipelineId.eq(id))
            .exec(&self.db)
            .await?;
        
        // Delete pipeline
        transformation_pipelines::Entity::delete_by_id(id)
            .exec(&self.db)
            .await?;
        
        info!("Deleted transformation pipeline: {}", id);
        Ok(())
    }
    
    /// Add a rule to a pipeline
    pub async fn add_rule_to_pipeline(&self, pipeline_id: &str, req: CreateRuleRequest) -> Result<transformation_rules::Model> {
        debug!("Adding rule to pipeline {}: {}", pipeline_id, req.name);
        
        // Verify pipeline exists
        let _pipeline = self.get_pipeline(pipeline_id).await?
            .ok_or_else(|| anyhow::anyhow!("Pipeline not found"))?;
        
        let rule = TransformationRule {
            id: Uuid::new_v4().to_string(),
            name: req.name.clone(),
            description: req.description.clone(),
            operation: req.operation,
            enabled: req.enabled.unwrap_or(true),
            conditions: req.conditions.unwrap_or_default(),
        };
        
        let rule_data = serde_json::to_string(&rule)?;
        
        let active_model = transformation_rules::ActiveModel {
            id: Set(rule.id.clone()),
            pipeline_id: Set(pipeline_id.to_string()),
            name: Set(req.name),
            description: Set(req.description),
            rule_data: Set(rule_data),
            enabled: Set(rule.enabled),
            order_index: Set(0), // Will be updated by reorder if needed
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
        };
        
        let result = active_model.insert(&self.db).await?;
        info!("Added rule to pipeline: {}", result.id);
        Ok(result)
    }
    
    /// Get rules for a pipeline
    pub async fn get_pipeline_rules(&self, pipeline_id: &str) -> Result<Vec<transformation_rules::Model>> {
        debug!("Getting rules for pipeline: {}", pipeline_id);
        
        let rules = transformation_rules::Entity::find()
            .filter(transformation_rules::Column::PipelineId.eq(pipeline_id))
            .all(&self.db)
            .await?;
        
        debug!("Found {} rules for pipeline {}", rules.len(), pipeline_id);
        Ok(rules)
    }
    
    /// Execute a transformation pipeline on a graph
    pub async fn execute_pipeline(&self, req: ExecutePipelineRequest) -> Result<PipelineExecutionResult> {
        let start_time = std::time::Instant::now();
        info!("Executing transformation pipeline on graph: {}", req.graph_id);
        
        // Get the graph
        let graph_entity = graphs::Entity::find_by_id(&req.graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;
        
        let graph: Graph = serde_json::from_str(&graph_entity.graph_data)?;
        
        // Get the pipeline (assuming we have the pipeline ID from somewhere)
        // For now, we'll create a simple pipeline for demonstration
        let mut pipeline = TransformationPipeline::new("Test Pipeline".to_string());
        
        // Set dry run mode if requested
        let mut engine = TransformationEngine::new();
        engine.set_dry_run(req.dry_run.unwrap_or(false));
        
        match engine.execute_pipeline(&pipeline, graph) {
            Ok(results) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                let rule_results: Vec<RuleExecutionResult> = results.iter().map(|r| {
                    RuleExecutionResult {
                        rule_id: r.rule_id.clone(),
                        rule_name: "Rule".to_string(), // Would get from actual rule
                        success: r.success,
                        statistics: r.statistics.clone(),
                        error: r.error.clone(),
                    }
                }).collect();
                
                let success = results.iter().all(|r| r.success);
                let total_stats = self.aggregate_statistics(&results);
                
                info!("Pipeline execution completed successfully in {}ms", execution_time);
                
                Ok(PipelineExecutionResult {
                    success,
                    results: rule_results,
                    total_statistics: total_stats,
                    execution_time_ms: execution_time,
                    error: None,
                })
            },
            Err(e) => {
                error!("Pipeline execution failed: {}", e);
                
                Ok(PipelineExecutionResult {
                    success: false,
                    results: Vec::new(),
                    total_statistics: TransformationStatistics::default(),
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    /// Execute a single transformation rule on a graph
    pub async fn execute_rule(&self, req: ExecuteRuleRequest) -> Result<RuleExecutionResult> {
        info!("Executing transformation rule {} on graph: {}", req.rule_id, req.graph_id);
        
        // Get the graph
        let graph_entity = graphs::Entity::find_by_id(&req.graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;
        
        let graph: Graph = serde_json::from_str(&graph_entity.graph_data)?;
        
        // Get the rule
        let rule_entity = transformation_rules::Entity::find_by_id(&req.rule_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Rule not found"))?;
        
        let rule: TransformationRule = serde_json::from_str(&rule_entity.rule_data)?;
        
        // Set dry run mode if requested
        let mut engine = TransformationEngine::new();
        engine.set_dry_run(req.dry_run.unwrap_or(false));
        
        match engine.execute_rule(&rule, graph) {
            Ok(result) => {
                info!("Rule execution completed: {}", result.success);
                
                Ok(RuleExecutionResult {
                    rule_id: req.rule_id,
                    rule_name: rule.name,
                    success: result.success,
                    statistics: result.statistics,
                    error: result.error,
                })
            },
            Err(e) => {
                error!("Rule execution failed: {}", e);
                
                Ok(RuleExecutionResult {
                    rule_id: req.rule_id,
                    rule_name: rule.name,
                    success: false,
                    statistics: TransformationStatistics::default(),
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    /// Get transformation statistics for a graph
    pub async fn get_graph_statistics(&self, graph_id: &str) -> Result<HashMap<String, serde_json::Value>> {
        debug!("Getting statistics for graph: {}", graph_id);
        
        let graph_entity = graphs::Entity::find_by_id(graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Graph not found"))?;
        
        let graph: Graph = serde_json::from_str(&graph_entity.graph_data)?;
        
        let mut stats = HashMap::new();
        stats.insert("node_count".to_string(), serde_json::Value::from(graph.nodes.len()));
        stats.insert("edge_count".to_string(), serde_json::Value::from(graph.edges.len()));
        stats.insert("layer_count".to_string(), serde_json::Value::from(graph.layers.len()));
        
        // Calculate basic graph metrics
        let isolated_nodes = graph.nodes.iter()
            .filter(|node| !graph.edges.iter().any(|edge| edge.source == node.id || edge.target == node.id))
            .count();
        
        stats.insert("isolated_nodes".to_string(), serde_json::Value::from(isolated_nodes));
        
        // Calculate average degree
        let total_degree: usize = graph.nodes.iter()
            .map(|node| graph.edges.iter()
                .filter(|edge| edge.source == node.id || edge.target == node.id)
                .count())
            .sum();
        
        let avg_degree = if graph.nodes.is_empty() { 0.0 } else { total_degree as f64 / graph.nodes.len() as f64 };
        stats.insert("average_degree".to_string(), serde_json::Value::from(avg_degree));
        
        Ok(stats)
    }
    
    /// Helper method to aggregate transformation statistics
    fn aggregate_statistics(&self, results: &[TransformationResult]) -> TransformationStatistics {
        let mut total = TransformationStatistics::default();
        
        for result in results {
            total.nodes_added += result.statistics.nodes_added;
            total.nodes_removed += result.statistics.nodes_removed;
            total.nodes_modified += result.statistics.nodes_modified;
            total.edges_added += result.statistics.edges_added;
            total.edges_removed += result.statistics.edges_removed;
            total.edges_modified += result.statistics.edges_modified;
            total.layers_added += result.statistics.layers_added;
            total.layers_removed += result.statistics.layers_removed;
            total.layers_modified += result.statistics.layers_modified;
            total.execution_time_ms += result.statistics.execution_time_ms;
        }
        
        total
    }
}