pub mod data_sources;
pub mod plan_dag_edges;
pub mod plan_dag_nodes;
pub mod plans;
pub mod project_collaborators;
pub mod projects;
pub mod user_sessions;
pub mod users;
// REMOVED: user_presence - ephemeral data now handled via WebSocket only
// REMOVED: nodes, edges - dead code, no longer used

// Pipeline entities for DAG execution
pub mod datasource_rows;
pub mod datasources;
pub mod execution_state;
pub mod graph_edges;
pub mod graph_edits;
pub mod graph_nodes;
pub mod graphs;
pub mod layers;

// Re-export specific entities to avoid naming conflicts
pub use execution_state::ExecutionState;
