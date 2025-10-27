//! Shared error message builders for GraphQL and MCP APIs
//!
//! This module provides consistent error message formatting across
//! both API surfaces without forcing them to share error types.
//!
//! # Examples
//!
//! ```rust
//! use layercake::common::error_helpers::*;
//! use sea_orm::DbErr;
//! use anyhow::anyhow;
//!
//! let db_err = DbErr::RecordNotFound("Project not found".to_string());
//! let err = anyhow!("Boom");
//!
//! // Database errors
//! let msg = db_error_msg("create project", &db_err);
//!
//! // Not found errors
//! let msg = not_found_msg("Project", 42);
//!
//! // Service errors
//! let msg = service_error_msg("DataSourceService", &err);
//!
//! // Validation errors
//! let msg = validation::required_field("email");
//! let msg = validation::invalid_format("email", "must be valid email");
//! ```

/// Create contextualized error message
///
/// Combines context string with an error message.
///
/// # Examples
///
/// ```
/// use layercake::common::error_helpers::context_error;
///
/// let msg = context_error("Failed to process", "invalid input");
/// assert_eq!(msg, "Failed to process: invalid input");
/// ```
pub fn context_error(context: &str, error: impl std::fmt::Display) -> String {
    format!("{}: {}", context, error)
}

/// Database error message
///
/// Creates a standardized database error message with operation context.
///
/// # Examples
///
/// ```
/// use layercake::common::error_helpers::db_error_msg;
///
/// let msg = db_error_msg("find user", "connection timeout");
/// assert_eq!(msg, "Database error during find user: connection timeout");
/// ```
pub fn db_error_msg(operation: &str, error: impl std::fmt::Display) -> String {
    format!("Database error during {}: {}", operation, error)
}

/// Service error message
///
/// Creates a standardized service error message.
///
/// # Examples
///
/// ```
/// use layercake::common::error_helpers::service_error_msg;
///
/// let msg = service_error_msg("DataSourceService", "file not found");
/// assert_eq!(msg, "Service 'DataSourceService' failed: file not found");
/// ```
pub fn service_error_msg(service: &str, error: impl std::fmt::Display) -> String {
    format!("Service '{}' failed: {}", service, error)
}

/// Not found message with ID
///
/// Creates a "resource not found" message including the ID.
///
/// # Examples
///
/// ```
/// use layercake::common::error_helpers::not_found_msg;
///
/// let msg = not_found_msg("Project", 42);
/// assert_eq!(msg, "Project with id '42' not found");
/// ```
pub fn not_found_msg(resource: &str, id: impl std::fmt::Display) -> String {
    format!("{} with id '{}' not found", resource, id)
}

/// Not found message without ID
///
/// Creates a simple "resource not found" message.
///
/// # Examples
///
/// ```
/// use layercake::common::error_helpers::not_found_simple;
///
/// let msg = not_found_simple("Project");
/// assert_eq!(msg, "Project not found");
/// ```
pub fn not_found_simple(resource: &str) -> String {
    format!("{} not found", resource)
}

/// Validation error message builders
pub mod validation {
    /// Missing required parameter error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::required_field;
    ///
    /// let msg = required_field("email");
    /// assert_eq!(msg, "Missing required parameter: email");
    /// ```
    pub fn required_field(field: &str) -> String {
        format!("Missing required parameter: {}", field)
    }

    /// Invalid type error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::invalid_type;
    ///
    /// let msg = invalid_type("age", "number");
    /// assert_eq!(msg, "age must be a number");
    /// ```
    pub fn invalid_type(field: &str, expected: &str) -> String {
        format!("{} must be a {}", field, expected)
    }

    /// Invalid format error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::invalid_format;
    ///
    /// let msg = invalid_format("email", "must contain @");
    /// assert_eq!(msg, "Invalid format for 'email': must contain @");
    /// ```
    pub fn invalid_format(field: &str, message: &str) -> String {
        format!("Invalid format for '{}': {}", field, message)
    }

    /// Resource already exists error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::already_exists;
    ///
    /// let msg = already_exists("User", "john@example.com");
    /// assert_eq!(msg, "User 'john@example.com' already exists");
    /// ```
    pub fn already_exists(resource: &str, identifier: &str) -> String {
        format!("{} '{}' already exists", resource, identifier)
    }

    /// Value out of range error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::out_of_range;
    ///
    /// let msg = out_of_range("age", 0, 120);
    /// assert_eq!(msg, "age must be between 0 and 120");
    /// ```
    pub fn out_of_range(
        field: &str,
        min: impl std::fmt::Display,
        max: impl std::fmt::Display,
    ) -> String {
        format!("{} must be between {} and {}", field, min, max)
    }

