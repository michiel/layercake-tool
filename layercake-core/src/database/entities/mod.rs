pub mod projects;
pub mod plans;
pub mod nodes;
pub mod edges;
pub mod layers;
pub mod plan_dag_nodes;
pub mod plan_dag_edges;

// Re-export specific entities to avoid naming conflicts
pub use projects::{Entity as ProjectEntity, Model as ProjectModel, ActiveModel as ProjectActiveModel};
pub use plans::{Entity as PlanEntity, Model as PlanModel, ActiveModel as PlanActiveModel};
pub use nodes::{Entity as NodeEntity, Model as NodeModel, ActiveModel as NodeActiveModel};
pub use edges::{Entity as EdgeEntity, Model as EdgeModel, ActiveModel as EdgeActiveModel};
pub use layers::{Entity as LayerEntity, Model as LayerModel, ActiveModel as LayerActiveModel};
pub use plan_dag_nodes::{Entity as PlanDagNodeEntity, Model as PlanDagNodeModel, ActiveModel as PlanDagNodeActiveModel};
pub use plan_dag_edges::{Entity as PlanDagEdgeEntity, Model as PlanDagEdgeModel, ActiveModel as PlanDagEdgeActiveModel};