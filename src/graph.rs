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
    pub fn from_row(row: &Row) -> Self {
        Node {
            id: row.0.get(0).unwrap().to_string(),
            label: row.0.get(1).unwrap().to_string(),
            layer: row.0.get(2).unwrap().to_string(),
            is_container: row.0.get(3).unwrap().to_string() == "true",
            belongs_to: row.0.get(4).unwrap().to_string(),
            comment: row.0.get(5).map(|c| c.to_string()),
        }
    }
}

impl Edge {
    pub fn from_row(row: &Row) -> Self {
        Edge {
            id: row.0.get(0).unwrap().to_string(),
            source: row.0.get(1).unwrap().to_string(),
            target: row.0.get(2).unwrap().to_string(),
            label: row.0.get(3).unwrap().to_string(),
            layer: row.0.get(4).unwrap().to_string(),
            comment: row.0.get(5).map(|c| c.to_string()),
        }
    }
}
