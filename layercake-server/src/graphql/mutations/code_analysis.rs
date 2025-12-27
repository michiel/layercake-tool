use async_graphql::{Context, Object, Result};

use crate::graphql::context::GraphQLContext;
use crate::graphql::errors::StructuredError;
use crate::graphql::types::{
    CodeAnalysisProfile, CodeAnalysisRunResult, CreateCodeAnalysisProfileInput,
    UpdateCodeAnalysisProfileInput,
};

#[derive(Default)]
pub struct CodeAnalysisMutation;

#[Object]
impl CodeAnalysisMutation {
    async fn create_code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        input: CreateCodeAnalysisProfileInput,
    ) -> Result<CodeAnalysisProfile> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let profile = context
            .app
            .code_analysis_service()
            .create(
                &actor,
                input.project_id,
                input.file_path,
                input.dataset_id,
                input.no_infra.unwrap_or(false),
                input.options.clone(),
                input
                    .analysis_type
                    .clone()
                    .unwrap_or_else(|| "code".to_string()),
                input.solution_options.clone(),
            )
            .await
            .map_err(Error::from)?;
        Ok(CodeAnalysisProfile::from(profile))
    }

    async fn update_code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateCodeAnalysisProfileInput,
    ) -> Result<CodeAnalysisProfile> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let profile = context
            .app
            .code_analysis_service()
            .update(
                &actor,
                &input.id,
                input.file_path,
                Some(input.dataset_id),
                input.no_infra,
                Some(input.options.clone()),
                input.analysis_type,
                Some(input.solution_options.clone()),
            )
            .await
            .map_err(Error::from)?;
        Ok(CodeAnalysisProfile::from(profile))
    }

    async fn delete_code_analysis_profile(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        context
            .app
            .code_analysis_service()
            .delete(&actor, &id)
            .await
            .map_err(Error::from)
    }

    async fn run_code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<CodeAnalysisRunResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let actor = context.actor_for_request(ctx).await;
        let profile = context
            .app
            .code_analysis_service()
            .run(&actor, &id)
            .await
            .map_err(Error::from)?;
        Ok(CodeAnalysisRunResult {
            profile: CodeAnalysisProfile::from(profile),
        })
    }
}
