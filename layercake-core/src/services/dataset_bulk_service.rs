use anyhow::{Context, Result};
use icu_locid::locale;
use rust_xlsxwriter::*;
use sea_orm::{DatabaseConnection, EntityTrait};
use spreadsheet_ods::{Sheet, Value, WorkBook};
use std::collections::HashSet;

use crate::database::entities::common_types::{DataType, FileFormat};
use crate::database::entities::data_sets;
use crate::services::data_set_service::DataSetService;

pub struct DataSetBulkService {
    db: DatabaseConnection,
}

impl DataSetBulkService {
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

    /// Convert calamine range to CSV string with proper escaping and consistent column counts
    fn range_to_csv(range: &calamine::Range<calamine::Data>) -> Result<Vec<u8>> {
        use calamine::Data;

        let mut csv_data = Vec::new();

        // Create a scope for the CSV writer to ensure it's dropped before we return csv_data
        {
            let mut csv_writer = csv::Writer::from_writer(&mut csv_data);

            // Ensure consistent width for all rows
            let width = range.width();

            for row_idx in 0..range.height() {
                let mut row_values = Vec::new();

                // Always write exactly 'width' columns to ensure consistency
                for col_idx in 0..width {
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

                // Write CSV row with proper escaping
                csv_writer.write_record(&row_values)?;
            }

            csv_writer.flush()?;
        } // csv_writer is dropped here, releasing the borrow on csv_data

        Ok(csv_data)
    }

    /// Convert graph_json to CSV rows
    fn json_array_to_csv_rows(array: &[serde_json::Value]) -> Result<Vec<Vec<String>>> {
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

    fn ensure_layer_alias_column(rows: &mut Vec<Vec<String>>) {
        if rows.is_empty() {
            return;
        }

        if rows[0].iter().any(|header| header == "alias") {
            return;
        }

        let insert_idx = rows[0]
            .iter()
            .position(|header| header == "label")
            .map(|idx| idx + 1)
            .unwrap_or(rows[0].len());

        for (row_idx, row) in rows.iter_mut().enumerate() {
            row.insert(
                insert_idx,
                if row_idx == 0 {
                    "alias".to_string()
                } else {
                    String::new()
                },
            );
        }
    }

    fn build_sheet_name(
        base: &str,
        suffix: &str,
        used: &mut HashSet<String>,
        max_len: Option<usize>,
    ) -> String {
        let mut raw = if suffix.is_empty() {
            base.to_string()
        } else {
            format!("{} - {}", base, suffix)
        };
        if let Some(limit) = max_len {
            raw = Self::truncate_to_len(&raw, limit);
        }
        let mut candidate = raw.clone();
        let mut counter = 2;
        while used.contains(&candidate) {
            let appendix = format!(" ({})", counter);
            counter += 1;
            let prefix_len = max_len
                .map(|limit| limit.saturating_sub(appendix.chars().count()))
                .unwrap_or(usize::MAX);
            let mut prefix = Self::truncate_to_len(&raw, prefix_len);
            prefix.push_str(&appendix);
            candidate = prefix;
        }
        used.insert(candidate.clone());
        candidate
    }

    fn truncate_to_len(name: &str, limit: usize) -> String {
        if name.chars().count() <= limit {
            return name.to_string();
        }
        name.chars().take(limit).collect()
    }

    /// Export datasets to XLSX format
    /// Each dataset becomes a separate sheet named with its name containing CSV data
    pub async fn export_to_xlsx(&self, dataset_ids: &[i32]) -> Result<Vec<u8>> {
        let mut workbook = Workbook::new();

        // Fetch all requested datasets
        let datasets = data_sets::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to fetch datasets")?
            .into_iter()
            .filter(|ds| dataset_ids.contains(&ds.id))
            .collect::<Vec<_>>();

        tracing::info!("Exporting {} datasets to XLSX", datasets.len());

        // Check for duplicate names
        let mut name_counts = std::collections::HashMap::new();
        for ds in &datasets {
            *name_counts.entry(&ds.name).or_insert(0) += 1;
        }
        let duplicates: Vec<&String> = name_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(&name, _)| name)
            .collect();

        if !duplicates.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot export: duplicate dataset names found: {}",
                duplicates
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let mut used_sheet_names = HashSet::new();
        for dataset in datasets {
            let parsed: serde_json::Value = serde_json::from_str(&dataset.graph_json)
                .context("Failed to parse graph_json during export")?;
            let sections = [("nodes", "Nodes"), ("edges", "Edges"), ("layers", "Layers")];
            let mut section_written = false;
            // Create a sheet named with the dataset name
            for (key, label) in sections {
                if let Some(array) = parsed.get(key).and_then(|v| v.as_array()) {
                    if array.is_empty() {
                        continue;
                    }
                    let mut rows = Self::json_array_to_csv_rows(array)?;
                    if key == "layers" {
                        Self::ensure_layer_alias_column(&mut rows);
                    }
                    let sheet_name = Self::build_sheet_name(
                        &dataset.name,
                        label,
                        &mut used_sheet_names,
                        Some(31),
                    );
                    let worksheet = workbook.add_worksheet();
                    worksheet.set_name(&sheet_name)?;

                    for (row_idx, row_data) in rows.iter().enumerate() {
                        for (col_idx, value) in row_data.iter().enumerate() {
                            if let Ok(num) = value.parse::<f64>() {
                                worksheet.write_number(row_idx as u32, col_idx as u16, num)?;
                            } else {
                                worksheet.write_string(row_idx as u32, col_idx as u16, value)?;
                            }
                        }
                    }
                    tracing::info!(
                        "Wrote {} rows to sheet {} ({})",
                        rows.len(),
                        sheet_name,
                        label
                    );
                    section_written = true;
                }
            }

            if !section_written {
                let sheet_name =
                    Self::build_sheet_name(&dataset.name, "Empty", &mut used_sheet_names, Some(31));
                let worksheet = workbook.add_worksheet();
                worksheet.set_name(&sheet_name)?;
                worksheet.write_string(0, 0, "Dataset contains no nodes, edges, or layers")?;
            }
        }

        // Save to buffer
        workbook.save_to_buffer().context("Failed to generate XLSX")
    }

    /// Export datasets to ODS format
    /// Each dataset becomes a separate sheet named with its name containing CSV data
    pub async fn export_to_ods(&self, dataset_ids: &[i32]) -> Result<Vec<u8>> {
        let mut workbook = WorkBook::new(locale!("en_US"));

        // Fetch all requested datasets
        let datasets = data_sets::Entity::find()
            .all(&self.db)
            .await
            .context("Failed to fetch datasets")?
            .into_iter()
            .filter(|ds| dataset_ids.contains(&ds.id))
            .collect::<Vec<_>>();

        tracing::info!("Exporting {} datasets to ODS", datasets.len());

        // Check for duplicate names
        let mut name_counts = std::collections::HashMap::new();
        for ds in &datasets {
            *name_counts.entry(&ds.name).or_insert(0) += 1;
        }
        let duplicates: Vec<&String> = name_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(&name, _)| name)
            .collect();

        if !duplicates.is_empty() {
            return Err(anyhow::anyhow!(
                "Cannot export: duplicate dataset names found: {}",
                duplicates
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        let mut used_sheet_names = HashSet::new();
        for dataset in datasets {
            let parsed: serde_json::Value = serde_json::from_str(&dataset.graph_json)
                .context("Failed to parse graph_json during export")?;
            let sections = [("nodes", "Nodes"), ("edges", "Edges"), ("layers", "Layers")];
            let mut section_written = false;

            for (key, label) in sections {
                if let Some(array) = parsed.get(key).and_then(|v| v.as_array()) {
                    if array.is_empty() {
                        continue;
                    }
                    let mut rows = Self::json_array_to_csv_rows(array)?;
                    if key == "layers" {
                        Self::ensure_layer_alias_column(&mut rows);
                    }
                    let sheet_name =
                        Self::build_sheet_name(&dataset.name, label, &mut used_sheet_names, None);
                    let mut sheet = Sheet::new(&sheet_name);

                    for (row_idx, row_data) in rows.iter().enumerate() {
                        for (col_idx, value) in row_data.iter().enumerate() {
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

                    workbook.push_sheet(sheet);
                    section_written = true;
                }
            }

            if !section_written {
                let sheet_name =
                    Self::build_sheet_name(&dataset.name, "Empty", &mut used_sheet_names, None);
                let mut sheet = Sheet::new(&sheet_name);
                sheet.set_value(
                    0,
                    0,
                    Value::Text("Dataset contains no nodes, edges, or layers".to_string()),
                );
                workbook.push_sheet(sheet);
            }
        }

        // Save to buffer
        let buffer = Vec::new();
        let result = spreadsheet_ods::write_ods_buf(&mut workbook, buffer)
            .context("Failed to generate ODS")?;
        Ok(result)
    }

    /// Import datasets from XLSX format
    /// Each sheet becomes a dataset containing the tabular data from that sheet
    pub async fn import_from_xlsx(
        &self,
        project_id: i32,
        xlsx_data: &[u8],
    ) -> Result<DataSetImportResult> {
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

        let service = DataSetService::new(self.db.clone());

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

                // Try to find existing dataset by name
                use sea_orm::ColumnTrait;
                use sea_orm::QueryFilter;
                if let Some(existing) = data_sets::Entity::find()
                    .filter(data_sets::Column::ProjectId.eq(project_id))
                    .filter(data_sets::Column::Name.eq(sheet_name.clone()))
                    .one(&self.db)
                    .await?
                {
                    tracing::info!(
                        "Found existing dataset '{}' (id: {}) - updating",
                        sheet_name,
                        existing.id
                    );

                    // Update the dataset with new CSV data
                    let filename = format!("{}.csv", existing.name);
                    let dataset = service.update_file(existing.id, filename, csv_data).await?;

                    updated_count += 1;
                    imported_ids.push(dataset.id);
                    tracing::info!("Updated dataset: {} (id: {})", dataset.name, dataset.id);
                    continue;
                }

                // Create new dataset
                let filename = format!("{}.csv", sheet_name);
                let dataset = service
                    .create_from_file(
                        project_id,
                        sheet_name.clone(),
                        Some("Imported from spreadsheet".to_string()),
                        filename,
                        FileFormat::Csv,
                        csv_data,
                        Some(data_type),
                    )
                    .await?;

                created_count += 1;
                imported_ids.push(dataset.id);
                tracing::info!("Created dataset: {} with id: {}", sheet_name, dataset.id);
            }
        }

        Ok(DataSetImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }

    /// Import datasets from ODS format
    /// Each sheet becomes a dataset containing the tabular data from that sheet
    pub async fn import_from_ods(
        &self,
        project_id: i32,
        ods_data: &[u8],
    ) -> Result<DataSetImportResult> {
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

        let service = DataSetService::new(self.db.clone());

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

                // Try to find existing dataset by name
                use sea_orm::ColumnTrait;
                use sea_orm::QueryFilter;
                if let Some(existing) = data_sets::Entity::find()
                    .filter(data_sets::Column::ProjectId.eq(project_id))
                    .filter(data_sets::Column::Name.eq(sheet_name.clone()))
                    .one(&self.db)
                    .await?
                {
                    tracing::info!(
                        "Found existing dataset '{}' (id: {}) - updating",
                        sheet_name,
                        existing.id
                    );

                    // Update the dataset with new CSV data
                    let filename = format!("{}.csv", existing.name);
                    let dataset = service.update_file(existing.id, filename, csv_data).await?;

                    updated_count += 1;
                    imported_ids.push(dataset.id);
                    tracing::info!("Updated dataset: {} (id: {})", dataset.name, dataset.id);
                    continue;
                }

                // Create new dataset
                let filename = format!("{}.csv", sheet_name);
                let dataset = service
                    .create_from_file(
                        project_id,
                        sheet_name.clone(),
                        Some("Imported from spreadsheet".to_string()),
                        filename,
                        FileFormat::Csv,
                        csv_data,
                        Some(data_type),
                    )
                    .await?;

                created_count += 1;
                imported_ids.push(dataset.id);
                tracing::info!("Created dataset: {} with id: {}", sheet_name, dataset.id);
            }
        }

        Ok(DataSetImportResult {
            created_count,
            updated_count,
            imported_ids,
        })
    }
}


pub struct DataSetImportResult {
    pub created_count: i32,
    pub updated_count: i32,
    pub imported_ids: Vec<i32>,
}
