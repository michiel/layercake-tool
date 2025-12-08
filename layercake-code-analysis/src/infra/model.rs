use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    Aws(String),
    Azure(String),
    Gcp(String),
    Other(String),
}

impl ResourceType {
    pub fn from_raw(raw: &str) -> Self {
        let lowered = raw.to_ascii_lowercase();
        if lowered.starts_with("aws") {
            ResourceType::Aws(raw.to_string())
        } else if lowered.starts_with("azure") {
            ResourceType::Azure(raw.to_string())
        } else if lowered.starts_with("gcp") {
            ResourceType::Gcp(raw.to_string())
        } else {
            ResourceType::Other(raw.to_string())
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNode {
    pub id: String,
    pub resource_type: ResourceType,
    pub name: String,
    pub source_file: String,
    pub properties: HashMap<String, String>,
    pub belongs_to: Option<String>,
}

impl ResourceNode {
    pub fn new(
        id: impl Into<String>,
        resource_type: ResourceType,
        name: impl Into<String>,
        source_file: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            resource_type,
            name: name.into(),
            source_file: source_file.into(),
            properties: HashMap::new(),
            belongs_to: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    DependsOn,
    References,
    CodeLink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CorrelationMatch {
    pub code_node: String,
    pub infra_node: String,
    pub reason: String,
    pub confidence: u8,
}
