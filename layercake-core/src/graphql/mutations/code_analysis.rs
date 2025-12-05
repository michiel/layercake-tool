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
        let profile = context
            .app
            .code_analysis_service()
            .create(input.project_id, input.file_path, input.dataset_id)
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::create", e))?;
        Ok(CodeAnalysisProfile::from(profile))
    }

    async fn update_code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        input: UpdateCodeAnalysisProfileInput,
    ) -> Result<CodeAnalysisProfile> {
        let context = ctx.data::<GraphQLContext>()?;
        let profile = context
            .app
            .code_analysis_service()
            .update(&input.id, input.file_path, Some(input.dataset_id))
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::update", e))?;
        Ok(CodeAnalysisProfile::from(profile))
    }

    async fn delete_code_analysis_profile(&self, ctx: &Context<'_>, id: String) -> Result<bool> {
        let context = ctx.data::<GraphQLContext>()?;
        context
            .app
            .code_analysis_service()
            .delete(&id)
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::delete", e))
    }

    async fn run_code_analysis_profile(
        &self,
        ctx: &Context<'_>,
        id: String,
    ) -> Result<CodeAnalysisRunResult> {
        let context = ctx.data::<GraphQLContext>()?;
        let profile = context
            .app
            .code_analysis_service()
            .run(&id)
            .await
            .map_err(|e| StructuredError::service("CodeAnalysisService::run", e))?;
        Ok(CodeAnalysisRunResult {
            profile: CodeAnalysisProfile::from(profile),
        })
    }
}