    /// Invalid value error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::validation::invalid_value;
    ///
    /// let msg = invalid_value("role", "admin, editor, viewer");
    /// assert_eq!(msg, "Invalid value for 'role'. Expected one of: admin, editor, viewer");
    /// ```
    pub fn invalid_value(field: &str, expected: &str) -> String {
        format!(
            "Invalid value for '{}'. Expected one of: {}",
            field, expected
        )
    }
}

/// Authentication/Authorization message builders
pub mod auth {
    /// Invalid credentials error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::invalid_credentials;
    ///
    /// let msg = invalid_credentials();
    /// assert_eq!(msg, "Invalid email or password");
    /// ```
    pub fn invalid_credentials() -> String {
        "Invalid email or password".to_string()
    }

    /// Account disabled error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::account_disabled;
    ///
    /// let msg = account_disabled();
    /// assert_eq!(msg, "Account is deactivated");
    /// ```
    pub fn account_disabled() -> String {
        "Account is deactivated".to_string()
    }

    /// Session expired error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::session_expired;
    ///
    /// let msg = session_expired();
    /// assert_eq!(msg, "Session has expired");
    /// ```
    pub fn session_expired() -> String {
        "Session has expired".to_string()
    }

    /// Insufficient permissions error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::insufficient_permissions;
    ///
    /// let msg = insufficient_permissions("delete project");
    /// assert_eq!(msg, "Insufficient permissions to delete project");
    /// ```
    pub fn insufficient_permissions(action: &str) -> String {
        format!("Insufficient permissions to {}", action)
    }

    /// Token expired error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::token_expired;
    ///
    /// let msg = token_expired();
    /// assert_eq!(msg, "Authentication token has expired");
    /// ```
    pub fn token_expired() -> String {
        "Authentication token has expired".to_string()
    }

    /// Invalid token error
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::error_helpers::auth::invalid_token;
    ///
    /// let msg = invalid_token();
    /// assert_eq!(msg, "Invalid authentication token");
    /// ```
    pub fn invalid_token() -> String {
        "Invalid authentication token".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_error() {
        let msg = context_error("Failed to process", "invalid input");
        assert_eq!(msg, "Failed to process: invalid input");
    }

    #[test]
    fn test_db_error_msg() {
        let msg = db_error_msg("create user", "duplicate key");
        assert_eq!(msg, "Database error during create user: duplicate key");
    }

    #[test]
    fn test_service_error_msg() {
        let msg = service_error_msg("EmailService", "connection timeout");
        assert_eq!(msg, "Service 'EmailService' failed: connection timeout");
    }

    #[test]
    fn test_not_found_msg() {
        let msg = not_found_msg("Project", 42);
        assert_eq!(msg, "Project with id '42' not found");
    }

    #[test]
    fn test_not_found_simple() {
        let msg = not_found_simple("User");
        assert_eq!(msg, "User not found");
    }

    #[test]
    fn test_validation_required_field() {
        let msg = validation::required_field("email");
        assert_eq!(msg, "Missing required parameter: email");
    }

    #[test]
    fn test_validation_invalid_type() {
        let msg = validation::invalid_type("age", "number");
        assert_eq!(msg, "age must be a number");
    }

    #[test]
    fn test_validation_invalid_format() {
        let msg = validation::invalid_format("email", "must contain @");
        assert_eq!(msg, "Invalid format for 'email': must contain @");
    }

    #[test]
    fn test_validation_already_exists() {
        let msg = validation::already_exists("User", "john@example.com");
        assert_eq!(msg, "User 'john@example.com' already exists");
    }

    #[test]
    fn test_validation_out_of_range() {
        let msg = validation::out_of_range("age", 0, 120);
        assert_eq!(msg, "age must be between 0 and 120");
    }

    #[test]
    fn test_validation_invalid_value() {
        let msg = validation::invalid_value("role", "admin, editor");
        assert_eq!(
            msg,
            "Invalid value for 'role'. Expected one of: admin, editor"
        );
    }

    #[test]
    fn test_auth_invalid_credentials() {
        let msg = auth::invalid_credentials();
        assert_eq!(msg, "Invalid email or password");
    }

    #[test]
    fn test_auth_account_disabled() {
        let msg = auth::account_disabled();
        assert_eq!(msg, "Account is deactivated");
    }

    #[test]
    fn test_auth_session_expired() {
        let msg = auth::session_expired();
        assert_eq!(msg, "Session has expired");
    }

    #[test]
    fn test_auth_insufficient_permissions() {
        let msg = auth::insufficient_permissions("delete project");
        assert_eq!(msg, "Insufficient permissions to delete project");
    }

    #[test]
    fn test_auth_token_expired() {
        let msg = auth::token_expired();
        assert_eq!(msg, "Authentication token has expired");
    }

    #[test]
    fn test_auth_invalid_token() {
        let msg = auth::invalid_token();
        assert_eq!(msg, "Invalid authentication token");
    }
}
