use crate::db::{establish_connection, repository::ProjectRepository};
use crate::graph::Graph as DomainGraph;
use crate::plan::Plan;
use async_graphql::{Context, InputObject, Object, Schema, SimpleObject, ID};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State, 
    routing::get,
    Router,
    Server,
    http::{Method, HeaderValue},
};
use tower_http::cors::CorsLayer;
use std::net::TcpListener;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

// GraphQL Types for your entities
struct Project {
    id: ID,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
}

#[Object]
impl Project {
    async fn id(&self) -> &ID {
        &self.id
    }
    
    async fn name(&self) -> &String {
        &self.name
    }
    
    async fn description(&self) -> &Option<String> {
        &self.description
    }
    
    async fn created_at(&self) -> &String {
        &self.created_at
    }
    
    async fn updated_at(&self) -> &String {
        &self.updated_at
    }
    
    async fn plan(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<PlanData>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let project_id = self.id.parse::<i32>()?;
        
        match repo.get_project(project_id).await {
            Ok((_, plan, _)) => {
                // Convert domain Plan to GraphQL PlanData
                let meta = plan.meta.map(|m| PlanMeta {
                    name: m.name,
                });
                
                let import_profiles = plan.import.profiles.iter().map(|p| ImportProfile {
                    filename: p.filename.clone(),
                    filetype: format!("{:?}", p.filetype), // Convert enum to string
                }).collect();
                
                let import = ImportConfig {
                    profiles: import_profiles,
                };
                
                let export_profiles = plan.export.profiles.iter().map(|p| ExportProfileItem {
                    filename: p.filename.clone(),
                    exporter: format!("{:?}", p.exporter), // Convert enum to string
                    graph_config: p.graph_config.map(|gc| ExportProfileGraphConfig {
                        generate_hierarchy: gc.generate_hierarchy,
                        max_partition_depth: gc.max_partition_depth,
                        max_partition_width: gc.max_partition_width,
                        invert_graph: gc.invert_graph,
                        node_label_max_length: gc.node_label_max_length.map(|v| v as i32),
                        node_label_insert_newlines_at: gc.node_label_insert_newlines_at.map(|v| v as i32),
                        edge_label_max_length: gc.edge_label_max_length.map(|v| v as i32),
                        edge_label_insert_newlines_at: gc.edge_label_insert_newlines_at.map(|v| v as i32),
                    }),
                }).collect();
                
                let export = ExportProfile {
                    profiles: export_profiles,
                };
                
                Ok(Some(PlanData {
                    meta,
                    import,
                    export,
                }))
            },
            Err(_) => Ok(None),
        }
    }
    
    async fn graph(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<GraphData>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let project_id = self.id.parse::<i32>()?;
        
        match repo.get_project(project_id).await {
            Ok((project, _, graph_data)) => {
                let nodes = graph_data.nodes.iter().map(|n| Node {
                    id: n.id.clone(),
                    label: n.label.clone(),
                    layer: n.layer.clone(),
                    is_partition: n.is_partition,
                    belongs_to: n.belongs_to.clone(),
                    weight: n.weight,
                    comment: n.comment.clone(),
                }).collect();
                
                let edges = graph_data.edges.iter().map(|e| Edge {
                    id: e.id.clone(),
                    source: e.source.clone(),
                    target: e.target.clone(),
                    label: e.label.clone(),
                    layer: e.layer.clone(),
                    weight: e.weight,
                    comment: e.comment.clone(),
                }).collect();
                
                let layers = graph_data.layers.iter().map(|l| Layer {
                    id: l.id.clone(),
                    label: l.label.clone(),
                    background_color: l.background_color.clone(),
                    text_color: l.text_color.clone(),
                    border_color: l.border_color.clone(),
                }).collect();
                
                Ok(Some(GraphData {
                    id: ID(format!("graph-{}", project.id)),
                    project_id: ID(project.id.to_string()),
                    nodes,
                    edges,
                    layers,
                }))
            },
            Err(_) => Ok(None),
        }
    }
}

struct GraphData {
    id: ID,
    project_id: ID,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    layers: Vec<Layer>,
}

