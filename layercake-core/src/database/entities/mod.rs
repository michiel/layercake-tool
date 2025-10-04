pub mod projects;
pub mod plans;
pub mod nodes;
pub mod edges;
pub mod layers;
pub mod plan_dag_nodes;
pub mod plan_dag_edges;
pub mod data_sources;
pub mod users;
pub mod user_sessions;
pub mod project_collaborators;
// REMOVED: user_presence - ephemeral data now handled via WebSocket only

// Pipeline entities for DAG execution
pub mod datasources;
pub mod datasource_rows;
pub mod graphs;
pub mod graph_nodes;
pub mod graph_edges;

// Re-export specific entities to avoid naming conflicts
