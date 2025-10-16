use anyhow::{Context, Result};
use icu_locid::locale;
use rust_xlsxwriter::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use spreadsheet_ods::{WorkBook, Sheet, Value};

use crate::database::entities::data_sources;

pub struct DataSourceBulkService {
    db: DatabaseConnection,
}

impl DataSourceBulkService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Export datasources to XLSX format
    /// Each datasource becomes a separate sheet named with its ID
    pub async fn export_to_xlsx(&self, datasource_ids: &[i32]) -> Result<Vec<u8>> {
        let mut workbook = Workbook::new();

        // Fetch all requested datasources
        let datasources = data_sources::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to fetch datasources")?
            .into_iter()
            .filter(|ds| datasource_ids.contains(&ds.id))
            .collect::<Vec<_>>();

        for datasource in datasources {
            // Create a sheet named with the datasource ID
            let sheet_name = format!("ds_{}", datasource.id);
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(&sheet_name)?;

            // Write datasource properties as rows
            let mut row = 0u32;

            // ID
            worksheet.write_string(row, 0, "id")?;
            worksheet.write_number(row, 1, datasource.id as f64)?;
            row += 1;

            // Name
            worksheet.write_string(row, 0, "name")?;
            worksheet.write_string(row, 1, &datasource.name)?;
            row += 1;

            // Description
            if let Some(desc) = &datasource.description {
                worksheet.write_string(row, 0, "description")?;
                worksheet.write_string(row, 1, desc)?;
                row += 1;
            }

            // File format
            worksheet.write_string(row, 0, "file_format")?;
            worksheet.write_string(row, 1, &datasource.file_format)?;
            row += 1;

            // Data type
            worksheet.write_string(row, 0, "data_type")?;
            worksheet.write_string(row, 1, &datasource.data_type)?;
            row += 1;

            // Filename
            worksheet.write_string(row, 0, "filename")?;
            worksheet.write_string(row, 1, &datasource.filename)?;
            row += 1;

            // File size
            worksheet.write_string(row, 0, "file_size")?;
            worksheet.write_number(row, 1, datasource.file_size as f64)?;
            row += 1;

            // Status
            worksheet.write_string(row, 0, "status")?;
            worksheet.write_string(row, 1, &datasource.status)?;
            row += 1;

            // Graph JSON (potentially large)
            worksheet.write_string(row, 0, "graph_json")?;
            worksheet.write_string(row, 1, &datasource.graph_json)?;
            row += 1;

            // Timestamps
            worksheet.write_string(row, 0, "created_at")?;
            worksheet.write_string(row, 1, &datasource.created_at.to_rfc3339())?;
            row += 1;

            worksheet.write_string(row, 0, "updated_at")?;
            worksheet.write_string(row, 1, &datasource.updated_at.to_rfc3339())?;
        }

        // Save to buffer
        workbook
            .save_to_buffer()
            .context("Failed to generate XLSX")
    }

    /// Export datasources to ODS format
    /// Each datasource becomes a separate sheet named with its ID
    pub async fn export_to_ods(&self, datasource_ids: &[i32]) -> Result<Vec<u8>> {
        let mut workbook = WorkBook::new(locale!("en_US"));

        // Fetch all requested datasources
        let datasources = data_sources::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to fetch datasources")?
            .into_iter()
            .filter(|ds| datasource_ids.contains(&ds.id))
            .collect::<Vec<_>>();

        for datasource in datasources {
            // Create a sheet named with the datasource ID
            let sheet_name = format!("ds_{}", datasource.id);
            let mut sheet = Sheet::new(&sheet_name);

            // Write datasource properties as rows
            let mut row = 0u32;

            // ID
            sheet.set_value(row, 0, Value::Text("id".to_string()));
            sheet.set_value(row, 1, Value::Number(datasource.id as f64));
            row += 1;

            // Name
            sheet.set_value(row, 0, Value::Text("name".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.name.clone()));
            row += 1;

            // Description
            if let Some(desc) = &datasource.description {
                sheet.set_value(row, 0, Value::Text("description".to_string()));
                sheet.set_value(row, 1, Value::Text(desc.clone()));
                row += 1;
            }

            // File format
            sheet.set_value(row, 0, Value::Text("file_format".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.file_format.clone()));
            row += 1;

            // Data type
            sheet.set_value(row, 0, Value::Text("data_type".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.data_type.clone()));
            row += 1;

            // Filename
            sheet.set_value(row, 0, Value::Text("filename".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.filename.clone()));
            row += 1;

            // File size
            sheet.set_value(row, 0, Value::Text("file_size".to_string()));
            sheet.set_value(row, 1, Value::Number(datasource.file_size as f64));
            row += 1;

            // Status
            sheet.set_value(row, 0, Value::Text("status".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.status.clone()));
            row += 1;

            // Graph JSON (potentially large)
            sheet.set_value(row, 0, Value::Text("graph_json".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.graph_json.clone()));
            row += 1;

            // Timestamps
            sheet.set_value(row, 0, Value::Text("created_at".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.created_at.to_rfc3339()));
            row += 1;

            sheet.set_value(row, 0, Value::Text("updated_at".to_string()));
            sheet.set_value(row, 1, Value::Text(datasource.updated_at.to_rfc3339()));

            workbook.push_sheet(sheet);
        }

        // Save to buffer
        let buffer = Vec::new();
        let result = spreadsheet_ods::write_ods_buf(&mut workbook, buffer)
            .context("Failed to generate ODS")?;
        Ok(result)
    }

    /// Import datasources from XLSX format
    /// Each sheet represents a datasource (identified by sheet name or ID field)
    /// If datasource exists (by ID), update it; otherwise create new
    pub async fn import_from_xlsx(
        &self,
        project_id: i32,
        xlsx_data: &[u8],
    ) -> Result<DataSourceImportResult> {
        use calamine::{open_workbook_from_rs, Reader, Xlsx};
        use std::io::Cursor;

        // Log file size for debugging
        tracing::debug!("Attempting to import XLSX file with {} bytes", xlsx_data.len());

        let cursor = Cursor::new(xlsx_data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .context(format!("Failed to open XLSX file ({} bytes)", xlsx_data.len()))?;

        let mut created_count = 0;
        let mut updated_count = 0;
        let mut imported_ids = Vec::new();

        // Iterate through all sheets
        for sheet_name in workbook.sheet_names() {
            let sheet_name = sheet_name.to_string();

            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // Parse datasource from sheet
                let datasource_data = self.parse_datasource_from_sheet(&range)?;

                // Check if datasource with this ID exists
                if let Some(existing_id) = datasource_data.id {
                    // Update existing
                    let existing = data_sources::Entity::find_by_id(existing_id)
                        .one(&self.db)
                        .await?;

                    if let Some(existing) = existing {
                        use sea_orm::ActiveModelTrait;
                        let mut active: data_sources::ActiveModel = existing.into();

                        // Update fields
                        if let Some(name) = datasource_data.name {
                            active.name = Set(name);
                        }
                        active.description = Set(datasource_data.description);

                        let updated = active.update(&self.db).await?;
                        updated_count += 1;
                        imported_ids.push(updated.id);
                    }
                } else {
                    // Create new datasource from imported data
                    if let (Some(name), Some(file_format), Some(data_type), Some(filename), Some(graph_json)) = (
                        datasource_data.name,
                        datasource_data.file_format,
                        datasource_data.data_type,
                        datasource_data.filename,
                        datasource_data.graph_json,
                    ) {
                        // Calculate file size from graph_json
                        let file_size = graph_json.len() as i64;

                        // Create new datasource with empty blob (we have graph_json which is what matters)
                        let new_datasource = data_sources::ActiveModel {
                            id: sea_orm::ActiveValue::NotSet,
                            project_id: Set(project_id),
                            name: Set(name),
                            description: Set(datasource_data.description),
                            file_format: Set(file_format),
                            data_type: Set(data_type),
                            filename: Set(filename),
                            blob: Set(Vec::new()), // Empty blob since we have graph_json
                            graph_json: Set(graph_json),
                            status: Set("active".to_string()),
                            error_message: Set(None),
                            file_size: Set(file_size),
                            processed_at: Set(Some(chrono::Utc::now())),
                            created_at: Set(chrono::Utc::now()),
                            updated_at: Set(chrono::Utc::now()),
                        };

                        let created = new_datasource.insert(&self.db).await?;
                        created_count += 1;
                        imported_ids.push(created.id);
                    }
                }
            }
        }

        Ok(DataSourceImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }

    /// Import datasources from ODS format
    /// Each sheet represents a datasource (identified by sheet name or ID field)
    /// If datasource exists (by ID), update it; otherwise create new
    pub async fn import_from_ods(
        &self,
        project_id: i32,
        ods_data: &[u8],
    ) -> Result<DataSourceImportResult> {
        use calamine::{open_workbook_from_rs, Reader, Ods};
        use std::io::Cursor;

        let cursor = Cursor::new(ods_data);
        let mut workbook: Ods<_> = open_workbook_from_rs(cursor)
            .context("Failed to open ODS file")?;

        let mut created_count = 0;
        let mut updated_count = 0;
        let mut imported_ids = Vec::new();

        // Iterate through all sheets
        for sheet_name in workbook.sheet_names() {
            let sheet_name = sheet_name.to_string();

            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                // Parse datasource from sheet
                let datasource_data = self.parse_datasource_from_sheet(&range)?;

                // Check if datasource with this ID exists
                if let Some(existing_id) = datasource_data.id {
                    // Update existing
                    let existing = data_sources::Entity::find_by_id(existing_id)
                        .one(&self.db)
                        .await?;

                    if let Some(existing) = existing {
                        use sea_orm::ActiveModelTrait;
                        let mut active: data_sources::ActiveModel = existing.into();

                        // Update fields
                        if let Some(name) = datasource_data.name {
                            active.name = Set(name);
                        }
                        active.description = Set(datasource_data.description);

                        let updated = active.update(&self.db).await?;
                        updated_count += 1;
                        imported_ids.push(updated.id);
                    }
                } else {
                    // Create new datasource from imported data
                    if let (Some(name), Some(file_format), Some(data_type), Some(filename), Some(graph_json)) = (
                        datasource_data.name,
                        datasource_data.file_format,
                        datasource_data.data_type,
                        datasource_data.filename,
                        datasource_data.graph_json,
                    ) {
                        // Calculate file size from graph_json
                        let file_size = graph_json.len() as i64;

                        // Create new datasource with empty blob (we have graph_json which is what matters)
                        let new_datasource = data_sources::ActiveModel {
                            id: sea_orm::ActiveValue::NotSet,
                            project_id: Set(project_id),
                            name: Set(name),
                            description: Set(datasource_data.description),
                            file_format: Set(file_format),
                            data_type: Set(data_type),
                            filename: Set(filename),
                            blob: Set(Vec::new()), // Empty blob since we have graph_json
                            graph_json: Set(graph_json),
                            status: Set("active".to_string()),
                            error_message: Set(None),
                            file_size: Set(file_size),
                            processed_at: Set(Some(chrono::Utc::now())),
                            created_at: Set(chrono::Utc::now()),
                            updated_at: Set(chrono::Utc::now()),
                        };

                        let created = new_datasource.insert(&self.db).await?;
                        created_count += 1;
                        imported_ids.push(created.id);
                    }
                }
            }
        }

        Ok(DataSourceImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }

    fn parse_datasource_from_sheet(
        &self,
        range: &calamine::Range<calamine::Data>,
    ) -> Result<DataSourceData> {
        use calamine::Data;

        let mut data = DataSourceData::default();

        // Read key-value pairs from rows
        for row_idx in 0..range.height() {
            if let (Some(key_cell), Some(value_cell)) =
                (range.get((row_idx, 0)), range.get((row_idx, 1)))
            {
                if let Data::String(ref key) = key_cell {
                    match key.as_str() {
                        "id" => {
                            if let Data::Int(id) = value_cell {
                                data.id = Some(*id as i32);
                            } else if let Data::Float(id) = value_cell {
                                data.id = Some(*id as i32);
                            }
                        }
                        "name" => {
                            if let Data::String(ref name) = value_cell {
                                data.name = Some(name.clone());
                            }
                        }
                        "description" => {
                            if let Data::String(ref desc) = value_cell {
                                data.description = Some(desc.clone());
                            }
                        }
                        "file_format" => {
                            if let Data::String(ref fmt) = value_cell {
                                data.file_format = Some(fmt.clone());
                            }
                        }
                        "data_type" => {
                            if let Data::String(ref dt) = value_cell {
                                data.data_type = Some(dt.clone());
                            }
                        }
                        "filename" => {
                            if let Data::String(ref fn_) = value_cell {
                                data.filename = Some(fn_.clone());
                            }
                        }
                        "graph_json" => {
                            if let Data::String(ref json) = value_cell {
                                data.graph_json = Some(json.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(data)
    }
}

#[derive(Default)]
struct DataSourceData {
    id: Option<i32>,
    name: Option<String>,
    description: Option<String>,
    file_format: Option<String>,
    data_type: Option<String>,
    filename: Option<String>,
    graph_json: Option<String>,
}

pub struct DataSourceImportResult {
    pub created_count: i32,
    pub updated_count: i32,
    pub imported_ids: Vec<i32>,
}
