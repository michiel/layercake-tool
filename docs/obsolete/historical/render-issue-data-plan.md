# Rendering Issue Investigation Plan

## Problem Statement

Occasionally datasets show correctly in the GraphQL graph preview but export incorrectly to Handlebars templates:
- Node labels are rendered as node IDs instead of labels
- Layer styling is not applied

This appears to be data-dependent, suggesting a parsing, sanitisation, or logic issue in the data preparation pipeline.

## Root Cause Analysis

### Issue Located
**File**: `layercake-core/src/services/graph_service.rs:85-86`

When converting database models to the core `Graph` struct for export:

```rust
label: db_node.label.unwrap_or_default(),  // None → ""
layer: db_node.layer.unwrap_or_else(|| "default".to_string()),  // None → "default"
```

### Data Flow Comparison

#### GraphQL Preview Path (✓ Works Correctly)
1. Database `graph_nodes` table → `Option<String>` for label/layer
2. GraphQL `GraphNodePreview` struct → `Option<String>` (preserves None)
3. Frontend displays correctly with null handling

#### Handlebars Export Path (✗ Fails)
1. Database `graph_nodes` table → `Option<String>` for label/layer
2. **`graph_service.rs:build_graph_from_dag_graph()`** → Converts None to:
   - `label: ""` (empty string)
   - `layer: "default"`
3. Core `Graph` struct → `String` (not Option)
4. Handlebars template → Receives empty labels and invalid layer references

### Consequences

1. **Empty Labels**: Templates may fall back to rendering node IDs when label is empty
2. **Missing Layer Styling**: Layer "default" likely doesn't exist in the layer map, causing styling to fail
3. **Silent Failures**: No error notifications when data is malformed

## Investigation Tasks

### 1. Audit Data Quality
- [ ] Identify which datasets have None values for label/layer
- [ ] Determine if None values are legitimate or indicate upstream data issues
- [ ] Check if CSV/JSON import validates required fields

### 2. Review Conversion Points
- [ ] `graph_service.rs:81-93` - Database to Graph conversion
- [ ] `graph_builder.rs:302-320` - Graph to data_set conversion
- [ ] `to_custom.rs:42-52` - Handlebars data preparation
- [ ] Handlebars helpers in `common/handlebars.rs` - Template rendering logic

### 3. Test Edge Cases
- [ ] Create test dataset with None label
- [ ] Create test dataset with None layer
- [ ] Create test dataset with invalid layer reference
- [ ] Verify preview vs export behaviour for each

### 4. Check Layer Map Construction
- [ ] Verify `graph.get_layer_map()` returns all layers from database
- [ ] Confirm layer IDs match between nodes and layer definitions
- [ ] Check if "default" layer exists or needs to be created

## Recommendations

### Immediate Fixes

#### 1. Add Validation at Import
**Location**: `layercake-core/src/pipeline/graph_builder.rs:428-439`

```rust
let node = NodeData {
    label: node_val["label"]
        .as_str()
        .filter(|s| !s.is_empty())  // Reject empty strings
        .map(|s| s.to_string()),
    layer: node_val["layer"]
        .as_str()
        .filter(|s| !s.is_empty())  // Reject empty strings
        .map(|s| s.to_string()),
    ...
};
```

**Then validate required fields**:
```rust
if node.label.is_none() {
    return Err(anyhow!("Node {} missing required 'label' field", id));
}
if node.layer.is_none() {
    return Err(anyhow!("Node {} missing required 'layer' field", id));
}
```

#### 2. Improve Fallback Logic
**Location**: `layercake-core/src/services/graph_service.rs:81-93`

```rust
label: db_node.label.unwrap_or_else(|| {
    tracing::warn!("Node {} missing label, using ID as fallback", db_node.id);
    db_node.id.clone()  // Use ID instead of empty string
}),
layer: db_node.layer.unwrap_or_else(|| {
    tracing::warn!("Node {} missing layer, using 'uncategorised'", db_node.id);
    "uncategorised".to_string()  // Ensure this layer exists
}),
```

**Add required layer creation**:
```rust
// Ensure fallback layer exists
if !graph_layers.iter().any(|l| l.id == "uncategorised") {
    graph_layers.push(Layer {
        id: "uncategorised".to_string(),
        label: "Uncategorised".to_string(),
        background_color: "cccccc".to_string(),
        text_color: "000000".to_string(),
        border_color: "999999".to_string(),
        dataset: None,
    });
}
```

#### 3. Add Handlebars Template Guards
**Location**: Handlebars templates using node data

```handlebars
{{#each hierarchy_nodes}}
  {{! Use label if present, otherwise fall back to id }}
  {{#if label}}
    {{label}}
  {{else}}
    {{id}} (unlabelled)
  {{/if}}

  {{! Validate layer exists before applying styles }}
  {{#if (lookup ../layers layer)}}
    style="{{! apply layer styling }}"
  {{/if}}
{{/each}}
```

