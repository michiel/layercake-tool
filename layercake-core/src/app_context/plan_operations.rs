use anyhow::{anyhow, Result};

use super::{AppContext, PlanSummary};
use crate::database::entities::plans;
use crate::services::plan_service::{PlanCreateRequest, PlanUpdateRequest};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

impl AppContext {
    #[allow(dead_code)]
    pub async fn list_plans(&self, project_id: Option<i32>) -> Result<Vec<PlanSummary>> {
        let plans = if let Some(project_id) = project_id {
            self.plan_service
                .list_plans(project_id)
                .await
                .map_err(|e| anyhow!("Failed to list plans: {}", e))?
        } else {
            plans::Entity::find()
                .order_by_desc(plans::Column::UpdatedAt)
                .all(&self.db)
                .await
                .map_err(|e| anyhow!("Failed to list plans: {}", e))?
        };

        Ok(plans.into_iter().map(PlanSummary::from).collect())
    }

    pub async fn get_plan(&self, id: i32) -> Result<Option<PlanSummary>> {
        let plan = self
            .plan_service
            .get_plan(id)
            .await
            .map_err(|e| anyhow!("Failed to load plan {}: {}", id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn get_plan_for_project(&self, project_id: i32) -> Result<Option<PlanSummary>> {
        let plan = self
            .plan_service
            .get_default_plan(project_id)
            .await
            .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn create_plan(&self, request: PlanCreateRequest) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .create_plan(request)
            .await
            .map_err(|e| anyhow!("Failed to create plan: {}", e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn update_plan(&self, id: i32, update: PlanUpdateRequest) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .update_plan(id, update)
            .await
            .map_err(|e| anyhow!("Failed to update plan {}: {}", id, e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn delete_plan(&self, id: i32) -> Result<()> {
        self.plan_service
            .delete_plan(id)
            .await
            .map_err(|e| anyhow!("Failed to delete plan {}: {}", id, e))
    }

    pub async fn duplicate_plan(&self, id: i32, name: String) -> Result<PlanSummary> {
        let plan = self
            .plan_service
            .duplicate_plan(id, name)
            .await
            .map_err(|e| anyhow!("Failed to duplicate plan {}: {}", id, e))?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn resolve_plan_model(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> Result<plans::Model> {
        if let Some(plan_id) = plan_id {
            let plan = self
                .plan_service
                .get_plan(plan_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan {}: {}", plan_id, e))?
                .ok_or_else(|| anyhow!("Plan {} not found", plan_id))?;

            if plan.project_id != project_id {
                return Err(anyhow!(
                    "Plan {} does not belong to project {}",
                    plan_id,
                    project_id
                ));
            }

            Ok(plan)
        } else {
            self.plan_service
                .get_default_plan(project_id)
                .await
                .map_err(|e| anyhow!("Failed to load plan for project {}: {}", project_id, e))?
                .ok_or_else(|| anyhow!("Project {} has no plan", project_id))
        }
    }

}
