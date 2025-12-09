# Layer Aliasing System

## Overview

Currently, the project layer system can identify "missing layers" - layers referenced in graph data but not defined in the project palette. This document describes an aliasing system that allows missing layers to be mapped to existing palette layers, eliminating the need to create duplicate layer definitions.

## Motivation

**Problem**: Graph data may reference layer IDs that don't exist in the project palette. Currently, users must add these as new layers, even if semantically they should use the same styling as an existing layer.

**Example Use Case**:
- Palette has layer: `attack` (pink background)
- Graph data references: `attack_node`, `attack_edge`, `offensive`
- Instead of creating 3 new pink layers, alias them all to `attack`

**Benefits**:
- Reduces layer duplication
- Maintains single source of truth for layer styling
- Simplifies layer management across multiple datasets
- Easier to update styling (change one layer, all aliases update)

## User Experience

### Missing Layers Tab

**Before** (Current):
```
Missing Layer: attack_node
[Add to Palette]
```

**After** (With Aliasing):
```
Missing Layer: attack_node
[Add to Palette]  [Alias]
```

### Alias Selection Flow

1. User clicks **[Alias]** button on a missing layer
2. Dialog opens with title: "Alias 'attack_node' to existing layer"
3. Dialog shows list of available palette layers:
   ```
   ┌─────────────────────────────────────────────────────┐
   │  Alias 'attack_node' to existing layer              │
   ├─────────────────────────────────────────────────────┤
   │                                                      │
   │  Select a layer to use for 'attack_node':          │
   │                                                      │
   │  ○  attack - Attack                                 │
   │     ███ #ED96AC  ███ #dddddd  ███ #000000          │
   │                                                      │
   │  ○  base - Base                                     │
   │     ███ #edffac  ███ #dddddd  ███ #000000          │
   │                                                      │
   │  ○  defense - Defense                               │
   │     ███ #ABD2FA  ███ #dddddd  ███ #000000          │
   │                                                      │
   │  ○  scope - Scope                                   │
   │     ███ #ffffff  ███ #cccccc  ███ #000000          │
   │                                                      │
   │                           [Cancel]  [Create Alias]  │
   └─────────────────────────────────────────────────────┘
   ```

4. User selects a layer (radio button)
5. User clicks **[Create Alias]**
6. Missing layer disappears from Missing tab
7. Success notification: "Layer 'attack_node' aliased to 'attack'"

### Viewing Aliases

**Palette Tab Enhancement**:
```
Layer: attack - Attack
Enabled: ☑
Background: #ED96AC  Border: #dddddd  Text: #000000
Source: Layer Dataset #9

Aliases (2):
  • attack_node
  • offensive
[Manage Aliases]
```

### Managing Aliases

Clicking **[Manage Aliases]** opens a dialog:
```
┌─────────────────────────────────────────────────────┐
│  Manage Aliases for 'attack'                        │
├─────────────────────────────────────────────────────┤
│                                                      │
│  These layers are aliased to 'attack':             │
│                                                      │
│  • attack_node                    [Remove Alias]   │
│  • offensive                      [Remove Alias]   │
│                                                      │
│                                          [Close]     │
└─────────────────────────────────────────────────────┘
```

## Data Model Changes

### Database Schema

Add new table `layer_aliases`:

```sql
CREATE TABLE layer_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    alias_layer_id TEXT NOT NULL,           -- The missing/referencing layer ID
    target_layer_id INTEGER NOT NULL,       -- FK to project_layers.id
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (target_layer_id) REFERENCES project_layers(id) ON DELETE CASCADE,

    -- Ensure each alias is unique per project
    UNIQUE(project_id, alias_layer_id)
);

CREATE INDEX idx_layer_aliases_project ON layer_aliases(project_id);
CREATE INDEX idx_layer_aliases_target ON layer_aliases(target_layer_id);
```

