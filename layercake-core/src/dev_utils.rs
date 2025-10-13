//! Development utilities for user and session management
//!
//! This module provides utilities for development mode to automatically create
//! and assign random users without requiring full authentication setup.

use crate::database::entities::{projects, user_sessions, users};
use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::OnceLock;

/// Names for development users
const DEV_USER_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack", "Kate",
    "Liam", "Maya", "Noah", "Olivia", "Peter", "Quinn", "Ruby", "Sam", "Tina", "Uma", "Victor",
    "Wendy", "Xander", "Yara", "Zoe",
];

/// Adjectives for development usernames
const DEV_ADJECTIVES: &[&str] = &[
    "creative",
    "brilliant",
    "focused",
    "curious",
    "energetic",
    "thoughtful",
    "dynamic",
    "innovative",
    "analytical",
    "collaborative",
    "strategic",
    "intuitive",
    "methodical",
    "adventurous",
    "precise",
    "versatile",
    "dedicated",
    "imaginative",
];

/// Global counter for ensuring unique usernames
static USER_COUNTER: OnceLock<std::sync::Mutex<u32>> = OnceLock::new();

fn get_next_counter() -> u32 {
    let counter = USER_COUNTER.get_or_init(|| std::sync::Mutex::new(1));
    let mut count = counter.lock().unwrap_or_else(|poisoned| {
        // If the mutex is poisoned, we recover by using the inner value
        poisoned.into_inner()
    });
    let result = *count;
    *count += 1;
    result
}

/// Creates a random development user with a unique username and email
pub async fn create_dev_user(db: &DatabaseConnection) -> Result<users::Model> {
    let counter = get_next_counter();

    // Generate random components
    let name_idx = (counter as usize) % DEV_USER_NAMES.len();
    let adj_idx = ((counter as usize) * 7) % DEV_ADJECTIVES.len(); // Use different multiplier for variation

    let name = DEV_USER_NAMES[name_idx];
    let adjective = DEV_ADJECTIVES[adj_idx];

    let username = format!("{}-{}-{}", adjective, name.to_lowercase(), counter);
    let email = format!("{}@dev.layercake.local", username);
    let display_name = format!("{} {} (Dev)", adjective.to_uppercase_first(), name);

    // Create user with a development password (not secure, only for dev)
    let mut user = users::ActiveModel::new();
    user.email = Set(email);
    user.username = Set(username);
    user.display_name = Set(display_name);
    user.password_hash = Set("dev_mode_password_not_secure".to_string()); // Placeholder for dev
    user.is_active = Set(true);

    let user_model = user.insert(db).await?;

    tracing::info!(
        "Created development user: {} (ID: {}) with email: {}",
        user_model.username,
        user_model.id,
        user_model.email
    );

    Ok(user_model)
}

/// Creates a session for a user in a specific project
pub async fn create_dev_session(
    db: &DatabaseConnection,
    user: &users::Model,
    project_id: i32,
) -> Result<user_sessions::Model> {
    let session = user_sessions::ActiveModel::new(user.id, user.display_name.clone(), project_id);

    let session_model = session.insert(db).await?;

    tracing::info!(
        "Created development session: {} for user {} in project {}",
        session_model.session_id,
        user.username,
        project_id
    );

    Ok(session_model)
}

/// Gets or creates a development project for testing
pub async fn get_or_create_dev_project(
    db: &DatabaseConnection,
    _user_id: i32,
) -> Result<projects::Model> {
    // Try to find an existing development project
    let existing_project = projects::Entity::find()
        .filter(projects::Column::Name.eq("Development Project"))
        .one(db)
        .await?;

    if let Some(project) = existing_project {
        return Ok(project);
    }

    // Create a new development project
    let mut project = projects::ActiveModel::new();
    project.name = Set("Development Project".to_string());
    project.description = Set(Some(
        "Auto-generated project for development and testing".to_string(),
    ));
    project.created_at = Set(Utc::now());
    project.updated_at = Set(Utc::now());

    let project_model = project.insert(db).await?;

    tracing::info!(
        "Created development project: {} (ID: {})",
        project_model.name,
        project_model.id
    );

    Ok(project_model)
}

/// Complete development setup: creates a random user, project, and session
///
/// This is the main function to call when you need a complete development setup
/// for testing collaborative features without authentication.
pub async fn setup_dev_user_session(db: &DatabaseConnection) -> Result<DevUserSession> {
    // Create random user
    let user = create_dev_user(db).await?;

    // Create or get development project
    let project = get_or_create_dev_project(db, user.id).await?;

    // Create session for this user in the project
    let session = create_dev_session(db, &user, project.id).await?;

    Ok(DevUserSession {
        user,
        project,
        session,
    })
}

/// Convenience struct containing all development entities
#[derive(Debug)]
pub struct DevUserSession {
    pub user: users::Model,
    pub project: projects::Model,
    pub session: user_sessions::Model,
}

impl DevUserSession {
    /// Get user information formatted for GraphQL responses
    pub fn user_info(&self) -> (String, String, String) {
        (
            self.user.id.to_string(),
            self.user.display_name.clone(),
            self.user.avatar_color.clone(),
        )
    }

    /// Get session ID for authentication
    pub fn session_id(&self) -> &str {
        &self.session.session_id
    }

    /// Get project ID
    pub fn project_id(&self) -> i32 {
        self.project.id
    }
}

/// Trait extension for string capitalization
trait StringExt {
    fn to_uppercase_first(&self) -> String;
}

impl StringExt for str {
    fn to_uppercase_first(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_capitalization() {
        assert_eq!("creative".to_uppercase_first(), "Creative");
        assert_eq!("BRILLIANT".to_uppercase_first(), "BRILLIANT");
        assert_eq!("".to_uppercase_first(), "");
    }

    #[test]
    fn test_counter_uniqueness() {
        let c1 = get_next_counter();
        let c2 = get_next_counter();
        let c3 = get_next_counter();

        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c1, c3);
    }
}
