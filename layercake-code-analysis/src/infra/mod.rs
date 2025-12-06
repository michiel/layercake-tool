mod correlation;
mod graph;
mod model;
mod scanner;

pub use correlation::{correlate_code_infra, CorrelationReport};
pub use graph::{slugify_id, InfrastructureGraph};
pub use model::{CorrelationMatch, EdgeType, GraphEdge, ResourceNode, ResourceType};
pub use scanner::{analyze_infra, InfraScanResult};
