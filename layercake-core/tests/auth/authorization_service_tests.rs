use chrono::{Duration, Utc};
use layercake_core::database::entities::{project_collaborators, projects, user_sessions, users};
use layercake_core::database::test_utils::setup_test_db;
use layercake_core::errors::CoreErrorKind;
use layercake_core::services::authorization::{AuthorizationService, ProjectRole};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};

async fn seed_project(db: &DatabaseConnection) -> projects::Model {
    let mut project = projects::ActiveModel::new();
    project.name = Set("Test Project".to_string());
    project.insert(db).await.expect("Failed to insert project")
}

async fn seed_user(
    db: &DatabaseConnection,
    email: &str,
    username: &str,
    is_active: bool,
) -> users::Model {
    let mut user = users::ActiveModel::new();
    user.email = Set(email.to_string());
    user.username = Set(username.to_string());
    user.display_name = Set(username.to_string());
    user.password_hash = Set("hash".to_string());
    user.is_active = Set(is_active);
    user.insert(db).await.expect("Failed to insert user")
}

async fn seed_session(
    db: &DatabaseConnection,
    user: &users::Model,
    project: &projects::Model,
    expires_at: Option<chrono::DateTime<Utc>>,
) -> user_sessions::Model {
    let mut session =
        user_sessions::ActiveModel::new(user.id, user.display_name.clone(), project.id);
    if let Some(expiry) = expires_at {
        session.expires_at = Set(expiry);
    }
    session.insert(db).await.expect("Failed to insert session")
}

async fn seed_collaborator(
    db: &DatabaseConnection,
    project: &projects::Model,
    user: &users::Model,
    role: project_collaborators::ProjectRole,
) -> project_collaborators::Model {
    let collaborator = project_collaborators::ActiveModel::new(
        project.id,
        user.id,
        role,
        Some(user.id),
    )
    .accept_invitation();

    collaborator
        .insert(db)
        .await
        .expect("Failed to insert collaborator")
}

#[tokio::test]
async fn get_user_from_session_returns_user() {
    let db = setup_test_db().await;
    let project = seed_project(&db).await;
    let user = seed_user(&db, "user@example.com", "user1", true).await;
    let session = seed_session(&db, &user, &project, None).await;

    let auth = AuthorizationService::new(db.clone());
    let resolved = auth
        .get_user_from_session(&session.session_id)
        .await
        .expect("Expected valid session");

    assert_eq!(resolved.id, user.id);
    assert_eq!(resolved.email, user.email);
}

#[tokio::test]
async fn get_user_from_session_rejects_expired_session() {
    let db = setup_test_db().await;
    let project = seed_project(&db).await;
    let user = seed_user(&db, "expired@example.com", "user2", true).await;
    let expires_at = Utc::now() - Duration::hours(1);
    let session = seed_session(&db, &user, &project, Some(expires_at)).await;

    let auth = AuthorizationService::new(db.clone());
    let err = auth
        .get_user_from_session(&session.session_id)
        .await
        .expect_err("Expected expired session to fail");

    assert_eq!(err.kind(), CoreErrorKind::Unauthorized);
}

#[tokio::test]
async fn get_user_from_session_rejects_inactive_user() {
    let db = setup_test_db().await;
    let project = seed_project(&db).await;
    let user = seed_user(&db, "inactive@example.com", "user3", false).await;
    let session = seed_session(&db, &user, &project, None).await;

    let auth = AuthorizationService::new(db.clone());
    let err = auth
        .get_user_from_session(&session.session_id)
        .await
        .expect_err("Expected inactive user to fail");

    assert_eq!(err.kind(), CoreErrorKind::Forbidden);
}

#[tokio::test]
async fn get_user_project_role_returns_collaborator_role() {
    let db = setup_test_db().await;
    let project = seed_project(&db).await;
    let user = seed_user(&db, "role@example.com", "user4", true).await;
    seed_collaborator(
        &db,
        &project,
        &user,
        project_collaborators::ProjectRole::Editor,
    )
    .await;

    let auth = AuthorizationService::new(db.clone());
    let role = auth
        .get_user_project_role(user.id, project.id)
        .await
        .expect("Expected role lookup")
        .expect("Expected collaborator role");

    assert_eq!(role, ProjectRole::Editor);
}
