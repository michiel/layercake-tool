#![allow(dead_code)]

use layercake_core::database::entities::users;
use crate::mcp::tools::{create_success_response, get_optional_param, get_required_param};
use layercake_core::errors::{CoreError, CoreErrorKind};
use layercake_core::services::auth_service::AuthService;
use axum_mcp::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::{json, Value};

/// User registration tool
pub async fn register_user(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let email = get_required_param(&arguments, "email")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Email must be a string".to_string(),
        })?
        .to_string();

    let username = get_required_param(&arguments, "username")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Username must be a string".to_string(),
        })?
        .to_string();

    let password = get_required_param(&arguments, "password")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Password must be a string".to_string(),
        })?;

    let display_name = get_optional_param(&arguments, "display_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| username.clone());

    // Validate input using auth service
    AuthService::validate_email(&email).map_err(mcp_error_from_core)?;

    AuthService::validate_username(&username).map_err(mcp_error_from_core)?;

    AuthService::validate_display_name(&display_name).map_err(mcp_error_from_core)?;

    // Check if user already exists
    let existing_user = users::Entity::find()
        .filter(users::Column::Email.eq(&email))
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?;

    if existing_user.is_some() {
        return Err(McpError::Validation {
            message: "User with this email already exists".to_string(),
        });
    }

    let existing_username = users::Entity::find()
        .filter(users::Column::Username.eq(&username))
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?;

    if existing_username.is_some() {
        return Err(McpError::Validation {
            message: "Username already taken".to_string(),
        });
    }

    // Hash password
    let password_hash = AuthService::hash_password(password).map_err(mcp_error_from_core)?;

    // Create user
    let new_user = users::ActiveModel {
        email: Set(email),
        username: Set(username),
        display_name: Set(display_name),
        password_hash: Set(password_hash),
        avatar_color: Set(AuthService::generate_avatar_color()),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let user = new_user.insert(db).await.map_err(|e| McpError::Internal {
        message: format!("Failed to create user: {}", e),
    })?;

    let result = json!({
        "user_id": user.id,
        "email": user.email,
        "username": user.username,
        "display_name": user.display_name,
        "avatar_color": user.avatar_color,
        "is_active": user.is_active,
        "created_at": user.created_at,
        "message": "User registered successfully"
    });

    create_success_response(&result)
}

/// User login tool
pub async fn login_user(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let email = get_required_param(&arguments, "email")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Email must be a string".to_string(),
        })?;

    let password = get_required_param(&arguments, "password")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Password must be a string".to_string(),
        })?;

    // Find user by email
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?
        .ok_or_else(|| McpError::Validation {
            message: "Invalid email or password".to_string(),
        })?;

    // Verify password
    let password_valid =
        AuthService::verify_password(password, &user.password_hash).map_err(mcp_error_from_core)?;

    if !password_valid {
        return Err(McpError::Validation {
            message: "Invalid email or password".to_string(),
        });
    }

    // Check if account is active
    if !user.is_active {
        return Err(McpError::Validation {
            message: "Account is deactivated".to_string(),
        });
    }

    // Generate session
    let session_id = AuthService::generate_session_id();
    let expires_at = AuthService::calculate_session_expiry();

    // In a real implementation, you would store the session in the database
    // For now, we'll return the session data

    let result = json!({
        "user": {
            "id": user.id,
            "email": user.email,
            "username": user.username,
            "display_name": user.display_name,
            "avatar_color": user.avatar_color,
            "is_active": user.is_active
        },
        "session": {
            "session_id": session_id,
            "expires_at": expires_at
        },
        "message": "Login successful"
    });

    create_success_response(&result)
}

/// Logout tool
pub async fn logout_user(
    arguments: Option<Value>,
    _db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let session_id = get_required_param(&arguments, "session_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Session ID must be a string".to_string(),
        })?;

    // In a real implementation, you would invalidate the session in the database
    tracing::info!("User logged out with session: {}", session_id);

    let result = json!({
        "message": "Logout successful",
        "session_id": session_id
    });

    create_success_response(&result)
}

/// Validate session tool
pub async fn validate_session(
    arguments: Option<Value>,
    _db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let session_id = get_required_param(&arguments, "session_id")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Session ID must be a string".to_string(),
        })?;

    // In a real implementation, you would validate the session against the database
    // For now, we'll perform basic validation
    if session_id.is_empty() || session_id.len() < 10 {
        return Err(McpError::Validation {
            message: "Invalid session ID".to_string(),
        });
    }

    // Mock user data - in real implementation, fetch from session store
    let result = json!({
        "valid": true,
        "user": {
            "id": 1,
            "email": "user@example.com",
            "username": "user",
            "display_name": "Test User"
        },
        "expires_at": chrono::Utc::now() + chrono::Duration::hours(24),
        "message": "Session is valid"
    });

    create_success_response(&result)
}

/// Change password tool
pub async fn change_password(
    arguments: Option<Value>,
    db: &DatabaseConnection,
) -> McpResult<ToolsCallResult> {
    let user_id = get_required_param(&arguments, "user_id")?
        .as_i64()
        .ok_or_else(|| McpError::Validation {
            message: "User ID must be a number".to_string(),
        })? as i32;

    let current_password = get_required_param(&arguments, "current_password")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "Current password must be a string".to_string(),
        })?;

    let new_password = get_required_param(&arguments, "new_password")?
        .as_str()
        .ok_or_else(|| McpError::Validation {
            message: "New password must be a string".to_string(),
        })?;

    // Find user
    let user = users::Entity::find_by_id(user_id)
        .one(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Database error: {}", e),
        })?
        .ok_or_else(|| McpError::Validation {
            message: "User not found".to_string(),
        })?;

    // Verify current password
    let password_valid =
        AuthService::verify_password(current_password, &user.password_hash)
            .map_err(mcp_error_from_core)?;

    if !password_valid {
        return Err(McpError::Validation {
            message: "Current password is incorrect".to_string(),
        });
    }

    // Hash new password
    let new_password_hash =
        AuthService::hash_password(new_password).map_err(mcp_error_from_core)?;

    // Update password
    let mut user_active: users::ActiveModel = user.into();
    user_active.password_hash = Set(new_password_hash);
    user_active.updated_at = Set(chrono::Utc::now());

    user_active
        .update(db)
        .await
        .map_err(|e| McpError::Internal {
            message: format!("Failed to update password: {}", e),
        })?;

    let result = json!({
        "message": "Password changed successfully",
        "user_id": user_id
    });

    create_success_response(&result)
}

fn mcp_error_from_core(err: CoreError) -> McpError {
    match err.kind() {
        CoreErrorKind::Validation => McpError::Validation {
            message: err.message().to_string(),
        },
        _ => McpError::Internal {
            message: err.message().to_string(),
        },
    }
}
