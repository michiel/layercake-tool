//! Format detection and validation utilities

use super::GraphFormat;
use std::path::Path;

/// Detect graph format from file content
pub fn detect_format_from_content(content: &str) -> Option<GraphFormat> {
    let content_trimmed = content.trim();
    
    // JSON detection
    if (content_trimmed.starts_with('{') && content_trimmed.ends_with('}')) ||
       (content_trimmed.starts_with('[') && content_trimmed.ends_with(']')) {
        // Check if it looks like Layercake format
        if content.contains("\"format\"") && content.contains("\"layercake\"") {
            return Some(GraphFormat::Layercake);
        }
        return Some(GraphFormat::JSON);
    }
    
    // XML detection (GraphML/GEXF)
    if content_trimmed.starts_with("<?xml") || content_trimmed.starts_with('<') {
        if content.contains("<graphml") {
            return Some(GraphFormat::GraphML);
        }
        if content.contains("<gexf") {
            return Some(GraphFormat::GEXF);
        }
    }
    
    // DOT detection
    if content.contains("digraph") || content.contains("graph") {
        return Some(GraphFormat::DOT);
    }
    
    // CSV detection (basic heuristic)
    let lines: Vec<&str> = content.lines().take(5).collect();
    if lines.len() > 1 {
        let first_line = lines[0];
        if first_line.contains(',') && 
           (first_line.to_lowercase().contains("id") || 
            first_line.to_lowercase().contains("source") ||
            first_line.to_lowercase().contains("target")) {
            return Some(GraphFormat::CSV);
        }
    }
    
    None
}

/// Validate that a file path matches the expected format
pub fn validate_format_compatibility(file_path: &Path, format: GraphFormat) -> bool {
    if let Some(extension) = file_path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        let valid_extensions = super::GraphIO::get_extensions(format);
        valid_extensions.contains(&ext.as_str())
    } else {
        false
    }
}

/// Get MIME type for a graph format
pub fn get_mime_type(format: GraphFormat) -> &'static str {
    match format {
        GraphFormat::JSON | GraphFormat::Layercake => "application/json",
        GraphFormat::CSV => "text/csv",
        GraphFormat::GraphML => "application/xml",
        GraphFormat::GEXF => "application/xml",
        GraphFormat::DOT => "text/vnd.graphviz",
    }
}

/// Check if format supports hierarchical data
pub fn supports_hierarchy(format: GraphFormat) -> bool {
    match format {
        GraphFormat::Layercake | GraphFormat::JSON => true,
        GraphFormat::GraphML | GraphFormat::GEXF => true, // With extensions
        GraphFormat::CSV | GraphFormat::DOT => false,
    }
}

/// Check if format supports layer information
pub fn supports_layers(format: GraphFormat) -> bool {
    match format {
        GraphFormat::Layercake | GraphFormat::JSON => true,
        GraphFormat::CSV => true, // Via separate files
        GraphFormat::GraphML | GraphFormat::GEXF => true, // With extensions
        GraphFormat::DOT => true, // Via subgraphs
    }
}

/// Get format description
pub fn get_format_description(format: GraphFormat) -> &'static str {
    match format {
        GraphFormat::CSV => "Comma-separated values (nodes.csv, edges.csv, layers.csv)",
        GraphFormat::JSON => "JavaScript Object Notation",
        GraphFormat::GraphML => "Graph Markup Language (XML-based)",
        GraphFormat::GEXF => "Graph Exchange XML Format",
        GraphFormat::DOT => "DOT graph description language (Graphviz)",
        GraphFormat::Layercake => "Layercake native format with metadata",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_detection_json() {
        let json_content = r#"{"nodes": [], "edges": []}"#;
        assert_eq!(detect_format_from_content(json_content), Some(GraphFormat::JSON));
    }
    
    #[test]
    fn test_format_detection_layercake() {
        let layercake_content = r#"{"format": "layercake", "graph": {"nodes": []}}"#;
        assert_eq!(detect_format_from_content(layercake_content), Some(GraphFormat::Layercake));
    }
    
    #[test]
    fn test_format_detection_dot() {
        let dot_content = r#"digraph G { A -> B; }"#;
        assert_eq!(detect_format_from_content(dot_content), Some(GraphFormat::DOT));
    }
    
    #[test]
    fn test_format_detection_csv() {
        let csv_content = "id,label,layer\n1,Node1,layer1\n";
        assert_eq!(detect_format_from_content(csv_content), Some(GraphFormat::CSV));
    }
    
    #[test]
    fn test_format_validation() {
        assert!(validate_format_compatibility(Path::new("graph.json"), GraphFormat::JSON));
        assert!(validate_format_compatibility(Path::new("data.csv"), GraphFormat::CSV));
        assert!(validate_format_compatibility(Path::new("graph.dot"), GraphFormat::DOT));
        assert!(!validate_format_compatibility(Path::new("graph.json"), GraphFormat::CSV));
    }
    
    #[test]
    fn test_format_capabilities() {
        assert!(supports_hierarchy(GraphFormat::Layercake));
        assert!(supports_hierarchy(GraphFormat::JSON));
        assert!(!supports_hierarchy(GraphFormat::CSV));
        
        assert!(supports_layers(GraphFormat::Layercake));
        assert!(supports_layers(GraphFormat::CSV));
        assert!(supports_layers(GraphFormat::DOT));
    }
}