# Layercake Graph Architecture Changes - Implementation Plan

## Overview

This plan addresses fundamental changes to the layercake graph model regarding partitions and hierarchy:

**Changes:**
1. **Flow nodes do NOT require partition membership** - `belongs_to` becomes truly optional
2. **Implicit global partition** - No explicit node needed for unattached nodes
3. **No synthetic root nodes by default** - Transformations will not introduce hard-coded nodes like "Hierarchy"
   - **Exception**: `GenerateHierarchy` transform may create synthetic root when multiple roots exist (explicit opt-in for single-root requirement)

**Goal:** Support both flat flow graphs and optional hierarchical organization without forcing partition structure.

**Key Requirements:**
1. **Tree Validation (NEW):** When `belongs_to` references exist, they MUST form a valid tree:
   - All `belongs_to` references must point to existing nodes
   - No cycles allowed in `belongs_to` chains
   - Validation errors when structure is invalid
2. **Forest Rendering:** Tree/hierarchy renderers display unattached nodes at top level alongside partition roots:
   - Top level = all nodes without `belongs_to` (flow nodes + partition roots)
   - Children recursively nested under their parents
   - Creates "forest" structure with multiple top-level nodes

## Current State Analysis

### Current Architecture

**Partition System:**
- `is_partition: bool` - Marks nodes as organizational containers
- `belongs_to: Option<String>` - References parent partition's external_id
- Partition nodes cannot have incoming/outgoing flow edges
- Flow nodes are expected (but not enforced) to belong to a partition

**Hierarchy Generation:**
- `generate_hierarchy()` creates a hard-coded "Hierarchy" root node
- Converts `belongs_to` relationships into explicit edges
- Forces ALL nodes to belong to synthetic root
- Flattens partition structure in the process

**Auto-Remediation:**
- `ensure_partition_hierarchy()` synthesizes partition metadata when missing
- Creates "synthetic_partition_root"
- Infers `belongs_to` from edge topology

### Current Validation Rules

**Enforced:**
- Edges cannot connect to partition nodes (graph_builder.rs:483-502)
- Edges cannot cross partition/non-partition boundaries (graph.rs:1610-1629)

**Not Enforced:**
- Flow nodes must belong to a partition (this becomes optional)

**Must Be Enforced (NEW):**
- When `belongs_to` references exist, they MUST reference existing nodes
- When `belongs_to` references exist, they MUST form a valid tree (no cycles, no orphans)
- Invalid `belongs_to` references should cause validation errors

## Impact Assessment

### High Risk Components

#### 1. Tree/Hierarchy Renderers
**Files:**
- `layercake-core/src/export/to_plantuml_mindmap.rs`
- `layercake-core/src/export/to_plantuml_wbs.rs`
- `layercake-core/src/export/to_mermaid_mindmap.rs`
- `layercake-core/src/export/to_mermaid_treemap.rs`
- `layercake-core/src/export/to_dot_hierarchy.rs`

**Current Dependency:** Rely on `build_tree()` which requires `belongs_to` structure

**Impact:** Will break if no partition structure exists

**Solution:** Render unattached nodes at top level alongside root partition nodes:
- Top level: all nodes without `belongs_to` (unattached flow nodes + root partition nodes)
- Recursively render children under each partition node based on `belongs_to` hierarchy
- Creates a "forest" structure with multiple top-level nodes

#### 2. Partition Transform Nodes
**Files:**
- `layercake-core/src/graph.rs:835-942` - `modify_graph_limit_partition_depth()`
- `layercake-core/src/graph.rs:944-1066` - `modify_graph_limit_partition_width()`
- `layercake-core/src/graphql/types/plan_dag/transforms.rs:177-241` - PartitionDepthLimit
- `layercake-core/src/graphql/types/plan_dag/transforms.rs:243-305` - PartitionWidthLimit

**Current Dependency:** Auto-generate partition structure via `ensure_partition_hierarchy()`

**Impact:** Currently compensates for missing partitions by synthesizing them

**Solution:** Make these transforms no-op when no partitions exist, or provide alternative grouping

#### 3. Hierarchy Generation Transform
**Files:**
- `layercake-core/src/graph.rs:1235-1314` - `generate_hierarchy()`
- `layercake-core/src/graphql/types/plan_dag/transforms.rs:260-267` - GenerateHierarchy

**Current Behavior:** Always creates "Hierarchy" root node, rewrites entire graph

