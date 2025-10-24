/// Shared types for pipeline operations

/// Layer data structure used during graph construction from data sources
#[derive(Debug, Clone)]
pub(crate) struct LayerData {
    pub name: String,
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub comment: Option<String>,
    pub properties: Option<String>, // JSON string
    pub datasource_id: Option<i32>,
}
