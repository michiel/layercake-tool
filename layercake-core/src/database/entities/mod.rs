pub mod chat_credentials;
pub mod chat_messages;
pub mod chat_sessions;
pub mod common_types;
pub mod data_sets;
pub mod library_items;
pub mod plan_dag_edges;
pub mod plan_dag_nodes;
pub mod plans;
pub mod project_collaborators;
pub mod project_layers;
pub mod projects;
pub mod system_settings;
pub mod user_sessions;
pub mod users;
// REMOVED: user_presence - ephemeral data now handled via WebSocket only
// REMOVED: nodes, edges - dead code, no longer used

// Pipeline entities for DAG execution
pub mod dataset_nodes;
pub mod dataset_rows;
pub mod execution_state;
pub mod graph_edges;
pub mod graph_edits;
pub mod graph_layers;
pub mod graph_nodes;
pub mod graphs;
pub mod layer_aliases;

// Re-export specific entities to avoid naming conflicts
pub use execution_state::ExecutionState;

// Backwards compatibility: dataset_nodes was previously called "datasets"
pub use dataset_nodes as datasets;
