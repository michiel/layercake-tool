//! Database error categorization and message formatting
//!
//! This module provides utilities for categorizing and formatting database errors
//! in a consistent way across both GraphQL and MCP APIs.
//!
//! # Examples
//!
//! ```rust
//! use layercake::common::db_errors::*;
//! use sea_orm::DbErr;
//!
//! // Categorize a database error
//! let kind = DbErrorKind::from_db_err(&db_err);
//!
//! // Format error with context
//! let (kind, message) = format_db_error("create user", &db_err);
//!
//! // Check if error is retryable
//! if kind.is_retryable() {
//!     // Retry the operation
//! }
//! ```

use sea_orm::DbErr;

/// Categories of database errors
///
/// This enum categorizes database errors into common types that can be
/// handled appropriately by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbErrorKind {
    /// Record not found (query returned no results)
    ///
    /// Typically indicates a 404 Not Found error for REST/GraphQL APIs.
    NotFound,

    /// Unique constraint violation
    ///
    /// Typically indicates a 409 Conflict error - resource already exists.
    UniqueViolation,

    /// Foreign key constraint violation
    ///
    /// Typically indicates a 400 Bad Request - invalid reference.
    ForeignKeyViolation,

    /// Database connection error
    ///
    /// Typically indicates a 503 Service Unavailable error.
    ConnectionError,

    /// Query timeout
    ///
    /// Typically indicates a 504 Gateway Timeout error.
    Timeout,

    /// Transaction deadlock
    ///
    /// Typically indicates a 503 Service Unavailable error (should retry).
    Deadlock,

    /// Unknown/other database error
    ///
    /// Typically indicates a 500 Internal Server Error.
    Unknown,
}

impl DbErrorKind {
    /// Categorize a sea_orm database error
    ///
    /// Analyzes a `DbErr` and categorizes it into one of the defined error kinds.
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::db_errors::DbErrorKind;
    /// use sea_orm::DbErr;
    ///
    /// let err = DbErr::RecordNotFound("User not found".to_string());
    /// let kind = DbErrorKind::from_db_err(&err);
    /// assert_eq!(kind, DbErrorKind::NotFound);
    /// ```
    pub fn from_db_err(err: &DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => Self::NotFound,
            DbErr::Conn(msg) if msg.to_lowercase().contains("timeout") => Self::Timeout,
            DbErr::Conn(_) => Self::ConnectionError,
            DbErr::Exec(msg) | DbErr::Query(msg) => {
                let msg_lower = msg.to_lowercase();
                if msg_lower.contains("unique") || msg_lower.contains("duplicate") {
                    Self::UniqueViolation
                } else if msg_lower.contains("foreign key") || msg_lower.contains("fk_") {
                    Self::ForeignKeyViolation
                } else if msg_lower.contains("deadlock") {
                    Self::Deadlock
                } else if msg_lower.contains("timeout") {
                    Self::Timeout
                } else {
                    Self::Unknown
                }
            }
            _ => Self::Unknown,
        }
    }

    /// Get appropriate HTTP status code for this error kind
    ///
    /// Maps database error kinds to standard HTTP status codes.
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::db_errors::DbErrorKind;
    ///
    /// assert_eq!(DbErrorKind::NotFound.http_status_code(), 404);
    /// assert_eq!(DbErrorKind::UniqueViolation.http_status_code(), 409);
    /// assert_eq!(DbErrorKind::ConnectionError.http_status_code(), 503);
    /// ```
    pub fn http_status_code(&self) -> u16 {
        match self {
            Self::NotFound => 404,                  // Not Found
            Self::UniqueViolation => 409,           // Conflict
            Self::ForeignKeyViolation => 400,       // Bad Request
            Self::ConnectionError => 503,           // Service Unavailable
            Self::Timeout => 504,                   // Gateway Timeout
            Self::Deadlock => 503,                  // Service Unavailable (retry)
            Self::Unknown => 500,                   // Internal Server Error
        }
    }

    /// Check if this error is retryable
    ///
    /// Returns `true` for transient errors that might succeed on retry.
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::db_errors::DbErrorKind;
    ///
    /// assert!(DbErrorKind::ConnectionError.is_retryable());
    /// assert!(DbErrorKind::Timeout.is_retryable());
    /// assert!(DbErrorKind::Deadlock.is_retryable());
    /// assert!(!DbErrorKind::UniqueViolation.is_retryable());
    /// assert!(!DbErrorKind::NotFound.is_retryable());
    /// ```
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionError | Self::Timeout | Self::Deadlock
        )
    }

    /// Check if this is a client error (4xx)
    ///
    /// Returns `true` for errors caused by client input.
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::db_errors::DbErrorKind;
    ///
    /// assert!(DbErrorKind::NotFound.is_client_error());
    /// assert!(DbErrorKind::ForeignKeyViolation.is_client_error());
    /// assert!(!DbErrorKind::ConnectionError.is_client_error());
    /// ```
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound | Self::UniqueViolation | Self::ForeignKeyViolation
        )
    }

    /// Check if this is a server error (5xx)
    ///
    /// Returns `true` for server-side errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use layercake::common::db_errors::DbErrorKind;
    ///
    /// assert!(DbErrorKind::ConnectionError.is_server_error());
    /// assert!(DbErrorKind::Timeout.is_server_error());
    /// assert!(!DbErrorKind::NotFound.is_server_error());
    /// ```
    pub fn is_server_error(&self) -> bool {
        !self.is_client_error()
    }
}

