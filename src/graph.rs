use polars::frame::row::Row;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Default for Graph {
    fn default() -> Self {
        Graph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub is_container: bool,
    pub belongs_to: String,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub comment: Option<String>,
}

impl Node {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            id: row.0.get(0).ok_or("Missing id")?.to_string(),
            label: row.0.get(1).ok_or("Missing label")?.to_string(),
            layer: row.0.get(2).ok_or("Missing layer")?.to_string(),
            is_container: row.0.get(3).ok_or("Missing is_container")?.to_string() == "true",
            belongs_to: row.0.get(4).ok_or("Missing belongs_to")?.to_string(),
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}

impl Edge {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Edge {
            id: row.0.get(0).ok_or("Missing id")?.to_string(),
            source: row.0.get(1).ok_or("Missing source")?.to_string(),
            target: row.0.get(2).ok_or("Missing target")?.to_string(),
            label: row.0.get(3).ok_or("Missing label")?.to_string(),
            layer: row.0.get(4).ok_or("Missing layer")?.to_string(),
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}
