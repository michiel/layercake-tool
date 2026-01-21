use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_graphql::{Request, Schema};
use layercake_core::database::entities::users;
use layercake_core::database::migrations::{Migrator, MigratorTrait};
use layercake_core::services::system_settings_service::SystemSettingsService;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, EntityTrait, QueryFilter, Set};

use layercake_server::graphql::context::GraphQLContext;
use layercake_server::graphql::mutations::Mutation;
use layercake_server::graphql::queries::Query;
use layercake_server::graphql::subscriptions::Subscription;
use layercake_server::graphql::chat_manager::ChatManager;

#[derive(serde::Serialize)]
struct ManifestCase<'a> {
    name: &'a str,
    path: &'a str,
    description: &'a str,
}

#[derive(serde::Serialize)]
struct Manifest<'a> {
    cases: Vec<ManifestCase<'a>>,
}

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("resources")
        .join("test-fixtures")
        .join("golden")
        .join("errors")
}

fn write_json(path: &Path, value: &serde_json::Value) -> anyhow::Result<()> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

async fn execute(schema: &Schema<Query, Mutation, Subscription>, request: &str) -> serde_json::Value {
    let response = schema.execute(Request::new(request)).await;
    serde_json::to_value(response).expect("response serializable")
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;

    let app = Arc::new(layercake_core::app_context::AppContext::new(db.clone()));
    let system_settings = Arc::new(
        SystemSettingsService::new(db.clone())
            .await
            .map_err(anyhow::Error::from)?,
    );
    let chat_manager = Arc::new(ChatManager::new());
    let gql_context = GraphQLContext::new(app, system_settings, chat_manager);

    let schema = Schema::build(Query, Mutation::default(), Subscription)
        .data(gql_context)
        .finish();

    let output_dir = fixtures_root();
    fs::create_dir_all(&output_dir)?;

    let register_valid = r#"
        mutation {
            register(input: {
                email: "baseline@example.com",
                username: "baseline",
                displayName: "Baseline",
                password: "Password123!"
            }) {
                user { id }
            }
        }
    "#;
    let _ = execute(&schema, register_valid).await;

    let invalid_email = execute(
        &schema,
        r#"
        mutation {
            register(input: {
                email: "not-an-email",
                username: "invalid",
                displayName: "Invalid",
                password: "Password123!"
            }) {
                user { id }
            }
        }
        "#,
    )
    .await;

    let duplicate_email = execute(
        &schema,
        r#"
        mutation {
            register(input: {
                email: "baseline@example.com",
                username: "baseline2",
                displayName: "Baseline Two",
                password: "Password123!"
            }) {
                user { id }
            }
        }
        "#,
    )
    .await;

    let unauthorized = execute(
        &schema,
        r#"
        mutation {
            login(input: { email: "missing@example.com", password: "Password123!" }) {
                sessionId
            }
        }
        "#,
    )
    .await;

    if let Some(user) = users::Entity::find()
        .filter(users::Column::Email.eq("baseline@example.com"))
        .one(&db)
        .await?
    {
        let mut active: users::ActiveModel = user.into();
        active.is_active = Set(false);
        active.update(&db).await?;
    }

    let forbidden = execute(
        &schema,
        r#"
        mutation {
            login(input: { email: "baseline@example.com", password: "Password123!" }) {
                sessionId
            }
        }
        "#,
    )
    .await;

    let not_found = execute(
        &schema,
        r#"
        query {
            getPlanDag(projectId: 9999) {
                version
            }
        }
        "#,
    )
    .await;

    write_json(&output_dir.join("validation_invalid_email.json"), &invalid_email)?;
    write_json(&output_dir.join("conflict_duplicate_email.json"), &duplicate_email)?;
    write_json(&output_dir.join("unauthorized_login.json"), &unauthorized)?;
    write_json(&output_dir.join("forbidden_login_inactive.json"), &forbidden)?;
    write_json(&output_dir.join("not_found_plan_dag.json"), &not_found)?;

    let manifest = Manifest {
        cases: vec![
            ManifestCase {
                name: "validation_invalid_email",
                path: "validation_invalid_email.json",
                description: "Register with invalid email format",
            },
            ManifestCase {
                name: "conflict_duplicate_email",
                path: "conflict_duplicate_email.json",
                description: "Register with an email that already exists",
            },
            ManifestCase {
                name: "unauthorized_login",
                path: "unauthorized_login.json",
                description: "Login with missing user credentials",
            },
            ManifestCase {
                name: "forbidden_login_inactive",
                path: "forbidden_login_inactive.json",
                description: "Login with a deactivated account",
            },
            ManifestCase {
                name: "not_found_plan_dag",
                path: "not_found_plan_dag.json",
                description: "Fetch plan DAG for missing project",
            },
        ],
    };

    let manifest_json = serde_json::to_value(&manifest)?;
    write_json(&output_dir.join("manifest.json"), &manifest_json)?;

    println!("Baseline errors captured in {}", output_dir.display());

    Ok(())
}
