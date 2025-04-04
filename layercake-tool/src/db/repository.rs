use super::entities::{graph, plan, project};
use crate::graph::Graph;
use crate::plan::Plan;
use anyhow::{anyhow, Result};
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};

pub struct ProjectRepository {
    db: DatabaseConnection,
}

impl ProjectRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // Create a new project with associated plan and graph
    pub async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        plan_data: &Plan,
        graph_data: &Graph,
    ) -> Result<i32> {
        // Start transaction
        let txn = self.db.begin().await?;

        // Create project
        let project = project::ActiveModel {
            id: NotSet,
            name: Set(name.to_string()),
            description: Set(description.map(|s| s.to_string())),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let project_result = project::Entity::insert(project).exec(&txn).await?;

        let project_id = project_result.last_insert_id;

        // Create plan
        let plan_model = plan::Model::from_plan(project_id, plan_data)?;
        let plan = plan::ActiveModel {
            id: NotSet,
            project_id: Set(project_id),
            plan_data: Set(plan_model.plan_data),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        plan::Entity::insert(plan).exec(&txn).await?;

        // Create graph
        let graph_model = graph::Model::from_graph(project_id, graph_data)?;
        let graph = graph::ActiveModel {
            id: NotSet,
            project_id: Set(project_id),
            graph_data: Set(graph_model.graph_data),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        graph::Entity::insert(graph).exec(&txn).await?;

        // Commit transaction
        txn.commit().await?;

        Ok(project_id)
    }

    // Get project by ID with associated plan and graph
    pub async fn get_project(&self, id: i32) -> Result<(project::Model, Plan, Graph)> {
        // Get project
        let project = project::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Project not found"))?;

        // Get plan
        let plan_model = plan::Entity::find()
            .filter(plan::Column::ProjectId.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Plan not found"))?;

        let plan = plan_model.to_plan()?;

        // Get graph
        let graph_model = graph::Entity::find()
            .filter(graph::Column::ProjectId.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Graph not found"))?;

        let graph = graph_model.to_graph()?;

        Ok((project, plan, graph))
    }

    // Update project plan
    pub async fn update_plan(&self, project_id: i32, plan_data: &Plan) -> Result<()> {
        // Find existing plan
        let plan_model = plan::Entity::find()
            .filter(plan::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Plan not found"))?;

        // Update plan
        let new_plan_data = serde_json::to_string(plan_data)?;
        let mut plan: plan::ActiveModel = plan_model.into();
        plan.plan_data = Set(new_plan_data);

        plan.update(&self.db).await?;

        Ok(())
    }

    // Update project graph
    pub async fn update_graph(&self, project_id: i32, graph_data: &Graph) -> Result<()> {
        // Find existing graph
        let graph_model = graph::Entity::find()
            .filter(graph::Column::ProjectId.eq(project_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow!("Graph not found"))?;

        // Update graph
        let new_graph_data = serde_json::to_string(graph_data)?;
        let mut graph: graph::ActiveModel = graph_model.into();
        graph.graph_data = Set(new_graph_data);

        graph.update(&self.db).await?;

        Ok(())
    }

    // List all projects
    pub async fn list_projects(&self) -> Result<Vec<project::Model>> {
        let projects = project::Entity::find().all(&self.db).await?;

        Ok(projects)
    }

    // Delete project
    pub async fn delete_project(&self, id: i32) -> Result<()> {
        // Start transaction
        let txn = self.db.begin().await?;

        // Delete plan
        plan::Entity::delete_many()
            .filter(plan::Column::ProjectId.eq(id))
            .exec(&txn)
            .await?;

        // Delete graph
        graph::Entity::delete_many()
            .filter(graph::Column::ProjectId.eq(id))
            .exec(&txn)
            .await?;

        // Delete project
        project::Entity::delete_by_id(id).exec(&txn).await?;

        // Commit transaction
        txn.commit().await?;

        Ok(())
    }
}