**Key Design Decisions**:
- `alias_layer_id` is TEXT (not FK) because it references a layer that doesn't exist in project_layers
- `target_layer_id` is FK to project_layers - the actual layer definition
- CASCADE DELETE ensures aliases are removed when target layer or project is deleted
- UNIQUE constraint prevents multiple aliases with same ID

### SeaORM Entity

Create `layercake-core/src/database/entities/layer_aliases.rs`:

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "layer_aliases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub project_id: i32,
    pub alias_layer_id: String,
    pub target_layer_id: i32,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::projects::Entity",
        from = "Column::ProjectId",
        to = "super::projects::Column::Id",
        on_delete = "Cascade"
    )]
    Project,

    #[sea_orm(
        belongs_to = "super::project_layers::Entity",
        from = "Column::TargetLayerId",
        to = "super::project_layers::Column::Id",
        on_delete = "Cascade"
    )]
    TargetLayer,
}

impl ActiveModelBehavior for ActiveModel {}
```

## API Changes

### GraphQL Schema Extensions

Add to `layercake-core/src/graphql/schema.rs`:

```graphql
# New type for layer alias
type LayerAlias {
  id: Int!
  projectId: Int!
  aliasLayerId: String!
  targetLayerId: Int!
  targetLayer: ProjectLayer!
  createdAt: String!
}

# Extend ProjectLayer to include aliases
type ProjectLayer {
  # ... existing fields ...
  aliases: [LayerAlias!]!
}

# Queries
extend type Query {
  # Get all aliases for a project
  listLayerAliases(projectId: Int!): [LayerAlias!]!

  # Get aliases for a specific target layer
  getLayerAliases(projectId: Int!, targetLayerId: Int!): [LayerAlias!]!
}

# Mutations
extend type Mutation {
  # Create an alias from a missing layer to an existing layer
  createLayerAlias(
    projectId: Int!
    aliasLayerId: String!
    targetLayerId: Int!
  ): LayerAlias!

  # Remove an alias
  removeLayerAlias(
    projectId: Int!
    aliasLayerId: String!
  ): Boolean!

  # Bulk remove all aliases for a target layer
  removeLayerAliases(
    projectId: Int!
    targetLayerId: Int!
  ): Int!  # Returns count of removed aliases
}
```

### GraphQL Resolvers

Add to `layercake-core/src/graphql/resolvers/layers.rs`:

```rust
// Query: List all aliases for a project
pub async fn list_layer_aliases(
    ctx: &Context<'_>,
    project_id: i32,
) -> GraphResult<Vec<LayerAlias>> {
    let db = ctx.data::<DatabaseConnection>()?;

    let aliases = layer_aliases::Entity::find()
        .filter(layer_aliases::Column::ProjectId.eq(project_id))
        .find_also_related(project_layers::Entity)
        .all(db)
        .await?;

    // Map to GraphQL type with target layer populated
    // ...
}

