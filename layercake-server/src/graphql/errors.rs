use async_graphql::*;

/// Error codes for structured error handling
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ErrorCode {
    /// Resource not found (404-equivalent)
    NotFound,
    /// Unauthorized access (401-equivalent)
    Unauthorized,
    /// Forbidden access (403-equivalent)
    Forbidden,
    /// Validation failed (400-equivalent)
    ValidationFailed,
    /// Database operation failed
    DatabaseError,
    /// External service error
    ServiceError,
    /// Internal server error
    InternalError,
    /// Conflict (409-equivalent)
    Conflict,
    /// Bad request (400-equivalent)
    BadRequest,
}

/// Structured error builder for consistent error handling
pub struct StructuredError;

impl StructuredError {
    /// Create a "not found" error
    pub fn not_found(resource: &str, id: impl std::fmt::Display) -> Error {
        Error::new(format!("{} with id '{}' not found", resource, id)).extend_with(|_, e| {
            e.set("code", "NOT_FOUND");
            e.set("resource", resource);
        })
    }

    /// Create a "not found" error with custom message
    #[allow(dead_code)]
    pub fn not_found_msg(message: impl Into<String>) -> Error {
        Error::new(message.into()).extend_with(|_, e| {
            e.set("code", "NOT_FOUND");
        })
    }

    /// Create an "unauthorized" error
    pub fn unauthorized(message: impl Into<String>) -> Error {
        Error::new(message.into()).extend_with(|_, e| {
            e.set("code", "UNAUTHORIZED");
        })
    }

    /// Create a "forbidden" error
    pub fn forbidden(message: impl Into<String>) -> Error {
        Error::new(message.into()).extend_with(|_, e| {
            e.set("code", "FORBIDDEN");
        })
    }

    /// Create a "validation failed" error
    pub fn validation(field: &str, message: impl Into<String>) -> Error {
        Error::new(format!(
            "Validation failed for '{}': {}",
            field,
            message.into()
        ))
        .extend_with(|_, e| {
            e.set("code", "VALIDATION_FAILED");
            e.set("field", field);
        })
    }

    /// Create a "database error"
    pub fn database(operation: &str, cause: impl std::fmt::Display) -> Error {
        Error::new(format!("Database error during {}: {}", operation, cause)).extend_with(|_, e| {
            e.set("code", "DATABASE_ERROR");
            e.set("operation", operation);
        })
    }

    /// Create a "service error"
    pub fn service(service: &str, cause: impl std::fmt::Display) -> Error {
        Error::new(format!("Service '{}' error: {}", service, cause)).extend_with(|_, e| {
            e.set("code", "SERVICE_ERROR");
            e.set("service", service);
        })
    }

    /// Create an "internal error"
    pub fn internal(message: impl Into<String>) -> Error {
        Error::new(message.into()).extend_with(|_, e| {
            e.set("code", "INTERNAL_ERROR");
        })
    }

    /// Create a "conflict" error
    pub fn conflict(resource: &str, message: impl Into<String>) -> Error {
        Error::new(format!("{}: {}", resource, message.into())).extend_with(|_, e| {
            e.set("code", "CONFLICT");
            e.set("resource", resource);
        })
    }

    /// Create a "bad request" error
    pub fn bad_request(message: impl Into<String>) -> Error {
        Error::new(message.into()).extend_with(|_, e| {
            e.set("code", "BAD_REQUEST");
        })
    }
}

/// Extension trait for Result to add context
#[allow(dead_code)]
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, message: impl Into<String>) -> Result<T>;

    /// Add context with format string
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E: std::fmt::Display> ResultExt<T> for std::result::Result<T, E> {
    fn context(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|e| Error::new(format!("{}: {}", message.into(), e)))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| Error::new(format!("{}: {}", f(), e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let error = StructuredError::not_found("Project", 42);
        assert!(error.message.contains("Project"));
        assert!(error.message.contains("42"));
    }

    #[test]
    fn test_validation_error() {
        let error = StructuredError::validation("email", "Invalid format");
        assert!(error.message.contains("email"));
        assert!(error.message.contains("Invalid format"));
    }
}
