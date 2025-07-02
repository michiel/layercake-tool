use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use anyhow::Result;

#[cfg(feature = "server")]
use utoipa::OpenApi;
#[cfg(feature = "server")]
use utoipa_swagger_ui::SwaggerUi;

use super::handlers::{health, projects, plans, graph_data};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[cfg(feature = "server")]
#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        projects::list_projects,
        projects::create_project,
        projects::get_project,
        projects::update_project,
        projects::delete_project,
    ),
    components(
        schemas(
            crate::database::entities::projects::Model,
            crate::server::handlers::projects::CreateProjectRequest,
            crate::server::handlers::projects::UpdateProjectRequest,
        )
    ),
    info(
        title = "Layercake API",
        version = "1.0.0",
        description = "Graph visualization and transformation tool API"
    ),
    servers(
        (url = "/", description = "Local server")
    )
)]
struct ApiDoc;

pub async fn create_app(db: DatabaseConnection, cors_origin: Option<&str>) -> Result<Router> {
    let state = AppState { db };

    let cors = match cors_origin {
        Some(origin) => CorsLayer::new()
            .allow_origin(origin.parse::<axum::http::HeaderValue>().unwrap())
            .allow_methods(Any)
            .allow_headers(Any),
        None => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    };

    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health::health_check))
        
        // API v1 routes
        .nest("/api/v1", api_v1_routes())
        
        // OpenAPI documentation
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        
        // Add middleware
        .layer(ServiceBuilder::new().layer(cors))
        .with_state(state);

    Ok(app)
}

fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Project routes
        .route("/projects", get(projects::list_projects))
        .route("/projects", post(projects::create_project))
        .route("/projects/:id", get(projects::get_project))
        .route("/projects/:id", put(projects::update_project))
        .route("/projects/:id", delete(projects::delete_project))
        
        // Plan routes
        .route("/projects/:id/plans", get(plans::list_plans))
        .route("/projects/:id/plans", post(plans::create_plan))
        .route("/projects/:id/plans/:plan_id", get(plans::get_plan))
        .route("/projects/:id/plans/:plan_id", put(plans::update_plan))
        .route("/projects/:id/plans/:plan_id", delete(plans::delete_plan))
        .route("/projects/:id/plans/:plan_id/execute", post(plans::execute_plan))
        
        // Graph data routes
        .route("/projects/:id/nodes", get(graph_data::list_nodes))
        .route("/projects/:id/nodes", post(graph_data::create_nodes))
        .route("/projects/:id/nodes", delete(graph_data::delete_nodes))
        
        .route("/projects/:id/edges", get(graph_data::list_edges))
        .route("/projects/:id/edges", post(graph_data::create_edges))
        .route("/projects/:id/edges", delete(graph_data::delete_edges))
        
        .route("/projects/:id/layers", get(graph_data::list_layers))
        .route("/projects/:id/layers", post(graph_data::create_layers))
        .route("/projects/:id/layers", delete(graph_data::delete_layers))
        
        .route("/projects/:id/import/csv", post(graph_data::import_csv))
        .route("/projects/:id/export/:format", get(graph_data::export_graph))
}