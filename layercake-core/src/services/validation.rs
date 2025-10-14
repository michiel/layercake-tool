#![allow(dead_code)]

use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;

/// Service for data validation and sanitization
#[allow(dead_code)] // Validation service reserved for future use
pub struct ValidationService;

impl ValidationService {
    /// Sanitize and validate project name
    pub fn validate_project_name(name: &str) -> Result<String> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }

        if trimmed.len() > 100 {
            return Err(anyhow!("Project name is too long (max 100 characters)"));
        }

        // Remove potentially dangerous characters
        let sanitized = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_().".contains(*c))
            .collect::<String>();

        if sanitized.is_empty() {
            return Err(anyhow!("Project name contains only invalid characters"));
        }

        Ok(sanitized)
    }

    /// Validate project description
    pub fn validate_project_description(description: &str) -> Result<String> {
        let trimmed = description.trim();

        if trimmed.len() > 1000 {
            return Err(anyhow!(
                "Project description is too long (max 1000 characters)"
            ));
        }

        // Basic HTML/script injection prevention
        let sanitized = trimmed
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('&', "&amp;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;");

        Ok(sanitized)
    }

    /// Validate Plan DAG node ID
    pub fn validate_node_id(node_id: &str) -> Result<String> {
        let trimmed = node_id.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("Node ID cannot be empty"));
        }

        if trimmed.len() > 50 {
            return Err(anyhow!("Node ID is too long (max 50 characters)"));
        }

        // Node IDs should be alphanumeric with underscores
        let regex = Regex::new(r"^[a-zA-Z0-9_]+$")
            .map_err(|e| anyhow!("Failed to compile node ID regex: {}", e))?;
        if !regex.is_match(trimmed) {
            return Err(anyhow!(
                "Node ID can only contain letters, numbers, and underscores"
            ));
        }

        Ok(trimmed.to_string())
    }

    /// Validate JSON configuration
    pub fn validate_json_config(json_str: &str) -> Result<Value> {
        if json_str.trim().is_empty() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| anyhow!("Invalid JSON configuration: {}", e))?;

        // Check JSON size (prevent excessively large configs)
        let serialized = serde_json::to_string(&value)?;
        if serialized.len() > 10_000 {
            return Err(anyhow!("JSON configuration is too large (max 10KB)"));
        }

        Ok(value)
    }

    /// Validate file path (for security)
    pub fn validate_file_path(path: &str) -> Result<String> {
        let trimmed = path.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("File path cannot be empty"));
        }

        if trimmed.len() > 500 {
            return Err(anyhow!("File path is too long (max 500 characters)"));
        }

        // Prevent directory traversal
        if trimmed.contains("..") {
            return Err(anyhow!(
                "File path cannot contain '..' (directory traversal)"
            ));
        }

        // Prevent absolute paths in user input
        if trimmed.starts_with('/') || trimmed.contains(':') {
            return Err(anyhow!("Absolute file paths are not allowed"));
        }

        // Basic path sanitization
        let sanitized = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || "/-_.".contains(*c))
            .collect::<String>();

        if sanitized.is_empty() {
            return Err(anyhow!("File path contains only invalid characters"));
        }

        Ok(sanitized)
    }

    /// Validate node label
    pub fn validate_node_label(label: &str) -> Result<String> {
        let trimmed = label.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("Node label cannot be empty"));
        }

        if trimmed.len() > 200 {
            return Err(anyhow!("Node label is too long (max 200 characters)"));
        }

        // Basic sanitization
        let sanitized = trimmed
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('&', "&amp;");

        Ok(sanitized)
    }

    /// Validate layer name
    pub fn validate_layer_name(name: &str) -> Result<String> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("Layer name cannot be empty"));
        }

        if trimmed.len() > 100 {
            return Err(anyhow!("Layer name is too long (max 100 characters)"));
        }

        // Layer names should be simple identifiers
        let regex = Regex::new(r"^[a-zA-Z0-9_\-\s]+$")
            .map_err(|e| anyhow!("Failed to compile display name regex: {}", e))?;
        if !regex.is_match(trimmed) {
            return Err(anyhow!(
                "Layer name can only contain letters, numbers, spaces, underscores, and hyphens"
            ));
        }

        Ok(trimmed.to_string())
    }

    /// Validate color code (hex format)
    pub fn validate_color_code(color: &str) -> Result<String> {
        let trimmed = color.trim();

        if trimmed.is_empty() {
            return Err(anyhow!("Color code cannot be empty"));
        }

        // Must be valid hex color
        let regex = Regex::new(r"^#[0-9A-Fa-f]{6}$")
            .map_err(|e| anyhow!("Failed to compile color regex: {}", e))?;
        if !regex.is_match(trimmed) {
            return Err(anyhow!("Color must be a valid hex code (e.g., #FF0000)"));
        }

        Ok(trimmed.to_uppercase())
    }

    /// Validate session data (cursor position, viewport, etc.)
    pub fn validate_session_data(data: &str) -> Result<String> {
        if data.trim().is_empty() {
            return Ok(String::new());
        }

        // Validate as JSON
        let _: Value =
            serde_json::from_str(data).map_err(|e| anyhow!("Invalid session data JSON: {}", e))?;

        // Limit size
        if data.len() > 1_000 {
            return Err(anyhow!("Session data is too large (max 1KB)"));
        }

        Ok(data.to_string())
    }

    /// Sanitize search query
    pub fn sanitize_search_query(query: &str) -> Result<String> {
        let trimmed = query.trim();

        if trimmed.len() > 200 {
            return Err(anyhow!("Search query is too long (max 200 characters)"));
        }

        // Remove potentially dangerous characters for search
        let sanitized = trimmed
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_()[]".contains(*c))
            .collect::<String>();

        Ok(sanitized)
    }

    /// Validate collaboration role
    pub fn validate_collaboration_role(role: &str) -> Result<String> {
        let valid_roles = ["owner", "editor", "viewer"];
        let role_lower = role.trim().to_lowercase();

        if !valid_roles.contains(&role_lower.as_str()) {
            return Err(anyhow!(
                "Invalid role. Must be one of: owner, editor, viewer"
            ));
        }

        Ok(role_lower)
    }

    /// Validate invitation status
    pub fn validate_invitation_status(status: &str) -> Result<String> {
        let valid_statuses = ["pending", "accepted", "declined", "revoked"];
        let status_lower = status.trim().to_lowercase();

        if !valid_statuses.contains(&status_lower.as_str()) {
            return Err(anyhow!(
                "Invalid status. Must be one of: pending, accepted, declined, revoked"
            ));
        }

        Ok(status_lower)
    }

    /// Validate user status
    pub fn validate_user_status(status: &str) -> Result<String> {
        let valid_statuses = ["active", "idle", "away", "offline"];
        let status_lower = status.trim().to_lowercase();

        if !valid_statuses.contains(&status_lower.as_str()) {
            return Err(anyhow!(
                "Invalid user status. Must be one of: active, idle, away, offline"
            ));
        }

        Ok(status_lower)
    }

    // ===== Plan DAG Validation Methods =====

    /// Validate Plan DAG node type
    pub fn validate_plan_dag_node_type(node_type: &str) -> Result<String> {
        let valid_types = [
            "DataSourceNode",
            "GraphNode",
            "TransformNode",
            "MergeNode",
            "CopyNode",
            "OutputNode",
        ];

        if !valid_types.contains(&node_type) {
            return Err(anyhow!(
                "Invalid node type '{}'. Must be one of: {}",
                node_type,
                valid_types.join(", ")
            ));
        }

        Ok(node_type.to_string())
    }

    /// Validate Plan DAG node position
    pub fn validate_plan_dag_position(x: f64, y: f64) -> Result<(f64, f64)> {
        // Reasonable canvas bounds to prevent extreme values
        const MAX_COORDINATE: f64 = 100_000.0;
        const MIN_COORDINATE: f64 = -100_000.0;

        if !x.is_finite() || !(MIN_COORDINATE..=MAX_COORDINATE).contains(&x) {
            return Err(anyhow!(
                "Invalid X coordinate: must be between {} and {}",
                MIN_COORDINATE,
                MAX_COORDINATE
            ));
        }

        if !y.is_finite() || !(MIN_COORDINATE..=MAX_COORDINATE).contains(&y) {
            return Err(anyhow!(
                "Invalid Y coordinate: must be between {} and {}",
                MIN_COORDINATE,
                MAX_COORDINATE
            ));
        }

        Ok((x, y))
    }

    /// Validate edge doesn't create self-loop
    pub fn validate_edge_no_self_loop(source: &str, target: &str) -> Result<()> {
        if source == target {
            return Err(anyhow!(
                "Edge cannot connect a node to itself (self-loop detected)"
            ));
        }
        Ok(())
    }

    /// Validate metadata JSON for Plan DAG nodes
    pub fn validate_plan_dag_metadata(metadata_json: &str) -> Result<Value> {
        if metadata_json.trim().is_empty() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let value: Value = serde_json::from_str(metadata_json)
            .map_err(|e| anyhow!("Invalid metadata JSON: {}", e))?;

        // Check if it's an object
        if !value.is_object() {
            return Err(anyhow!("Metadata must be a JSON object"));
        }

        // Check JSON size (prevent excessively large metadata)
        let serialized = serde_json::to_string(&value)?;
        if serialized.len() > 50_000 {
            return Err(anyhow!("Metadata JSON is too large (max 50KB)"));
        }

        Ok(value)
    }

    /// Validate config JSON for Plan DAG nodes
    pub fn validate_plan_dag_config(config_json: &str) -> Result<Value> {
        if config_json.trim().is_empty() {
            return Ok(Value::Object(serde_json::Map::new()));
        }

        let value: Value =
            serde_json::from_str(config_json).map_err(|e| anyhow!("Invalid config JSON: {}", e))?;

        // Check if it's an object
        if !value.is_object() {
            return Err(anyhow!("Config must be a JSON object"));
        }

        // Check JSON size (prevent excessively large configs)
        let serialized = serde_json::to_string(&value)?;
        if serialized.len() > 100_000 {
            return Err(anyhow!("Config JSON is too large (max 100KB)"));
        }

        Ok(value)
    }

    /// Validate Plan DAG doesn't exceed reasonable limits
    pub fn validate_plan_dag_limits(node_count: usize, edge_count: usize) -> Result<()> {
        const MAX_NODES: usize = 1000;
        const MAX_EDGES: usize = 5000;

        if node_count > MAX_NODES {
            return Err(anyhow!(
                "Plan DAG has too many nodes ({} > max {})",
                node_count,
                MAX_NODES
            ));
        }

        if edge_count > MAX_EDGES {
            return Err(anyhow!(
                "Plan DAG has too many edges ({} > max {})",
                edge_count,
                MAX_EDGES
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_name_validation() {
        // Valid names
        assert!(ValidationService::validate_project_name("My Project").is_ok());
        assert!(ValidationService::validate_project_name("Test_Project-1").is_ok());

        // Invalid names
        assert!(ValidationService::validate_project_name("").is_err());
        assert!(ValidationService::validate_project_name("   ").is_err());
        assert!(ValidationService::validate_project_name(&"a".repeat(101)).is_err());
    }

    #[test]
    fn test_node_id_validation() {
        // Valid IDs
        assert!(ValidationService::validate_node_id("node_1").is_ok());
        assert!(ValidationService::validate_node_id("inputNode").is_ok());

        // Invalid IDs
        assert!(ValidationService::validate_node_id("").is_err());
        assert!(ValidationService::validate_node_id("node-with-dashes").is_err());
        assert!(ValidationService::validate_node_id("node with spaces").is_err());
    }

    #[test]
    fn test_json_validation() {
        // Valid JSON
        assert!(ValidationService::validate_json_config("{}").is_ok());
        assert!(ValidationService::validate_json_config(r#"{"key": "value"}"#).is_ok());
        assert!(ValidationService::validate_json_config("").is_ok()); // Empty is OK

        // Invalid JSON
        assert!(ValidationService::validate_json_config("{invalid}").is_err());
        assert!(ValidationService::validate_json_config("not json").is_err());
    }

    #[test]
    fn test_file_path_validation() {
        // Valid paths
        assert!(ValidationService::validate_file_path("data/file.csv").is_ok());
        assert!(ValidationService::validate_file_path("output.json").is_ok());

        // Invalid paths
        assert!(ValidationService::validate_file_path("").is_err());
        assert!(ValidationService::validate_file_path("../etc/passwd").is_err());
        assert!(ValidationService::validate_file_path("/absolute/path").is_err());
    }

    #[test]
    fn test_color_validation() {
        // Valid colors
        assert!(ValidationService::validate_color_code("#FF0000").is_ok());
        assert!(ValidationService::validate_color_code("#ff0000").is_ok());

        // Invalid colors
        assert!(ValidationService::validate_color_code("").is_err());
        assert!(ValidationService::validate_color_code("red").is_err());
        assert!(ValidationService::validate_color_code("#FF00").is_err());
        assert!(ValidationService::validate_color_code("#GGGGGG").is_err());
    }

    #[test]
    fn test_role_validation() {
        // Valid roles
        assert!(ValidationService::validate_collaboration_role("owner").is_ok());
        assert!(ValidationService::validate_collaboration_role("EDITOR").is_ok());
        assert!(ValidationService::validate_collaboration_role(" viewer ").is_ok());

        // Invalid roles
        assert!(ValidationService::validate_collaboration_role("admin").is_err());
        assert!(ValidationService::validate_collaboration_role("").is_err());
    }

    // ===== Plan DAG Validation Tests =====

    #[test]
    fn test_plan_dag_node_type_validation() {
        // Valid node types
        assert!(ValidationService::validate_plan_dag_node_type("DataSourceNode").is_ok());
        assert!(ValidationService::validate_plan_dag_node_type("GraphNode").is_ok());
        assert!(ValidationService::validate_plan_dag_node_type("TransformNode").is_ok());
        assert!(ValidationService::validate_plan_dag_node_type("MergeNode").is_ok());
        assert!(ValidationService::validate_plan_dag_node_type("CopyNode").is_ok());
        assert!(ValidationService::validate_plan_dag_node_type("OutputNode").is_ok());

        // Invalid node types
        assert!(ValidationService::validate_plan_dag_node_type("InvalidNode").is_err());
        assert!(ValidationService::validate_plan_dag_node_type("").is_err());
        assert!(ValidationService::validate_plan_dag_node_type("dataSourceNode").is_err());
        // Wrong case
    }

    #[test]
    fn test_plan_dag_position_validation() {
        // Valid positions
        assert!(ValidationService::validate_plan_dag_position(0.0, 0.0).is_ok());
        assert!(ValidationService::validate_plan_dag_position(100.0, 200.0).is_ok());
        assert!(ValidationService::validate_plan_dag_position(-100.0, -200.0).is_ok());
        assert!(ValidationService::validate_plan_dag_position(99_999.0, 99_999.0).is_ok());

        // Invalid positions - out of bounds
        assert!(ValidationService::validate_plan_dag_position(100_001.0, 0.0).is_err());
        assert!(ValidationService::validate_plan_dag_position(0.0, -100_001.0).is_err());

        // Invalid positions - non-finite
        assert!(ValidationService::validate_plan_dag_position(f64::INFINITY, 0.0).is_err());
        assert!(ValidationService::validate_plan_dag_position(0.0, f64::NAN).is_err());
    }

    #[test]
    fn test_edge_self_loop_validation() {
        // Valid edges (different nodes)
        assert!(ValidationService::validate_edge_no_self_loop("node1", "node2").is_ok());
        assert!(ValidationService::validate_edge_no_self_loop("a", "b").is_ok());

        // Invalid edges (self-loops)
        assert!(ValidationService::validate_edge_no_self_loop("node1", "node1").is_err());
        assert!(ValidationService::validate_edge_no_self_loop("same", "same").is_err());
    }

    #[test]
    fn test_plan_dag_metadata_validation() {
        // Valid metadata
        assert!(ValidationService::validate_plan_dag_metadata("{}").is_ok());
        assert!(ValidationService::validate_plan_dag_metadata(r#"{"key": "value"}"#).is_ok());
        assert!(ValidationService::validate_plan_dag_metadata("").is_ok()); // Empty is valid

        // Invalid metadata - not an object
        assert!(ValidationService::validate_plan_dag_metadata("[]").is_err());
        assert!(ValidationService::validate_plan_dag_metadata(r#""string""#).is_err());
        assert!(ValidationService::validate_plan_dag_metadata("123").is_err());

        // Invalid metadata - malformed JSON
        assert!(ValidationService::validate_plan_dag_metadata("{invalid}").is_err());

        // Invalid metadata - too large (>50KB)
        let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(50_000));
        assert!(ValidationService::validate_plan_dag_metadata(&large_json).is_err());
    }

    #[test]
    fn test_plan_dag_config_validation() {
        // Valid config
        assert!(ValidationService::validate_plan_dag_config("{}").is_ok());
        assert!(ValidationService::validate_plan_dag_config(r#"{"setting": true}"#).is_ok());
        assert!(ValidationService::validate_plan_dag_config("").is_ok()); // Empty is valid

        // Invalid config - not an object
        assert!(ValidationService::validate_plan_dag_config("[]").is_err());
        assert!(ValidationService::validate_plan_dag_config("null").is_err());

        // Invalid config - malformed JSON
        assert!(ValidationService::validate_plan_dag_config("{bad json}").is_err());

        // Invalid config - too large (>100KB)
        let large_json = format!(r#"{{"config": "{}"}}"#, "x".repeat(100_000));
        assert!(ValidationService::validate_plan_dag_config(&large_json).is_err());
    }

    #[test]
    fn test_plan_dag_limits_validation() {
        // Valid limits
        assert!(ValidationService::validate_plan_dag_limits(0, 0).is_ok());
        assert!(ValidationService::validate_plan_dag_limits(100, 500).is_ok());
        assert!(ValidationService::validate_plan_dag_limits(1000, 5000).is_ok()); // At max

        // Invalid limits - too many nodes
        assert!(ValidationService::validate_plan_dag_limits(1001, 0).is_err());
        assert!(ValidationService::validate_plan_dag_limits(10_000, 0).is_err());

        // Invalid limits - too many edges
        assert!(ValidationService::validate_plan_dag_limits(0, 5001).is_err());
        assert!(ValidationService::validate_plan_dag_limits(0, 10_000).is_err());

        // Invalid limits - both exceeded
        assert!(ValidationService::validate_plan_dag_limits(1001, 5001).is_err());
    }

    #[test]
    fn test_plan_dag_metadata_size_limits() {
        // Test exact boundary cases
        let almost_50kb = "x".repeat(49_900);
        let metadata_49kb = format!(r#"{{"data": "{}"}}"#, almost_50kb);
        assert!(ValidationService::validate_plan_dag_metadata(&metadata_49kb).is_ok());

        let over_50kb = "x".repeat(50_100);
        let metadata_over = format!(r#"{{"data": "{}"}}"#, over_50kb);
        assert!(ValidationService::validate_plan_dag_metadata(&metadata_over).is_err());
    }

    #[test]
    fn test_plan_dag_config_size_limits() {
        // Test exact boundary cases
        let almost_100kb = "x".repeat(99_900);
        let config_99kb = format!(r#"{{"data": "{}"}}"#, almost_100kb);
        assert!(ValidationService::validate_plan_dag_config(&config_99kb).is_ok());

        let over_100kb = "x".repeat(100_100);
        let config_over = format!(r#"{{"data": "{}"}}"#, over_100kb);
        assert!(ValidationService::validate_plan_dag_config(&config_over).is_err());
    }

    #[test]
    fn test_plan_dag_position_boundary_values() {
        // Test exact boundary values
        assert!(ValidationService::validate_plan_dag_position(100_000.0, 100_000.0).is_ok());
        assert!(ValidationService::validate_plan_dag_position(-100_000.0, -100_000.0).is_ok());

        // Just over boundary
        assert!(ValidationService::validate_plan_dag_position(100_000.1, 0.0).is_err());
        assert!(ValidationService::validate_plan_dag_position(0.0, -100_000.1).is_err());
    }
}
