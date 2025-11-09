//! Authentication and authorisation error types
//!
//! This module provides structured error types for authentication and
//! authorisation operations, including user management, sessions, and permissions.
//!
//! # Examples
//!
//! ```rust
//! use layercake::errors::AuthError;
//!
//! // Create an invalid credentials error
//! let err = AuthError::InvalidCredentials;
//!
//! // Create a permission denied error
//! let err = AuthError::PermissionDenied("delete project".to_string());
//!
//! // Create a session expired error
//! let err = AuthError::SessionExpired;
//! ```

use thiserror::Error;

/// Authentication and authorisation errors
#[derive(Error, Debug)]
pub enum AuthError {
    /// Invalid credentials provided
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User not found
    #[error("User not found")]
    UserNotFound,

    /// Session not found
    #[error("Session not found")]
    SessionNotFound,

    /// Session has expired
    #[error("Session expired")]
    SessionExpired,

    /// Invalid email format
    #[error("Invalid email: {0}")]
    InvalidEmail(String),

    /// Invalid username format
    #[error("Invalid username: {0}")]
    InvalidUsername(String),

    /// User already exists
    #[error("User already exists")]
    UserAlreadyExists,

    /// Permission denied for specific action
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Not authorised to access resource
    #[error("Not authorised to access resource")]
    Unauthorised,

    /// Invalid role specified
    #[error("Invalid role: {0}")]
    InvalidRole(String),

    /// Database operation failed
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    /// Account is deactivated
    #[error("Account is deactivated")]
    AccountDeactivated,

    /// Invalid token
    #[error("Invalid authentication token")]
    InvalidToken,

    /// Token has expired
    #[error("Authentication token has expired")]
    TokenExpired,

    /// Missing authentication
    #[error("Authentication required")]
    AuthenticationRequired,

    /// Password does not meet requirements
    #[error("Password does not meet requirements: {0}")]
    WeakPassword(String),

    /// Email verification required
    #[error("Email verification required")]
    EmailVerificationRequired,

    /// Invalid password reset token
    #[error("Invalid or expired password reset token")]
    InvalidResetToken,

    /// Too many authentication attempts
    #[error("Too many authentication attempts. Please try again later")]
    TooManyAttempts,

    /// Insufficient permissions for operation
    #[error("Insufficient permissions to {0}")]
    InsufficientPermissions(String),
}

