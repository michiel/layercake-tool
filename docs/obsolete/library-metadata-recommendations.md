# Library Metadata System Recommendations

## Current State

The current `DatasetMetadata` structure stores:
```rust
pub struct DatasetMetadata {
    pub format: String,           // "csv", "tsv", "json"
    pub data_type: String,        // "nodes", "edges", "layers", "graph"
    pub filename: String,
    pub row_count: Option<usize>,
    pub column_count: Option<usize>,
    pub headers: Option<Vec<String>>,
}
```

### Problems
1. **Type safety**: Format and data_type are strings, not enums
2. **Validation**: No validation at creation time
3. **Inference dependency**: System falls back to inference when metadata is missing/incorrect
4. **Limited extensibility**: No version field for schema evolution
5. **No user control**: Users can't manually set/correct type information

## Recommendations

### 1. Enhance Metadata Structure (Short-term)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum LibraryMetadata {
    #[serde(rename = "1")]
    V1(DatasetMetadataV1),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetMetadataV1 {
    // Explicit type information (required)
    pub format: FileFormat,        // Use enum instead of string
    pub data_type: DataType,       // Use enum instead of string

    // File information
    pub filename: String,
    pub original_filename: Option<String>, // Preserve upload name

    // Content statistics
    pub row_count: Option<usize>,
    pub column_count: Option<usize>,
    pub headers: Option<Vec<String>>,
    pub file_size_bytes: Option<i64>,

    // Detection metadata
    pub type_source: TypeDetectionSource, // How was type determined?
    pub inferred_from: Option<String>,    // "filename" | "headers" | "manual"
    pub confidence: Option<f32>,          // 0.0-1.0 confidence score

    // Validation
    pub validated: bool,              // Has user confirmed this is correct?
    pub validation_errors: Vec<String>, // Any issues found

    // User overrides
    pub user_override: bool,          // Did user manually set type?
    pub notes: Option<String>,        // User notes about the dataset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDetectionSource {
    Filename,      // Inferred from filename
    Headers,       // Inferred from CSV headers
    Manual,        // User specified
    Automatic,     // System default
    Migrated,      // From old metadata format
}
```

### 2. Validation at Upload (Medium-term)

```rust
impl LibraryItemService {
    pub async fn create_dataset_with_validation(
        &self,
        name: String,
        description: Option<String>,
        tags: Vec<String>,
        file_name: String,
        // User can optionally specify format/type
        format_hint: Option<FileFormat>,
        type_hint: Option<DataType>,
        content_type: Option<String>,
        bytes: Vec<u8>,
    ) -> Result<(library_items::Model, Vec<String>)> {
        // 1. Determine format
        let format = format_hint
            .or_else(|| FileFormat::from_extension(&file_name))
            .ok_or_else(|| anyhow!("Cannot determine format"))?;

        // 2. Infer data type from content
        let inferred_type = infer_data_type(&file_name, &format, &bytes)?;

        // 3. Use type hint if provided and compatible
        let data_type = type_hint
            .filter(|t| t.is_compatible_with_format(&format))
            .unwrap_or(inferred_type);

        // 4. Validate the combination
        let warnings = self.validate_dataset(&format, &data_type, &bytes)?;

        // 5. Build rich metadata
        let metadata = self.build_rich_metadata(
            &format,
            &data_type,
            &file_name,
            &bytes,
            type_hint.is_some(),
        )?;

        // 6. Create item
        let item = self.create_dataset_item_internal(
            name,
            description,
            tags,
            file_name,
            format,
            data_type,
            content_type,
            bytes,
            metadata,
        ).await?;

        Ok((item, warnings))
    }
}
```

### 3. UI Improvements (Short-term - Immediate)

**Library Item Edit Dialog** should include:
- **Description** field (editable)
- **Tags** selector (editable)
- **Metadata viewer/editor**:
  - Format (dropdown: csv, tsv, json)
  - Data Type (dropdown: nodes, edges, layers, graph)
  - Detected headers (read-only)
  - Row/column counts (read-only)
  - Validation status indicator
  - "Re-detect Type" button to re-run inference
  - Notes field for user comments

**Library Item Upload Dialog** should include:
- Format auto-detection with override option
- Type auto-detection with override option
- Validation preview before upload
- Warning messages if inference is uncertain

### 4. Migration Strategy (Medium-term)

```rust
pub fn migrate_metadata(old: &str) -> Result<LibraryMetadata> {
    // Try to parse as old format
    if let Ok(old_meta) = serde_json::from_str::<OldDatasetMetadata>(old) {
        let format = old_meta.format.parse()
            .unwrap_or(FileFormat::Csv);
        let data_type = old_meta.data_type.parse()
            .unwrap_or(DataType::Nodes);

        return Ok(LibraryMetadata::V1(DatasetMetadataV1 {
            format,
            data_type,
            filename: old_meta.filename,
            original_filename: None,
            row_count: old_meta.row_count,
            column_count: old_meta.column_count,
            headers: old_meta.headers,
            file_size_bytes: None,
            type_source: TypeDetectionSource::Migrated,
            inferred_from: None,
            confidence: None,
            validated: false,
            validation_errors: vec![],
            user_override: false,
            notes: None,
        }));
    }

    // If already new format, parse directly
    serde_json::from_str(old)
}
```

### 5. Content Validation (Long-term)

```rust
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<String>,
}

pub enum ValidationError {
    MissingRequiredColumn(String),
    InvalidDataType { column: String, expected: String, found: String },
    DuplicateIds(Vec<String>),
    BrokenReferences { edge_id: String, missing_node: String },
}

pub enum ValidationWarning {
    EmptyColumn(String),
    SuspiciousValues { column: String, examples: Vec<String> },
    InconsistentFormatting(String),
}

impl LibraryItemService {
    pub async fn validate_dataset(
        &self,
        format: &FileFormat,
        data_type: &DataType,
        bytes: &[u8],
    ) -> Result<ValidationResult> {
        // Validate CSV structure
        // Check required columns
        // Validate data types
        // Check for common issues
        // Return detailed report
    }
}
```

## Implementation Priority

### Phase 1 (Immediate) ✓
- [x] Fix immediate import issues with inference priority
- [ ] Add edit dialog for library items (description, tags, metadata viewer)
- [ ] Show current metadata in UI
- [ ] Add validation indicators

### Phase 2 (Next Sprint)
- [ ] Enhance metadata structure with versioning
- [ ] Add format/type override in upload UI
- [ ] Implement validation at upload
- [ ] Add "Re-detect Type" functionality
- [ ] Show warnings when metadata is uncertain

### Phase 3 (Future)
- [ ] Implement content validation
- [ ] Add metadata migration system
- [ ] Build confidence scoring
- [ ] Add batch validation tools
- [ ] Export validation reports

## Benefits

1. **Reliability**: Explicit metadata reduces inference failures
2. **User Control**: Users can correct misdetections
3. **Transparency**: Show how types were detected
4. **Extensibility**: Versioned metadata allows future enhancements
5. **Debugging**: Rich metadata helps troubleshoot import issues
6. **Validation**: Catch errors before import into projects
