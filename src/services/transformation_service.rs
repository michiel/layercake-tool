//! Service layer for graph transformations using plan-centric model

use anyhow::Result;
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, error};
use uuid::Uuid;

use crate::database::entities::{graphs, plans, plan_nodes};
use crate::graph::Graph;
use crate::transformations::{
    TransformationEngine, 
    TransformationType,
    TransformationResult,
    TransformationStatistics,
};

/// API request/response types for transformations in plan-centric model

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTransformationNodeRequest {
    pub name: String,
    pub description: Option<String>,
    pub transformation_type: TransformationType,
    pub input_graph_id: String,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTransformationNodeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub transformation_type: Option<TransformationType>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationNodeResponse {
    pub id: String,
    pub plan_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub transformation_type: TransformationType,
    pub input_graph_id: Option<String>,
    pub output_graph_id: Option<String>,
    pub position_x: Option<f64>,
    pub position_y: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTransformationRequest {
    pub dry_run: Option<bool>,
    pub validate_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteTransformationResponse {
    pub success: bool,
    pub output_graph_id: Option<String>,
    pub statistics: TransformationStatistics,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Service for managing graph transformations through plan nodes
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

    /// Create a new transformation node in a plan
    pub async fn create_transformation_node(
        &self,
        plan_id: i32,
        req: CreateTransformationNodeRequest,
    ) -> Result<TransformationNodeResponse> {
        debug!("Creating transformation node for plan {}", plan_id);

        // Validate that the plan exists
        let plan = plans::Entity::find_by_id(plan_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Plan not found"))?;

        // Validate that the input graph exists
        let input_graph = graphs::Entity::find_by_id(&req.input_graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Input graph not found"))?;

        // Create transformation node
        let node_id = Uuid::new_v4().to_string();
        let configuration = serde_json::to_string(&req.transformation_type)?;
        
        let active_model = plan_nodes::ActiveModel {
            id: sea_orm::Set(node_id.clone()),
            plan_id: sea_orm::Set(plan_id),
            node_type: sea_orm::Set("transformation".to_string()),
            name: sea_orm::Set(req.name.clone()),
            description: sea_orm::Set(req.description.clone()),
            configuration: sea_orm::Set(configuration),
            graph_id: sea_orm::Set(None), // Will be set after execution
            position_x: sea_orm::Set(req.position_x),
            position_y: sea_orm::Set(req.position_y),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
        };

        let node = active_model.insert(&self.db).await?;
        
        info!("Created transformation node {} for plan {}", node_id, plan_id);

        Ok(TransformationNodeResponse {
            id: node.id,
            plan_id: node.plan_id,
            name: node.name,
            description: node.description,
            transformation_type: req.transformation_type,
            input_graph_id: Some(req.input_graph_id),
            output_graph_id: node.graph_id,
            position_x: node.position_x,
            position_y: node.position_y,
            created_at: node.created_at,
            updated_at: node.updated_at,
        })
    }

    /// Update a transformation node
    pub async fn update_transformation_node(
        &self,
        node_id: &str,
        req: UpdateTransformationNodeRequest,
    ) -> Result<TransformationNodeResponse> {
        debug!("Updating transformation node {}", node_id);

        let node = plan_nodes::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Transformation node not found"))?;

        if node.node_type != "transformation" {
            return Err(anyhow::anyhow!("Node is not a transformation node"));
        }

        let mut active_model = plan_nodes::ActiveModel {
            id: sea_orm::Set(node.id.clone()),
            plan_id: sea_orm::Set(node.plan_id),
            node_type: sea_orm::Set(node.node_type.clone()),
            name: sea_orm::Set(req.name.unwrap_or(node.name.clone())),
            description: sea_orm::Set(req.description.or(node.description.clone())),
            configuration: sea_orm::Set(node.configuration.clone()), // Update if transformation_type changed
            graph_id: sea_orm::Set(node.graph_id.clone()),
            position_x: sea_orm::Set(req.position_x.or(node.position_x)),
            position_y: sea_orm::Set(req.position_y.or(node.position_y)),
            created_at: sea_orm::Set(node.created_at),
            updated_at: sea_orm::Set(chrono::Utc::now()),
        };

        // Update configuration if transformation_type changed
        if let Some(transformation_type) = req.transformation_type {
            let configuration = serde_json::to_string(&transformation_type)?;
            active_model.configuration = sea_orm::Set(configuration);
        }

        let updated_node = active_model.update(&self.db).await?;
        
        // Parse the transformation type from configuration
        let transformation_type: TransformationType = serde_json::from_str(&updated_node.configuration)?;

        info!("Updated transformation node {}", node_id);

        Ok(TransformationNodeResponse {
            id: updated_node.id,
            plan_id: updated_node.plan_id,
            name: updated_node.name,
            description: updated_node.description,
            transformation_type,
            input_graph_id: None, // Would need to track this separately
            output_graph_id: updated_node.graph_id,
            position_x: updated_node.position_x,
            position_y: updated_node.position_y,
            created_at: updated_node.created_at,
            updated_at: updated_node.updated_at,
        })
    }

    /// Execute a transformation node
    pub async fn execute_transformation_node(
        &self,
        node_id: &str,
        input_graph_id: &str,
        req: ExecuteTransformationRequest,
    ) -> Result<ExecuteTransformationResponse> {
        debug!("Executing transformation node {} with input graph {}", node_id, input_graph_id);

        let node = plan_nodes::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Transformation node not found"))?;

        if node.node_type != "transformation" {
            return Err(anyhow::anyhow!("Node is not a transformation node"));
        }

        // Load input graph
        let input_graph_entity = graphs::Entity::find_by_id(input_graph_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Input graph not found"))?;

        let input_graph_data = input_graph_entity.get_graph_data_json()?;
        let input_graph = Graph::from_json(&input_graph_data)?;

        // Parse transformation configuration
        let transformation_type: TransformationType = serde_json::from_str(&node.configuration)?;

        if req.validate_only.unwrap_or(false) {
            // Just validate the transformation without executing
            match self.engine.validate_transformation(&input_graph, &transformation_type) {
                Ok(_) => {
                    return Ok(ExecuteTransformationResponse {
                        success: true,
                        output_graph_id: None,
                        statistics: TransformationStatistics::default(),
                        warnings: vec![],
                        errors: vec![],
                    });
                }
                Err(e) => {
                    return Ok(ExecuteTransformationResponse {
                        success: false,
                        output_graph_id: None,
                        statistics: TransformationStatistics::default(),
                        warnings: vec![],
                        errors: vec![e.to_string()],
                    });
                }
            }
        }

        // Execute transformation
        let result = self.engine.execute_transformation(input_graph, transformation_type).await?;

        if req.dry_run.unwrap_or(false) {
            // Return results without persisting
            return Ok(ExecuteTransformationResponse {
                success: result.success,
                output_graph_id: None,
                statistics: result.statistics,
                warnings: vec![], // TransformationResult doesn't have warnings field
                errors: result.error.map(|e| vec![e]).unwrap_or_else(Vec::new),
            });
        }

        // Create output graph
        let output_graph_id = Uuid::new_v4().to_string();
        let transformed_graph = result.transformed_graph.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No transformed graph in result"))?;
        let output_graph_data = transformed_graph.to_json()?;
        
        let graph_active_model = graphs::ActiveModel {
            id: sea_orm::Set(output_graph_id.clone()),
            plan_id: sea_orm::Set(node.plan_id),
            plan_node_id: sea_orm::Set(node.id.clone()),
            name: sea_orm::Set(format!("{} - Output", node.name)),
            description: sea_orm::Set(Some(format!("Output graph from transformation node {}", node.id))),
            graph_data: sea_orm::Set(serde_json::to_string(&output_graph_data)?),
            metadata: sea_orm::Set(Some(serde_json::to_string(&result.statistics)?)),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
        };

        graph_active_model.insert(&self.db).await?;

        // Update the transformation node to reference the output graph
        let mut node_active_model = plan_nodes::ActiveModel {
            id: sea_orm::Set(node.id.clone()),
            plan_id: sea_orm::Set(node.plan_id),
            node_type: sea_orm::Set(node.node_type.clone()),
            name: sea_orm::Set(node.name.clone()),
            description: sea_orm::Set(node.description.clone()),
            configuration: sea_orm::Set(node.configuration.clone()),
            graph_id: sea_orm::Set(Some(output_graph_id.clone())),
            position_x: sea_orm::Set(node.position_x),
            position_y: sea_orm::Set(node.position_y),
            created_at: sea_orm::Set(node.created_at),
            updated_at: sea_orm::Set(chrono::Utc::now()),
        };

        node_active_model.update(&self.db).await?;

        info!("Executed transformation node {} -> output graph {}", node_id, output_graph_id);

        Ok(ExecuteTransformationResponse {
            success: result.success,
            output_graph_id: Some(output_graph_id),
            statistics: result.statistics,
            warnings: vec![], // TransformationResult doesn't have warnings field
            errors: result.error.map(|e| vec![e]).unwrap_or_else(Vec::new),
        })
    }

    /// Get transformation nodes for a plan
    pub async fn get_transformation_nodes(
        &self,
        plan_id: i32,
    ) -> Result<Vec<TransformationNodeResponse>> {
        debug!("Getting transformation nodes for plan {}", plan_id);

        let nodes = plan_nodes::Entity::find()
            .filter(plan_nodes::Column::PlanId.eq(plan_id))
            .filter(plan_nodes::Column::NodeType.eq("transformation"))
            .all(&self.db)
            .await?;

        let mut responses = Vec::new();
        for node in nodes {
            let transformation_type: TransformationType = serde_json::from_str(&node.configuration)?;
            
            responses.push(TransformationNodeResponse {
                id: node.id,
                plan_id: node.plan_id,
                name: node.name,
                description: node.description,
                transformation_type,
                input_graph_id: None, // Would need to track this separately
                output_graph_id: node.graph_id,
                position_x: node.position_x,
                position_y: node.position_y,
                created_at: node.created_at,
                updated_at: node.updated_at,
            });
        }

        Ok(responses)
    }

    /// Delete a transformation node
    pub async fn delete_transformation_node(&self, node_id: &str) -> Result<()> {
        debug!("Deleting transformation node {}", node_id);

        let node = plan_nodes::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Transformation node not found"))?;

        if node.node_type != "transformation" {
            return Err(anyhow::anyhow!("Node is not a transformation node"));
        }

        // Delete associated output graph if it exists
        if let Some(graph_id) = &node.graph_id {
            graphs::Entity::delete_by_id(graph_id)
                .exec(&self.db)
                .await?;
        }

        // Delete the transformation node
        plan_nodes::Entity::delete_by_id(node_id)
            .exec(&self.db)
            .await?;

        info!("Deleted transformation node {}", node_id);
        Ok(())
    }
}