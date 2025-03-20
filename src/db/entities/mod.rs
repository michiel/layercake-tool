pub mod graph;
pub mod plan;
pub mod project;

// Export with specific names to avoid conflicts
pub use graph::{Entity as GraphEntity, Model as GraphModel, ActiveModel as GraphActiveModel};
pub use plan::{Entity as PlanEntity, Model as PlanModel, ActiveModel as PlanActiveModel};
pub use project::{Entity as ProjectEntity, Model as ProjectModel, ActiveModel as ProjectActiveModel};