// Mutation: Create alias
pub async fn create_layer_alias(
    ctx: &Context<'_>,
    project_id: i32,
    alias_layer_id: String,
    target_layer_id: i32,
) -> GraphResult<LayerAlias> {
    let db = ctx.data::<DatabaseConnection>()?;

    // Validate target layer exists and belongs to project
    let target_layer = project_layers::Entity::find_by_id(target_layer_id)
        .filter(project_layers::Column::ProjectId.eq(project_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Target layer not found"))?;

    // Create alias
    let alias = layer_aliases::ActiveModel {
        id: NotSet,
        project_id: Set(project_id),
        alias_layer_id: Set(alias_layer_id.clone()),
        target_layer_id: Set(target_layer_id),
        created_at: Set(Utc::now().naive_utc()),
    };

    let result = alias.insert(db).await?;

    // Return with target layer populated
    // ...
}

// Mutation: Remove alias
pub async fn remove_layer_alias(
    ctx: &Context<'_>,
    project_id: i32,
    alias_layer_id: String,
) -> GraphResult<bool> {
    let db = ctx.data::<DatabaseConnection>()?;

    let result = layer_aliases::Entity::delete_many()
        .filter(layer_aliases::Column::ProjectId.eq(project_id))
        .filter(layer_aliases::Column::AliasLayerId.eq(alias_layer_id))
        .exec(db)
        .await?;

    Ok(result.rows_affected > 0)
}
```

### Layer Resolution Service

Add to `layercake-core/src/services/graph_service.rs`:

```rust
/// Resolve layer information, including aliases
pub async fn resolve_layer(
    &self,
    project_id: i32,
    layer_id: &str,
) -> GraphResult<Option<Layer>> {
    // 1. Try to find direct match in project_layers
    if let Some(layer) = self.get_project_layer_by_id(project_id, layer_id).await? {
        return Ok(Some(layer));
    }

    // 2. Check if this layer_id is aliased
    let alias = layer_aliases::Entity::find()
        .filter(layer_aliases::Column::ProjectId.eq(project_id))
        .filter(layer_aliases::Column::AliasLayerId.eq(layer_id))
        .find_also_related(project_layers::Entity)
        .one(&self.db)
        .await?;

    if let Some((alias_record, Some(target_layer))) = alias {
        // Return the target layer's styling but with the alias's ID
        return Ok(Some(Layer {
            id: layer_id.to_string(),  // Use alias ID
            label: target_layer.name,
            background_color: target_layer.background_color,
            text_color: target_layer.text_color,
            border_color: target_layer.border_color,
            dataset: target_layer.source_dataset_id,
        }));
    }

    // 3. Layer not found and not aliased
    Ok(None)
}

/// Get all layers for a project, including aliases
pub async fn get_all_resolved_layers(
    &self,
    project_id: i32,
) -> GraphResult<Vec<Layer>> {
    let mut layers = Vec::new();

    // Get all direct layers
    let direct_layers = self.get_project_layers_palette(project_id).await?;
    layers.extend(direct_layers);

    // Get all aliases and resolve them
    let aliases = layer_aliases::Entity::find()
        .filter(layer_aliases::Column::ProjectId.eq(project_id))
        .find_also_related(project_layers::Entity)
        .all(&self.db)
        .await?;

    for (alias_record, target_layer_opt) in aliases {
        if let Some(target_layer) = target_layer_opt {
            // Only include enabled target layers
            if target_layer.enabled {
                layers.push(Layer {
                    id: alias_record.alias_layer_id,
                    label: target_layer.name.clone(),
                    background_color: target_layer.background_color.clone(),
                    text_color: target_layer.text_color.clone(),
                    border_color: target_layer.border_color.clone(),
                    dataset: target_layer.source_dataset_id,
                });
            }
        }
    }

    Ok(layers)
}
```

## Pipeline Integration

### Export Pipeline Changes

Update `layercake-core/src/pipeline/graph_builder.rs`:

```rust
// In graph_to_data_set() function

// OLD: Only load direct project layers
let db_layers = project_layers::Entity::find()
    .filter(project_layers::Column::ProjectId.eq(graph.project_id))
    .filter(project_layers::Column::Enabled.eq(true))
    .all(&self.db)
    .await?;

// NEW: Load both direct layers AND aliased layers
let graph_service = GraphService::new(self.db.clone());
let all_layers = graph_service
    .get_all_resolved_layers(graph.project_id)
    .await?;

// Convert to JSON format for templates
let layers_json: Vec<_> = all_layers.iter().map(|l| {
    serde_json::json!({
        "id": l.id.clone(),
        "label": l.label.clone(),
        "background_color": l.background_color.trim_start_matches('#'),
        "text_color": l.text_color.trim_start_matches('#'),
        "border_color": l.border_color.trim_start_matches('#'),
    })
}).collect();
```

### DAG Executor Changes

Update `layercake-core/src/pipeline/dag_executor.rs`:

```rust
// When loading cached graph data, ensure aliases are resolved
// This might not need changes if we use get_all_resolved_layers()
// during graph building, as the JSON will already contain resolved layers
```

## Frontend Changes

### TypeScript Types

Add to `frontend/src/types/layers.ts`:

```typescript
export interface LayerAlias {
  id: number
  projectId: number
  aliasLayerId: string
  targetLayerId: number
  targetLayer: ProjectLayer
  createdAt: string
}

export interface ProjectLayer {
  // ... existing fields ...
  aliases?: LayerAlias[]
}
```

### GraphQL Queries/Mutations

Add to `frontend/src/graphql/layers.ts`:

```typescript
export const LIST_LAYER_ALIASES = gql`
  query ListLayerAliases($projectId: Int!) {
    listLayerAliases(projectId: $projectId) {
      id
      projectId
      aliasLayerId
      targetLayerId
      targetLayer {
        id
        layerId
        name
        backgroundColor
        textColor
        borderColor
      }
      createdAt
    }
  }
`

export const CREATE_LAYER_ALIAS = gql`
  mutation CreateLayerAlias(
    $projectId: Int!
    $aliasLayerId: String!
    $targetLayerId: Int!
  ) {
    createLayerAlias(
      projectId: $projectId
      aliasLayerId: $aliasLayerId
      targetLayerId: $targetLayerId
    ) {
      id
      aliasLayerId
      targetLayerId
      targetLayer {
        id
        name
        backgroundColor
        textColor
        borderColor
      }
    }
  }
`

export const REMOVE_LAYER_ALIAS = gql`
  mutation RemoveLayerAlias($projectId: Int!, $aliasLayerId: String!) {
    removeLayerAlias(projectId: $projectId, aliasLayerId: $aliasLayerId)
  }
`
```

### UI Components

#### 1. Missing Tab Enhancement

Update `frontend/src/pages/ProjectLayersPage.tsx`:

```typescript
// In the Missing tab section, add Alias button

const MissingLayerRow = ({ layer, onAddToPalette, onAlias }) => {
  return (
    <div className="flex items-center justify-between p-2 border-b">
      <div>
        <span className="font-mono text-sm">{layer.id}</span>
        <span className="text-xs text-muted-foreground ml-2">
          Used by {layer.nodeCount} nodes, {layer.edgeCount} edges
        </span>
      </div>
      <div className="flex gap-2">
        <Button size="sm" variant="outline" onClick={() => onAlias(layer)}>
          Alias
        </Button>
        <Button size="sm" onClick={() => onAddToPalette(layer)}>
          Add to Palette
        </Button>
      </div>
    </div>
  )
}
```

#### 2. Alias Selection Dialog

Create `frontend/src/components/layers/AliasLayerDialog.tsx`:

```typescript
import { useState } from 'react'
import { useMutation, useQuery } from '@apollo/client'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import { Label } from '@/components/ui/label'
import { CREATE_LAYER_ALIAS } from '@/graphql/layers'
import { GET_PROJECT_LAYERS } from '@/graphql/layers'

interface AliasLayerDialogProps {
  open: boolean
  onClose: () => void
  projectId: number
  missingLayerId: string
  onSuccess: () => void
}

export const AliasLayerDialog = ({
  open,
  onClose,
  projectId,
  missingLayerId,
  onSuccess,
}: AliasLayerDialogProps) => {
  const [selectedLayerId, setSelectedLayerId] = useState<number | null>(null)

  const { data: layersData, loading } = useQuery(GET_PROJECT_LAYERS, {
    variables: { projectId },
    skip: !open,
  })

  const [createAlias, { loading: creating }] = useMutation(CREATE_LAYER_ALIAS, {
    onCompleted: () => {
      onSuccess()
      onClose()
    },
  })

  const handleCreate = () => {
    if (!selectedLayerId) return

    createAlias({
      variables: {
        projectId,
        aliasLayerId: missingLayerId,
        targetLayerId: selectedLayerId,
      },
    })
  }

  const paletteLayer = layersData?.listProjectLayers?.filter(
    (l) => l.enabled
  ) || []

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            Alias '{missingLayerId}' to existing layer
          </DialogTitle>
        </DialogHeader>

        <div className="py-4">
          <p className="text-sm text-muted-foreground mb-4">
            Select a layer to use for '{missingLayerId}':
          </p>

          {loading ? (
            <p className="text-sm text-muted-foreground">Loading layers...</p>
          ) : (
            <RadioGroup
              value={selectedLayerId?.toString()}
              onValueChange={(value) => setSelectedLayerId(parseInt(value))}
            >
              <div className="space-y-3">
                {paletteLayer.map((layer) => (
                  <div
                    key={layer.id}
                    className="flex items-center space-x-3 p-3 border rounded hover:bg-muted/50"
                  >
                    <RadioGroupItem value={layer.id.toString()} id={`layer-${layer.id}`} />
                    <Label
                      htmlFor={`layer-${layer.id}`}
                      className="flex-1 cursor-pointer"
                    >
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-sm">{layer.layerId}</span>
                        <span className="text-sm">-</span>
                        <span className="text-sm">{layer.name}</span>
                      </div>
                      <div className="flex gap-2 mt-2">
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.backgroundColor }}
                            title="Background"
                          />
                          <span className="text-xs text-muted-foreground">BG</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.borderColor }}
                            title="Border"
                          />
                          <span className="text-xs text-muted-foreground">Border</span>
                        </div>
                        <div className="flex items-center gap-1">
                          <div
                            className="w-6 h-6 border rounded"
                            style={{ backgroundColor: layer.textColor }}
                            title="Text"
                          />
                          <span className="text-xs text-muted-foreground">Text</span>
                        </div>
                      </div>
                    </Label>
                  </div>
                ))}
              </div>
            </RadioGroup>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={handleCreate}
            disabled={!selectedLayerId || creating}
          >
            {creating ? 'Creating...' : 'Create Alias'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
```

#### 3. Alias Management Dialog

Create `frontend/src/components/layers/ManageAliasesDialog.tsx`:

```typescript
import { useMutation, useQuery } from '@apollo/client'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { REMOVE_LAYER_ALIAS, GET_LAYER_ALIASES } from '@/graphql/layers'
import { IconX } from '@tabler/icons-react'

interface ManageAliasesDialogProps {
  open: boolean
  onClose: () => void
  projectId: number
  targetLayerId: number
  layerName: string
}

export const ManageAliasesDialog = ({
  open,
  onClose,
  projectId,
  targetLayerId,
  layerName,
}: ManageAliasesDialogProps) => {
  const { data, refetch } = useQuery(GET_LAYER_ALIASES, {
    variables: { projectId, targetLayerId },
    skip: !open,
  })

  const [removeAlias] = useMutation(REMOVE_LAYER_ALIAS, {
    onCompleted: () => {
      refetch()
    },
  })

  const aliases = data?.getLayerAliases || []

  const handleRemove = (aliasLayerId: string) => {
    if (confirm(`Remove alias '${aliasLayerId}'?`)) {
      removeAlias({
        variables: { projectId, aliasLayerId },
      })
    }
  }

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Manage Aliases for '{layerName}'</DialogTitle>
        </DialogHeader>

        <div className="py-4">
          {aliases.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              No aliases for this layer.
            </p>
          ) : (
            <div className="space-y-2">
              <p className="text-sm text-muted-foreground mb-3">
                These layers are aliased to '{layerName}':
              </p>
              {aliases.map((alias) => (
                <div
                  key={alias.id}
                  className="flex items-center justify-between p-2 border rounded"
                >
                  <span className="font-mono text-sm">{alias.aliasLayerId}</span>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => handleRemove(alias.aliasLayerId)}
                  >
                    <IconX className="h-4 w-4" />
                    Remove Alias
                  </Button>
                </div>
              ))}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
```

#### 4. Update Palette Tab

Enhance the palette layer display to show aliases:

```typescript
const PaletteLayerCard = ({ layer, projectId }) => {
  const [showAliases, setShowAliases] = useState(false)

  const { data } = useQuery(GET_LAYER_ALIASES, {
    variables: { projectId, targetLayerId: layer.id },
  })

  const aliasCount = data?.getLayerAliases?.length || 0

  return (
    <div className="border rounded p-4">
      {/* Existing layer info */}

      {aliasCount > 0 && (
        <div className="mt-3 pt-3 border-t">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">
              Aliases ({aliasCount}):
            </span>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => setShowAliases(true)}
            >
              Manage Aliases
            </Button>
          </div>
          <div className="flex flex-wrap gap-1 mt-2">
            {data?.getLayerAliases?.map((alias) => (
              <Badge key={alias.id} variant="secondary">
                {alias.aliasLayerId}
              </Badge>
            ))}
          </div>
        </div>
      )}

      <ManageAliasesDialog
        open={showAliases}
        onClose={() => setShowAliases(false)}
        projectId={projectId}
        targetLayerId={layer.id}
        layerName={layer.name}
      />
    </div>
  )
}
```

## Implementation Plan

### Phase 1: Database & Backend Core (Week 1)

1. **Database Migration**
   - Create migration file for `layer_aliases` table
   - Add SeaORM entity
   - Update entities mod.rs exports

2. **GraphQL API**
   - Add LayerAlias type to schema
   - Implement query resolvers (listLayerAliases, getLayerAliases)
   - Implement mutation resolvers (createLayerAlias, removeLayerAlias)

3. **Layer Resolution Service**
   - Add `resolve_layer()` function
   - Add `get_all_resolved_layers()` function
   - Add unit tests for layer resolution

### Phase 2: Pipeline Integration (Week 1-2)

1. **Export Pipeline**
   - Update graph_builder.rs to use resolved layers
   - Test with aliased layers

2. **Validation**
   - Ensure aliases work in all export formats (DOT, Mermaid, PlantUML)
   - Verify layer colors are applied correctly

### Phase 3: Frontend UI (Week 2)

1. **GraphQL Client**
   - Add TypeScript types
   - Add queries/mutations
   - Update Apollo cache handling

2. **Alias Selection Dialog**
   - Implement AliasLayerDialog component
   - Add color preview blocks
   - Wire up to mutations

3. **Missing Tab Updates**
   - Add Alias button to missing layers
   - Integrate AliasLayerDialog
   - Update missing layer detection to exclude aliases

4. **Palette Tab Updates**
   - Show alias count and list
   - Add ManageAliasesDialog component
   - Add remove alias functionality

### Phase 4: Testing & Documentation (Week 2-3)

1. **Backend Tests**
   - Test alias creation/removal
   - Test layer resolution with various scenarios
   - Test CASCADE delete behavior

2. **Frontend Tests**
   - Test UI interactions
   - Test error handling
   - Test refetch behavior

3. **Integration Tests**
   - End-to-end alias workflow
   - Export with aliased layers
   - Multiple aliases to same layer

4. **Documentation**
   - Update user guide
   - Add API documentation
   - Create migration guide

## Testing Plan

### Unit Tests

**Backend** (`layercake-core/src/services/graph_service.rs`):
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_resolve_direct_layer() {
        // Test that direct layers are found
    }

    #[tokio::test]
    async fn test_resolve_aliased_layer() {
        // Test that aliased layers resolve to target
    }

    #[tokio::test]
    async fn test_resolve_nonexistent_layer() {
        // Test that missing layers return None
    }

    #[tokio::test]
    async fn test_get_all_resolved_layers_includes_aliases() {
        // Test that both direct and aliased layers are returned
    }

    #[tokio::test]
    async fn test_alias_disabled_layer_not_included() {
        // Test that aliases to disabled layers are excluded
    }
}
```

### Integration Tests

1. **Alias Creation Flow**
   - Create project
   - Add layer to palette
   - Create alias
   - Verify alias appears in list
   - Verify missing layer disappears

2. **Export with Aliases**
   - Create graph with aliased layer references
   - Export to DOT
   - Verify correct colors applied
   - Verify alias ID used in output

3. **Alias Removal**
   - Create alias
   - Remove alias
   - Verify layer appears in missing again
   - Verify export handles missing layer

4. **Target Layer Deletion**
   - Create alias
   - Delete target layer
   - Verify alias is CASCADE deleted
   - Verify database integrity

### Manual Testing Scenarios

1. **Basic Alias Flow**
   - [ ] Missing tab shows layers not in palette
   - [ ] Click Alias button opens dialog
   - [ ] Dialog shows all palette layers with colors
   - [ ] Create alias removes from missing tab
   - [ ] Export uses aliased layer colors

2. **Alias Management**
   - [ ] Palette shows alias count
   - [ ] Manage Aliases shows all aliases
   - [ ] Remove alias returns to missing tab
   - [ ] Multiple aliases to same layer work

3. **Edge Cases**
   - [ ] Alias to disabled layer (should not appear in exports)
   - [ ] Enable/disable target layer (aliases should follow)
   - [ ] Delete target layer (aliases CASCADE deleted)
   - [ ] Duplicate alias attempt (should error gracefully)

## Migration Strategy

### For Existing Projects

1. **Automatic Alias Suggestion**
   - When missing layers are detected, suggest potential aliases based on:
     - Similar layer ID (e.g., "attack" → "attack_node")
     - Prefix matching (e.g., "attack" → "attack_*")

2. **Bulk Alias Creation**
   - Add option to create multiple aliases at once
   - Useful for projects with many similar layers

### Database Migration

Migration file: `layercake-core/migrations/XXX_add_layer_aliases.sql`

```sql
-- Up Migration
CREATE TABLE IF NOT EXISTS layer_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    alias_layer_id TEXT NOT NULL,
    target_layer_id INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (target_layer_id) REFERENCES project_layers(id) ON DELETE CASCADE,

    UNIQUE(project_id, alias_layer_id)
);

CREATE INDEX idx_layer_aliases_project ON layer_aliases(project_id);
CREATE INDEX idx_layer_aliases_target ON layer_aliases(target_layer_id);

-- Down Migration (for rollback)
DROP INDEX IF EXISTS idx_layer_aliases_target;
DROP INDEX IF EXISTS idx_layer_aliases_project;
DROP TABLE IF EXISTS layer_aliases;
```

## Future Enhancements

### Phase 2 Features (Future Work)

1. **Alias Groups**
   - Group multiple aliases together
   - Bulk operations on alias groups

2. **Smart Alias Suggestions**
   - ML-based layer similarity detection
   - Automatic alias recommendations

3. **Alias Templates**
   - Save common alias patterns
   - Apply alias templates across projects

4. **Alias Import/Export**
   - Export alias mappings
   - Import alias mappings from other projects

5. **Layer Inheritance**
   - Allow aliases to override specific properties
   - E.g., use target layer colors but different name

## Security Considerations

1. **Validation**
   - Ensure project_id authorization for all operations
   - Validate target_layer_id belongs to project
   - Prevent circular alias references (if we add alias-to-alias in future)

2. **SQL Injection**
   - Use SeaORM parameterized queries (already handled)
   - Validate input strings

3. **CASCADE DELETE**
   - Properly configured to maintain referential integrity
   - Test thoroughly to ensure no orphaned records

## Performance Considerations

1. **Layer Resolution Caching**
   - Cache resolved layers during export
   - Avoid repeated database lookups

2. **Batch Queries**
   - Load all aliases for project in single query
   - Use DataLoader pattern in GraphQL if needed

3. **Indexes**
   - Indexed on project_id for fast project queries
   - Indexed on target_layer_id for cascade operations

## Summary

This layer aliasing system provides a clean solution for mapping missing layer references to existing palette layers. It:

- ✅ Reduces layer duplication
- ✅ Maintains single source of truth for styling
- ✅ Integrates seamlessly with existing layer system
- ✅ Provides intuitive UI for managing aliases
- ✅ Works with all export formats
- ✅ Scales to multiple aliases per layer
- ✅ Handles CASCADE deletes properly

The implementation is straightforward and builds on the existing project layer infrastructure without breaking changes to the current system.
