# Input Validation Implementation Plan

**Date**: 2025-10-26
**Status**: Planned (Not Yet Implemented)
**Priority**: High (Phase 1.2 - Deferred to future sprint)

## Overview

Adding comprehensive input validation to GraphQL schema requires splitting dual-purpose types (both `SimpleObject` and `InputObject`) into separate input and output types. This is more invasive than initially estimated.

## Problem

Current schema uses dual-purpose types like:

```rust
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
```

Adding validators causes compilation errors:
```rust
#[graphql(validator(minimum = "-100000.0", maximum = "100000.0"))]
pub x: f64,  // ❌ Breaks SimpleObject derivation
```

## Required Changes

### 1. Split Types into Input/Output Variants

**Before:**
```rust
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "PositionInput")]
pub struct Position {
    pub x: f64,
    pub y: f64,
}
```

**After:**
```rust
// Output type (for queries)
#[derive(SimpleObject, Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

// Input type with validation (for mutations)
#[derive(InputObject, Clone, Debug)]
pub struct PositionInput {
    #[graphql(validator(minimum = "-100000.0", maximum = "100000.0"))]
    pub x: f64,
    #[graphql(validator(minimum = "-100000.0", maximum = "100000.0"))]
    pub y: f64,
}

impl From<PositionInput> for Position {
    fn from(input: PositionInput) -> Self {
        Position { x: input.x, y: input.y }
    }
}
```

### 2. Types Requiring Splitting

All found in `layercake-core/src/graphql/types/plan_dag.rs`:

1. **Position** (used everywhere in DAG nodes)
   - Impact: HIGH - used in many mutations
   - Lines: ~20 usages across codebase

2. **NodeMetadata**
   - Impact: MEDIUM - used in node creation/updates
   - Lines: ~10 usages

3. **EdgeMetadata**
   - Impact: MEDIUM - used in edge creation/updates
   - Lines: ~8 usages

### 3. Validation Rules to Apply

#### Project Input Types
```rust
#[derive(InputObject)]
pub struct CreateProjectInput {
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub name: String,
    #[graphql(validator(max_length = 2000))]
    pub description: Option<String>,
}
```

#### Position Input
```rust
#[derive(InputObject)]
pub struct PositionInput {
    #[graphql(validator(minimum = "-100000.0", maximum = "100000.0"))]
    pub x: f64,
    #[graphql(validator(minimum = "-100000.0", maximum = "100000.0"))]
    pub y: f64,
}
```

#### Node Metadata Input
```rust
#[derive(InputObject)]
pub struct NodeMetadataInput {
    #[graphql(validator(min_length = 1, max_length = 255))]
    pub label: String,
    #[graphql(validator(max_length = 2000))]
    pub description: Option<String>,
}
```

#### Edge Metadata Input
```rust
#[derive(InputObject)]
pub struct EdgeMetadataInput {
    #[graphql(validator(max_length = 255))]
    pub label: Option<String>,
    #[graphql(name = "dataType")]
    pub data_type: DataType,
}
```

#### Node ID Validation
```rust
#[derive(InputObject)]
pub struct NodePositionInput {
    #[graphql(validator(regex = r"^[a-z_]+_\d{3}$"))]
    pub node_id: String,
    pub position: PositionInput,
}
```

## Implementation Strategy

### Phase A: Non-Breaking Preparations (Week 1)
1. Create new `*Input` types alongside existing types
2. Add `From` trait implementations
3. Keep existing types unchanged
4. Add validators to new types
5. Update internal services to use new types

### Phase B: Migration (Week 2)
1. Update all mutations to use new `*Input` types
2. Update GraphQL schema introspection
3. Test with frontend (may require frontend changes)
4. Document breaking changes

### Phase C: Cleanup (Week 3)
1. Deprecate dual-purpose types (if any remain)
2. Remove old code after migration period
3. Update documentation

## Impact Analysis

### Backend Changes Required

**Files to modify:**
- `layercake-core/src/graphql/types/plan_dag.rs` (PRIMARY)
- `layercake-core/src/graphql/types/project.rs`
- `layercake-core/src/graphql/mutations/mod.rs` (update all mutation signatures)
- `layercake-core/src/graphql/mutations/plan_dag_delta.rs`

**Estimated lines changed**: ~300-500 lines

### Frontend Changes Required

**Breaking changes:**
- Input types in mutations will have different names
- `PositionInput` instead of `Position` in mutations
- Code generation will create new TypeScript types

**Mitigation:**
- Frontend uses generated types from GraphQL schema
- Re-run code generation will update types automatically
- Minor TypeScript compilation fixes may be needed

### Testing Required

1. **Unit tests**: All mutations with new input types
2. **Integration tests**: Frontend→Backend full flow
3. **Validation tests**: Ensure validators reject invalid inputs
4. **Regression tests**: Existing functionality still works

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Breaking frontend | HIGH | Coordinate with frontend team, staged rollout |
| Type confusion | MEDIUM | Clear naming (`*Input` suffix), good documentation |
| Performance impact | LOW | Validation is fast, negligible overhead |
| Incomplete migration | MEDIUM | Comprehensive test coverage, code review |

## Alternatives Considered

### Alternative 1: Custom Validation in Mutation Resolvers
**Pros:**
- No type splitting required
- Easier to implement

**Cons:**
- Validation logic scattered across resolvers
- Less declarative, harder to maintain
- No automatic GraphQL schema documentation
- Errors occur late (at resolver level, not schema level)

### Alternative 2: Use `#[guard]` Instead of `#[validator]`
**Pros:**
- More flexible validation logic
- Can access context

**Cons:**
- Still requires separate input types
- More complex implementation
- Less standardised

### Alternative 3: Accept Invalid Data (Current State)
**Pros:**
- No changes needed

**Cons:**
- Security risk
- Data integrity issues
- Poor user experience (late error feedback)

**Decision**: Full implementation required, but deferred to allow proper planning and coordination.

## Timeline

**Total estimated effort**: 2-3 weeks with proper testing

**Proposed schedule:**
- Week 1: Implementation (backend types + mutations)
- Week 2: Frontend updates + integration testing
- Week 3: QA + deployment

**Dependencies:**
- Frontend team availability for coordination
- QA resources for testing
- Staging environment for validation

## Success Criteria

- [ ] All input types have validators
- [ ] All mutations use validated input types
- [ ] Frontend updated with new types
- [ ] All tests passing
- [ ] Zero production incidents related to validation
- [ ] GraphQL schema documentation shows validation rules

## References

- **async-graphql validators docs**: https://async-graphql.github.io/async-graphql/en/input_value_validators.html
- **Related issue**: See GRAPHQL_SCHEMA_REVIEW.md Section 10
- **Original PR**: (to be created)

---

**Status**: This plan is documented but implementation is deferred. Phase 1 will focus on lower-risk improvements first (deprecation directives, documentation, pagination).
