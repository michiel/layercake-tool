# Node Configuration Migration Guide

**Date**: 2025-10-26
**Status**: Documented (Phase 3.2)

## Overview

This document explains the migration logic for node configurations in the PlanDAG system. The migration logic handles backward compatibility when loading legacy plan configurations.

## Background

The PlanDAG node configuration schema has evolved through two major versions:

- **Schema v1** (Legacy): Single transform/filter with configuration object
- **Schema v2** (Current): Array-based transforms with query builder for filters

To maintain backward compatibility, the deserialization code automatically migrates v1 configurations to v2 format.

## Migration Architecture

### Location

Migration logic is located in `layercake-core/src/graphql/types/plan_dag.rs`:

- **Transform Node**: Lines 155-178 (TransformNodeConfig deserializer)
- **Filter Node**: Lines 632-668 (FilterNodeConfig deserializer)

### Design Decision

**Why migration logic is in deserializers:**

1. **Avoid Circular Dependencies**: plan_dag types depend on services, services depend on plan_dag
2. **Single Responsibility**: Deserialization *is* the migration point - data enters system here
3. **Type Safety**: Wire types and domain types stay together
4. **Performance**: No additional service layer overhead for every load

**Trade-offs:**
- ✅ Simple, no circular dependencies
- ✅ Automatic migration on load
- ⚠️ Business logic in trait implementation (acceptable for schema migration)
- ⚠️ Harder to unit test migration logic independently

## Transform Node Migration

### Schema Evolution

**Schema v1 (Legacy)**:
```json
{
  "transformType": "PartitionDepthLimit",
  "transformConfig": {
    "maxPartitionDepth": 3,
    "generateHierarchy": true
  }
}
```

**Schema v2 (Current)**:
```json
{
  "transforms": [
    {
      "kind": "PartitionDepthLimit",
      "params": {
        "maxPartitionDepth": 3,
        "enabled": true
      }
    },
    {
      "kind": "GenerateHierarchy",
      "params": {
        "enabled": true
      }
    }
  ]
}
```

### Migration Logic

The `TransformNodeConfig` deserializer handles three cases:

1. **Current Schema (v2)**: `{ transforms: [...] }`
   - Deserializes array directly
   - Applies `with_default_enabled` to set `enabled: true` if missing

2. **Legacy Schema (v1)**: `{ transformType: "...", transformConfig: {...} }`
   - Converts single transform_type to array of transforms
   - Uses `LegacyTransformConfig::into_graph_transforms()` for conversion
   - Extracts multiple transforms from monolithic config

3. **Empty**: `{}`
   - Returns empty transforms array

### Implementation Details

#### Wire Type

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformNodeConfigWire {
    transforms: Option<Vec<GraphTransform>>,
    transform_type: Option<LegacyTransformType>,
    transform_config: Option<LegacyTransformConfig>,
}
```

#### Legacy Transform Types

```rust
enum LegacyTransformType {
    PartitionDepthLimit,
    InvertGraph,
    FilterNodes,
    FilterEdges,
}
```

#### Conversion Examples

**PartitionDepthLimit** (v1 → v2):
```rust
// v1 input
transformType: "PartitionDepthLimit"
transformConfig: { maxPartitionDepth: 3, generateHierarchy: true }

// Converts to v2
transforms: [
    { kind: "PartitionDepthLimit", params: { maxPartitionDepth: 3, enabled: true } },
    { kind: "GenerateHierarchy", params: { enabled: true } }
]
```

**InvertGraph** (v1 → v2):
```rust
// v1 input
transformType: "InvertGraph"
transformConfig: {}

// Converts to v2
transforms: [
    { kind: "InvertGraph", params: { enabled: true } }
]
```

**FilterNodes** (v1 → v2):
```rust
// v1 input
transformType: "FilterNodes"
transformConfig: { nodeFilter: "category = 'data'" }

// Converts to v2
transforms: [
    { kind: "FilterNodes", params: { nodeFilter: "category = 'data'", enabled: true } }
]
```

## Filter Node Migration

### Schema Evolution

**Schema v1 (Legacy)**:
```json
{
  "filters": [
    {
      "kind": "Query",
      "params": {
        "queryConfig": {
          "targets": [...],
          "mode": "And"
        }
      }
    }
  ]
}
```

**Schema v2 (Current)**:
```json
{
  "query": {
    "targets": [...],
    "mode": "And"
  }
}
```

### Migration Logic

The `FilterNodeConfig` deserializer handles two cases:

1. **Current Schema (v2)**: `{ query: {...} }`
   - Deserializes query directly
   - Applies normalization via `query.normalized()`

2. **Legacy Schema (v1)**: `{ filters: [{kind: "Query", params: {...}}] }`
   - Searches filters array for "Query" kind
   - Extracts queryConfig from params
   - Applies normalization

**Error Handling**: Returns error if no valid query configuration found in either format.

### Implementation Details

#### Wire Type

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FilterNodeConfigWire {
    query: Option<QueryFilterConfig>,
    filters: Option<Vec<LegacyGraphFilter>>,
}
```

#### Legacy Filter Type

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyGraphFilter {
    kind: Option<String>,
    params: Option<LegacyGraphFilterParams>,
}

