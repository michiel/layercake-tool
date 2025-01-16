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
        unimplemented!()
    }
}

impl Edge {
    pub fn from_row(row: &Row) -> Self {
        unimplemented!()
    }
}