**Impact:** Creating synthetic root when there's already a single root violates new rules

**Solution (EXCEPTION TO NO-SYNTHETIC-ROOT RULE):** Modify transform to be smart:
- **Single root already exists**: No synthetic root created, keep existing structure
- **Multiple roots exist**: Create "Hierarchy" synthetic root to unify them
- **Purpose**: This transform explicitly creates single-root trees when needed for renderers that require it

### Medium Risk Components

#### 4. Graph Export Preparation
**Files:**
- `layercake-core/src/export/mod.rs:42-54` - `prepare_graph_data()`
- `layercake-core/src/graph.rs:356-385` - Hierarchy accessors

**Current Dependency:** Methods like `get_hierarchy_nodes()`, `get_hierarchy_edges()`

**Impact:** Assumes partition structure for hierarchy exports

**Solution:** Handle empty hierarchy gracefully, treat as flat graph

#### 5. Graph Validation
**Files:**
- `layercake-core/src/graph.rs:1599-1679` - `validate()`
- `layercake-core/src/pipeline/graph_builder.rs:483-502` - Edge validation

**Current Rules:** Prevents edges connecting to/from partition nodes

**Impact:** Rules are overly restrictive for optional partition model

**Solution:** Relax or remove partition edge restrictions

### Low Risk Components

#### 6. Standard Flow Renderers
**Files:** DOT, GML, JSON, PlantUML, Mermaid (flow diagrams)

**Current Behavior:** Use `get_non_partition_nodes()` and `get_non_partition_edges()`

**Impact:** None - already ignore partition structure

**Solution:** No changes needed

#### 7. Projections
**Files:** `layercake-projections/src/service.rs`

**Current Behavior:** Renders all nodes regardless of partition

**Impact:** None

**Solution:** No changes needed

#### 8. Filtering and Merging
**Files:**
- `layercake-core/src/graphql/types/plan_dag/filter.rs`
- `layercake-core/src/pipeline/merge_builder.rs`

**Current Behavior:** Operate on node/edge properties, preserve partition metadata

**Impact:** None - work with or without partitions

**Solution:** No changes needed

## Design Decisions

### Decision 1: Partition Semantics

**Option A: Partitions as Optional Organizational Metadata**
- `belongs_to` remains on Node but is truly optional
- No validation of `belongs_to` references
- Renderers that need hierarchy skip nodes without `belongs_to`
- **RECOMMENDED**

**Option B: Explicit Containment Edges**
- Remove `belongs_to` field entirely
- Add explicit "contains" edge type for hierarchy
- More explicit, but requires migration
- NOT RECOMMENDED - breaks existing data

**Decision:** Go with Option A for backwards compatibility

### Decision 2: Handling Unattached Nodes

**Option A: Implicit Global Partition**
- Nodes without `belongs_to` implicitly belong to global/root
- No explicit node created
- Tree renderers treat null `belongs_to` as root level
- **RECOMMENDED**

**Option B: Virtual Root Node**
- Create ephemeral root during rendering only
- Never persisted to database
- NOT RECOMMENDED - still violates "no synthetic nodes" rule

**Decision:** Go with Option A - implicit global partition

### Decision 3: Partition Transforms

**Option A: Graceful Degradation**
- PartitionDepthLimit/WidthLimit transforms check if partitions exist
- If no partitions: transform becomes no-op with warning
- **RECOMMENDED**

**Option B: Auto-Group by Layer**
- Use layer as implicit partition when no `belongs_to` exists
- More automatic but potentially surprising
- NOT RECOMMENDED

**Decision:** Go with Option A - explicit opt-in for partition features

### Decision 4: Hierarchy Generation

**Option A: Remove GenerateHierarchy Transform**
- Simply remove the feature
- Breaking change for existing plans using it
- NOT RECOMMENDED - removes useful functionality

**Option B: Redesign as Containment Edge Generator**
- Convert `belongs_to` to explicit edges without synthetic root
- Allow multiple roots
- More complex implementation
- NOT RECOMMENDED - doesn't solve single-root requirement

**Option C: Smart Synthetic Root (Conditional)**
- Check how many roots exist before generating
- Single root: No synthetic root, use existing structure
- Multiple roots: Create "Hierarchy" root to unify them
- Purpose: Generate single-root trees when required by downstream tools
- **RECOMMENDED** - Exception to no-synthetic-root rule, but only when necessary

