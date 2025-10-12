/// Shared types for pipeline operations

/// Layer data structure used during graph construction from data sources
#[derive(Debug, Clone)]
pub(crate) struct LayerData {
    pub name: String,
    pub color: Option<String>,
    pub properties: Option<String>, // JSON string
}
