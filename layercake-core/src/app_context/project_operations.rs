use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};

use super::{AppContext, ProjectSummary, ProjectUpdate};
use crate::auth::Actor;
use crate::database::entities::projects;
use crate::errors::{CoreError, CoreResult};

impl AppContext {
    // ----- Project helpers -------------------------------------------------
    pub async fn list_projects(&self) -> CoreResult<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to list projects: {}", e)))?;

        Ok(projects.into_iter().map(ProjectSummary::from).collect())
    }

    pub async fn list_projects_filtered(
        &self,
        tags: Option<Vec<String>>,
    ) -> CoreResult<Vec<ProjectSummary>> {
        let projects = projects::Entity::find()
            .order_by_desc(projects::Column::UpdatedAt)
            .all(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to list projects: {}", e)))?;

        // If tags filter is provided, filter projects by tags
        let filtered_projects = if let Some(filter_tags) = tags {
            if filter_tags.is_empty() {
                projects
            } else {
                projects
                    .into_iter()
                    .filter(|project| {
                        let project_tags: Vec<String> =
                            serde_json::from_str(&project.tags).unwrap_or_default();
                        // Check if any filter tag matches any project tag
                        filter_tags
                            .iter()
                            .any(|filter_tag| project_tags.contains(filter_tag))
                    })
                    .collect()
            }
        } else {
            projects
        };

        Ok(filtered_projects
            .into_iter()
            .map(ProjectSummary::from)
            .collect())
    }

    pub async fn get_project(&self, id: i32) -> CoreResult<Option<ProjectSummary>> {
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load project {}: {}", id, e)))?;

        Ok(project.map(ProjectSummary::from))
    }

    /// Export a full project (datasets, plan DAG, stories, sequences, layers) as
    /// a single JSON document — for snapshotting a session or reproducing a bug.
    /// Read access is authorized on the project.
    pub async fn export_project_json(
        &self,
        actor: &Actor,
        project_id: i32,
    ) -> CoreResult<serde_json::Value> {
        use crate::database::entities::{
            data_sets, plan_dag_edges, plan_dag_nodes, plans, project_layers, sequences, stories,
        };
        use sea_orm::{ColumnTrait, QueryFilter};

        self.authorize_project_read(actor, project_id).await?;

        fn load_err(what: &'static str) -> impl Fn(sea_orm::DbErr) -> CoreError {
            move |e| CoreError::internal(format!("export: {}: {}", what, e))
        }

        let project = projects::Entity::find_by_id(project_id)
            .one(&self.db)
            .await
            .map_err(load_err("project"))?
            .ok_or_else(|| CoreError::not_found("Project", project_id.to_string()))?;

        let datasets = data_sets::Entity::find()
            .filter(data_sets::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(load_err("datasets"))?;

        let plan_models = plans::Entity::find()
            .filter(plans::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(load_err("plans"))?;
        let plan_ids: Vec<i32> = plan_models.iter().map(|p| p.id).collect();

        let (dag_nodes, dag_edges) = if plan_ids.is_empty() {
            (Vec::new(), Vec::new())
        } else {
            let nodes = plan_dag_nodes::Entity::find()
                .filter(plan_dag_nodes::Column::PlanId.is_in(plan_ids.clone()))
                .all(&self.db)
                .await
                .map_err(load_err("plan_dag_nodes"))?;
            let edges = plan_dag_edges::Entity::find()
                .filter(plan_dag_edges::Column::PlanId.is_in(plan_ids.clone()))
                .all(&self.db)
                .await
                .map_err(load_err("plan_dag_edges"))?;
            (nodes, edges)
        };

        let story_models = stories::Entity::find()
            .filter(stories::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(load_err("stories"))?;
        let story_ids: Vec<i32> = story_models.iter().map(|s| s.id).collect();
        let sequence_models = if story_ids.is_empty() {
            Vec::new()
        } else {
            sequences::Entity::find()
                .filter(sequences::Column::StoryId.is_in(story_ids))
                .all(&self.db)
                .await
                .map_err(load_err("sequences"))?
        };

        let layers = project_layers::Entity::find()
            .filter(project_layers::Column::ProjectId.eq(project_id))
            .all(&self.db)
            .await
            .map_err(load_err("project_layers"))?;

        Ok(serde_json::json!({
            "formatVersion": 1,
            "project": project,
            "datasets": datasets,
            "plans": plan_models,
            "planDagNodes": dag_nodes,
            "planDagEdges": dag_edges,
            "stories": story_models,
            "sequences": sequence_models,
            "projectLayers": layers,
        }))
    }

    pub async fn create_project(
        &self,
        actor: &Actor,
        name: String,
        description: Option<String>,
        tags: Option<Vec<String>>,
    ) -> CoreResult<ProjectSummary> {
        self.authorize(actor, "write:project")?;
        let now = Utc::now();
        let tags_json =
            serde_json::to_string(&tags.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string());
        let project = projects::ActiveModel {
            name: Set(name),
            description: Set(description),
            tags: Set(tags_json),
            import_export_path: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let project = project
            .insert(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to create project: {}", e)))?;

        // Create default "Main plan" for new project if no plans exist
        let _ = self.plan_service.ensure_default_plan(project.id).await;

        Ok(ProjectSummary::from(project))
    }

    pub async fn update_project(
        &self,
        actor: &Actor,
        id: i32,
        update: ProjectUpdate,
    ) -> CoreResult<ProjectSummary> {
        self.authorize_project_write(actor, id).await?;
        let project = projects::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to load project {}: {}", id, e)))?
            .ok_or_else(|| CoreError::not_found("Project", id.to_string()))?;

        let mut active: projects::ActiveModel = project.into();
        if let Some(name) = update.name {
            active.name = Set(name);
        }
        if update.description_is_set {
            active.description = Set(update.description);
        }
        if let Some(tags) = update.tags {
            let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
            active.tags = Set(tags_json);
        }
        if let Some(path) = update.import_export_path {
            active.import_export_path = Set(path);
        }
        active.updated_at = Set(Utc::now());

        let project = active
            .update(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to update project {}: {}", id, e)))?;

        Ok(ProjectSummary::from(project))
    }

    pub async fn delete_project(&self, actor: &Actor, id: i32) -> CoreResult<()> {
        self.authorize_project_admin(actor, id).await?;
        let result = projects::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| CoreError::internal(format!("Failed to delete project {}: {}", id, e)))?;

        if result.rows_affected == 0 {
            return Err(CoreError::not_found("Project", id.to_string()));
        }

        Ok(())
    }
}