**Decision:** Go with Option C - conditional synthetic root creation

### Decision 5: Tree Renderers

**Option A: Forest Structure (Multi-Root Support)**
- `build_tree()` returns `Vec<TreeNode>` instead of single TreeNode
- Top level contains: unattached flow nodes + root partition nodes (all nodes without `belongs_to`)
- Each TreeNode recursively contains its children based on `belongs_to` references
- Renderers display all top-level nodes as siblings, with partition hierarchies nested beneath
- Creates a "forest" view: multiple trees side-by-side
- **RECOMMENDED**

**Option B: Skip Nodes Without Hierarchy**
- Only render nodes with `belongs_to` structure
- Simpler but loses data (unattached nodes invisible)
- NOT RECOMMENDED

**Decision:** Go with Option A - forest structure with unattached nodes at top level

## Implementation Plan

### Phase 1: Core Model Changes

**Goal:** Make `belongs_to` truly optional without breaking existing functionality

#### 1.1 Update Validation Rules

**File:** `layercake-core/src/graph.rs`

**Changes:**
```rust
// Remove or relax partition edge restrictions, ADD tree validation
pub fn validate(&self) -> Result<Vec<String>, Vec<String>> {
    let mut errors = Vec::new();

    // REMOVE: Partition/non-partition edge checks
    // These restrictions don't make sense with optional partitions

    // KEEP: Dangling edge detection
    // KEEP: Self-loop detection

    // ADD (REQUIRED): Validate belongs_to references when present
    for node in &self.nodes {
        if let Some(parent_id) = &node.belongs_to {
            if !parent_id.is_empty() {
                // Check parent exists
                if !self.nodes.iter().any(|n| n.id == *parent_id) {
                    errors.push(format!(
                        "Node '{}' has belongs_to='{}' but parent does not exist",
                        node.id, parent_id
                    ));
                }
            }
        }
    }

    // ADD (REQUIRED): Detect cycles in belongs_to chains
    for node in &self.nodes {
        if let Err(cycle) = self.check_belongs_to_cycle(&node.id) {
            errors.push(format!(
                "Cycle detected in belongs_to chain: {}",
                cycle
            ));
        }
    }

    // ...
}

fn check_belongs_to_cycle(&self, start_id: &str) -> Result<(), String> {
    let mut visited = std::collections::HashSet::new();
    let mut current_id = start_id.to_string();

    while let Some(node) = self.nodes.iter().find(|n| n.id == current_id) {
        if let Some(parent_id) = &node.belongs_to {
            if !parent_id.is_empty() {
                if !visited.insert(current_id.clone()) {
                    return Err(format!("cycle involving '{}'", current_id));
                }
                current_id = parent_id.clone();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Ok(())
}
```

**File:** `layercake-core/src/pipeline/graph_builder.rs:483-502`

**Changes:**
```rust
// Remove hard validation that edges can't connect to partition nodes
// Make it a warning instead of error

if source_node.is_partition {
    warn!(
        "Edge {} has source partition node {}. This is allowed but unusual.",
        edge_id, source_id
    );
    // Don't return error
}
```

#### 1.2 Update Documentation

**Files:**
- `docs/GRAPH_MODEL.md` (if exists)
- `README.md`
- Code comments in `graph.rs`

**Changes:**
- Clarify that `belongs_to` is optional
- Explain implicit global partition
- Document when partition structure is useful vs not needed

**Test:**
- Import graph with no `belongs_to` values
- Verify no validation errors
- Verify graph can be saved and loaded

### Phase 2: Hierarchy System Redesign

**Goal:** Support graphs with partial or no partition structure

#### 2.1 Update build_tree() for Forest Structure

**File:** `layercake-core/src/graph.rs:480+`

**Current:**
```rust
pub fn build_tree(&self) -> TreeNode {
    // Returns single TreeNode
    // Assumes all nodes eventually belong to one root
}
```