#[Object]
impl GraphData {
    async fn id(&self) -> &ID {
        &self.id
    }
    
    async fn project_id(&self) -> &ID {
        &self.project_id
    }
    
    async fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }
    
    async fn edges(&self) -> &Vec<Edge> {
        &self.edges
    }
    
    async fn layers(&self) -> &Vec<Layer> {
        &self.layers
    }
    
    async fn project(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Project>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let project_id = self.project_id.parse::<i32>()?;
        
        match repo.get_project(project_id).await {
            Ok((project, _, _)) => Ok(Some(Project {
                id: ID(project.id.to_string()),
                name: project.name,
                description: project.description,
                created_at: project.created_at.to_string(),
                updated_at: project.updated_at.to_string(),
            })),
            Err(_) => Ok(None),
        }
    }
}

#[derive(SimpleObject, Clone)]
struct Node {
    id: String,
    label: String,
    layer: String,
    is_partition: bool,
    belongs_to: Option<String>,
    weight: i32,
    comment: Option<String>,
}

#[derive(SimpleObject, Clone)]
struct Edge {
    id: String,
    source: String,
    target: String,
    label: String,
    layer: String,
    weight: i32,
    comment: Option<String>,
}

#[derive(SimpleObject, Clone)]
struct Layer {
    id: String,
    label: String,
    background_color: String,
    text_color: String,
    border_color: String,
}

// GraphQL types for Plan
#[derive(SimpleObject)]
struct PlanMeta {
    name: Option<String>,
}

#[derive(SimpleObject)]
struct ImportProfile {
    filename: String,
    filetype: String,
}

#[derive(SimpleObject)]
struct ImportConfig {
    profiles: Vec<ImportProfile>,
}

#[derive(SimpleObject)]
struct ExportProfileGraphConfig {
    generate_hierarchy: Option<bool>,
    max_partition_depth: Option<i32>,
    max_partition_width: Option<i32>,
    invert_graph: Option<bool>,
    node_label_max_length: Option<i32>,
    node_label_insert_newlines_at: Option<i32>,
    edge_label_max_length: Option<i32>,
    edge_label_insert_newlines_at: Option<i32>,
}

#[derive(SimpleObject)]
struct ExportProfileItem {
    filename: String,
    exporter: String,
    graph_config: Option<ExportProfileGraphConfig>,
}

#[derive(SimpleObject)]
struct ExportProfile {
    profiles: Vec<ExportProfileItem>,
}

struct PlanData {
    meta: Option<PlanMeta>,
    import: ImportConfig,
    export: ExportProfile,
}

#[Object]
impl PlanData {
    async fn meta(&self) -> &Option<PlanMeta> {
        &self.meta
    }
    
    async fn import(&self) -> &ImportConfig {
        &self.import
    }
    
    async fn export(&self) -> &ExportProfile {
        &self.export
    }
}

// Input types for mutations
#[derive(InputObject)]
struct ProjectInput {
    name: String,
    description: Option<String>,
}

#[derive(InputObject)]
struct NodeInput {
    id: Option<String>,      // Optional for create, required for update
    label: String,
    layer: String,
    is_partition: bool,
    belongs_to: Option<String>,
    weight: Option<i32>,
    comment: Option<String>,
}

#[derive(InputObject)]
struct EdgeInput {
    id: Option<String>,      // Optional for create, required for update
    source: String,
    target: String,
    label: String,
    layer: String,
    weight: Option<i32>,
    comment: Option<String>,
}

#[derive(InputObject)]
struct LayerInput {
    id: Option<String>,      // Optional for create, required for update
    label: String,
    background_color: String,
    text_color: String,
    border_color: String,
}

// State shared across the application
struct AppState {
    db: DatabaseConnection,
}