impl AuthError {
    /// Check if this is an authentication error (401)
    pub fn is_authentication_error(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidCredentials
                | AuthError::SessionExpired
                | AuthError::SessionNotFound
                | AuthError::InvalidToken
                | AuthError::TokenExpired
                | AuthError::AuthenticationRequired
        )
    }

    /// Check if this is an authorisation error (403)
    pub fn is_authorisation_error(&self) -> bool {
        matches!(
            self,
            AuthError::PermissionDenied(_)
                | AuthError::Unauthorised
                | AuthError::InsufficientPermissions(_)
                | AuthError::AccountDeactivated
                | AuthError::EmailVerificationRequired
        )
    }

    /// Check if this is a validation error (400)
    pub fn is_validation_error(&self) -> bool {
        matches!(
            self,
            AuthError::InvalidEmail(_)
                | AuthError::InvalidUsername(_)
                | AuthError::InvalidRole(_)
                | AuthError::WeakPassword(_)
                | AuthError::InvalidResetToken
        )
    }

    /// Check if this is a not found error (404)
    pub fn is_not_found(&self) -> bool {
        matches!(self, AuthError::UserNotFound)
    }

    /// Check if this is a conflict error (409)
    pub fn is_conflict(&self) -> bool {
        matches!(self, AuthError::UserAlreadyExists)
    }

    /// Get HTTP status code for this error
    pub fn http_status_code(&self) -> u16 {
        match self {
            AuthError::InvalidCredentials
            | AuthError::SessionExpired
            | AuthError::SessionNotFound
            | AuthError::InvalidToken
            | AuthError::TokenExpired
            | AuthError::AuthenticationRequired => 401, // Unauthorised
            AuthError::PermissionDenied(_)
            | AuthError::Unauthorised
            | AuthError::InsufficientPermissions(_)
            | AuthError::AccountDeactivated
            | AuthError::EmailVerificationRequired => 403, // Forbidden
            AuthError::UserNotFound => 404, // Not Found
            AuthError::InvalidEmail(_)
            | AuthError::InvalidUsername(_)
            | AuthError::InvalidRole(_)
            | AuthError::WeakPassword(_)
            | AuthError::InvalidResetToken => 400, // Bad Request
            AuthError::UserAlreadyExists => 409, // Conflict
            AuthError::TooManyAttempts => 429, // Too Many Requests
            AuthError::Database(_) => 500,  // Internal Server Error
        }
    }

    /// Get error code for GraphQL/API responses
    pub fn error_code(&self) -> &'static str {
        match self {
            AuthError::InvalidCredentials => "INVALID_CREDENTIALS",
            AuthError::UserNotFound => "USER_NOT_FOUND",
            AuthError::SessionNotFound => "SESSION_NOT_FOUND",
            AuthError::SessionExpired => "SESSION_EXPIRED",
            AuthError::InvalidEmail(_) => "INVALID_EMAIL",
            AuthError::InvalidUsername(_) => "INVALID_USERNAME",
            AuthError::UserAlreadyExists => "USER_ALREADY_EXISTS",
            AuthError::PermissionDenied(_) => "PERMISSION_DENIED",
            AuthError::Unauthorised => "UNAUTHORISED",
            AuthError::InvalidRole(_) => "INVALID_ROLE",
            AuthError::Database(_) => "DATABASE_ERROR",
            AuthError::AccountDeactivated => "ACCOUNT_DEACTIVATED",
            AuthError::InvalidToken => "INVALID_TOKEN",
            AuthError::TokenExpired => "TOKEN_EXPIRED",
            AuthError::AuthenticationRequired => "AUTHENTICATION_REQUIRED",
            AuthError::WeakPassword(_) => "WEAK_PASSWORD",
            AuthError::EmailVerificationRequired => "EMAIL_VERIFICATION_REQUIRED",
            AuthError::InvalidResetToken => "INVALID_RESET_TOKEN",
            AuthError::TooManyAttempts => "TOO_MANY_ATTEMPTS",
            AuthError::InsufficientPermissions(_) => "INSUFFICIENT_PERMISSIONS",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_credentials() {
        let err = AuthError::InvalidCredentials;
        assert_eq!(err.to_string(), "Invalid credentials");
        assert!(err.is_authentication_error());
        assert_eq!(err.http_status_code(), 401);
        assert_eq!(err.error_code(), "INVALID_CREDENTIALS");
    }

    #[test]
    fn test_permission_denied() {
        let err = AuthError::PermissionDenied("delete project".to_string());
        assert_eq!(err.to_string(), "Permission denied: delete project");
        assert!(err.is_authorisation_error());
        assert_eq!(err.http_status_code(), 403);
        assert_eq!(err.error_code(), "PERMISSION_DENIED");
    }

    #[test]
    fn test_session_expired() {
        let err = AuthError::SessionExpired;
        assert_eq!(err.to_string(), "Session expired");
        assert!(err.is_authentication_error());
        assert_eq!(err.http_status_code(), 401);
        assert_eq!(err.error_code(), "SESSION_EXPIRED");
    }

    #[test]
    fn test_user_not_found() {
        let err = AuthError::UserNotFound;
        assert_eq!(err.to_string(), "User not found");
        assert!(err.is_not_found());
        assert_eq!(err.http_status_code(), 404);
        assert_eq!(err.error_code(), "USER_NOT_FOUND");
    }

    #[test]
    fn test_user_already_exists() {
        let err = AuthError::UserAlreadyExists;
        assert_eq!(err.to_string(), "User already exists");
        assert!(err.is_conflict());
        assert_eq!(err.http_status_code(), 409);
        assert_eq!(err.error_code(), "USER_ALREADY_EXISTS");
    }

    #[test]
    fn test_invalid_email() {
        let err = AuthError::InvalidEmail("not-an-email".to_string());
        assert_eq!(err.to_string(), "Invalid email: not-an-email");
        assert!(err.is_validation_error());
        assert_eq!(err.http_status_code(), 400);
        assert_eq!(err.error_code(), "INVALID_EMAIL");
    }

    #[test]
    fn test_too_many_attempts() {
        let err = AuthError::TooManyAttempts;
        assert_eq!(
            err.to_string(),
            "Too many authentication attempts. Please try again later"
        );
        assert_eq!(err.http_status_code(), 429);
        assert_eq!(err.error_code(), "TOO_MANY_ATTEMPTS");
    }
}
