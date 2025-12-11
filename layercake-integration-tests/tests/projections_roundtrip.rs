use anyhow::Result;
use layercake::database::migrations::Migrator;
use layercake::database::migrations::MigratorTrait;
use layercake::services::projection_service::ProjectionService as CoreProjectionService;
use layercake_projections::graphql::{
    ProjectionMutation, ProjectionQuery, ProjectionSchemaContext, ProjectionSubscription,
};
use layercake_projections::service::ProjectionService;
use sea_orm::Database;

#[tokio::test]
async fn projections_schema_builds_with_core_db() -> Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;

    // Core service still compiles against the same DB connection
    let _core_service = CoreProjectionService::new(db.clone());

    // Cross-crate service and schema build
    let service = ProjectionService::new(db);
    let context = ProjectionSchemaContext::new(std::sync::Arc::new(service));
    let schema = async_graphql::Schema::build(
        ProjectionQuery::default(),
        ProjectionMutation,
        ProjectionSubscription,
    )
    .data(context)
    .finish();

    // Smoke test the schema root fields
    let doc = "{ __schema { queryType { name } } }";
    let response = schema.execute(doc).await;
    assert!(response.errors.is_empty(), "GraphQL errors: {:?}", response.errors);
    Ok(())
}
