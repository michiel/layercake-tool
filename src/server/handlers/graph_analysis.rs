use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use chrono;

use crate::services::{
    GraphAnalysisService, GraphMetrics, NodeMetrics, PathResult,
    ConnectivityResult, CommunityDetectionResult, LayerAnalysis,
};
use crate::server::app::AppState;

#[derive(Debug, Deserialize)]
pub struct PathQuery {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
pub struct AnalysisOptions {
    pub include_centrality: Option<bool>,
    pub include_clustering: Option<bool>,
    pub algorithm: Option<String>,
}

/// Get comprehensive graph metrics
pub async fn get_graph_metrics(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Getting graph metrics for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service.analyze_graph(project_id).await {
        Ok(metrics) => Ok(Json(metrics)),
        Err(e) => {
            error!("Failed to analyze graph: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get detailed node metrics including centrality measures
pub async fn get_node_metrics(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Query(options): Query<AnalysisOptions>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Getting node metrics for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service.analyze_nodes(project_id).await {
        Ok(mut metrics) => {
            // Filter metrics based on options
            if options.include_centrality == Some(false) {
                for metric in &mut metrics {
                    metric.betweenness_centrality = 0.0;
                    metric.closeness_centrality = 0.0;
                    metric.eigenvector_centrality = 0.0;
                    metric.pagerank = 0.0;
                }
            }
            
            if options.include_clustering == Some(false) {
                for metric in &mut metrics {
                    metric.clustering_coefficient = 0.0;
                }
            }

            Ok(Json(metrics))
        }
        Err(e) => {
            error!("Failed to analyze nodes: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Find shortest path between two nodes
pub async fn find_shortest_path(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Query(query): Query<PathQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    info!(
        "Finding shortest path from {} to {} in project {}",
        query.source, query.target, project_id
    );

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service
        .find_shortest_path(project_id, query.source, query.target)
        .await
    {
        Ok(path_result) => Ok(Json(path_result)),
        Err(e) => {
            error!("Failed to find shortest path: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Analyze graph connectivity
pub async fn analyze_connectivity(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Analyzing connectivity for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service.analyze_connectivity(project_id).await {
        Ok(connectivity) => Ok(Json(connectivity)),
        Err(e) => {
            error!("Failed to analyze connectivity: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Detect communities in the graph
pub async fn detect_communities(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
    Query(options): Query<AnalysisOptions>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Detecting communities for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service.detect_communities(project_id).await {
        Ok(communities) => Ok(Json(communities)),
        Err(e) => {
            error!("Failed to detect communities: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Analyze individual layers
pub async fn analyze_layers(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Analyzing layers for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    match analysis_service.analyze_layers(project_id).await {
        Ok(layer_analyses) => Ok(Json(layer_analyses)),
        Err(e) => {
            error!("Failed to analyze layers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get comprehensive analysis report
pub async fn get_analysis_report(
    State(state): State<AppState>,
    Path(project_id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    info!("Generating analysis report for project {}", project_id);

    let analysis_service = GraphAnalysisService::new(state.db.clone());

    // Run all analyses in parallel
    let (graph_metrics, node_metrics, connectivity, communities, layer_analyses) = tokio::try_join!(
        analysis_service.analyze_graph(project_id),
        analysis_service.analyze_nodes(project_id),
        analysis_service.analyze_connectivity(project_id),
        analysis_service.detect_communities(project_id),
        analysis_service.analyze_layers(project_id),
    )
    .map_err(|e| {
        error!("Failed to generate analysis report: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let report = AnalysisReport {
        project_id,
        graph_metrics,
        node_metrics,
        connectivity,
        communities,
        layer_analyses,
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(report))
}

#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    pub project_id: i32,
    pub graph_metrics: GraphMetrics,
    pub node_metrics: Vec<NodeMetrics>,
    pub connectivity: ConnectivityResult,
    pub communities: CommunityDetectionResult,
    pub layer_analyses: Vec<LayerAnalysis>,
    pub generated_at: String,
}
