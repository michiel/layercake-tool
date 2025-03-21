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

