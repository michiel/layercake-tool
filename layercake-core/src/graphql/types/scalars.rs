// Re-export commonly used types that are already supported by async-graphql

// Define JSON as serde_json::Value for GraphQL usage
pub type JSON = serde_json::Value;
