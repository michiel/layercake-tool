use anyhow::{Context, Result};
use icu_locid::locale;
use rust_xlsxwriter::*;
use sea_orm::{DatabaseConnection, EntityTrait};
use spreadsheet_ods::{Sheet, Value, WorkBook};

use crate::database::entities::common_types::{DataType, FileFormat};
use crate::database::entities::data_sources;
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

    /// Convert graph_json to CSV rows
    fn graph_json_to_csv_rows(graph_json: &str, data_type: &str) -> Result<Vec<Vec<String>>> {
        let data: serde_json::Value =
            serde_json::from_str(graph_json).context("Failed to parse graph_json")?;

        let array = match data_type {
            "nodes" => data.get("nodes"),
            "edges" => data.get("edges"),
            "layers" => data.get("layers"),
            _ => None,
        }
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("No {} array in graph_json", data_type))?;

        if array.is_empty() {
            return Ok(Vec::new());
        }

        // Extract all unique keys for headers
        let mut all_keys = std::collections::BTreeSet::new();
        for item in array {
            if let Some(obj) = item.as_object() {
                for key in obj.keys() {
                    all_keys.insert(key.clone());
                }
            }
        }

        let headers: Vec<String> = all_keys.into_iter().collect();
        let mut rows = vec![headers.clone()];

        // Convert each object to a row
        for item in array {
            if let Some(obj) = item.as_object() {
                let mut row = Vec::new();
                for header in &headers {
                    let value = obj
                        .get(header)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => String::new(),
                            _ => serde_json::to_string(v).unwrap_or_default(),
                        })
                        .unwrap_or_default();
                    row.push(value);
                }
                rows.push(row);
            }
        }

        Ok(rows)
    }

    /// Export datasources to XLSX format
    /// Each datasource becomes a separate sheet named with its name containing CSV data
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

        tracing::info!("Exporting {} datasources to XLSX", datasources.len());

        // Check for duplicate names
        let mut name_counts = std::collections::HashMap::new();
        for ds in &datasources {
            *name_counts.entry(&ds.name).or_insert(0) += 1;
        }
        let duplicates: Vec<&String> = name_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(&name, _)| name)
            .collect();

        if !duplicates.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot export: duplicate datasource names found: {}",
                duplicates
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        for datasource in datasources {
            // Create a sheet named with the datasource name
            let sheet_name = datasource.name.clone();
            let worksheet = workbook.add_worksheet();
            worksheet.set_name(&sheet_name)?;

            tracing::info!(
                "Exporting datasource {} ({}) to sheet {}",
                datasource.id,
                datasource.name,
                sheet_name
            );

            // Convert graph_json to CSV rows
            match Self::graph_json_to_csv_rows(&datasource.graph_json, &datasource.data_type) {
                Ok(rows) => {
                    // Write rows to sheet
                    for (row_idx, row_data) in rows.iter().enumerate() {
                        for (col_idx, value) in row_data.iter().enumerate() {
                            // Try to parse as number, otherwise write as string
                            if let Ok(num) = value.parse::<f64>() {
                                worksheet.write_number(row_idx as u32, col_idx as u16, num)?;
                            } else {
                                worksheet.write_string(row_idx as u32, col_idx as u16, value)?;
                            }
                        }
                    }
                    tracing::info!("Wrote {} rows to sheet {}", rows.len(), sheet_name);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to convert datasource {} to CSV: {}",
                        datasource.id,
                        e
                    );
                    // Write error message to sheet
                    worksheet.write_string(0, 0, "Error")?;
                    worksheet.write_string(0, 1, format!("Failed to export: {}", e))?;
                }
            }
        }

        // Save to buffer
        workbook.save_to_buffer().context("Failed to generate XLSX")
    }

    /// Export datasources to ODS format
    /// Each datasource becomes a separate sheet named with its name containing CSV data
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

        tracing::info!("Exporting {} datasources to ODS", datasources.len());

        // Check for duplicate names
        let mut name_counts = std::collections::HashMap::new();
        for ds in &datasources {
            *name_counts.entry(&ds.name).or_insert(0) += 1;
        }
        let duplicates: Vec<&String> = name_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(&name, _)| name)
            .collect();

        if !duplicates.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot export: duplicate datasource names found: {}",
                duplicates
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        for datasource in datasources {
            // Create a sheet named with the datasource name
            let sheet_name = datasource.name.clone();
            let mut sheet = Sheet::new(&sheet_name);

            tracing::info!(
                "Exporting datasource {} ({}) to sheet {}",
                datasource.id,
                datasource.name,
                sheet_name
            );

            // Convert graph_json to CSV rows
            match Self::graph_json_to_csv_rows(&datasource.graph_json, &datasource.data_type) {
                Ok(rows) => {
                    // Write rows to sheet
                    for (row_idx, row_data) in rows.iter().enumerate() {
                        for (col_idx, value) in row_data.iter().enumerate() {
                            // Try to parse as number, otherwise write as string
                            if let Ok(num) = value.parse::<f64>() {
                                sheet.set_value(row_idx as u32, col_idx as u32, Value::Number(num));
                            } else {
                                sheet.set_value(
                                    row_idx as u32,
                                    col_idx as u32,
                                    Value::Text(value.clone()),
                                );
                            }
                        }
                    }
                    tracing::info!("Wrote {} rows to sheet {}", rows.len(), sheet_name);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to convert datasource {} to CSV: {}",
                        datasource.id,
                        e
                    );
                    // Write error message to sheet
                    sheet.set_value(0, 0, Value::Text("Error".to_string()));
                    sheet.set_value(0, 1, Value::Text(format!("Failed to export: {}", e)));
                }
            }

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
        use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
        use std::io::Cursor;

        tracing::info!("Importing XLSX file with {} bytes", xlsx_data.len());

        let cursor = Cursor::new(xlsx_data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor).map_err(|e| {
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
                tracing::info!(
                    "Sheet '{}' dimensions: {}x{}",
                    sheet_name,
                    range.height(),
                    range.width()
                );

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
                let data_type = Self::infer_data_type(&sheet_name, &headers).ok_or_else(|| {
                    anyhow::anyhow!("Could not infer data type for sheet: {}", sheet_name)
                })?;

                tracing::info!("Inferred data type: {:?}", data_type);

                // Convert sheet to CSV
                let csv_data = Self::range_to_csv(&range)?;
                tracing::info!("Converted sheet to {} bytes of CSV", csv_data.len());

                // Try to find existing datasource by name
                use sea_orm::ColumnTrait;
                use sea_orm::QueryFilter;
                if let Some(existing) = data_sources::Entity::find()
                    .filter(data_sources::Column::ProjectId.eq(project_id))
                    .filter(data_sources::Column::Name.eq(sheet_name.clone()))
                    .one(&self.db)
                    .await?
                {
                    tracing::info!(
                        "Found existing datasource '{}' (id: {}) - updating",
                        sheet_name,
                        existing.id
                    );

                    // Update the datasource with new CSV data
                    let filename = format!("{}.csv", existing.name);
                    let datasource = service.update_file(existing.id, filename, csv_data).await?;

                    updated_count += 1;
                    imported_ids.push(datasource.id);
                    tracing::info!(
                        "Updated datasource: {} (id: {})",
                        datasource.name,
                        datasource.id
                    );
                    continue;
                }

                // Create new datasource
                let filename = format!("{}.csv", sheet_name);
                let datasource = service
                    .create_from_file(
                        project_id,
                        sheet_name.clone(),
                        Some("Imported from spreadsheet".to_string()),
                        filename,
                        FileFormat::Csv,
                        data_type,
                        csv_data,
                    )
                    .await?;

                created_count += 1;
                imported_ids.push(datasource.id);
                tracing::info!(
                    "Created datasource: {} with id: {}",
                    sheet_name,
                    datasource.id
                );
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
        use calamine::{open_workbook_from_rs, Data, Ods, Reader};
        use std::io::Cursor;

        tracing::info!("Importing ODS file with {} bytes", ods_data.len());

        let cursor = Cursor::new(ods_data);
        let mut workbook: Ods<_> = open_workbook_from_rs(cursor).map_err(|e| {
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
                tracing::info!(
                    "Sheet '{}' dimensions: {}x{}",
                    sheet_name,
                    range.height(),
                    range.width()
                );

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
                let data_type = Self::infer_data_type(&sheet_name, &headers).ok_or_else(|| {
                    anyhow::anyhow!("Could not infer data type for sheet: {}", sheet_name)
                })?;

                tracing::info!("Inferred data type: {:?}", data_type);

                // Convert sheet to CSV
                let csv_data = Self::range_to_csv(&range)?;
                tracing::info!("Converted sheet to {} bytes of CSV", csv_data.len());

                // Try to find existing datasource by name
                use sea_orm::ColumnTrait;
                use sea_orm::QueryFilter;
                if let Some(existing) = data_sources::Entity::find()
                    .filter(data_sources::Column::ProjectId.eq(project_id))
                    .filter(data_sources::Column::Name.eq(sheet_name.clone()))
                    .one(&self.db)
                    .await?
                {
                    tracing::info!(
                        "Found existing datasource '{}' (id: {}) - updating",
                        sheet_name,
                        existing.id
                    );

                    // Update the datasource with new CSV data
                    let filename = format!("{}.csv", existing.name);
                    let datasource = service.update_file(existing.id, filename, csv_data).await?;

                    updated_count += 1;
                    imported_ids.push(datasource.id);
                    tracing::info!(
                        "Updated datasource: {} (id: {})",
                        datasource.name,
                        datasource.id
                    );
                    continue;
                }

                // Create new datasource
                let filename = format!("{}.csv", sheet_name);
                let datasource = service
                    .create_from_file(
                        project_id,
                        sheet_name.clone(),
                        Some("Imported from spreadsheet".to_string()),
                        filename,
                        FileFormat::Csv,
                        data_type,
                        csv_data,
                    )
                    .await?;

                created_count += 1;
                imported_ids.push(datasource.id);
                tracing::info!(
                    "Created datasource: {} with id: {}",
                    sheet_name,
                    datasource.id
                );
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
