# Data Source Upload & Import Guidelines

Layercake’s data-source pipeline now runs through the shared `AppContext`. GraphQL, MCP, and the console CLI all call the same helpers, so the runtime expectations below apply everywhere.

## Supported Formats

| Flow | Accepted File Types | Notes |
| ---- | ------------------- | ----- |
| `createDataSetFromFile` / MCP `create_data_source_from_file` | CSV, TSV, JSON | The `file_format` field must match the file extension. Data type is inferred from the `data_type` argument. |
| `bulkUploadDataSets` / MCP bulk upload | Base64-encoded CSV payloads (per file) | Each entry in the request is decoded and auto-detected; supply one file per data source. |
| `importDataSets` / MCP `import_data_sources` | XLSX or ODS spreadsheets | Each sheet becomes a data source. Sheet names must be unique within the workbook. |
| `createEmptyDataSet` | N/A | Produces an empty JSON graph placeholder (nodes/edges/layers) ready for manual editing. |

## Size Guidance

Uploads are processed entirely in memory. To keep the MCP runtime responsive:

- **Single-file uploads**: stay under ~5 MB per request. Larger CSV/JSON files should be uploaded via the console CLI, which streams to disk before processing.
- **Spreadsheet imports (XLSX/ODS)**: use workbooks under ~10 MB. Split very large data sets into multiple imports to avoid long-running conversions.

The platform does not currently enforce hard limits, but exceeding the recommendations above can stall long-running agent sessions. Both GraphQL and MCP will surface a “payload too large” error once gateway limits are introduced.

## MIME / Content-Type Expectations

- GraphQL callers send the raw Base64 string as input; HTTP clients should still set `Content-Type: application/json` because the payload is embedded in the mutation variables.
- MCP tools accept JSON arguments and expect the file payloads to be Base64 strings. No additional MIME metadata is required.
- Exports return Base64 strings plus the generated filename and format (`xlsx` or `ods`). Callers should decode the `fileContent` field and write it to disk.

## Verifying Parity

The parity smoke test (`layercake-core/tests/parity_smoke_test.rs`) covers:

1. Creating an empty data source via GraphQL and reading it back via MCP.
2. Updating the data source through MCP and verifying the GraphQL response.
3. Comparing MCP `list_data_sources` output with the GraphQL `dataSources` query.

If you extend the data-source API (new formats, validation rules, etc.), update this document and add matching assertions to the smoke test.
