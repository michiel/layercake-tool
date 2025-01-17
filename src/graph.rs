use polars::frame::row::Row;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub layer: String,
    pub is_container: bool,
    pub belongs_to: String,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub layer: String,
    pub comment: Option<String>,
}

fn is_truthy(s: &str) -> bool {
    let trimmed_lowercase = s.trim().to_lowercase();
    // TODO memoize
    let re = Regex::new(r"(true|y|yes)").unwrap();
    let res = re.is_match(&trimmed_lowercase);
    // println!("is_truthy({}) -> {}", trimmed_lowercase, res); // Clone, Debug print
    res
}

fn strip_quotes_and_whitespace(s: &str) -> &str {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        &trimmed[1..trimmed.len() - 1].trim()
    } else {
        trimmed
    }
}

fn get_stripped_value(
    row: &Row,
    idx: usize,
    label: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let value = row
        .0
        .get(idx)
        .ok_or(format!("Missing {}", label))?
        .to_string();
    Ok(strip_quotes_and_whitespace(&value).to_string())
}

impl Node {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Node {
            id: get_stripped_value(row, 0, "id")?,
            label: get_stripped_value(row, 1, "label")?,
            layer: get_stripped_value(row, 2, "layer")?,
            is_container: is_truthy(&get_stripped_value(row, 3, "is_container")?),
            belongs_to: get_stripped_value(row, 4, "belongs_to")?,
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}

impl Edge {
    pub fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Edge {
            id: get_stripped_value(row, 0, "id")?,
            source: get_stripped_value(row, 1, "source")?,
            target: get_stripped_value(row, 2, "target")?,
            label: get_stripped_value(row, 3, "label")?,
            layer: get_stripped_value(row, 4, "layer")?,
            comment: row.0.get(5).map(|c| c.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy("true"));
        assert!(is_truthy("True"));
        assert!(is_truthy("TRUE"));
        assert!(is_truthy("y"));
        assert!(is_truthy("Y"));
        assert!(is_truthy("yes"));
        assert!(is_truthy("Yes"));
        assert!(is_truthy("YES"));
        assert!(is_truthy(" true "));
        assert!(is_truthy("\ntrue\n"));
        assert!(is_truthy("  YES  "));

        assert!(!is_truthy("false"));
        assert!(!is_truthy("False"));
        assert!(!is_truthy("FALSE"));
        assert!(!is_truthy("n"));
        assert!(!is_truthy("N"));
        assert!(!is_truthy("no"));
        assert!(!is_truthy("No"));
        assert!(!is_truthy("NO"));
        assert!(!is_truthy("  false  "));
        assert!(!is_truthy("\nfalse\n"));
        assert!(!is_truthy("  NO  "));
    }
}
