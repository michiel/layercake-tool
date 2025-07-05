use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "execution_outputs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub execution_id: String,
    pub file_name: String,
    pub file_type: String,
    pub file_path: Option<String>,
    pub file_size: Option<i32>,
    pub created_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFileType {
    Gml,
    Dot,
    PlantUml,
    CsvMatrix,
    Json,
    Yaml,
    Html,
    Svg,
    Png,
    Pdf,
}

impl From<OutputFileType> for String {
    fn from(file_type: OutputFileType) -> Self {
        match file_type {
            OutputFileType::Gml => "gml".to_string(),
            OutputFileType::Dot => "dot".to_string(),
            OutputFileType::PlantUml => "plantuml".to_string(),
            OutputFileType::CsvMatrix => "csv_matrix".to_string(),
            OutputFileType::Json => "json".to_string(),
            OutputFileType::Yaml => "yaml".to_string(),
            OutputFileType::Html => "html".to_string(),
            OutputFileType::Svg => "svg".to_string(),
            OutputFileType::Png => "png".to_string(),
            OutputFileType::Pdf => "pdf".to_string(),
        }
    }
}

impl From<String> for OutputFileType {
    fn from(file_type: String) -> Self {
        match file_type.to_lowercase().as_str() {
            "gml" => OutputFileType::Gml,
            "dot" => OutputFileType::Dot,
            "plantuml" | "puml" => OutputFileType::PlantUml,
            "csv_matrix" | "csv" => OutputFileType::CsvMatrix,
            "json" => OutputFileType::Json,
            "yaml" | "yml" => OutputFileType::Yaml,
            "html" | "htm" => OutputFileType::Html,
            "svg" => OutputFileType::Svg,
            "png" => OutputFileType::Png,
            "pdf" => OutputFileType::Pdf,
            _ => OutputFileType::Json,
        }
    }
}

impl Model {
    pub fn get_file_type(&self) -> OutputFileType {
        OutputFileType::from(self.file_type.clone())
    }

    pub fn file_extension(&self) -> &'static str {
        match self.get_file_type() {
            OutputFileType::Gml => ".gml",
            OutputFileType::Dot => ".dot",
            OutputFileType::PlantUml => ".puml",
            OutputFileType::CsvMatrix => ".csv",
            OutputFileType::Json => ".json",
            OutputFileType::Yaml => ".yaml",
            OutputFileType::Html => ".html",
            OutputFileType::Svg => ".svg",
            OutputFileType::Png => ".png",
            OutputFileType::Pdf => ".pdf",
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self.get_file_type(),
            OutputFileType::Svg | OutputFileType::Png | OutputFileType::Pdf
        )
    }

    pub fn is_text(&self) -> bool {
        matches!(
            self.get_file_type(),
            OutputFileType::Gml
                | OutputFileType::Dot
                | OutputFileType::PlantUml
                | OutputFileType::CsvMatrix
                | OutputFileType::Json
                | OutputFileType::Yaml
                | OutputFileType::Html
        )
    }

    pub fn formatted_size(&self) -> String {
        match self.file_size {
            Some(size) => {
                if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                }
            }
            None => "Unknown size".to_string(),
        }
    }
}