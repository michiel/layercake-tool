use async_graphql::{Request, Schema, Variables};
use axum_mcp::protocol::{ToolContent, ToolsCallResult};
use chrono::{DateTime, Utc};
use layercake as layercake_core;
use layercake_core::app_context::AppContext;
use layercake_core::console::chat::ChatConfig;
use layercake_core::database::migrations::Migrator;
use layercake_core::graphql::{
    chat_manager::ChatManager, context::GraphQLContext, mutations::Mutation, queries::Query,
    subscriptions::Subscription,
};
use layercake_core::mcp::tools::{plans as mcp_plans, projects as mcp_projects};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde_json::{json, Value};
use std::sync::Arc;

type GraphQLSchema = Schema<Query, Mutation, Subscription>;

async fn setup_test_app() -> anyhow::Result<(Arc<AppContext>, GraphQLSchema)> {
    let db: DatabaseConnection = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;

    let app = Arc::new(AppContext::new(db.clone()));
    let chat_config = Arc::new(ChatConfig::load(&db).await?);
    let chat_manager = Arc::new(ChatManager::new());
    let graphql_context = GraphQLContext::new(app.clone(), chat_config, chat_manager);

    let schema = Schema::build(Query, Mutation, Subscription)
        .data(graphql_context)
        .finish();

    Ok((app, schema))
}

fn tool_result_json(result: ToolsCallResult) -> Value {
    result
        .content
        .into_iter()
        .find_map(|item| match item {
            ToolContent::Text { text } => Some(text),
            _ => None,
        })
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .expect("tool response should include JSON text content")
}

fn parse_timestamp(value: &Value, label: &str) -> DateTime<Utc> {
    let raw = value
        .as_str()
        .unwrap_or_else(|| panic!("{} should be a string timestamp", label));
    DateTime::parse_from_rfc3339(raw)
        .unwrap_or_else(|_| panic!("{} should be valid RFC3339", label))
        .with_timezone(&Utc)
}

