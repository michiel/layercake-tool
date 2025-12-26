pub mod config;
pub mod edge;
pub mod filter;
pub mod metadata;
pub mod node;
pub mod position;
pub mod transforms;

pub use config::*;
pub use edge::*;
pub use filter::*;
pub use metadata::*;
pub use node::*;
pub use position::*;
pub use transforms::*;

use serde::{Deserialize, Serialize};

// Plan DAG Node Types (matching frontend enum values)
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PlanDagNodeType {
    #[serde(rename = "DataSetNode")]
    DataSet,
    #[serde(rename = "GraphNode")]
    Graph,
    #[serde(rename = "TransformNode")]
    Transform,
    #[serde(rename = "FilterNode")]
    Filter,
    #[serde(rename = "MergeNode")]
    Merge,
    #[serde(rename = "GraphArtefactNode", alias = "OutputNode", alias = "Output")]
    GraphArtefact,
    #[serde(rename = "TreeArtefactNode")]
    TreeArtefact,
    #[serde(rename = "ProjectionNode")]
    Projection,
    #[serde(rename = "StoryNode")]
    Story,
    #[serde(rename = "SequenceArtefactNode")]
    SequenceArtefact,
}