// GraphQL Query Root
struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn projects(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Project>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let projects = repo.list_projects().await?;

        Ok(projects
            .into_iter()
            .map(|p| Project {
                id: ID(p.id.to_string()),
                name: p.name,
                description: p.description,
                created_at: p.created_at.to_string(),
                updated_at: p.updated_at.to_string(),
            })
            .collect())
    }

    async fn project(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<Project>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let project_id = id.parse::<i32>()?;

        match repo.get_project(project_id).await {
            Ok((project, _, _)) => Ok(Some(Project {
                id: ID(project.id.to_string()),
                name: project.name,
                description: project.description,
                created_at: project.created_at.to_string(),
                updated_at: project.updated_at.to_string(),
            })),
            Err(_) => Ok(None),
        }
    }
    
    async fn graph(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> async_graphql::Result<Option<GraphData>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;

        match repo.get_project(pid).await {
            Ok((project, _, graph_data)) => {
                let nodes = graph_data.nodes.iter().map(|n| Node {
                    id: n.id.clone(),
                    label: n.label.clone(),
                    layer: n.layer.clone(),
                    is_partition: n.is_partition,
                    belongs_to: n.belongs_to.clone(),
                    weight: n.weight,
                    comment: n.comment.clone(),
                }).collect();
                
                let edges = graph_data.edges.iter().map(|e| Edge {
                    id: e.id.clone(),
                    source: e.source.clone(),
                    target: e.target.clone(),
                    label: e.label.clone(),
                    layer: e.layer.clone(),
                    weight: e.weight,
                    comment: e.comment.clone(),
                }).collect();
                
                let layers = graph_data.layers.iter().map(|l| Layer {
                    id: l.id.clone(),
                    label: l.label.clone(),
                    background_color: l.background_color.clone(),
                    text_color: l.text_color.clone(),
                    border_color: l.border_color.clone(),
                }).collect();
                
                Ok(Some(GraphData {
                    id: ID(format!("graph-{}", project.id)),
                    project_id: ID(project.id.to_string()),
                    nodes,
                    edges,
                    layers,
                }))
            },
            Err(_) => Ok(None),
        }
    }

    async fn nodes(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> async_graphql::Result<Vec<Node>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;

        let (_, _, graph) = repo.get_project(pid).await?;

        Ok(graph
            .nodes
            .iter()
            .map(|n| Node {
                id: n.id.clone(),
                label: n.label.clone(),
                layer: n.layer.clone(),
                is_partition: n.is_partition,
                belongs_to: n.belongs_to.clone(),
                weight: n.weight,
                comment: n.comment.clone(),
            })
            .collect())
    }

    async fn edges(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> async_graphql::Result<Vec<Edge>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;

        let (_, _, graph) = repo.get_project(pid).await?;

        Ok(graph
            .edges
            .iter()
            .map(|e| Edge {
                id: e.id.clone(),
                source: e.source.clone(),
                target: e.target.clone(),
                label: e.label.clone(),
                layer: e.layer.clone(),
                weight: e.weight,
                comment: e.comment.clone(),
            })
            .collect())
    }

    async fn layers(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> async_graphql::Result<Vec<Layer>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;

        let (_, _, graph) = repo.get_project(pid).await?;

        Ok(graph
            .layers
            .iter()
            .map(|l| Layer {
                id: l.id.clone(),
                label: l.label.clone(),
                background_color: l.background_color.clone(),
                text_color: l.text_color.clone(),
                border_color: l.border_color.clone(),
            })
            .collect())
    }
    
    async fn plan(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> async_graphql::Result<Option<PlanData>> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;

        match repo.get_project(pid).await {
            Ok((_, plan, _)) => {
                // Convert domain Plan to GraphQL PlanData
                let meta = plan.meta.map(|m| PlanMeta {
                    name: m.name,
                });
                
                let import_profiles = plan.import.profiles.iter().map(|p| ImportProfile {
                    filename: p.filename.clone(),
                    filetype: format!("{:?}", p.filetype),
                }).collect();
                
                let import = ImportConfig {
                    profiles: import_profiles,
                };
                
                let export_profiles = plan.export.profiles.iter().map(|p| ExportProfileItem {
                    filename: p.filename.clone(),
                    exporter: format!("{:?}", p.exporter),
                    graph_config: p.graph_config.map(|gc| ExportProfileGraphConfig {
                        generate_hierarchy: gc.generate_hierarchy,
                        max_partition_depth: gc.max_partition_depth,
                        max_partition_width: gc.max_partition_width,
                        invert_graph: gc.invert_graph,
                        node_label_max_length: gc.node_label_max_length.map(|v| v as i32),
                        node_label_insert_newlines_at: gc.node_label_insert_newlines_at.map(|v| v as i32),
                        edge_label_max_length: gc.edge_label_max_length.map(|v| v as i32),
                        edge_label_insert_newlines_at: gc.edge_label_insert_newlines_at.map(|v| v as i32),
                    }),
                }).collect();
                
                let export = ExportProfile {
                    profiles: export_profiles,
                };
                
                Ok(Some(PlanData {
                    meta,
                    import,
                    export,
                }))
            },
            Err(_) => Ok(None),
        }
    }
}