#### 4. Add Error Notifications
**Location**: Multiple points in the pipeline

```rust
// In graph_service.rs
let mut validation_warnings = Vec::new();

for db_node in &db_graph_nodes {
    if db_node.label.is_none() {
        validation_warnings.push(format!("Node {} has no label", db_node.id));
    }
    if db_node.layer.is_none() {
        validation_warnings.push(format!("Node {} has no layer", db_node.id));
    }
    // Check layer exists
    if let Some(layer_id) = &db_node.layer {
        if !db_layers.iter().any(|l| l.layer_id == *layer_id) {
            validation_warnings.push(format!(
                "Node {} references non-existent layer '{}'",
                db_node.id, layer_id
            ));
        }
    }
}

// Add warnings to graph annotations
if !validation_warnings.is_empty() {
    tracing::warn!("Graph {} has {} data quality issues", graph_id, validation_warnings.len());
    // Optionally append to graph.annotations for visibility
}
```

### Long-term Improvements

#### 1. Schema Validation
- Add database constraints to require label/layer (or make them truly optional in core Graph)
- Implement JSON schema validation for imported data
- Add pre-import data quality checks with user-friendly error messages

#### 2. Core Type Consistency
**Option A**: Make Graph types use Option<String> consistently
```rust
pub struct Node {
    pub id: String,
    pub label: Option<String>,  // Change from String
    pub layer: Option<String>,  // Change from String
    ...
}
```

**Option B**: Validate and normalise at boundaries
- Enforce non-null at database insert time
- Use database defaults for missing values
- Make core Graph struct truly non-optional

#### 3. Export Pre-flight Checks
Add validation step before export:
```rust
fn validate_for_export(graph: &Graph) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for node in &graph.nodes {
        if node.label.is_empty() {
            errors.push(format!("Node {} has empty label", node.id));
        }
        if !graph.layers.iter().any(|l| l.id == node.layer) {
            errors.push(format!("Node {} layer '{}' not found", node.id, node.layer));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

#### 4. User Notifications
- Add GraphQL subscription for data quality warnings
- Display validation errors in the UI before export
- Provide data repair suggestions (e.g., "Node X has no label - click to edit")

#### 5. Comprehensive Testing
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_export_with_none_label() {
        // Create graph with None label
        // Verify export handles gracefully
    }

    #[test]
    fn test_export_with_invalid_layer() {
        // Create node with non-existent layer
        // Verify error or fallback
    }

    #[test]
    fn test_preview_matches_export() {
        // Compare GraphQL preview data with export data
        // Ensure consistency
    }
}
```

## Implementation Status

### ✅ Completed (P0 Immediate Fixes)
1. ✅ Use node ID as label fallback (not empty string) - `graph_service.rs:88-94`
2. ✅ Improve logging for None values (warnings, not silent) - `graph_service.rs:138-151`
3. ✅ Empty layer means inherit default styling - `graph_service.rs:97,117-122`

**Changes Made:**
- Modified `graph_service.rs:build_graph_from_dag_graph()` to:
  - Use node ID as label when label is None (improves visibility in exports)
  - Use empty string for layer when layer is None (inherits default styling)
  - Track and log warnings when nodes have missing labels
  - Log debug info when edges have no layer (common, not an error)

### P0 (Remaining)
1. ~~Add fallback layer "uncategorised"~~ - Not needed, empty layer is valid and inherits default styling

### P1 (Short-term)
1. Add validation warnings to graph annotations
2. Implement export pre-flight check
3. Add Handlebars template guards

### P2 (Medium-term)
1. Decide on core type strategy (Option vs. validated non-null)
2. Add database constraints or schema validation
3. Implement comprehensive test suite

### P3 (Long-term)
1. Add user-facing data quality notifications
2. Build data repair/editing UI
3. Implement JSON schema validation for imports

## Testing Strategy

1. **Create problematic dataset**:
   - CSV with empty label column
   - CSV with non-existent layer references
   - JSON with null values

2. **Verify behaviour**:
   - GraphQL preview display
   - Export to DOT format
   - Export to custom Handlebars template
   - Check for error notifications

3. **After fixes**:
   - Confirm data quality warnings appear
   - Verify fallbacks work correctly
   - Ensure layer styling applies with fallback layer
   - Validate error messages are actionable

## Success Criteria

- [ ] No silent failures when data has None values
- [ ] GraphQL preview and exports show consistent data
- [ ] User-facing error messages when data quality issues exist
- [ ] Fallback layer styling always applies
- [ ] Node labels never render as IDs unexpectedly
- [ ] Comprehensive test coverage for edge cases
- [ ] Documentation updated with data quality requirements
