use crate::services::code_analysis_service::CodeAnalysisProfile as ServiceProfile;
use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};

#[derive(SimpleObject, Clone)]
pub struct CodeAnalysisProfile {
    pub id: String,
    pub project_id: i32,
    pub file_path: String,
    pub dataset_id: Option<i32>,
    pub last_run: Option<DateTime<Utc>>,
    pub report: Option<String>,
    #[graphql(name = "noInfra")]
    pub no_infra: bool,
    pub options: Option<String>,
}

impl From<ServiceProfile> for CodeAnalysisProfile {
    fn from(profile: ServiceProfile) -> Self {
        Self {
            id: profile.id,
            project_id: profile.project_id,
            file_path: profile.file_path,
            dataset_id: profile.dataset_id,
            last_run: profile.last_run,
            report: profile.report,
            no_infra: profile.no_infra,
            options: profile.options,
        }
    }
}

#[derive(InputObject)]
pub struct CreateCodeAnalysisProfileInput {
    pub project_id: i32,
    pub file_path: String,
    pub dataset_id: Option<i32>,
    #[graphql(name = "noInfra")]
    pub no_infra: Option<bool>,
    pub options: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateCodeAnalysisProfileInput {
    pub id: String,
    pub file_path: Option<String>,
    pub dataset_id: Option<i32>,
    #[graphql(name = "noInfra")]
    pub no_infra: Option<bool>,
    pub options: Option<String>,
}

#[derive(SimpleObject)]
pub struct CodeAnalysisRunResult {
    pub profile: CodeAnalysisProfile,
}