#[tokio::test]
async fn graphql_mcp_parity_smoke_test() -> anyhow::Result<()> {
    let (app, schema) = setup_test_app().await?;

    // Create a project via GraphQL
    let create_project_mutation = r#"
        mutation CreateProject($input: CreateProjectInput!) {
            createProject(input: $input) {
                id
                name
                description
                createdAt
                updatedAt
            }
        }
    "#;

    let project_variables = Variables::from_json(json!({
        "input": {
            "name": "Parity Smoke Project",
            "description": "Created via GraphQL mutation"
        }
    }));

    let project_response = schema
        .execute(Request::new(create_project_mutation).variables(project_variables))
        .await;
    assert!(
        project_response.errors.is_empty(),
        "GraphQL project mutation errored: {:?}",
        project_response.errors
    );

    let project_json = serde_json::to_value(&project_response)?;
    let gql_project = project_json["data"]["createProject"].clone();
    assert!(
        !gql_project.is_null(),
        "createProject response payload missing"
    );
    let project_id = gql_project["id"]
        .as_i64()
        .expect("project id should be an integer") as i32;

    // Fetch the project via MCP tooling and compare fields
    let mcp_project_result =
        mcp_projects::get_project(Some(json!({ "project_id": project_id })), &app).await?;
    let mcp_project_payload = tool_result_json(mcp_project_result);
    let mcp_project = mcp_project_payload["project"].clone();
    assert!(
        !mcp_project.is_null(),
        "MCP project payload missing 'project'"
    );

    assert_eq!(gql_project["id"], mcp_project["id"]);
    assert_eq!(gql_project["name"], mcp_project["name"]);
    assert_eq!(gql_project["description"], mcp_project["description"]);
    assert_eq!(
        parse_timestamp(&gql_project["createdAt"], "graphql project createdAt"),
        parse_timestamp(&mcp_project["createdAt"], "mcp project createdAt")
    );
    assert_eq!(
        parse_timestamp(&gql_project["updatedAt"], "graphql project updatedAt"),
        parse_timestamp(&mcp_project["updatedAt"], "mcp project updatedAt")
    );

    // Create a plan via GraphQL
    let create_plan_mutation = r#"
        mutation CreatePlan($input: CreatePlanInput!) {
            createPlan(input: $input) {
                id
                projectId
                name
                yamlContent
                dependencies
                status
                createdAt
                updatedAt
            }
        }
    "#;

    let plan_variables = Variables::from_json(json!({
        "input": {
            "projectId": project_id,
            "name": "GraphQL Plan",
            "yamlContent": "steps:\n  - id: step1\n    run: example",
            "dependencies": null
        }
    }));

    let plan_response = schema
        .execute(Request::new(create_plan_mutation).variables(plan_variables))
        .await;
    assert!(
        plan_response.errors.is_empty(),
        "GraphQL plan mutation errored: {:?}",
        plan_response.errors
    );

    let plan_json = serde_json::to_value(&plan_response)?;
    let gql_plan = plan_json["data"]["createPlan"].clone();
    assert!(!gql_plan.is_null(), "createPlan response payload missing");
    let plan_id = gql_plan["id"]
        .as_i64()
        .expect("plan id should be an integer") as i32;

    // Fetch the plan via MCP and ensure parity with GraphQL response
    let mcp_plan_result = mcp_plans::get_plan(Some(json!({ "plan_id": plan_id })), &app).await?;
    let mcp_plan_payload = tool_result_json(mcp_plan_result);
    let mcp_plan_initial = mcp_plan_payload["plan"].clone();
    assert!(
        !mcp_plan_initial.is_null(),
        "MCP plan payload missing 'plan'"
    );

    assert_eq!(gql_plan["id"], mcp_plan_initial["id"]);
    assert_eq!(gql_plan["projectId"], mcp_plan_initial["projectId"]);
    assert_eq!(gql_plan["name"], mcp_plan_initial["name"]);
    assert_eq!(gql_plan["yamlContent"], mcp_plan_initial["yamlContent"]);
    assert_eq!(gql_plan["dependencies"], mcp_plan_initial["dependencies"]);
    assert_eq!(gql_plan["status"], mcp_plan_initial["status"]);
    assert_eq!(
        parse_timestamp(&gql_plan["createdAt"], "graphql plan createdAt"),
        parse_timestamp(&mcp_plan_initial["createdAt"], "mcp plan createdAt")
    );
    assert_eq!(
        parse_timestamp(&gql_plan["updatedAt"], "graphql plan updatedAt"),
        parse_timestamp(&mcp_plan_initial["updatedAt"], "mcp plan updatedAt")
    );

    // Update the plan via MCP tooling
    let updated_dependencies = vec![project_id];
    let update_plan_result = mcp_plans::update_plan(
        Some(json!({
            "plan_id": plan_id,
            "name": "Updated Plan via MCP",
            "yaml_content": "steps:\n  - id: updated-step\n    run: updated",
            "dependencies": updated_dependencies,
        })),
        &app,
    )
    .await?;
    let update_payload = tool_result_json(update_plan_result);
    let mcp_plan_updated = update_payload["plan"].clone();
    assert!(
        !mcp_plan_updated.is_null(),
        "MCP update response missing 'plan'"
    );

    // Fetch the plan via GraphQL after MCP update
    let get_plan_query = r#"
        query GetPlan($id: Int!) {
            plan(id: $id) {
                id
                projectId
                name
                yamlContent
                dependencies
                status
                createdAt
                updatedAt
            }
        }
    "#;

    let get_plan_vars = Variables::from_json(json!({ "id": plan_id }));
    let plan_after_update = schema
        .execute(Request::new(get_plan_query).variables(get_plan_vars))
        .await;
    assert!(
        plan_after_update.errors.is_empty(),
        "GraphQL plan query errored: {:?}",
        plan_after_update.errors
    );

    let plan_after_update_json = serde_json::to_value(&plan_after_update)?;
    let gql_plan_after_update = plan_after_update_json["data"]["plan"].clone();
    assert!(
        !gql_plan_after_update.is_null(),
        "plan query response payload missing"
    );

    assert_eq!(
        gql_plan_after_update["id"], mcp_plan_updated["id"],
        "plan id mismatch after update"
    );
    assert_eq!(
        gql_plan_after_update["projectId"], mcp_plan_updated["projectId"],
        "plan projectId mismatch after update"
    );
    assert_eq!(
        gql_plan_after_update["name"], mcp_plan_updated["name"],
        "plan name mismatch after update"
    );
    assert_eq!(
        gql_plan_after_update["yamlContent"], mcp_plan_updated["yamlContent"],
        "plan yamlContent mismatch after update"
    );
    assert_eq!(
        gql_plan_after_update["dependencies"], mcp_plan_updated["dependencies"],
        "plan dependencies mismatch after update"
    );
    assert_eq!(
        gql_plan_after_update["status"], mcp_plan_updated["status"],
        "plan status mismatch after update"
    );
    assert_eq!(
        parse_timestamp(
            &gql_plan_after_update["createdAt"],
            "graphql plan createdAt after update"
        ),
        parse_timestamp(
            &mcp_plan_updated["createdAt"],
            "mcp plan createdAt after update"
        )
    );
    assert_eq!(
        parse_timestamp(
            &gql_plan_after_update["updatedAt"],
            "graphql plan updatedAt after update"
        ),
        parse_timestamp(
            &mcp_plan_updated["updatedAt"],
            "mcp plan updatedAt after update"
        )
    );

    Ok(())
}
