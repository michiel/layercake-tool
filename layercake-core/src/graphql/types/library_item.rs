use async_graphql::{Enum, InputObject, Json, SimpleObject};
use serde_json::Value;

use crate::database::entities::library_items;

use super::{DataSetDataType, FileFormat};

#[derive(Copy, Clone, Eq, PartialEq, Enum, Debug)]
pub enum LibraryItemType {
    #[graphql(name = "DATASET")]
    Dataset,
    #[graphql(name = "PROJECT")]
    Project,
    #[graphql(name = "PROJECT_TEMPLATE")]
    ProjectTemplate,
}

impl LibraryItemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LibraryItemType::Dataset => "dataset",
            LibraryItemType::Project => "project",
            LibraryItemType::ProjectTemplate => "project_template",
        }
    }
}

#[derive(Clone, Debug, SimpleObject)]
pub struct LibraryItem {
    pub id: i32,
    #[graphql(name = "type")]
    pub item_type: LibraryItemType,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub metadata: Json<Value>,
    #[graphql(name = "contentSize")]
    pub content_size: Option<i64>,
    #[graphql(name = "createdAt")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[graphql(name = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(SimpleObject)]
pub struct ExportProjectArchivePayload {
    pub filename: String,
    #[graphql(name = "fileContent")]
    pub file_content: String,
}

impl From<library_items::Model> for LibraryItem {
    fn from(model: library_items::Model) -> Self {
        let tags: Vec<String> = serde_json::from_str(&model.tags).unwrap_or_default();
        let metadata: Value = serde_json::from_str(&model.metadata).unwrap_or(Value::Null);
        let item_type = match model.item_type.as_str() {
            "dataset" => LibraryItemType::Dataset,
            "project" => LibraryItemType::Project,
            "project_template" => LibraryItemType::ProjectTemplate,
            _ => LibraryItemType::Dataset,
        };

        Self {
            id: model.id,
            item_type,
            name: model.name,
            description: model.description,
            tags,
            metadata: Json(metadata),
            content_size: model.content_size,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(InputObject, Default)]
pub struct LibraryItemFilterInput {
    pub types: Option<Vec<LibraryItemType>>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "searchQuery")]
    pub search_query: Option<String>,
}

#[derive(InputObject)]
pub struct UploadLibraryItemInput {
    #[graphql(name = "type")]
    pub item_type: LibraryItemType,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    #[graphql(name = "fileName")]
    pub file_name: String,
    #[graphql(name = "fileContent")]
    pub file_content: String,
    #[graphql(name = "fileFormat")]
    pub file_format: Option<FileFormat>,
    #[graphql(name = "dataType")]
    pub data_type: Option<DataSetDataType>,
    #[graphql(name = "contentType")]
    pub content_type: Option<String>,
}
