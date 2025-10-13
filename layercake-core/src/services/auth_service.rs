#![allow(dead_code)]

use anyhow::{anyhow, Result};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

/// Service for handling authentication operations
#[allow(dead_code)] // Authentication service reserved for future use
#[derive(Clone)]
pub struct AuthService {
    db: DatabaseConnection,
}

impl AuthService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String> {
        if password.is_empty() {
            return Err(anyhow!("Password cannot be empty"));
        }

        if password.len() < 8 {
            return Err(anyhow!("Password must be at least 8 characters long"));
        }

        hash(password, DEFAULT_COST).map_err(|e| anyhow!("Failed to hash password: {}", e))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        verify(password, hash).map_err(|e| anyhow!("Failed to verify password: {}", e))
    }

    /// Generate a secure session ID
    pub fn generate_session_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Calculate session expiration time (24 hours from now)
    pub fn calculate_session_expiry() -> chrono::DateTime<Utc> {
        Utc::now() + Duration::hours(24)
    }

    /// Validate email format
    pub fn validate_email(email: &str) -> Result<()> {
        if email.is_empty() {
            return Err(anyhow!("Email cannot be empty"));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid email format: must contain exactly one @"));
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        if local_part.is_empty() {
            return Err(anyhow!("Invalid email format: local part cannot be empty"));
        }

        if domain_part.is_empty() {
            return Err(anyhow!("Invalid email format: domain part cannot be empty"));
        }

        if !domain_part.contains('.') {
            return Err(anyhow!("Invalid email format: domain must contain a dot"));
        }

        if domain_part.starts_with('.') || domain_part.ends_with('.') {
            return Err(anyhow!(
                "Invalid email format: domain cannot start or end with a dot"
            ));
        }

        // Check for reasonable length
        if email.len() > 254 {
            return Err(anyhow!("Email is too long"));
        }

        Ok(())
    }

    /// Validate username format
    pub fn validate_username(username: &str) -> Result<()> {
        if username.is_empty() {
            return Err(anyhow!("Username cannot be empty"));
        }

        if username.len() < 3 {
            return Err(anyhow!("Username must be at least 3 characters long"));
        }

        if username.len() > 50 {
            return Err(anyhow!("Username is too long (max 50 characters)"));
        }

        // Check for valid characters (alphanumeric, underscore, hyphen)
        if !username
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(anyhow!(
                "Username can only contain letters, numbers, underscores, and hyphens"
            ));
        }

        Ok(())
    }

    /// Validate display name
    pub fn validate_display_name(display_name: &str) -> Result<()> {
        if display_name.is_empty() {
            return Err(anyhow!("Display name cannot be empty"));
        }

        if display_name.len() > 100 {
            return Err(anyhow!("Display name is too long (max 100 characters)"));
        }

        // Trim whitespace and check if still valid
        let trimmed = display_name.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("Display name cannot be only whitespace"));
        }

        Ok(())
    }

    /// Check if session is expired
    pub fn is_session_expired(expires_at: chrono::DateTime<Utc>) -> bool {
        Utc::now() > expires_at
    }

    /// Generate a random avatar color
    pub fn generate_avatar_color() -> String {
        let colors = [
            "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
            "#BB8FCE", "#85C1E9", "#F8C471", "#82E0AA", "#F1948A", "#85C1E9", "#F4D03F",
        ];

        let index = (Uuid::new_v4().as_u128() % colors.len() as u128) as usize;
        colors[index].to_string()
    }
}

/// Authentication-related errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Session expired")]
    SessionExpired,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Email already exists")]
    EmailExists,

    #[error("Username already exists")]
    UsernameExists,

    #[error("Account is deactivated")]
    AccountDeactivated,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<sea_orm::DbErr> for AuthError {
    fn from(err: sea_orm::DbErr) -> Self {
        AuthError::DatabaseError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test_password_123";
        let hash = AuthService::hash_password(password).expect("Failed to hash password");

        assert!(AuthService::verify_password(password, &hash).expect("Failed to verify password"));
        assert!(!AuthService::verify_password("wrong_password", &hash)
            .expect("Failed to verify wrong password"));
    }

    #[test]
    fn test_password_validation() {
        // Too short
        assert!(AuthService::hash_password("short").is_err());

        // Empty
        assert!(AuthService::hash_password("").is_err());

        // Valid
        assert!(AuthService::hash_password("valid_password").is_ok());
    }

    #[test]
    fn test_email_validation() {
        // Valid emails
        assert!(AuthService::validate_email("test@example.com").is_ok());
        assert!(AuthService::validate_email("user.name+tag@domain.co.uk").is_ok());

        // Invalid emails
        assert!(AuthService::validate_email("").is_err());
        assert!(AuthService::validate_email("notanemail").is_err());
        assert!(AuthService::validate_email("@example.com").is_err());
        assert!(AuthService::validate_email("test@").is_err());
    }

    #[test]
    fn test_username_validation() {
        // Valid usernames
        assert!(AuthService::validate_username("valid_user").is_ok());
        assert!(AuthService::validate_username("user123").is_ok());
        assert!(AuthService::validate_username("user-name").is_ok());

        // Invalid usernames
        assert!(AuthService::validate_username("").is_err());
        assert!(AuthService::validate_username("ab").is_err()); // Too short
        assert!(AuthService::validate_username("user@name").is_err()); // Invalid character
        assert!(AuthService::validate_username("user name").is_err()); // Space
    }

    #[test]
    fn test_session_expiry() {
        let future_time = Utc::now() + Duration::hours(1);
        let past_time = Utc::now() - Duration::hours(1);

        assert!(!AuthService::is_session_expired(future_time));
        assert!(AuthService::is_session_expired(past_time));
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = AuthService::generate_session_id();
        let id2 = AuthService::generate_session_id();

        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_avatar_color_generation() {
        let color1 = AuthService::generate_avatar_color();
        let color2 = AuthService::generate_avatar_color();

        assert!(color1.starts_with('#'));
        assert!(color2.starts_with('#'));
        assert_eq!(color1.len(), 7); // #RRGGBB format
    }
}