/// Format database error with operation context
///
/// Analyzes a database error, categorizes it, and creates a human-readable
/// error message with context about the operation that failed.
///
/// # Arguments
///
/// * `operation` - Description of the database operation (e.g., "create user", "find project")
/// * `err` - The database error to format
///
/// # Returns
///
/// A tuple of `(DbErrorKind, String)` where:
/// - `DbErrorKind` is the categorized error type
/// - `String` is the formatted error message
///
/// # Examples
///
/// ```
/// use layercake::common::db_errors::*;
/// use sea_orm::DbErr;
///
/// let err = DbErr::RecordNotFound("User not found".to_string());
/// let (kind, message) = format_db_error("find user by email", &err);
///
/// assert_eq!(kind, DbErrorKind::NotFound);
/// assert_eq!(message, "find user by email: record not found");
/// ```
pub fn format_db_error(operation: &str, err: &DbErr) -> (DbErrorKind, String) {
    let kind = DbErrorKind::from_db_err(err);

    let message = match kind {
        DbErrorKind::NotFound => format!("{}: record not found", operation),
        DbErrorKind::UniqueViolation => format!("{}: duplicate key violation", operation),
        DbErrorKind::ForeignKeyViolation => {
            format!("{}: foreign key constraint violation", operation)
        }
        DbErrorKind::ConnectionError => format!("{}: database connection failed", operation),
        DbErrorKind::Timeout => format!("{}: query timeout", operation),
        DbErrorKind::Deadlock => format!("{}: transaction deadlock", operation),
        DbErrorKind::Unknown => format!("{}: database error - {}", operation, err),
    };

    (kind, message)
}

