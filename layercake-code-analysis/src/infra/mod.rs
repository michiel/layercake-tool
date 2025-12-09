mod api_gateway;
mod correlation;
mod enhanced_correlation;
mod event_sources;
mod graph;
mod model;
mod scanner;

pub use api_gateway::{enrich_with_api_routes, ApiRoute};
pub use correlation::{correlate_code_infra, CorrelationReport};
pub use enhanced_correlation::{
    enhanced_correlate, DataFlowCorrelation, EnhancedCorrelationReport, EnvVarCorrelation,
    ExternalCallCorrelation,
};
pub use event_sources::{enrich_with_event_sources, EventSourceMapping};
pub use graph::{slugify_id, InfrastructureGraph};
pub use model::{CorrelationMatch, EdgeType, GraphEdge, ResourceNode, ResourceType};
pub use scanner::{analyze_infra, InfraScanResult};
