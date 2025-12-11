pub mod chat_credentials;
pub mod chat_messages;
pub mod chat_sessions;
pub mod code_analysis_profiles;
pub mod common_types;
pub mod data_sets;
pub mod library_items;
pub mod plan_dag_edges;
pub mod plan_dag_nodes;
pub mod plans;
pub mod project_collaborators;
pub mod project_layers;
pub mod projections;
pub mod projects;
pub mod sequence_contexts;
pub mod sequences;
pub mod stories;
pub mod system_settings;
pub mod user_sessions;
pub mod users;
// REMOVED: user_presence - ephemeral data now handled via WebSocket only
// REMOVED: nodes, edges - dead code, no longer used

// Pipeline entities for DAG execution
pub mod dataset_graph_edges;
pub mod dataset_graph_layers;
pub mod dataset_graph_nodes;
pub mod dataset_nodes;
pub mod dataset_rows;
pub mod execution_state;
pub mod graph_edges;
pub mod graph_edits;
pub mod graph_layers;
pub mod graph_nodes;
pub mod graphs;
pub mod layer_aliases;

// Unified graph data model (Phase 1 of refactoring)
pub mod graph_data;
pub mod graph_data_edges;
pub mod graph_data_nodes;

// Re-export specific entities to avoid naming conflicts
pub use execution_state::ExecutionState;

// Backwards compatibility: dataset_nodes was previously called "datasets"
pub use dataset_nodes as datasets;