**New:**
```rust
pub fn build_tree(&self) -> Vec<TreeNode> {
    let mut roots = Vec::new();

    // Find all top-level nodes (no belongs_to = unattached nodes + root partition nodes)
    // These form the "forest" - multiple trees side-by-side at the top level
    let root_nodes = self.nodes.iter()
        .filter(|n| n.belongs_to.is_none() || n.belongs_to.as_ref().map(|s| s.is_empty()).unwrap_or(false))
        .collect::<Vec<_>>();

    // Build tree for each top-level node
    // Each TreeNode recursively contains its children based on belongs_to
    for root in root_nodes {
        let tree = self.build_tree_from_node(&root.id);
        roots.push(tree);
    }

    roots
}

fn build_tree_from_node(&self, node_id: &str) -> TreeNode {
    let node = self.nodes.iter()
        .find(|n| n.id == node_id)
        .expect("Node must exist");

    // Find children (nodes whose belongs_to points to this node)
    let children = self.nodes.iter()
        .filter(|n| n.belongs_to.as_ref() == Some(&node_id.to_string()))
        .map(|child| self.build_tree_from_node(&child.id))
        .collect();

    TreeNode {
        node: node.clone(),
        children,
    }
}
```

**Test:**
- Graph with multiple partition roots (multiple top-level trees)
- Graph with no partitions at all (all nodes at top level)
- Graph with mixed: some unattached nodes + some partition trees
- Validation catches orphaned nodes (invalid belongs_to) before build_tree

#### 2.2 Update Hierarchy Accessors

**File:** `layercake-core/src/graph.rs:356-385`

**Changes:**
```rust
pub fn get_hierarchy_nodes(&self) -> Vec<&Node> {
    // BEFORE: Returns all nodes
    // AFTER: Returns nodes with belongs_to structure OR all if no hierarchy

    if self.has_partition_structure() {
        self.nodes.iter()
            .filter(|n| n.is_partition || n.belongs_to.is_some())
            .collect()
    } else {
        // No partition structure: return all nodes as flat hierarchy
        self.nodes.iter().collect()
    }
}

pub fn get_hierarchy_edges(&self) -> Vec<Edge> {
    // BEFORE: Converts belongs_to to edges
    // AFTER: Only for nodes with belongs_to

    let mut edges = Vec::new();
    for node in &self.nodes {
        if let Some(parent_id) = node.belongs_to.as_ref().filter(|p| !p.is_empty()) {
            // Create edge from parent to node
            edges.push(Edge {
                id: format!("hierarchy_{}_{}", parent_id, node.id),
                source: parent_id.clone(),
                target: node.id.clone(),
                label: "contains".to_string(),
                layer: "hierarchy".to_string(),
                weight: 1,
                comment: None,
                dataset: None,
                attributes: None,
            });
        }
    }
    edges
}

fn has_partition_structure(&self) -> bool {
    self.nodes.iter().any(|n| n.is_partition || n.belongs_to.is_some())
}
```

**Test:**
- Graph with partitions returns filtered nodes
- Graph without partitions returns all nodes
- Hierarchy edges only created for nodes with belongs_to

### Phase 3: Transform Updates

**Goal:** Handle partition transforms gracefully when no partition structure exists

#### 3.1 Remove ensure_partition_hierarchy()

**File:** `layercake-core/src/graph.rs:1319-1377`

**Action:** Delete this function entirely

**Rationale:** Violates new rule against synthetic nodes

#### 3.2 Update generate_hierarchy() for Conditional Root

**File:** `layercake-core/src/graph.rs:1235-1314`

**Action:** Modify to only create synthetic root when multiple roots exist

**Rationale:** Support single-root tree requirement for downstream tools, but avoid unnecessary synthetic nodes

**Changes:**
```rust
pub fn generate_hierarchy(&self) -> Graph {
    // Count how many root nodes exist (nodes without belongs_to)
    let root_count = self.nodes.iter()
        .filter(|n| n.belongs_to.is_none() || n.belongs_to.as_ref().map(|s| s.is_empty()).unwrap_or(false))
        .count();

    // If single root already exists, no need for synthetic root
    if root_count <= 1 {
        return self.clone_with_hierarchy_edges();
    }

    // Multiple roots: create synthetic "Hierarchy" root to unify them
    let mut new_nodes = Vec::new();

    // Add synthetic root node
    new_nodes.push(Node {
        id: "Hierarchy".to_string(),
        label: "Hierarchy".to_string(),
        layer: "hierarchy".to_string(),
        is_partition: true,
        belongs_to: None,
        weight: 1,
        comment: Some("Synthetic root for multi-root tree".to_string()),
        dataset: None,
        attributes: None,
    });

    // Add all existing nodes, updating root nodes to belong to "Hierarchy"
    for node in &self.nodes {
        let mut new_node = node.clone();
        if node.belongs_to.is_none() || node.belongs_to.as_ref().map(|s| s.is_empty()).unwrap_or(false) {
            new_node.belongs_to = Some("Hierarchy".to_string());
        }
        new_nodes.push(new_node);
    }

    Graph {
        name: self.name.clone(),
        nodes: new_nodes,
        edges: self.get_hierarchy_edges(),
        layers: self.layers.clone(),
        annotations: self.annotations.clone(),
    }
}

fn clone_with_hierarchy_edges(&self) -> Graph {
    // Single root: just convert belongs_to to edges, no synthetic root
    Graph {
        name: self.name.clone(),
        nodes: self.nodes.clone(),
        edges: self.get_hierarchy_edges(),
        layers: self.layers.clone(),
        annotations: self.annotations.clone(),
    }
}
```

