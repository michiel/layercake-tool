# Migration Guide: belongs_to Hierarchy Support

## What Changed

The graph editor now supports hierarchical subflows based on `belongs_to` references and `is_partition` flags. This requires the `graph_nodes` table to have a `belongs_to` column.

## Migration Steps

### 1. Run Database Migrations

The migration `m20251009_000001_add_belongs_to_to_graph_nodes` adds the `belongs_to` column to the `graph_nodes` table.

```bash
# Migrations run automatically on application startup
# Or manually trigger with:
cargo run --bin layercake -- migrate
```

### 2. Rebuild Existing Graphs

**Option A: Delete and Recreate Graphs (Recommended)**

Since backwards compatibility is not required, the simplest approach is:

1. Delete existing graphs through the UI or API
2. The graphs will be automatically rebuilt from datasources with correct `belongs_to` values

**Option B: SQL Update (Advanced)**

If you need to preserve graph IDs, you can manually populate `belongs_to` by re-running the graph build process:

```sql
-- This will trigger a rebuild of all graphs from their datasources
-- (Application-specific SQL - adjust based on your rebuild mechanism)
UPDATE graphs SET execution_state = 'pending';
```

## Verification

After migration, check the browser console when viewing a graph. You should see:

```
Graph nodes: [
  { id: "root", label: "Root", isPartition: true, belongsTo: null },
  { id: "child1", label: "Child 1", isPartition: false, belongsTo: "root" },
  ...
]
```

If `belongsTo` is `null` or `undefined` for child nodes, the migration hasn't completed properly.

## Troubleshooting

**Problem**: Partition nodes render as empty boxes, children are not nested

**Cause**: `belongs_to` values are `null` in the database

**Solution**: Delete and recreate the graph from its datasources

**Debug**: Check browser console logs added in `graphUtils.ts`:
- "Graph nodes:" - shows all node data including `belongsTo` and `isPartition`
- "Root nodes:" - shows which nodes are identified as roots
- "ELK graph before layout:" - shows the hierarchical structure sent to ELK

## CSV Data Format

Ensure your CSV files have the correct format:

```csv
id,label,layer,is_partition,belongs_to,weight,comment
root,Root,layer1,true,,1,
child1,Child 1,layer1,no,root,1,
```

- `is_partition`: "true", "yes", "y", "1" → partition (subflow)
- `is_partition`: "false", "no", "n", "0" → regular node
- `belongs_to`: empty → root node, otherwise → parent node ID
