// Re-export commonly used types that are already supported by async-graphql
pub use chrono::{DateTime, Utc};

// Define JSON as serde_json::Value for GraphQL usage
pub type JSON = serde_json::Value;