#### 3.3 Update PartitionDepthLimit Transform

**File:** `layercake-core/src/graph.rs:835-942`

**Changes:**
```rust
pub fn modify_graph_limit_partition_depth(
    &mut self,
    max_depth: usize,
) -> Result<PartitionDepthAggregationResult> {
    // Check if graph has partition structure
    if !self.has_partition_structure() {
        warn!("PartitionDepthLimit transform skipped: no partition structure in graph");
        return Ok(PartitionDepthAggregationResult {
            aggregations: vec![],
            total_nodes_aggregated: 0,
        });
    }

    // Existing logic...
    // Remove call to ensure_partition_hierarchy()
}
```

**File:** `layercake-core/src/graphql/types/plan_dag/transforms.rs:177-241`

**Changes:**
```rust
GraphTransformKind::PartitionDepthLimit => {
    // Add check and user-friendly message
    if !graph.has_partition_structure() {
        return Ok(Some(
            "### Transform: Partition Depth Limit (Skipped)\n\
             No partition structure found in graph. \
             Add `belongs_to` relationships to use this transform."
                .to_string()
        ));
    }
    // Existing logic...
}
```

#### 3.4 Update PartitionWidthLimit Transform

**File:** `layercake-core/src/graph.rs:944-1066`

**Changes:**
```rust
pub fn modify_graph_limit_partition_width(
    &mut self,
    max_width: usize,
) -> Vec<PartitionWidthAggregation> {
    // Check if graph has partition structure
    if !self.has_partition_structure() {
        warn!("PartitionWidthLimit transform skipped: no partition structure in graph");
        return vec![];
    }

    // Existing logic...
    // Remove call to ensure_partition_hierarchy()
}
```

**File:** `layercake-core/src/graphql/types/plan_dag/transforms.rs:243-305`

**Changes:**
```rust
GraphTransformKind::PartitionWidthLimit => {
    if !graph.has_partition_structure() {
        return Ok(Some(
            "### Transform: Partition Width Limit (Skipped)\n\
             No partition structure found in graph."
                .to_string()
        ));
    }
    // Existing logic...
}
```

#### 3.5 Update GenerateHierarchy Transform

**File:** `layercake-core/src/graphql/types/plan_dag/transforms.rs:260-267`

**Action:** Keep the transform but update to use new conditional logic

**Changes:**
```rust
pub enum GraphTransformKind {
    AggregateEdges,
    PartitionDepthLimit,
    PartitionWidthLimit,
    GenerateHierarchy,  // KEEP - but update behavior
    // ... other transforms
}

// Update the apply_to match arm for GenerateHierarchy
GraphTransformKind::GenerateHierarchy => {
    let root_count = graph.nodes.iter()
        .filter(|n| n.belongs_to.is_none() || n.belongs_to.as_ref().map(|s| s.is_empty()).unwrap_or(false))
        .count();

    if root_count <= 1 {
        // Single root or no nodes - just convert to hierarchy edges
        let output = format!(
            "### Transform: Generate Hierarchy\n\
             Graph already has single root - no synthetic root needed.\n\
             Converting belongs_to references to explicit edges."
        );
        return Ok(Some(output));
    }

    // Multiple roots - create synthetic root
    let output = format!(
        "### Transform: Generate Hierarchy\n\
         Graph has {} root nodes - creating synthetic 'Hierarchy' root to unify them.",
        root_count
    );

    *graph = graph.generate_hierarchy();
    Ok(Some(output))
}
```

