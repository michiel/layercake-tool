use async_graphql::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::app_context::{GraphLayerUpdateRequest, GraphNodeUpdateRequest};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph::{
    CreateGraphInput, CreateLayerInput, Graph, GraphValidationResult, UpdateGraphInput,
};
use crate::services::graph_edit_service::GraphEditService;
use crate::services::graph_service::GraphService;

#[derive(Default)]
pub struct GraphMutation;

#[Object]
impl GraphMutation {
    /// Create a new Graph
    async fn create_graph(&self, ctx: &Context<'_>, input: CreateGraphInput) -> Result<Graph> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        let graph = graph_service
            .create_graph(input.project_id, input.name, None)
            .await
            .map_err(|e| StructuredError::service("GraphService::create_graph", e))?;

        Ok(Graph::from(graph))
    }

    /// Update Graph metadata
    async fn update_graph(
        &self,
        ctx: &Context<'_>,
        id: i32,
        input: UpdateGraphInput,
    ) -> Result<Graph> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        let graph = graph_service
            .update_graph(id, input.name)
            .await
            .map_err(|e| StructuredError::service("GraphService::update_graph", e))?;

        Ok(Graph::from(graph))
    }

    /// Validate persisted graph structure
    async fn validate_graph(
        &self,
        ctx: &Context<'_>,
        id: i32,
    ) -> Result<GraphValidationResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let summary = context
            .app
            .validate_graph(id)
            .await
            .map_err(|e| StructuredError::service("AppContext::validate_graph", e))?;

        Ok(GraphValidationResult::from(summary))
    }

    /// Delete Graph
    async fn delete_graph(&self, ctx: &Context<'_>, id: i32) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        graph_service
            .delete_graph(id)
            .await
            .map_err(|e| StructuredError::service("GraphService::delete_graph", e))?;

        Ok(true)
    }

    /// Create a new Layer
    async fn create_layer(
        &self,
        ctx: &Context<'_>,
        input: CreateLayerInput,
    ) -> Result<crate::graphql::types::Layer> {
        let context = ctx.data::<GraphQLContext>()?;

        use crate::database::entities::graph_layers;

        let layer = graph_layers::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_id: Set(input.graph_id),
            layer_id: Set(input.layer_id),
            name: Set(input.name),
            background_color: Set(None),
            text_color: Set(None),
            border_color: Set(None),
            alias: Set(None),
            comment: Set(None),
            properties: Set(None),
            dataset_id: Set(None),
        };

        let inserted_layer = layer
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_layers::Entity::insert", e))?;

        Ok(crate::graphql::types::Layer::from(inserted_layer))
    }

    /// Update a graph node's properties
    async fn update_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        node_id: String,
        label: Option<String>,
        layer: Option<String>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
        belongs_to: Option<String>,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;

        let node = context
            .app
            .update_graph_node(graph_id, node_id, label, layer, attrs, belongs_to)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_graph_node", e))?;

        Ok(node)
    }

    /// Update layer properties (name, colors, etc.)
    async fn update_layer_properties(
        &self,
        ctx: &Context<'_>,
        id: i32,
        name: Option<String>,
        alias: Option<String>,
        properties: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::layer::Layer> {
        let context = ctx.data::<GraphQLContext>()?;

        let layer = context
            .app
            .update_layer_properties(id, name, alias, properties)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_layer_properties", e))?;

        Ok(layer)
    }

    /// Add a new node to a graph
    async fn add_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        id: String,
        label: Option<String>,
        layer: Option<String>,
        is_partition: bool,
        belongs_to: Option<String>,
        weight: Option<f64>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Create the new node
        let node = graph_service
            .add_graph_node(
                graph_id,
                id.clone(),
                label.clone(),
                layer.clone(),
                is_partition,
                belongs_to.clone(),
                weight,
                attrs.clone(),
            )
            .await
            .map_err(|e| StructuredError::service("GraphService::add_graph_node", e))?;

        // Create edit record for the new node
        let node_data = serde_json::json!({
            "id": id,
            "label": label,
            "layer": layer,
            "is_partition": is_partition,
            "belongs_to": belongs_to,
            "weight": weight,
            "attrs": attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "node".to_string(),
                id.clone(),
                "create".to_string(),
                None,
                None,
                Some(node_data),
                None,
                true,
            )
            .await;

        Ok(crate::graphql::types::graph_node::GraphNode::from(node))
    }

    /// Add a new edge to a graph
    async fn add_graph_edge(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        id: String,
        source: String,
        target: String,
        label: Option<String>,
        layer: Option<String>,
        weight: Option<f64>,
        attrs: Option<crate::graphql::types::scalars::JSON>,
    ) -> Result<crate::graphql::types::graph_edge::GraphEdge> {
        let context = ctx.data::<GraphQLContext>()?;
        let edit_service = GraphEditService::new(context.db.clone());

        use crate::database::entities::graph_edges::{
            ActiveModel as GraphEdgeActiveModel, Entity as GraphEdges,
        };
        use sea_orm::{ActiveValue::Set, EntityTrait};

        // Create the new edge
        let now = chrono::Utc::now();
        let edge_model = GraphEdgeActiveModel {
            id: Set(id.clone()),
            graph_id: Set(graph_id),
            source: Set(source.clone()),
            target: Set(target.clone()),
            label: Set(label.clone()),
            layer: Set(layer.clone()),
            weight: Set(weight),
            attrs: Set(attrs.clone()),
            dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(now),
        };

        GraphEdges::insert(edge_model)
            .exec_without_returning(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_edges::Entity::insert", e))?;

        // Create edit record for the new edge
        let edge_data = serde_json::json!({
            "id": id,
            "source": source,
            "target": target,
            "label": label,
            "layer": layer,
            "weight": weight,
            "attrs": attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "edge".to_string(),
                id.clone(),
                "create".to_string(),
                None,
                None,
                Some(edge_data),
                None,
                true,
            )
            .await;

        // Fetch the inserted edge to return
        use crate::database::entities::graph_edges::Column as EdgeColumn;
        use sea_orm::{ColumnTrait, QueryFilter};

        let edge = GraphEdges::find()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .filter(EdgeColumn::Id.eq(&id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_edges::Entity::find", e))?
            .ok_or_else(|| StructuredError::not_found("Graph edge", &id))?;

        Ok(crate::graphql::types::graph_edge::GraphEdge::from(edge))
    }

    /// Delete an edge from a graph
    async fn delete_graph_edge(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        edge_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let edit_service = GraphEditService::new(context.db.clone());

        use crate::database::entities::graph_edges::{Column as EdgeColumn, Entity as GraphEdges};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        // Fetch current edge to get old values for edit record
        let old_edge = GraphEdges::find()
            .filter(EdgeColumn::GraphId.eq(graph_id))
            .filter(EdgeColumn::Id.eq(&edge_id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_edges::Entity::find", e))?;

        if let Some(old_edge) = old_edge {
            // Create edit record for the deletion
            let edge_data = serde_json::json!({
                "id": old_edge.id,
                "source": old_edge.source,
                "target": old_edge.target,
                "label": old_edge.label,
                "layer": old_edge.layer,
                "weight": old_edge.weight,
                "attrs": old_edge.attrs,
            });

            let _ = edit_service
                .create_edit(
                    graph_id,
                    "edge".to_string(),
                    edge_id.clone(),
                    "delete".to_string(),
                    None,
                    Some(edge_data),
                    None,
                    None,
                    true,
                )
                .await;

            // Delete the edge
            GraphEdges::delete_many()
                .filter(EdgeColumn::GraphId.eq(graph_id))
                .filter(EdgeColumn::Id.eq(&edge_id))
                .exec(&context.db)
                .await
                .map_err(|e| StructuredError::database("graph_edges::Entity::delete_many", e))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a node from a graph
    async fn delete_graph_node(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        node_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());
        let edit_service = GraphEditService::new(context.db.clone());

        // Fetch current node to get old values for edit record
        let old_node = graph_service
            .delete_graph_node(graph_id, node_id.clone())
            .await
            .map_err(|e| StructuredError::service("GraphService::delete_graph_node", e))?;

        // Create edit record for the deletion
        let node_data = serde_json::json!({
            "id": old_node.id,
            "label": old_node.label,
            "layer": old_node.layer,
            "is_partition": old_node.is_partition,
            "belongs_to": old_node.belongs_to,
            "weight": old_node.weight,
            "attrs": old_node.attrs,
        });

        let _ = edit_service
            .create_edit(
                graph_id,
                "node".to_string(),
                node_id,
                "delete".to_string(),
                None,
                Some(node_data),
                None,
                None,
                true,
            )
            .await;

        Ok(true)
    }

    /// Bulk update graph nodes and layers in a single transaction
    async fn bulk_update_graph_data(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        nodes: Option<Vec<crate::graphql::types::graph_node::GraphNodeUpdateInput>>,
        layers: Option<Vec<crate::graphql::types::layer::LayerUpdateInput>>,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        let node_requests = nodes
            .unwrap_or_default()
            .into_iter()
            .map(|node_update| GraphNodeUpdateRequest {
                node_id: node_update.node_id,
                label: node_update.label,
                layer: node_update.layer,
                attrs: node_update.attrs,
                belongs_to: None,
            })
            .collect();

        let layer_requests = layers
            .unwrap_or_default()
            .into_iter()
            .map(|layer_update| GraphLayerUpdateRequest {
                id: layer_update.id,
                name: layer_update.name,
                properties: layer_update.properties,
                alias: layer_update.alias,
            })
            .collect();

        context
            .app
            .bulk_update_graph_data(graph_id, node_requests, layer_requests)
            .await
            .map_err(|e| StructuredError::service("AppContext::bulk_update_graph_data", e))?;

        Ok(true)
    }
}
