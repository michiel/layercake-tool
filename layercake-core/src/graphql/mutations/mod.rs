// Mutation module - refactored into focused submodules
//
// This module provides all GraphQL mutations for the application,
// organized by functional area for better maintainability.

pub mod helpers;
pub mod plan_dag_delta;

// Mutation modules
mod auth;
mod chat;
mod code_analysis;
mod collaboration;
mod data_acquisition;
mod data_set;
mod graph;
mod graph_data;
mod graph_edit;
mod layer;
mod library;
mod mcp;
mod plan;
mod plan_dag;
mod plan_dag_edges;
mod plan_dag_nodes;
mod project;
mod projection;
mod sequence;
mod story;
mod system;

// Re-export public types from helpers

use async_graphql::*;

/// Main mutation root that combines all mutation submodules
#[derive(Default, MergedObject)]
pub struct Mutation(
    pub auth::AuthMutation,
    pub chat::ChatMutation,
    pub collaboration::CollaborationMutation,
    pub code_analysis::CodeAnalysisMutation,
    pub data_set::DataSetMutation,
    pub data_acquisition::DataAcquisitionMutation,
    pub graph::GraphMutation,
    pub graph_data::GraphDataMutation,
    pub graph_edit::GraphEditMutation,
    pub library::LibraryMutation,
    pub mcp::McpMutation,
    pub plan::PlanMutation,
    pub plan_dag::PlanDagMutation,
    pub plan_dag_edges::PlanDagEdgesMutation,
    pub plan_dag_nodes::PlanDagNodesMutation,
    pub layer::LayerMutation,
    pub project::ProjectMutation,
    pub projection::ProjectionMutation,
    pub sequence::SequenceMutation,
    pub story::StoryMutation,
    pub system::SystemMutation,
);