// GraphQL Mutation Root
struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_project(
        &self,
        ctx: &Context<'_>,
        input: ProjectInput,
        plan_file: String,
    ) -> async_graphql::Result<Project> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        // Load plan from file
        let plan_path = std::path::Path::new(&plan_file);
        let plan_content = std::fs::read_to_string(plan_path)?;
        let plan: Plan = serde_yaml::from_str(&plan_content)?;

        // Create graph from plan
        let mut graph_data = crate::graph_utils::create_graph_from_plan(&plan);
        crate::graph_utils::load_data_into_graph(&mut graph_data, &plan, plan_path)?;

        // Create project in database
        let project_id = repo
            .create_project(&input.name, input.description.as_deref(), &plan, &graph_data)
            .await?;
            
        // Get the created project
        let (project, _, _) = repo.get_project(project_id).await?;
        
        Ok(Project {
            id: ID(project.id.to_string()),
            name: project.name,
            description: project.description,
            created_at: project.created_at.to_string(),
            updated_at: project.updated_at.to_string(),
        })
    }

    async fn update_graph(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
        graph_data: String,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let pid = project_id.parse::<i32>()?;
        let graph_data: DomainGraph = serde_json::from_str(&graph_data)?;

        repo.update_graph(pid, &graph_data).await?;

        Ok(true)
    }

    async fn delete_project(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());

        let project_id = id.parse::<i32>()?;

        repo.delete_project(project_id).await?;

        Ok(true)
    }

    // Node CRUD operations
    async fn add_node(&self, ctx: &Context<'_>, project_id: ID, node: NodeInput) -> async_graphql::Result<Node> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Create a new node
        let new_node = crate::graph::Node {
            id: node.id.unwrap_or_else(|| format!("n{}", chrono::Utc::now().timestamp_millis())),
            label: node.label,
            layer: node.layer,
            is_partition: node.is_partition,
            belongs_to: node.belongs_to,
            weight: node.weight.unwrap_or(1),
            comment: node.comment,
        };
        
        // Add the node to the graph
        graph.set_node(new_node.clone());
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Node to GraphQL Node
        Ok(Node {
            id: new_node.id,
            label: new_node.label,
            layer: new_node.layer,
            is_partition: new_node.is_partition,
            belongs_to: new_node.belongs_to,
            weight: new_node.weight,
            comment: new_node.comment,
        })
    }
    
    async fn update_node(&self, ctx: &Context<'_>, project_id: ID, node_id: String, node: NodeInput) -> async_graphql::Result<Node> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Check if the node exists
        if graph.get_node_by_id(&node_id).is_none() {
            return Err(async_graphql::Error::new(format!("Node with id {} not found", node_id)));
        }
        
        // Create updated node
        let updated_node = crate::graph::Node {
            id: node_id,
            label: node.label,
            layer: node.layer,
            is_partition: node.is_partition,
            belongs_to: node.belongs_to,
            weight: node.weight.unwrap_or(1),
            comment: node.comment,
        };
        
        // Update the node
        graph.set_node(updated_node.clone());
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Node to GraphQL Node
        Ok(Node {
            id: updated_node.id,
            label: updated_node.label,
            layer: updated_node.layer,
            is_partition: updated_node.is_partition,
            belongs_to: updated_node.belongs_to,
            weight: updated_node.weight,
            comment: updated_node.comment,
        })
    }
    
    async fn delete_node(&self, ctx: &Context<'_>, project_id: ID, node_id: String) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Check if the node exists
        if graph.get_node_by_id(&node_id).is_none() {
            return Err(async_graphql::Error::new(format!("Node with id {} not found", node_id)));
        }
        
        // Remove the node and related edges
        graph.remove_node(node_id);
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        Ok(true)
    }
    
    // Edge CRUD operations
    async fn add_edge(&self, ctx: &Context<'_>, project_id: ID, edge: EdgeInput) -> async_graphql::Result<Edge> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Verify that source and target nodes exist
        if graph.get_node_by_id(&edge.source).is_none() {
            return Err(async_graphql::Error::new(format!("Source node with id {} not found", edge.source)));
        }
        
        if graph.get_node_by_id(&edge.target).is_none() {
            return Err(async_graphql::Error::new(format!("Target node with id {} not found", edge.target)));
        }
        
        // Create a new edge
        let new_edge = crate::graph::Edge {
            id: edge.id.unwrap_or_else(|| format!("e{}", chrono::Utc::now().timestamp_millis())),
            source: edge.source,
            target: edge.target,
            label: edge.label,
            layer: edge.layer,
            weight: edge.weight.unwrap_or(1),
            comment: edge.comment,
        };
        
        // Add the edge to the graph
        graph.edges.push(new_edge.clone());
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Edge to GraphQL Edge
        Ok(Edge {
            id: new_edge.id,
            source: new_edge.source,
            target: new_edge.target,
            label: new_edge.label,
            layer: new_edge.layer,
            weight: new_edge.weight,
            comment: new_edge.comment,
        })
    }
    
    async fn update_edge(&self, ctx: &Context<'_>, project_id: ID, edge_id: String, edge: EdgeInput) -> async_graphql::Result<Edge> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Find the edge index
        let edge_index = graph.edges.iter().position(|e| e.id == edge_id)
            .ok_or_else(|| async_graphql::Error::new(format!("Edge with id {} not found", edge_id)))?;
        
        // Verify that source and target nodes exist
        if graph.get_node_by_id(&edge.source).is_none() {
            return Err(async_graphql::Error::new(format!("Source node with id {} not found", edge.source)));
        }
        
        if graph.get_node_by_id(&edge.target).is_none() {
            return Err(async_graphql::Error::new(format!("Target node with id {} not found", edge.target)));
        }
        
        // Create updated edge
        let updated_edge = crate::graph::Edge {
            id: edge_id,
            source: edge.source,
            target: edge.target,
            label: edge.label,
            layer: edge.layer,
            weight: edge.weight.unwrap_or(1),
            comment: edge.comment,
        };
        
        // Update the edge
        graph.edges[edge_index] = updated_edge.clone();
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Edge to GraphQL Edge
        Ok(Edge {
            id: updated_edge.id,
            source: updated_edge.source,
            target: updated_edge.target,
            label: updated_edge.label,
            layer: updated_edge.layer,
            weight: updated_edge.weight,
            comment: updated_edge.comment,
        })
    }
    
    async fn delete_edge(&self, ctx: &Context<'_>, project_id: ID, edge_id: String) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Remove the edge
        let initial_count = graph.edges.len();
        graph.edges.retain(|e| e.id != edge_id);
        
        if graph.edges.len() == initial_count {
            return Err(async_graphql::Error::new(format!("Edge with id {} not found", edge_id)));
        }
        
        // Validate the graph integrity
        if let Err(errors) = graph.verify_graph_integrity() {
            return Err(async_graphql::Error::new(format!("Graph integrity validation failed: {:?}", errors)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        Ok(true)
    }
    
    // Layer CRUD operations
    async fn add_layer(&self, ctx: &Context<'_>, project_id: ID, layer: LayerInput) -> async_graphql::Result<Layer> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Create a new layer
        let new_layer = crate::graph::Layer {
            id: layer.id.unwrap_or_else(|| format!("layer{}", chrono::Utc::now().timestamp_millis())),
            label: layer.label,
            background_color: layer.background_color,
            text_color: layer.text_color,
            border_color: layer.border_color,
        };
        
        // Check if layer with same ID already exists
        if graph.layers.iter().any(|l| l.id == new_layer.id) {
            return Err(async_graphql::Error::new(format!("Layer with id {} already exists", new_layer.id)));
        }
        
        // Add the layer to the graph
        graph.layers.push(new_layer.clone());
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Layer to GraphQL Layer
        Ok(Layer {
            id: new_layer.id,
            label: new_layer.label,
            background_color: new_layer.background_color,
            text_color: new_layer.text_color,
            border_color: new_layer.border_color,
        })
    }
    
    async fn update_layer(&self, ctx: &Context<'_>, project_id: ID, layer_id: String, layer: LayerInput) -> async_graphql::Result<Layer> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Find the layer index
        let layer_index = graph.layers.iter().position(|l| l.id == layer_id)
            .ok_or_else(|| async_graphql::Error::new(format!("Layer with id {} not found", layer_id)))?;
        
        // Create updated layer
        let updated_layer = crate::graph::Layer {
            id: layer_id,
            label: layer.label,
            background_color: layer.background_color,
            text_color: layer.text_color,
            border_color: layer.border_color,
        };
        
        // Update the layer
        graph.layers[layer_index] = updated_layer.clone();
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        // Convert domain Layer to GraphQL Layer
        Ok(Layer {
            id: updated_layer.id,
            label: updated_layer.label,
            background_color: updated_layer.background_color,
            text_color: updated_layer.text_color,
            border_color: updated_layer.border_color,
        })
    }
    
    async fn delete_layer(&self, ctx: &Context<'_>, project_id: ID, layer_id: String) -> async_graphql::Result<bool> {
        let state = ctx.data::<Arc<AppState>>().unwrap();
        let repo = ProjectRepository::new(state.db.clone());
        
        let pid = project_id.parse::<i32>()?;
        
        // Fetch current graph
        let (_, _, mut graph) = repo.get_project(pid).await?;
        
        // Check if any nodes or edges are using this layer
        let used_by_nodes = graph.nodes.iter().any(|n| n.layer == layer_id);
        let used_by_edges = graph.edges.iter().any(|e| e.layer == layer_id);
        
        if used_by_nodes || used_by_edges {
            return Err(async_graphql::Error::new(
                format!("Cannot delete layer {} as it is in use by nodes or edges", layer_id)
            ));
        }
        
        // Remove the layer
        let initial_count = graph.layers.len();
        graph.layers.retain(|l| l.id != layer_id);
        
        if graph.layers.len() == initial_count {
            return Err(async_graphql::Error::new(format!("Layer with id {} not found", layer_id)));
        }
        
        // Save the updated graph
        repo.update_graph(pid, &graph).await?;
        
        Ok(true)
    }
}

// Define a GraphQL schema
type GraphQLSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;

// Handler for GraphQL requests
async fn graphql_handler(
    State(schema): State<GraphQLSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

// Handler for the GraphQL playground
async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

// Main function to start the GraphQL server
pub async fn serve_graph(
    plan: Plan,
    graph: DomainGraph,
    port: u16,
    db_path: &str,
) -> Result<(), anyhow::Error> {
    // Connect to database
    let db = establish_connection(db_path).await?;

    // Create repository
    let repo = ProjectRepository::new(db.clone());

    // Create a default project if none exists
    let projects = repo.list_projects().await?;
    if projects.is_empty() {
        repo.create_project(
            "Default Project",
            Some("Created on server start"),
            &plan,
            &graph,
        )
        .await?;
        tracing::info!("Created default project");
    }

    // Set up shared state
    let state = Arc::new(AppState { db });

    // Set up GraphQL schema
    let schema = Schema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(state.clone())
        .finish();

    // Configure CORS middleware
    let cors = CorsLayer::new()
        // Allow requests from specific origins for credentials
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        // Allow standard methods
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // Allow specific headers instead of Any
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::AUTHORIZATION,
        ])
        // Allow credentials
        .allow_credentials(true);

    // Set up web server routes with CORS
    let app = Router::new()
        .route("/", get(graphql_playground))
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        .layer(cors)
        .with_state(schema);

    // Start the server
    tracing::info!(
        "GraphQL server running at http://localhost:{}/graphql",
        port
    );
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    Server::from_tcp(listener)?.serve(app.into_make_service()).await?;

    Ok(())
}

