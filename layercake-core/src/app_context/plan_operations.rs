use super::{AppContext, PlanSummary};
use crate::auth::Actor;
use crate::database::entities::plans;
use crate::errors::{CoreError, CoreResult};
use crate::services::plan_service::{PlanCreateRequest, PlanUpdateRequest};
use sea_orm::{EntityTrait, QueryOrder};

impl AppContext {
    #[allow(dead_code)]
    pub async fn list_plans(&self, project_id: Option<i32>) -> CoreResult<Vec<PlanSummary>> {
        let plans = if let Some(project_id) = project_id {
            self.plan_service.list_plans(project_id).await?
        } else {
            plans::Entity::find()
                .order_by_desc(plans::Column::UpdatedAt)
                .all(&self.db)
                .await
                .map_err(|e| CoreError::internal(format!("Failed to list plans: {}", e)))?
        };

        Ok(plans.into_iter().map(PlanSummary::from).collect())
    }

    pub async fn get_plan(&self, id: i32) -> CoreResult<Option<PlanSummary>> {
        let plan = self.plan_service.get_plan(id).await?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn get_plan_for_project(&self, project_id: i32) -> CoreResult<Option<PlanSummary>> {
        let plan = self.plan_service.get_default_plan(project_id).await?;

        Ok(plan.map(PlanSummary::from))
    }

    pub async fn create_plan(
        &self,
        actor: &Actor,
        request: PlanCreateRequest,
    ) -> CoreResult<PlanSummary> {
        self.authorize(actor, "write:plan")?;
        let plan = self
            .plan_service
            .create_plan(request)
            .await?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn update_plan(
        &self,
        actor: &Actor,
        id: i32,
        update: PlanUpdateRequest,
    ) -> CoreResult<PlanSummary> {
        self.authorize(actor, "write:plan")?;
        let plan = self
            .plan_service
            .update_plan(id, update)
            .await?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn delete_plan(&self, actor: &Actor, id: i32) -> CoreResult<()> {
        self.authorize(actor, "write:plan")?;
        self.plan_service.delete_plan(id).await
    }

    pub async fn duplicate_plan(
        &self,
        _actor: &Actor,
        id: i32,
        name: String,
    ) -> CoreResult<PlanSummary> {
        let plan = self
            .plan_service
            .duplicate_plan(id, name)
            .await?;

        Ok(PlanSummary::from(plan))
    }

    pub async fn resolve_plan_model(
        &self,
        project_id: i32,
        plan_id: Option<i32>,
    ) -> CoreResult<plans::Model> {
        if let Some(plan_id) = plan_id {
            let plan = self
                .plan_service
                .get_plan(plan_id)
                .await?
                .ok_or_else(|| CoreError::not_found("Plan", plan_id.to_string()))?;

            if plan.project_id != project_id {
                return Err(CoreError::validation(format!(
                    "Plan {} does not belong to project {}",
                    plan_id, project_id
                )));
            }

            Ok(plan)
        } else {
            self.plan_service
                .get_default_plan(project_id)
                .await?
                .ok_or_else(|| CoreError::not_found("Plan", project_id.to_string()))
        }
    }
}