**Behavior:**
- **Single root**: No synthetic root created, just converts belongs_to to edges
- **Multiple roots**: Creates "Hierarchy" synthetic root to unify them
- **Purpose**: Generate single-root trees for tools that require it

**Test:**
- Partition transforms skip gracefully on flat graphs
- Partition transforms work normally on graphs with partitions
- GenerateHierarchy creates synthetic root only when multiple roots exist
- GenerateHierarchy preserves structure when single root exists

### Phase 4: Renderer Updates

**Goal:** Tree/hierarchy renderers support multiple roots and missing hierarchy

#### 4.1 Update PlantUML Mindmap

**File:** `layercake-core/src/export/to_plantuml_mindmap.rs`

**Current:**
```rust
pub fn to_plantuml_mindmap(graph: &Graph) -> String {
    let tree = graph.build_tree();  // Single root
    // ...
}
```

**New:**
```rust
pub fn to_plantuml_mindmap(graph: &Graph) -> String {
    let trees = graph.build_tree();  // Forest: top-level = unattached nodes + partition roots

    if trees.is_empty() {
        return format!(
            "@startmindmap\n\
             * {}\n\
             ** (No hierarchy structure)\n\
             @endmindmap",
            graph.name
        );
    }

    let mut output = String::from("@startmindmap\n");

    if trees.len() == 1 {
        // Single top-level node - render directly
        output.push_str(&render_mindmap_node(&trees[0], 0));
    } else {
        // Multiple top-level nodes (forest structure):
        // - Unattached flow nodes appear at this level
        // - Root partition nodes appear at this level
        // - Children are recursively nested under partition nodes
        // Create implicit container for PlantUML syntax
        output.push_str(&format!("* {}\n", graph.name));
        for tree in trees {
            output.push_str(&render_mindmap_node(&tree, 1));
        }
    }

    output.push_str("@endmindmap");
    output
}
```

#### 4.2 Update PlantUML WBS

**File:** `layercake-core/src/export/to_plantuml_wbs.rs`

**Changes:** Similar to mindmap - support multiple roots

#### 4.3 Update Mermaid Mindmap

**File:** `layercake-core/src/export/to_mermaid_mindmap.rs`

**Changes:** Similar to PlantUML mindmap

#### 4.4 Update Mermaid Treemap

**File:** `layercake-core/src/export/to_mermaid_treemap.rs`

**Changes:** Similar to other tree renderers

#### 4.5 Update DOT Hierarchy

**File:** `layercake-core/src/export/to_dot_hierarchy.rs`

**Changes:**
```rust
pub fn to_dot_hierarchy(graph: &Graph, _config: &GraphConfig) -> String {
    let trees = graph.build_tree();

    let mut output = String::from("digraph G {\n");
    output.push_str("  rankdir=TB;\n");
    output.push_str("  node [shape=box];\n\n");

    if trees.is_empty() {
        // No hierarchy - render as flat graph
        output.push_str("  \"No Hierarchy\" [shape=plaintext];\n");
    } else {
        // Render each tree
        for (idx, tree) in trees.iter().enumerate() {
            output.push_str(&format!("  // Tree {}\n", idx + 1));
            output.push_str(&render_dot_tree_node(tree, 0));
        }
    }

    output.push_str("}\n");
    output
}
```

**Test:**
- Mindmap/WBS/Treemap renderers handle multiple roots
- Mindmap/WBS/Treemap renderers handle no hierarchy (flat graph)
- DOT hierarchy gracefully handles missing structure

### Phase 5: Migration and Compatibility

**Goal:** Ensure existing data works and provide migration path

#### 5.1 Data Migration

**No database migration needed** - `is_partition` and `belongs_to` remain as-is

**Behavioral changes only:**
- Graphs without partitions now valid
- Partition transforms become optional/no-op

#### 5.2 Update Examples and Documentation

**Files to update:**
- `README.md` - Update graph examples
- `docs/` - Update any architecture docs
- Example datasets in `test-data/` or similar

**Add examples:**
1. Flat graph (no partitions)
2. Partial hierarchy (some nodes partitioned)
3. Full hierarchy (all nodes partitioned)

#### 5.3 Deprecation Warnings

**Generate warnings for:**
- Plans using GenerateHierarchy transform
- Graphs assuming auto-generation of partition structure

**Log warnings:**
```rust
if self.nodes.iter().all(|n| n.belongs_to.is_none()) {
    info!("Graph has no partition structure - hierarchy features unavailable");
}
```

