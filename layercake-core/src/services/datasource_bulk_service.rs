use anyhow::{Context, Result};
use icu_locid::locale;
use rust_xlsxwriter::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use spreadsheet_ods::{WorkBook, Sheet, Value};

use crate::database::entities::data_sources;
use crate::database::entities::data_sources::{FileFormat, DataType};
use crate::services::data_source_service::DataSourceService;

pub struct DataSourceBulkService {
    db: DatabaseConnection,
}

impl DataSourceBulkService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Infer data type from sheet name or headers
    fn infer_data_type(sheet_name: &str, headers: &[String]) -> Option<DataType> {
        let name_lower = sheet_name.to_lowercase();

        // Check sheet name first
        if name_lower.contains("node") {
            return Some(DataType::Nodes);
        }
        if name_lower.contains("edge") || name_lower.contains("link") {
            return Some(DataType::Edges);
        }
        if name_lower.contains("layer") {
            return Some(DataType::Layers);
        }

        // Check headers
        if headers.contains(&"source".to_string()) && headers.contains(&"target".to_string()) {
            return Some(DataType::Edges);
        }
        if headers.contains(&"layer".to_string()) && headers.contains(&"background".to_string()) {
            return Some(DataType::Layers);
        }
        if headers.contains(&"label".to_string()) && !headers.contains(&"source".to_string()) {
            return Some(DataType::Nodes);
        }

        None
    }

    /// Convert calamine range to CSV string
    fn range_to_csv(range: &calamine::Range<calamine::Data>) -> Result<Vec<u8>> {
        use calamine::Data;
        use std::io::Write;

        let mut csv_data = Vec::new();

        for row_idx in 0..range.height() {
            let mut row_values = Vec::new();

            for col_idx in 0..range.width() {
                let cell = range.get((row_idx, col_idx));
                let value = match cell {
                    Some(Data::String(s)) => s.clone(),
                    Some(Data::Int(i)) => i.to_string(),
                    Some(Data::Float(f)) => f.to_string(),
                    Some(Data::Bool(b)) => b.to_string(),
                    Some(Data::Empty) | None => String::new(),
                    _ => String::new(),
                };
                row_values.push(value);
            }

            // Write CSV row
            let row_str = row_values.join(",");
            writeln!(csv_data, "{}", row_str)?;
        }

        Ok(csv_data)
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
    /// Each sheet becomes a datasource containing the tabular data from that sheet
    pub async fn import_from_xlsx(
        &self,
        project_id: i32,
        xlsx_data: &[u8],
    ) -> Result<DataSourceImportResult> {
        use calamine::{open_workbook_from_rs, Reader, Xlsx, Data};
        use std::io::Cursor;

        tracing::info!("Importing XLSX file with {} bytes", xlsx_data.len());

        let cursor = Cursor::new(xlsx_data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
            .map_err(|e| {
                tracing::error!("Failed to open XLSX: {:?}", e);
                anyhow::anyhow!("Failed to open XLSX file: {:?}", e)
            })?;

        let mut created_count = 0;
        let mut updated_count = 0;
        let mut imported_ids = Vec::new();

        let service = DataSourceService::new(self.db.clone());

        // Iterate through all sheets
        let sheet_names = workbook.sheet_names();
        tracing::info!("Found {} sheets in XLSX", sheet_names.len());

        for sheet_name in sheet_names {
            let sheet_name = sheet_name.to_string();
            tracing::info!("Processing sheet: {}", sheet_name);

            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                tracing::info!("Sheet '{}' dimensions: {}x{}", sheet_name, range.height(), range.width());

                if range.height() == 0 || range.width() == 0 {
                    tracing::warn!("Skipping empty sheet: {}", sheet_name);
                    continue;
                }

                // Extract headers from first row
                let mut headers = Vec::new();
                for col_idx in 0..range.width() {
                    if let Some(Data::String(s)) = range.get((0, col_idx)) {
                        headers.push(s.clone());
                    }
                }

                tracing::info!("Sheet headers: {:?}", headers);

                // Infer data type
                let data_type = Self::infer_data_type(&sheet_name, &headers)
                    .ok_or_else(|| anyhow::anyhow!("Could not infer data type for sheet: {}", sheet_name))?;

                tracing::info!("Inferred data type: {:?}", data_type);

                // Convert sheet to CSV
                let csv_data = Self::range_to_csv(&range)?;
                tracing::info!("Converted sheet to {} bytes of CSV", csv_data.len());

                // Create datasource using existing service
                let filename = format!("{}.csv", sheet_name);
                let datasource = service.create_from_file(
                    project_id,
                    sheet_name.clone(),
                    Some(format!("Imported from spreadsheet")),
                    filename,
                    FileFormat::Csv,
                    data_type,
                    csv_data,
                ).await?;

                created_count += 1;
                imported_ids.push(datasource.id);
                tracing::info!("Created datasource: {} with id: {}", sheet_name, datasource.id);
            }
        }

        Ok(DataSourceImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }

    /// Import datasources from ODS format
    /// Each sheet becomes a datasource containing the tabular data from that sheet
    pub async fn import_from_ods(
        &self,
        project_id: i32,
        ods_data: &[u8],
    ) -> Result<DataSourceImportResult> {
        use calamine::{open_workbook_from_rs, Reader, Ods, Data};
        use std::io::Cursor;

        tracing::info!("Importing ODS file with {} bytes", ods_data.len());

        let cursor = Cursor::new(ods_data);
        let mut workbook: Ods<_> = open_workbook_from_rs(cursor)
            .map_err(|e| {
                tracing::error!("Failed to open ODS: {:?}", e);
                anyhow::anyhow!("Failed to open ODS file: {:?}", e)
            })?;

        let mut created_count = 0;
        let mut updated_count = 0;
        let mut imported_ids = Vec::new();

        let service = DataSourceService::new(self.db.clone());

        // Iterate through all sheets
        let sheet_names = workbook.sheet_names();
        tracing::info!("Found {} sheets in ODS", sheet_names.len());

        for sheet_name in sheet_names {
            let sheet_name = sheet_name.to_string();
            tracing::info!("Processing sheet: {}", sheet_name);

            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                tracing::info!("Sheet '{}' dimensions: {}x{}", sheet_name, range.height(), range.width());

                if range.height() == 0 || range.width() == 0 {
                    tracing::warn!("Skipping empty sheet: {}", sheet_name);
                    continue;
                }

                // Extract headers from first row
                let mut headers = Vec::new();
                for col_idx in 0..range.width() {
                    if let Some(Data::String(s)) = range.get((0, col_idx)) {
                        headers.push(s.clone());
                    }
                }

                tracing::info!("Sheet headers: {:?}", headers);

                // Infer data type
                let data_type = Self::infer_data_type(&sheet_name, &headers)
                    .ok_or_else(|| anyhow::anyhow!("Could not infer data type for sheet: {}", sheet_name))?;

                tracing::info!("Inferred data type: {:?}", data_type);

                // Convert sheet to CSV
                let csv_data = Self::range_to_csv(&range)?;
                tracing::info!("Converted sheet to {} bytes of CSV", csv_data.len());

                // Create datasource using existing service
                let filename = format!("{}.csv", sheet_name);
                let datasource = service.create_from_file(
                    project_id,
                    sheet_name.clone(),
                    Some(format!("Imported from spreadsheet")),
                    filename,
                    FileFormat::Csv,
                    data_type,
                    csv_data,
                ).await?;

                created_count += 1;
                imported_ids.push(datasource.id);
                tracing::info!("Created datasource: {} with id: {}", sheet_name, datasource.id);
            }
        }

        Ok(DataSourceImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }
}

pub struct DataSourceImportResult {
    pub created_count: i32,
    pub updated_count: i32,
    pub imported_ids: Vec<i32>,
}