/// Format database error with detailed context
///
/// Similar to `format_db_error` but includes the original error details
/// for debugging purposes.
///
/// # Examples
///
/// ```
/// use layercake::common::db_errors::*;
/// use sea_orm::DbErr;
///
/// let err = DbErr::Conn("connection timeout after 30s".to_string());
/// let (kind, message) = format_db_error_detailed("connect to database", &err);
///
/// assert_eq!(kind, DbErrorKind::Timeout);
/// assert!(message.contains("connection timeout"));
/// ```
pub fn format_db_error_detailed(operation: &str, err: &DbErr) -> (DbErrorKind, String) {
    let kind = DbErrorKind::from_db_err(err);

    let message = match kind {
        DbErrorKind::NotFound => format!("{}: record not found - {}", operation, err),
        DbErrorKind::UniqueViolation => {
            format!("{}: duplicate key violation - {}", operation, err)
        }
        DbErrorKind::ForeignKeyViolation => {
            format!("{}: foreign key constraint violation - {}", operation, err)
        }
        DbErrorKind::ConnectionError => {
            format!("{}: database connection failed - {}", operation, err)
        }
        DbErrorKind::Timeout => format!("{}: query timeout - {}", operation, err),
        DbErrorKind::Deadlock => format!("{}: transaction deadlock - {}", operation, err),
        DbErrorKind::Unknown => format!("{}: database error - {}", operation, err),
    };

    (kind, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_record_not_found() {
        let err = DbErr::RecordNotFound("User not found".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::NotFound);
        assert_eq!(kind.http_status_code(), 404);
        assert!(!kind.is_retryable());
    }

    #[test]
    fn test_categorize_connection_error() {
        let err = DbErr::Conn("Connection refused".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::ConnectionError);
        assert_eq!(kind.http_status_code(), 503);
        assert!(kind.is_retryable());
    }

    #[test]
    fn test_categorize_timeout() {
        let err = DbErr::Conn("connection timeout after 30s".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::Timeout);
        assert_eq!(kind.http_status_code(), 504);
        assert!(kind.is_retryable());
    }

    #[test]
    fn test_categorize_unique_violation() {
        let err = DbErr::Query("UNIQUE constraint failed: users.email".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::UniqueViolation);
        assert_eq!(kind.http_status_code(), 409);
        assert!(!kind.is_retryable());
    }

    #[test]
    fn test_categorize_foreign_key_violation() {
        let err = DbErr::Exec("FOREIGN KEY constraint failed".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::ForeignKeyViolation);
        assert_eq!(kind.http_status_code(), 400);
        assert!(!kind.is_retryable());
    }

    #[test]
    fn test_categorize_deadlock() {
        let err = DbErr::Query("Deadlock detected".to_string());
        let kind = DbErrorKind::from_db_err(&err);
        assert_eq!(kind, DbErrorKind::Deadlock);
        assert_eq!(kind.http_status_code(), 503);
        assert!(kind.is_retryable());
    }

    #[test]
    fn test_format_db_error() {
        let err = DbErr::RecordNotFound("User not found".to_string());
        let (kind, message) = format_db_error("find user by email", &err);

        assert_eq!(kind, DbErrorKind::NotFound);
        assert_eq!(message, "find user by email: record not found");
    }

    #[test]
    fn test_format_db_error_unique_violation() {
        let err = DbErr::Query("UNIQUE constraint failed".to_string());
        let (kind, message) = format_db_error("create user", &err);

        assert_eq!(kind, DbErrorKind::UniqueViolation);
        assert_eq!(message, "create user: duplicate key violation");
    }

    #[test]
    fn test_is_client_error() {
        assert!(DbErrorKind::NotFound.is_client_error());
        assert!(DbErrorKind::UniqueViolation.is_client_error());
        assert!(DbErrorKind::ForeignKeyViolation.is_client_error());
        assert!(!DbErrorKind::ConnectionError.is_client_error());
        assert!(!DbErrorKind::Timeout.is_client_error());
    }

    #[test]
    fn test_is_server_error() {
        assert!(!DbErrorKind::NotFound.is_server_error());
        assert!(DbErrorKind::ConnectionError.is_server_error());
        assert!(DbErrorKind::Timeout.is_server_error());
        assert!(DbErrorKind::Deadlock.is_server_error());
        assert!(DbErrorKind::Unknown.is_server_error());
    }

    #[test]
    fn test_format_db_error_detailed() {
        let err = DbErr::Conn("connection timeout after 30s".to_string());
        let (kind, message) = format_db_error_detailed("connect", &err);

        assert_eq!(kind, DbErrorKind::Timeout);
        assert!(message.contains("query timeout"));
        assert!(message.contains("connection timeout"));
    }
}
