use serde::{Deserialize, Serialize};

/// Scope of a tag. Stored as string for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagScope {
    File,
    Dataset,
    GraphNode,
    GraphEdge,
}

impl TagScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagScope::File => "file",
            TagScope::Dataset => "dataset",
            TagScope::GraphNode => "graph_node",
            TagScope::GraphEdge => "graph_edge",
        }
    }
}

impl AsRef<str> for TagScope {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for TagScope {
    fn from(value: &str) -> Self {
        match value {
            "dataset" => TagScope::Dataset,
            "graph_node" => TagScope::GraphNode,
            "graph_edge" => TagScope::GraphEdge,
            _ => TagScope::File,
        }
    }
}