impl LegacyGraphFilter {
    fn is_query(&self) -> bool {
        // Matches "query" or "querytext" (case-insensitive)
        self.kind
            .as_deref()
            .map(|k| k.eq_ignore_ascii_case("query") || k.eq_ignore_ascii_case("querytext"))
            .unwrap_or(false)
    }
}
```

#### Query Normalization

The `QueryFilterConfig::normalized()` method ensures default values are set:

```rust
pub fn normalized(self) -> Self {
    Self {
        targets: self.targets.into_iter().map(|t| t.normalized()).collect(),
        mode: self.mode, // Already has default via serde
    }
}
```

## Testing Migration Logic

### Current Test Coverage

The migration logic currently has **implicit** test coverage through:
- Integration tests loading legacy plan files
- GraphQL query tests with old data formats

### Recommended Explicit Tests

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform_migration_v1_to_v2() {
        let v1_json = json!({
            "transformType": "PartitionDepthLimit",
            "transformConfig": {
                "maxPartitionDepth": 3
            }
        });

        let config: TransformNodeConfig = serde_json::from_value(v1_json).unwrap();
        assert_eq!(config.transforms.len(), 1);
        assert_eq!(config.transforms[0].kind, GraphTransformKind::PartitionDepthLimit);
        assert_eq!(config.transforms[0].params.max_partition_depth, Some(3));
        assert_eq!(config.transforms[0].params.enabled, Some(true));
    }

    #[test]
    fn test_filter_migration_v1_to_v2() {
        let v1_json = json!({
            "filters": [{
                "kind": "Query",
                "params": {
                    "queryConfig": {
                        "targets": [],
                        "mode": "And"
                    }
                }
            }]
        });

        let config: FilterNodeConfig = serde_json::from_value(v1_json).unwrap();
        assert!(config.query.targets.is_empty());
    }

    #[test]
    fn test_filter_migration_missing_query_error() {
        let invalid_json = json!({
            "filters": [{
                "kind": "Other",
                "params": {}
            }]
        });

        let result: Result<FilterNodeConfig, _> = serde_json::from_value(invalid_json);
        assert!(result.is_err());
    }
}
```

## Migration Path for New Developers

### Understanding Legacy Formats

1. **Check Plan Files**: Look in `sample/` directory for old plan YAML files
2. **Read Wire Types**: Understand what legacy formats look like in `plan_dag.rs`
3. **Trace Migration**: Follow deserializer logic to see how conversion happens

### Adding New Node Types

When adding a new node type with configuration:

1. **Design Current Schema**: Define v2 format first
2. **Consider Backward Compatibility**: Will users have v1 data?
3. **Implement Migration**: If yes, add wire type + migration logic
4. **Document**: Add to this guide with examples
5. **Test**: Add explicit migration tests

### Deprecating Legacy Support

When removing v1 support (future):

1. **Announce Deprecation**: Give users 6+ months notice
2. **Provide Migration Tool**: CLI command to convert old plans to v2
3. **Remove Wire Types**: Delete Legacy* types
4. **Simplify Deserializers**: Replace custom impl with `#[derive(Deserialize)]`
5. **Update Documentation**: Remove migration guide

## Performance Considerations

### Deserialization Overhead

Migration logic runs on **every deserialization**, including:
- Loading plans from database
- Receiving GraphQL mutations
- Importing plan files

**Current Impact**: Negligible (< 1ms per node for typical configs)

### Optimization Opportunities

If migration becomes a bottleneck:

1. **Cache Deserialized Configs**: Store normalized v2 in database
2. **Lazy Migration**: Keep v1 format until first access
3. **Batch Migration**: Background job to upgrade all plans to v2

## Future Schema Versions

### Schema v3 Planning

When designing schema v3:

1. **Add Version Field**: Explicit `schema_version: 3` in all configs
2. **Chain Migrations**: v1 → v2 → v3 (don't skip versions)
3. **Migration Service**: Extract to dedicated `NodeConfigMigrationService`
4. **Versioned Wire Types**: `TransformNodeConfigWireV1`, `V2`, `V3`

### Recommended Approach

```rust
pub enum TransformNodeConfigWire {
    V1(TransformNodeConfigWireV1),
    V2(TransformNodeConfigWireV2),
    V3(TransformNodeConfigWireV3),
}

impl NodeConfigMigrationService {
    pub fn migrate_transform(wire: TransformNodeConfigWire) -> TransformNodeConfig {
        match wire {
            V1(v1) => Self::migrate_v1_to_current(v1),
            V2(v2) => Self::migrate_v2_to_current(v2),
            V3(v3) => Self::v3_to_current(v3),  // Might be identity
        }
    }
}
```

## Summary

### Current State

- Migration logic embedded in deserializers (lines 155-178, 632-668 of plan_dag.rs)
- Handles v1 → v2 automatically on load
- No explicit version tracking
- Works well for current needs

### Improvements Made (Phase 3.2)

- ✅ Comprehensive documentation of migration logic
- ✅ Clear comments in source code
- ✅ Explicit test recommendations
- ✅ Future schema evolution guidance

### Next Steps

- Add explicit migration tests (recommended for Phase 3.5)
- Consider extracting to service when adding schema v3
- Monitor performance as plan sizes grow

---

**Related Files**:
- `layercake-core/src/graphql/types/plan_dag.rs` - Migration implementation
- `sample/ref/plan.yaml` - Example plans with both v1 and v2 formats

**Status**: Migration logic documented and clarified
**Phase**: 3.2 Complete
