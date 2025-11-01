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
use layercake_core::mcp::tools::{data_sources, plans as mcp_plans, projects as mcp_projects};
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

    // --- Data source parity -------------------------------------------------
    let create_empty_data_source_mutation = r#"
        mutation CreateEmptyDataSource($input: CreateEmptyDataSourceInput!) {
            createEmptyDataSource(input: $input) {
                id
                projectId
                name
                dataType
                status
                createdAt
                updatedAt
            }
        }
    "#;

    let data_source_variables = Variables::from_json(json!({
        "input": {
            "projectId": project_id,
            "name": "GraphQL Empty Source",
            "description": "Created via GraphQL",
            "dataType": "NODES"
        }
    }));

    let data_source_response = schema
        .execute(Request::new(create_empty_data_source_mutation).variables(data_source_variables))
        .await;
    assert!(
        data_source_response.errors.is_empty(),
        "GraphQL createEmptyDataSource mutation errored: {:?}",
        data_source_response.errors
    );

    let data_source_json = serde_json::to_value(&data_source_response)?;
    let gql_data_source = data_source_json["data"]["createEmptyDataSource"].clone();
    assert!(
        !gql_data_source.is_null(),
        "createEmptyDataSource response payload missing"
    );
    let data_source_id = gql_data_source["id"]
        .as_i64()
        .expect("data source id should be an integer") as i32;

    let mcp_data_source = tool_result_json(
        data_sources::get_data_source(Some(json!({ "data_source_id": data_source_id })), &app)
            .await?,
    )["dataSource"]
        .clone();
    assert!(
        !mcp_data_source.is_null(),
        "MCP get_data_source payload missing 'dataSource'"
    );

    assert_eq!(gql_data_source["id"], mcp_data_source["id"]);
    assert_eq!(gql_data_source["projectId"], mcp_data_source["projectId"]);
    assert_eq!(gql_data_source["dataType"], mcp_data_source["dataType"]);
    assert_eq!(gql_data_source["status"], mcp_data_source["status"]);
    assert_eq!(
        parse_timestamp(
            &gql_data_source["createdAt"],
            "graphql data source createdAt"
        ),
        parse_timestamp(&mcp_data_source["createdAt"], "mcp data source createdAt")
    );

    let updated_data_source = tool_result_json(
        data_sources::update_data_source(
            Some(json!({
                "data_source_id": data_source_id,
                "name": "Updated MCP Source",
                "description": "Name updated via MCP"
            })),
            &app,
        )
        .await?,
    )["dataSource"]
        .clone();
    assert!(
        !updated_data_source.is_null(),
        "MCP update_data_source payload missing 'dataSource'"
    );
    assert_eq!(updated_data_source["name"], json!("Updated MCP Source"));
    assert_eq!(
        updated_data_source["description"],
        json!("Name updated via MCP")
    );

    let get_data_source_query = r#"
        query GetDataSource($id: Int!) {
            dataSource(id: $id) {
                id
                projectId
                name
                description
                dataType
                status
                createdAt
                updatedAt
            }
        }
    "#;
    let gql_ds_after = schema
        .execute(
            Request::new(get_data_source_query).variables(Variables::from_json(json!({
                "id": data_source_id
            }))),
        )
        .await;
    assert!(
        gql_ds_after.errors.is_empty(),
        "GraphQL dataSource query errored: {:?}",
        gql_ds_after.errors
    );
    let gql_ds_after_json = serde_json::to_value(&gql_ds_after)?;
    let gql_data_source_after = gql_ds_after_json["data"]["dataSource"].clone();
    assert_eq!(gql_data_source_after["name"], json!("Updated MCP Source"));
    assert_eq!(
        gql_data_source_after["description"],
        json!("Name updated via MCP")
    );

    let mcp_list = tool_result_json(
        data_sources::list_data_sources(Some(json!({ "project_id": project_id })), &app).await?,
    );
    let mcp_count = mcp_list["count"]
        .as_u64()
        .expect("count should be a number");

    let gql_list_query = r#"
        query ListProjectDataSources($projectId: Int!) {
            dataSources(projectId: $projectId) {
                id
            }
        }
    "#;
    let gql_list = schema
        .execute(
            Request::new(gql_list_query).variables(Variables::from_json(json!({
                "projectId": project_id
            }))),
        )
        .await;
    assert!(
        gql_list.errors.is_empty(),
        "GraphQL dataSources query errored: {:?}",
        gql_list.errors
    );
    let gql_list_json = serde_json::to_value(&gql_list)?;
    let gql_list_count = gql_list_json["data"]["dataSources"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);

    assert_eq!(mcp_count as usize, gql_list_count);

    Ok(())
}
