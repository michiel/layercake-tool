use serde::de::DeserializeOwned;
use axum::{
    extract::Request,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    pub field: Option<String>,
    pub message: String,
}

impl ValidationError {
    pub fn field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: Some(field.into()),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(field) = &self.field {
            write!(f, "{}: {}", field, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ValidationError {}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let error_response = serde_json::json!({
            "error": "validation_failed",
            "message": self.message,
            "field": self.field
        });

        (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
    }
}


// Project validation structures
#[derive(serde::Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

impl Validate for CreateProjectRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.name.trim().is_empty() {
            return Err(ValidationError::field("name", "Project name cannot be empty"));
        }
        if self.name.len() > 100 {
            return Err(ValidationError::field("name", "Project name too long (max 100 chars)"));
        }
        if self.name.chars().any(|c| c.is_control()) {
            return Err(ValidationError::field("name", "Project name cannot contain control characters"));
        }

        if let Some(description) = &self.description {
            if description.len() > 1000 {
                return Err(ValidationError::field("description", "Description too long (max 1000 chars)"));
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Validate for UpdateProjectRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err(ValidationError::field("name", "Project name cannot be empty"));
            }
            if name.len() > 100 {
                return Err(ValidationError::field("name", "Project name too long (max 100 chars)"));
            }
            if name.chars().any(|c| c.is_control()) {
                return Err(ValidationError::field("name", "Project name cannot contain control characters"));
            }
        }

        if let Some(description) = &self.description {
            if description.len() > 1000 {
                return Err(ValidationError::field("description", "Description too long (max 1000 chars)"));
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct NodeRequest {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub metadata: Option<Value>,
}

impl Validate for NodeRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::field("id", "Node ID cannot be empty"));
        }
        if self.id.len() > 50 {
            return Err(ValidationError::field("id", "Node ID too long (max 50 chars)"));
        }
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::field("id", "Node ID can only contain alphanumeric characters, hyphens, and underscores"));
        }

        if self.label.trim().is_empty() {
            return Err(ValidationError::field("label", "Node label cannot be empty"));
        }
        if self.label.len() > 200 {
            return Err(ValidationError::field("label", "Node label too long (max 200 chars)"));
        }

        if self.node_type.trim().is_empty() {
            return Err(ValidationError::field("node_type", "Node type cannot be empty"));
        }

        // Validate that node_type is one of the allowed types
        let allowed_types = ["data_source", "transform", "merge", "copy", "output", "graph"];
        if !allowed_types.contains(&self.node_type.as_str()) {
            return Err(ValidationError::field("node_type", "Invalid node type"));
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct EdgeRequest {
    pub id: String,
    pub source: String,
    pub target: String,
    pub connection_type: Option<String>,
}

impl Validate for EdgeRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::field("id", "Edge ID cannot be empty"));
        }
        if self.id.len() > 50 {
            return Err(ValidationError::field("id", "Edge ID too long (max 50 chars)"));
        }
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ValidationError::field("id", "Edge ID can only contain alphanumeric characters, hyphens, and underscores"));
        }

        if self.source.trim().is_empty() {
            return Err(ValidationError::field("source", "Source node ID cannot be empty"));
        }
        if self.target.trim().is_empty() {
            return Err(ValidationError::field("target", "Target node ID cannot be empty"));
        }

        if self.source == self.target {
            return Err(ValidationError::field("target", "Source and target cannot be the same node"));
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_validation() {
        // Valid project
        let project = CreateProjectRequest {
            name: "Test Project".to_string(),
            description: Some("A test project".to_string()),
        };
        assert!(project.validate().is_ok());

        // Empty name
        let project = CreateProjectRequest {
            name: "".to_string(),
            description: None,
        };
        assert!(project.validate().is_err());

        // Name too long
        let project = CreateProjectRequest {
            name: "x".repeat(101),
            description: None,
        };
        assert!(project.validate().is_err());

        // Description too long
        let project = CreateProjectRequest {
            name: "Test".to_string(),
            description: Some("x".repeat(1001)),
        };
        assert!(project.validate().is_err());
    }

    #[test]
    fn test_node_validation() {
        // Valid node
        let node = NodeRequest {
            id: "node-1".to_string(),
            label: "Test Node".to_string(),
            node_type: "data_source".to_string(),
            metadata: None,
        };
        assert!(node.validate().is_ok());

        // Invalid node type
        let node = NodeRequest {
            id: "node-1".to_string(),
            label: "Test Node".to_string(),
            node_type: "invalid_type".to_string(),
            metadata: None,
        };
        assert!(node.validate().is_err());

        // Invalid ID characters
        let node = NodeRequest {
            id: "node@1".to_string(),
            label: "Test Node".to_string(),
            node_type: "data_source".to_string(),
            metadata: None,
        };
        assert!(node.validate().is_err());
    }

    #[test]
    fn test_edge_validation() {
        // Valid edge
        let edge = EdgeRequest {
            id: "edge-1".to_string(),
            source: "node-1".to_string(),
            target: "node-2".to_string(),
            connection_type: None,
        };
        assert!(edge.validate().is_ok());

        // Self-referencing edge
        let edge = EdgeRequest {
            id: "edge-1".to_string(),
            source: "node-1".to_string(),
            target: "node-1".to_string(),
            connection_type: None,
        };
        assert!(edge.validate().is_err());
    }

}