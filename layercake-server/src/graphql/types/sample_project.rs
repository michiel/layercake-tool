use async_graphql::SimpleObject;

use layercake_core::services::sample_project_service::SampleProjectMetadata;

#[derive(SimpleObject, Clone, Debug)]
pub struct SampleProject {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
}

impl From<SampleProjectMetadata> for SampleProject {
    fn from(metadata: SampleProjectMetadata) -> Self {
        Self {
            key: metadata.key,
            name: metadata.name,
            description: metadata.description,
        }
    }
}
