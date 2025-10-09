pub mod projects;
pub mod plans;
pub mod plan_dag_nodes;
pub mod plan_dag_edges;
pub mod data_sources;
pub mod users;
pub mod user_sessions;
pub mod project_collaborators;
// REMOVED: user_presence - ephemeral data now handled via WebSocket only
// REMOVED: nodes, edges - dead code, no longer used

// Pipeline entities for DAG execution
pub mod execution_state;
pub mod datasources;
pub mod datasource_rows;
pub mod graphs;
pub mod graph_nodes;
pub mod graph_edges;
pub mod layers;

// Re-export specific entities to avoid naming conflicts
pub use execution_state::ExecutionState;