### Phase 6: Testing

**Goal:** Comprehensive testing of new optional partition behavior

#### 6.1 Unit Tests

**File:** `layercake-core/tests/graph_partition_optional.rs` (new)

**Tests:**
```rust
#[test]
fn test_flat_graph_no_partitions() {
    // Graph with no is_partition nodes and no belongs_to
    // Verify validation passes
    // Verify exports work
}

#[test]
fn test_multi_root_partition_hierarchy() {
    // Graph with 2+ partition roots (separate trees in forest)
    // Verify build_tree returns multiple roots
    // Verify renderers show all roots at top level
}

#[test]
fn test_partition_transform_skips_on_flat_graph() {
    // Apply PartitionDepthLimit to flat graph
    // Verify no-op behavior
    // Verify warning logged
}

#[test]
fn test_invalid_belongs_to_rejected() {
    // Node with belongs_to pointing to non-existent parent
    // Verify validation FAILS with error
    // Graph must be rejected
}

#[test]
fn test_belongs_to_cycle_rejected() {
    // Create cycle: A -> B -> C -> A
    // Verify validation FAILS with cycle error
    // Graph must be rejected
}

#[test]
fn test_mixed_hierarchy() {
    // Some nodes partitioned, some not
    // Unattached nodes appear at top level
    // Partition hierarchies nested beneath their roots
}

#[test]
fn test_forest_structure() {
    // Graph with: 2 unattached flow nodes + 2 partition roots with children
    // Verify build_tree returns 4 top-level TreeNodes
    // Verify first 2 have no children, last 2 have children
}
```

#### 6.2 Integration Tests

**File:** `layercake-core/tests/pipeline_partition_optional_e2e.rs` (new)

**Tests:**
```rust
#[tokio::test]
async fn test_import_flat_graph() {
    // Import JSON/CSV with no partition structure
    // Verify loads successfully
    // Run through pipeline
    // Export to various formats
}

#[tokio::test]
async fn test_partition_transform_pipeline() {
    // Graph with partitions -> PartitionDepthLimit -> export
    // Graph without partitions -> PartitionDepthLimit -> export
    // Verify both work
}

#[tokio::test]
async fn test_tree_renderers_without_hierarchy() {
    // Flat graph -> export to mindmap/WBS
    // Verify graceful output
}
```

#### 6.3 Regression Tests

**Verify:**
- Existing graphs with full partition structure still work
- Existing exports produce same output (or documented differences)
- Existing transforms behave correctly

#### 6.4 Frontend Tests

**If UI depends on partition structure:**
- Test graph editor with non-partition graphs
- Test visualizations without hierarchy
- Test filters/searches work regardless of partitions

## Rollout Plan

### Week 1: Phase 1 - Core Model Changes
- Update validation rules (remove/relax partition restrictions)
- Update documentation
- Run initial tests
- **Deliverable:** Flat graphs can be imported and validated

### Week 2: Phase 2 - Hierarchy Redesign
- Update `build_tree()` for multiple roots
- Update hierarchy accessors
- Add `has_partition_structure()` helper
- **Deliverable:** Hierarchy queries handle optional partitions

### Week 3: Phase 3 - Transform Updates
- Remove `ensure_partition_hierarchy()`
- Update `generate_hierarchy()` to conditionally create synthetic root
- Update PartitionDepthLimit and PartitionWidthLimit to skip gracefully
- Update GenerateHierarchy transform behavior (conditional root creation)
- **Deliverable:** Transforms work with or without partitions

### Week 4: Phase 4 - Renderer Updates
- Update all tree/hierarchy renderers
- Support multiple roots
- Handle missing hierarchy
- **Deliverable:** All exports work with optional partitions

### Week 5: Phase 5-6 - Migration, Testing, Polish
- Write comprehensive tests
- Update examples
- Add deprecation warnings
- Documentation review
- **Deliverable:** Production-ready release

## Breaking Changes and Mitigation

### Breaking Changes

1. **`build_tree()` signature change**
   - **Before:** `pub fn build_tree(&self) -> TreeNode`
   - **After:** `pub fn build_tree(&self) -> Vec<TreeNode>`
   - **Impact:** Code calling `build_tree()` will break
   - **Mitigation:** Provide `build_tree_single()` helper that returns first root or error

