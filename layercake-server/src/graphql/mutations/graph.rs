use async_graphql::*;

use layercake_core::app_context::{GraphLayerUpdateRequest, GraphNodeUpdateRequest};
use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::graph::{
    CreateGraphInput, CreateLayerInput, Graph, GraphValidationResult, UpdateGraphInput,
};
use layercake_core::services::graph_service::GraphService;
use sea_orm::{ActiveModelTrait, Set};
use serde_json::Value;

#[derive(Default)]
pub struct GraphMutation;

fn merge_and_validate_attributes(
    attrs: Option<crate::graphql::types::scalars::JSON>,
    attributes: Option<crate::graphql::types::scalars::JSON>,
) -> Result<Option<Value>> {
    let candidate = attributes.or(attrs);
    if let Some(value) = candidate {
        validate_attributes(&value).map_err(|message| Error::new(message))?;
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn validate_attributes(value: &Value) -> Result<(), String> {
    let map = value.as_object().ok_or_else(|| {
        "attributes must be a JSON object with string keys and string/integer values".to_string()
    })?;

    for (key, val) in map {
        if key.trim().is_empty() {
            return Err("attribute keys must be non-empty strings".to_string());
        }
        if val.is_string() {
            continue;
        }
        if let Some(n) = val.as_i64() {
            // ensure value is an integer (reject floats)
            if val.as_f64().map(|f| f.fract() == 0.0).unwrap_or(true)
                && n >= i64::MIN
                && n <= i64::MAX
            {
                continue;
            }
        }
        return Err(format!(
            "attribute '{}' must be a string or integer value",
            key
        ));
    }

    Ok(())
}

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
    async fn validate_graph(&self, ctx: &Context<'_>, id: i32) -> Result<GraphValidationResult> {
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

        use layercake_core::database::entities::graph_layers;

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
        #[graphql(name = "attributes")] attributes_arg: Option<
            crate::graphql::types::scalars::JSON,
        >,
        belongs_to: Option<String>,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;
        let attributes = merge_and_validate_attributes(attrs, attributes_arg)?;

        let node = context
            .app
            .update_graph_node(graph_id, node_id, label, layer, attributes, belongs_to)
            .await
            .map_err(|e| StructuredError::service("AppContext::update_graph_node", e))?;

        Ok(crate::graphql::types::graph_node::GraphNode::from(node))
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

        Ok(crate::graphql::types::layer::Layer::from(layer))
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
        #[graphql(name = "attributes")] attributes_arg: Option<
            crate::graphql::types::scalars::JSON,
        >,
    ) -> Result<crate::graphql::types::graph_node::GraphNode> {
        let context = ctx.data::<GraphQLContext>()?;
        let graph_service = GraphService::new(context.db.clone());

        let attributes = merge_and_validate_attributes(attrs, attributes_arg)?;

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
                attributes.clone(),
            )
            .await
            .map_err(|e| StructuredError::service("GraphService::add_graph_node", e))?;

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
        #[graphql(name = "attributes")] attributes_arg: Option<
            crate::graphql::types::scalars::JSON,
        >,
    ) -> Result<crate::graphql::types::graph_edge::GraphEdge> {
        let context = ctx.data::<GraphQLContext>()?;
        use layercake_core::database::entities::graph_data_edges::ActiveModel as GraphEdgeActiveModel;
        use sea_orm::{ActiveModelTrait, ActiveValue::Set};

        let attributes = merge_and_validate_attributes(attrs, attributes_arg)?;

        // Create the new edge
        let now = chrono::Utc::now();
        let edge_model = GraphEdgeActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            graph_data_id: Set(graph_id),
            external_id: Set(id.clone()),
            source: Set(source.clone()),
            target: Set(target.clone()),
            label: Set(label.clone()),
            layer: Set(layer.clone()),
            weight: Set(weight),
            attributes: Set(attributes.clone()),
            source_dataset_id: Set(None),
            comment: Set(None),
            created_at: Set(now),
        };

        let inserted = edge_model
            .insert(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data_edges::Entity::insert", e))?;

        Ok(crate::graphql::types::graph_edge::GraphEdge::from(inserted))
    }

    /// Delete an edge from a graph
    async fn delete_graph_edge(
        &self,
        ctx: &Context<'_>,
        graph_id: i32,
        edge_id: String,
    ) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;

        use layercake_core::database::entities::graph_data_edges::{
            Column as EdgeColumn, Entity as GraphEdges,
        };
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        // Fetch current edge to get old values for edit record
        let old_edge = GraphEdges::find()
            .filter(EdgeColumn::GraphDataId.eq(graph_id))
            .filter(EdgeColumn::ExternalId.eq(&edge_id))
            .one(&context.db)
            .await
            .map_err(|e| StructuredError::database("graph_data_edges::Entity::find", e))?;

        if let Some(_old_edge) = old_edge {
            // Delete the edge
            GraphEdges::delete_many()
                .filter(EdgeColumn::GraphDataId.eq(graph_id))
                .filter(EdgeColumn::ExternalId.eq(&edge_id))
                .exec(&context.db)
                .await
                .map_err(|e| {
                    StructuredError::database("graph_data_edges::Entity::delete_many", e)
                })?;

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

        // Fetch current node to get old values for edit record
        let _old_node = graph_service
            .delete_graph_node(graph_id, node_id.clone())
            .await
            .map_err(|e| StructuredError::service("GraphService::delete_graph_node", e))?;

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
            .map(|node_update| {
                let attributes =
                    merge_and_validate_attributes(node_update.attrs, node_update.attributes)?;

                Ok(GraphNodeUpdateRequest {
                    node_id: node_update.node_id,
                    label: node_update.label,
                    layer: node_update.layer,
                    attributes,
                    belongs_to: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;

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
