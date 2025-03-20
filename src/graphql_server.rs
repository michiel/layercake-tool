use crate::graph::{Graph, Node};
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, InputObject, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, routing::get, Router, Server};
use std::path::Path;
use std::sync::{Arc, RwLock};

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use axum::response::{Html, IntoResponse};

// Handler for the GraphQL playground
async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

// Make your graph types compatible with GraphQL
#[derive(SimpleObject, Clone)]
struct GraphQLNode {
    id: String,
    label: String,
    layer: String,
    is_partition: bool,
    belongs_to: Option<String>,
    weight: i32,
    comment: Option<String>,
}

#[derive(SimpleObject, Clone)]
struct GraphQLEdge {
    id: String,
    source: String,
    target: String,
    label: String,
    layer: String,
    weight: i32,
    comment: Option<String>,
}

#[derive(SimpleObject, Clone)]
struct GraphQLLayer {
    id: String,
    label: String,
    background_color: String,
    text_color: String,
    border_color: String,
}

// State shared across the application
struct AppState {
    graph: Arc<RwLock<Graph>>,
    persist_path: Option<String>,
}

// GraphQL Query Root
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn nodes(&self, ctx: &Context<'_>) -> Vec<GraphQLNode> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let graph = state.graph.read().unwrap();

        graph
            .nodes
            .iter()
            .map(|n| GraphQLNode {
                id: n.id.clone(),
                label: n.label.clone(),
                layer: n.layer.clone(),
                is_partition: n.is_partition,
                belongs_to: n.belongs_to.clone(),
                weight: n.weight,
                comment: n.comment.clone(),
            })
            .collect()
    }

    async fn edges(&self, ctx: &Context<'_>) -> Vec<GraphQLEdge> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let graph = state.graph.read().unwrap();

        graph
            .edges
            .iter()
            .map(|e| GraphQLEdge {
                id: e.id.clone(),
                source: e.source.clone(),
                target: e.target.clone(),
                label: e.label.clone(),
                layer: e.layer.clone(),
                weight: e.weight,
                comment: e.comment.clone(),
            })
            .collect()
    }

    async fn layers(&self, ctx: &Context<'_>) -> Vec<GraphQLLayer> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let graph = state.graph.read().unwrap();

        graph
            .layers
            .iter()
            .map(|l| GraphQLLayer {
                id: l.id.clone(),
                label: l.label.clone(),
                background_color: l.background_color.clone(),
                text_color: l.text_color.clone(),
                border_color: l.border_color.clone(),
            })
            .collect()
    }

    // Add more query fields as needed
}

// Define a GraphQL schema
type GraphQLSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

// Handler for GraphQL requests
async fn graphql_handler(
    State(schema): State<GraphQLSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// Persistence functions
fn load_graph(path: &str) -> Option<Graph> {
    if Path::new(path).exists() {
        match std::fs::read_to_string(path) {
            Ok(json) => match serde_json::from_str(&json) {
                Ok(graph) => return Some(graph),
                Err(e) => tracing::error!("Failed to deserialize graph: {}", e),
            },
            Err(e) => tracing::error!("Failed to read graph file: {}", e),
        }
    }
    None
}

fn save_graph(graph: &Graph, path: &str) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(graph)?;
    std::fs::write(path, json)
}

// Main function to start the GraphQL server
pub async fn serve_graph(graph: Graph, port: u16, persist: bool) -> Result<(), anyhow::Error> {
    let persist_path = if persist {
        Some("layercake_graph.json".to_string())
    } else {
        None
    };

    // Load persisted graph if it exists
    let graph = if let Some(path) = &persist_path {
        load_graph(path).unwrap_or(graph)
    } else {
        graph
    };

    // Set up shared state
    let state = Arc::new(AppState {
        graph: Arc::new(RwLock::new(graph)),
        persist_path,
    });

    // Set up GraphQL schema
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state.clone())
        .finish();

    // Set up web server routes
    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        .with_state(schema);

    // Start the server
    tracing::info!(
        "GraphQL server running at http://localhost:{}/graphql",
        port
    );
    let addr = format!("0.0.0.0:{}", port).parse()?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// In src/graphql_server.rs, add these functions:

// Create a MutationRoot to allow modifying the graph
struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn persist_graph(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();

        if let Some(path) = &state.persist_path {
            let graph = state.graph.read().unwrap();
            if let Err(e) = save_graph(&graph, path) {
                tracing::error!("Failed to save graph: {}", e);
                return Ok(false);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Add mutations to modify the graph
    async fn add_node(
        &self,
        ctx: &Context<'_>,
        node: NodeInput,
    ) -> async_graphql::Result<GraphQLNode> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let mut graph = state.graph.write().unwrap();

        let new_node = Node {
            id: node.id,
            label: node.label,
            layer: node.layer,
            is_partition: node.is_partition,
            belongs_to: node.belongs_to,
            weight: node.weight,
            comment: node.comment,
        };

        graph.nodes.push(new_node.clone());

        // Auto-persist if enabled
        if let Some(path) = &state.persist_path {
            if let Err(e) = save_graph(&graph, path) {
                tracing::error!("Failed to save graph: {}", e);
            }
        }

        Ok(GraphQLNode {
            id: new_node.id,
            label: new_node.label,
            layer: new_node.layer,
            is_partition: new_node.is_partition,
            belongs_to: new_node.belongs_to,
            weight: new_node.weight,
            comment: new_node.comment,
        })
    }

    // Implement similar mutations for edges and layers
}

// Add a NodeInput type for mutations
#[derive(InputObject)]
struct NodeInput {
    id: String,
    label: String,
    layer: String,
    is_partition: bool,
    belongs_to: Option<String>,
    weight: i32,
    comment: Option<String>,
}