2. **GenerateHierarchy transform behavior changed**
   - **Before:** Always creates synthetic "Hierarchy" root node
   - **After:** Only creates synthetic root when multiple roots exist; preserves single-root structure
   - **Impact:** Plans using it will produce different output (fewer synthetic nodes)
   - **Mitigation:** This is generally an improvement (less clutter), but document behavior change in release notes

3. **Partition edge validation removed**
   - **Impact:** Graphs with edges to/from partitions now allowed
   - **Mitigation:** This is a relaxation, not a restriction - no breaking change for valid data

### Non-Breaking Changes

1. **Partition transforms become optional**
   - **Impact:** Transforms skip with warning instead of failing
   - **Mitigation:** Clear logging, no data loss

2. **Hierarchy renderers output changes**
   - **Impact:** Multi-root trees render differently
   - **Mitigation:** Documented in release notes

## Success Criteria

- [ ] Flat graphs (no partitions) import and validate successfully
- [ ] Graphs with partial partition structure work correctly
- [ ] Graphs with full partition structure continue to work (regression test)
- [ ] **Validation enforces tree structure when `belongs_to` references exist**
- [ ] **Validation rejects invalid `belongs_to` references (non-existent parent)**
- [ ] **Validation rejects cycles in `belongs_to` chains**
- [ ] Multi-root partition hierarchies render correctly (forest structure)
- [ ] **Tree renderers show unattached nodes at top level alongside partition roots**
- [ ] Partition transforms skip gracefully when no partitions exist
- [ ] Partition transforms work normally when partitions exist
- [ ] All tree/hierarchy renderers handle missing hierarchy
- [ ] **No synthetic root nodes created by system (except GenerateHierarchy when multiple roots)**
- [ ] **GenerateHierarchy creates synthetic root only when multiple roots exist**
- [ ] **GenerateHierarchy preserves structure when single root already exists**
- [ ] Documentation reflects optional partition model
- [ ] All tests pass (unit + integration + regression)
- [ ] Performance unchanged or improved

## Future Enhancements (Out of Scope)

1. **Explicit containment edges** - Replace `belongs_to` with typed edges
2. **Multiple hierarchy types** - Layer-based, property-based grouping
3. **Auto-grouping strategies** - Smart defaults when no partitions
4. **Visual hierarchy editor** - UI for managing partition structure

## Open Questions

1. **Q:** Should we add cycle detection for `belongs_to` chains?
   **A:** YES - REQUIRED. When `belongs_to` references exist, they MUST form a valid tree structure. Add validation in Phase 1.

2. **Q:** What happens to nodes with invalid `belongs_to` (references non-existent node)?
   **A:** Validation ERROR - this is no longer allowed. Phase 1 validation catches this.

3. **Q:** Should we keep support for empty string `belongs_to` meaning root?
   **A:** Yes, treat `""` same as `None` for backwards compatibility

4. **Q:** Should partition transforms error or warn when skipped?
   **A:** Warn only - not an error condition to lack partitions

5. **Q:** How to handle transition period where some graphs have synthetic roots?
   **A:** Leave existing data as-is, but don't create new synthetic roots

## References

### Code Locations

**Partition Core:**
- `layercake-core/src/graph.rs` - Graph model, hierarchy methods
- `layercake-core/src/database/entities/graph_data_nodes.rs` - Node entity

**Validation:**
- `layercake-core/src/graph.rs:1599-1679` - Graph validation
- `layercake-core/src/pipeline/graph_builder.rs:483-502` - Edge validation

**Transforms:**
- `layercake-core/src/graphql/types/plan_dag/transforms.rs` - Transform types
- `layercake-core/src/graph.rs:835-1066` - Partition transform implementations
- `layercake-core/src/graph.rs:1235-1377` - Hierarchy generation (TO BE REMOVED)

**Renderers:**
- `layercake-core/src/export/to_plantuml_mindmap.rs`
- `layercake-core/src/export/to_plantuml_wbs.rs`
- `layercake-core/src/export/to_mermaid_mindmap.rs`
- `layercake-core/src/export/to_mermaid_treemap.rs`
- `layercake-core/src/export/to_dot_hierarchy.rs`

**Import:**
- `layercake-core/src/pipeline/dataset_importer.rs` - CSV/JSON import

### Related Documents

- Deep review exploration report (this analysis)
- Dataset source tracking plan (completed - similar methodology)
