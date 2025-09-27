# DAG Plan Editor Implementation Plan

## Overview
This plan implements the enhanced DAG Plan Editor functionality as specified in SPECIFICATION.md, focusing on the complete node type system, configuration dialogs, toolbar functionality, and DataSource integration.

## Current State Analysis

### ✅ What's Already Implemented
- **Backend**: Complete GraphQL types and database schema for Plan DAG nodes/edges
- **Frontend**: Basic PlanVisualEditor with ReactFlow integration
- **Node Types**: All 5 node types defined (DataSource, Graph, Transform, Merge, Copy, Output)
- **Database**: Plan DAG nodes/edges tables with proper relationships
- **GraphQL**: Full CRUD operations for Plan DAG management
- **Collaboration**: Real-time subscriptions and user presence
- **Project Management**: Complete project creation and management

### ❌ What Needs Implementation
- **Node Configuration System**: Specific configuration dialogs for each node type
- **Toolbar with Draggable Nodes**: Top toolbar with node type icons
- **DataSource Integration**: Connection between DataSource entities and DataSourceNodes
- **Node Validation**: Visual indicators for unconfigured nodes
- **Transform Engine**: Backend processing for TransformNode operations
- **Output Generation**: Backend rendering system for OutputNodes

## Implementation Phases

## Phase 1: Enhanced Node Configuration System (Priority: Critical)
**Estimated Effort**: 8-10 hours

### Backend Changes

#### 1.1 Enhanced Node Configuration Types
**File**: `layercake-core/src/graphql/types/plan_dag.rs`

Add missing configuration structures:

```rust
// Add to existing file after line 175
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "DataSourceReferenceInput")]
pub struct DataSourceReference {
    pub data_source_id: i32,
    pub project_id: i32,
    pub name: String,
    pub source_type: String,
}

// Enhanced Transform Rules
#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "TransformRuleInput")]
pub struct TransformRule {
    pub rule_type: TransformRuleType,
    pub parameters: TransformRuleParameters,
    pub order: i32,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum TransformRuleType {
    InvertGraph,
    MaxPartitionWidth,
    MaxPartitionDepth,
    NodeLabelMaxLength,
    EdgeLabelMaxLength,
    FilterByLayer,
    FilterByProperty,
}

#[derive(SimpleObject, InputObject, Clone, Debug, Serialize, Deserialize)]
#[graphql(input_name = "TransformRuleParametersInput")]
pub struct TransformRuleParameters {
    pub max_width: Option<i32>,
    pub max_depth: Option<i32>,
    pub max_length: Option<i32>,
    pub layer_filter: Option<String>,
    pub property_filter: Option<String>,
    pub property_value: Option<String>,
}
```

#### 1.2 DataSource Query Integration
**File**: `layercake-core/src/graphql/queries/mod.rs`

Add after line 381:

```rust
/// Get available DataSources for a DataSourceNode configuration
async fn available_data_sources(&self, ctx: &Context<'_>, project_id: i32) -> Result<Vec<DataSourceReference>> {
    let context = ctx.data::<GraphQLContext>()?;
    let data_sources = data_sources::Entity::find()
        .filter(data_sources::Column::ProjectId.eq(project_id))
        .filter(data_sources::Column::Status.eq("active"))
        .all(&context.db)
        .await?;

    Ok(data_sources.into_iter().map(|ds| DataSourceReference {
        data_source_id: ds.id,
        project_id: ds.project_id,
        name: ds.name,
        source_type: ds.source_type.to_string(),
    }).collect())
}
```

### Frontend Changes

#### 1.3 Node Configuration Dialogs
**File**: `frontend/src/components/editors/PlanVisualEditor/dialogs/NodeConfigDialog.tsx`

Create comprehensive configuration system:

```tsx
import React, { useState, useEffect } from 'react'
import { Modal, Stack, TextInput, Select, Button, Group, Text, Accordion } from '@mantine/core'
import { PlanDagNodeType, NodeConfig } from '../../../../types/plan-dag'

interface NodeConfigDialogProps {
  opened: boolean
  onClose: () => void
  nodeType: PlanDagNodeType
  currentConfig?: NodeConfig
  projectId: number
  onSave: (config: NodeConfig) => void
}

export const NodeConfigDialog: React.FC<NodeConfigDialogProps> = ({
  opened,
  onClose,
  nodeType,
  currentConfig,
  projectId,
  onSave,
}) => {
  const [config, setConfig] = useState<NodeConfig | null>(null)

  const renderConfigForm = () => {
    switch (nodeType) {
      case 'DataSourceNode':
        return <DataSourceNodeConfigForm />
      case 'TransformNode':
        return <TransformNodeConfigForm />
      case 'MergeNode':
        return <MergeNodeConfigForm />
      case 'CopyNode':
        return <CopyNodeConfigForm />
      case 'OutputNode':
        return <OutputNodeConfigForm />
      default:
        return <Text>Configuration not available for this node type</Text>
    }
  }

  return (
    <Modal opened={opened} onClose={onClose} title={`Configure ${nodeType}`} size="lg">
      <Stack gap="md">
        {renderConfigForm()}
        <Group justify="flex-end">
          <Button variant="subtle" onClick={onClose}>Cancel</Button>
          <Button onClick={() => config && onSave(config)}>Save Configuration</Button>
        </Group>
      </Stack>
    </Modal>
  )
}
```

#### 1.4 DataSource Configuration Form
**File**: `frontend/src/components/editors/PlanVisualEditor/dialogs/DataSourceNodeConfigForm.tsx`

```tsx
import React, { useState, useEffect } from 'react'
import { useQuery } from '@apollo/client'
import { Stack, Select, Text, Alert } from '@mantine/core'
import { IconAlertCircle } from '@tabler/icons-react'
import { gql } from '@apollo/client'

const GET_AVAILABLE_DATA_SOURCES = gql`
  query GetAvailableDataSources($projectId: Int!) {
    availableDataSources(projectId: $projectId) {
      dataSourceId
      name
      sourceType
    }
  }
`

interface DataSourceNodeConfigFormProps {
  projectId: number
  onChange: (config: DataSourceNodeConfig) => void
  initialConfig?: DataSourceNodeConfig
}

export const DataSourceNodeConfigForm: React.FC<DataSourceNodeConfigFormProps> = ({
  projectId,
  onChange,
  initialConfig,
}) => {
  const [selectedDataSource, setSelectedDataSource] = useState<number | null>(
    initialConfig?.dataSourceId || null
  )

  const { data, loading, error } = useQuery(GET_AVAILABLE_DATA_SOURCES, {
    variables: { projectId },
  })

  const dataSources = data?.availableDataSources || []

  return (
    <Stack gap="md">
      <Text fw={500}>DataSource Configuration</Text>

      {error && (
        <Alert icon={<IconAlertCircle size={16} />} color="red">
          Failed to load available data sources
        </Alert>
      )}

      <Select
        label="Select DataSource"
        placeholder="Choose a data source"
        value={selectedDataSource?.toString()}
        onChange={(value) => {
          const id = value ? parseInt(value) : null
          setSelectedDataSource(id)
          if (id) {
            const ds = dataSources.find(d => d.dataSourceId === id)
            if (ds) {
              onChange({
                dataSourceId: id,
                outputGraphRef: `datasource_${id}_output`,
                sourceType: ds.sourceType,
              })
            }
          }
        }}
        data={dataSources.map(ds => ({
          value: ds.dataSourceId.toString(),
          label: `${ds.name} (${ds.sourceType})`,
        }))}
        loading={loading}
        required
      />

      {selectedDataSource && (
        <Text size="sm" c="dimmed">
          This DataSource will output graph data that can be used by other nodes
        </Text>
      )}
    </Stack>
  )
}
```

## Phase 2: Interactive Toolbar with Draggable Nodes (Priority: High)
**Estimated Effort**: 4-6 hours

#### 2.1 DAG Editor Toolbar
**File**: `frontend/src/components/editors/PlanVisualEditor/PlanEditorToolbar.tsx`

```tsx
import React from 'react'
import { Group, ActionIcon, Tooltip, Paper, Divider, Badge } from '@mantine/core'
import {
  IconDatabase,
  IconShare,
  IconTransform,
  IconCopy,
  IconFileExport,
  IconEye,
  IconPlayerPlay,
  IconSettings,
  IconUsers,
} from '@tabler/icons-react'
import { PlanDagNodeType } from '../../../types/plan-dag'

interface PlanEditorToolbarProps {
  onNodeDrop: (nodeType: PlanDagNodeType, position: { x: number; y: number }) => void
  onPreview: () => void
  onRun: () => void
  onSettings: () => void
  userCount?: number
}

export const PlanEditorToolbar: React.FC<PlanEditorToolbarProps> = ({
  onNodeDrop,
  onPreview,
  onRun,
  onSettings,
  userCount = 0,
}) => {
  const handleDragStart = (event: React.DragEvent, nodeType: PlanDagNodeType) => {
    event.dataTransfer.setData('application/reactflow', nodeType)
    event.dataTransfer.effectAllowed = 'move'
  }

  const nodeTypes = [
    { type: 'DataSourceNode' as PlanDagNodeType, icon: IconDatabase, label: 'Data Source', color: 'blue' },
    { type: 'GraphNode' as PlanDagNodeType, icon: IconShare, label: 'Graph', color: 'green' },
    { type: 'TransformNode' as PlanDagNodeType, icon: IconTransform, label: 'Transform', color: 'orange' },
    { type: 'MergeNode' as PlanDagNodeType, icon: IconCopy, label: 'Merge', color: 'purple' },
    { type: 'CopyNode' as PlanDagNodeType, icon: IconCopy, label: 'Copy', color: 'teal' },
    { type: 'OutputNode' as PlanDagNodeType, icon: IconFileExport, label: 'Output', color: 'red' },
  ]

  return (
    <Paper p="sm" shadow="sm" style={{ borderBottom: '1px solid #e0e0e0' }}>
      <Group justify="space-between">
        <Group gap="xs">
          <Tooltip.Group openDelay={300} closeDelay={100}>
            {nodeTypes.map(({ type, icon: Icon, label, color }) => (
              <Tooltip key={type} label={`Drag to add ${label} node`}>
                <ActionIcon
                  size="lg"
                  variant="light"
                  color={color}
                  style={{ cursor: 'grab' }}
                  draggable
                  onDragStart={(e) => handleDragStart(e, type)}
                >
                  <Icon size={20} />
                </ActionIcon>
              </Tooltip>
            ))}
          </Tooltip.Group>
        </Group>

        <Divider orientation="vertical" />

        <Group gap="xs">
          <Tooltip label="Preview Plan">
            <ActionIcon size="lg" variant="light" onClick={onPreview}>
              <IconEye size={20} />
            </ActionIcon>
          </Tooltip>

          <Tooltip label="Execute Plan">
            <ActionIcon size="lg" variant="light" color="green" onClick={onRun}>
              <IconPlayerPlay size={20} />
            </ActionIcon>
          </Tooltip>

          <Tooltip label="Settings">
            <ActionIcon size="lg" variant="light" onClick={onSettings}>
              <IconSettings size={20} />
            </ActionIcon>
          </Tooltip>

          {userCount > 0 && (
            <Group gap={4}>
              <IconUsers size={16} />
              <Badge size="sm" variant="light">{userCount}</Badge>
            </Group>
          )}
        </Group>
      </Group>
    </Paper>
  )
}
```

#### 2.2 Enhanced PlanVisualEditor with Drag & Drop
**File**: `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`

Add to existing component (around line 100):

```tsx
// Add drag and drop functionality
const onDragOver = useCallback((event: React.DragEvent) => {
  event.preventDefault()
  event.dataTransfer.dropEffect = 'move'
}, [])

const onDrop = useCallback((event: React.DragEvent) => {
  event.preventDefault()

  const reactFlowBounds = reactFlowWrapper.current?.getBoundingClientRect()
  const nodeType = event.dataTransfer.getData('application/reactflow') as PlanDagNodeType

  if (!nodeType || !reactFlowBounds) return

  const position = reactFlowInstance?.project({
    x: event.clientX - reactFlowBounds.left,
    y: event.clientY - reactFlowBounds.top,
  })

  if (position) {
    handleCreateNode(nodeType, position)
  }
}, [reactFlowInstance])

const handleCreateNode = async (nodeType: PlanDagNodeType, position: { x: number, y: number }) => {
  const newNodeId = `${nodeType.toLowerCase()}_${Date.now()}`

  const newNode: PlanDagNodeInput = {
    id: newNodeId,
    nodeType,
    position,
    metadata: {
      label: `New ${nodeType}`,
      description: null,
    },
    config: JSON.stringify({}), // Empty config - will be highlighted as unconfigured
  }

  await addPlanDagNode(newNode)
  setConfigDialogNode({ nodeId: newNodeId, nodeType }) // Auto-open config dialog
}
```

## Phase 3: Node Validation & Visual Indicators (Priority: High)
**Estimated Effort**: 3-4 hours

#### 3.1 Node Status Validation
**File**: `frontend/src/utils/nodeValidation.ts`

```tsx
import { PlanDagNode, PlanDagNodeType } from '../types/plan-dag'

export interface NodeValidationResult {
  isConfigured: boolean
  hasErrors: boolean
  warnings: string[]
  errors: string[]
}

export const validateNode = (node: PlanDagNode): NodeValidationResult => {
  const result: NodeValidationResult = {
    isConfigured: false,
    hasErrors: false,
    warnings: [],
    errors: [],
  }

  try {
    const config = JSON.parse(node.config)

    switch (node.nodeType) {
      case 'DataSourceNode':
        result.isConfigured = !!config.dataSourceId
        if (!config.dataSourceId) {
          result.errors.push('No DataSource selected')
        }
        break

      case 'TransformNode':
        result.isConfigured = !!(config.inputGraphRef && config.outputGraphRef && config.transformRules?.length)
        if (!config.inputGraphRef) result.errors.push('No input graph specified')
        if (!config.outputGraphRef) result.errors.push('No output graph specified')
        if (!config.transformRules?.length) result.errors.push('No transform rules defined')
        break

      case 'OutputNode':
        result.isConfigured = !!(config.sourceGraphRef && config.renderTarget)
        if (!config.sourceGraphRef) result.errors.push('No source graph specified')
        if (!config.renderTarget) result.errors.push('No render target specified')
        break

      default:
        result.isConfigured = Object.keys(config).length > 0
    }

  } catch (e) {
    result.hasErrors = true
    result.errors.push('Invalid configuration JSON')
  }

  result.hasErrors = result.errors.length > 0
  return result
}
```

#### 3.2 Enhanced Node Components with Status Indicators
**File**: `frontend/src/components/editors/PlanVisualEditor/nodes/BaseNode.tsx`

```tsx
import React from 'react'
import { Handle, Position } from 'reactflow'
import { Paper, Group, Text, ActionIcon, Badge, Tooltip } from '@mantine/core'
import { IconSettings, IconAlertTriangle, IconCheck } from '@tabler/icons-react'
import { NodeValidationResult } from '../../../../utils/nodeValidation'

interface BaseNodeProps {
  data: {
    label: string
    description?: string
    validation: NodeValidationResult
    onConfigure: () => void
  }
  color: string
  icon: React.ReactNode
}

export const BaseNode: React.FC<BaseNodeProps> = ({ data, color, icon }) => {
  const { validation } = data

  const getStatusColor = () => {
    if (validation.hasErrors) return 'red'
    if (!validation.isConfigured) return 'orange'
    return 'green'
  }

  return (
    <Paper
      p="sm"
      shadow="md"
      style={{
        border: `2px solid ${validation.isConfigured ? color : '#ff9500'}`,
        minWidth: 180,
        backgroundColor: validation.isConfigured ? 'white' : '#fff8f0'
      }}
    >
      <Handle type="target" position={Position.Top} />

      <Group justify="space-between" mb="xs">
        <Group gap="xs">
          {icon}
          <Badge size="xs" color={getStatusColor()}>
            {validation.hasErrors ? 'Error' : validation.isConfigured ? 'Ready' : 'Configure'}
          </Badge>
        </Group>

        <Tooltip label="Configure node">
          <ActionIcon size="sm" variant="light" onClick={data.onConfigure}>
            <IconSettings size={14} />
          </ActionIcon>
        </Tooltip>
      </Group>

      <Text size="sm" fw={500}>{data.label}</Text>
      {data.description && (
        <Text size="xs" c="dimmed">{data.description}</Text>
      )}

      {validation.errors.length > 0 && (
        <Group gap="xs" mt="xs">
          <IconAlertTriangle size={12} color="red" />
          <Text size="xs" c="red">{validation.errors[0]}</Text>
        </Group>
      )}

      <Handle type="source" position={Position.Bottom} />
    </Paper>
  )
}
```

## Phase 4: Backend Processing Engine (Priority: Medium)
**Estimated Effort**: 10-12 hours

#### 4.1 Transform Service Implementation
**File**: `layercake-core/src/services/transform_service.rs`

```rust
use anyhow::Result;
use crate::graph::Graph;
use crate::graphql::types::plan_dag::{TransformRule, TransformRuleType};

pub struct TransformService;

impl TransformService {
    pub fn new() -> Self {
        Self
    }

    pub async fn apply_transform_rules(
        &self,
        input_graph: &Graph,
        rules: &[TransformRule],
    ) -> Result<Graph> {
        let mut result_graph = input_graph.clone();

        // Apply rules in order
        for rule in rules.iter() {
            result_graph = self.apply_single_rule(result_graph, rule).await?;
        }

        Ok(result_graph)
    }

    async fn apply_single_rule(&self, graph: Graph, rule: &TransformRule) -> Result<Graph> {
        match rule.rule_type {
            TransformRuleType::InvertGraph => {
                self.invert_graph(graph).await
            }
            TransformRuleType::MaxPartitionWidth => {
                if let Some(width) = rule.parameters.max_width {
                    self.limit_partition_width(graph, width).await
                } else {
                    Ok(graph)
                }
            }
            TransformRuleType::MaxPartitionDepth => {
                if let Some(depth) = rule.parameters.max_depth {
                    self.limit_partition_depth(graph, depth).await
                } else {
                    Ok(graph)
                }
            }
            TransformRuleType::NodeLabelMaxLength => {
                if let Some(max_len) = rule.parameters.max_length {
                    self.truncate_node_labels(graph, max_len).await
                } else {
                    Ok(graph)
                }
            }
            // ... implement other transform types
            _ => Ok(graph), // TODO: Implement remaining transforms
        }
    }

    async fn invert_graph(&self, mut graph: Graph) -> Result<Graph> {
        graph.invert_graph()?;
        Ok(graph)
    }

    async fn limit_partition_width(&self, mut graph: Graph, width: i32) -> Result<Graph> {
        graph.modify_graph_limit_partition_width(width as usize)?;
        Ok(graph)
    }

    async fn limit_partition_depth(&self, mut graph: Graph, depth: i32) -> Result<Graph> {
        graph.modify_graph_limit_partition_depth(depth as usize)?;
        Ok(graph)
    }

    async fn truncate_node_labels(&self, mut graph: Graph, max_len: i32) -> Result<Graph> {
        for node in &mut graph.nodes {
            if node.label.len() > max_len as usize {
                node.label.truncate(max_len as usize);
                node.label.push_str("...");
            }
        }
        Ok(graph)
    }
}
```

#### 4.2 Plan Execution Engine
**File**: `layercake-core/src/services/plan_execution_service.rs`

```rust
use anyhow::Result;
use std::collections::HashMap;
use sea_orm::DatabaseConnection;
use crate::graph::Graph;
use crate::graphql::types::plan_dag::{PlanDag, PlanDagNodeType};
use crate::services::{DataSourceService, TransformService};

pub struct PlanExecutionService {
    db: DatabaseConnection,
    transform_service: TransformService,
    data_source_service: DataSourceService,
}

impl PlanExecutionService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            db: db.clone(),
            transform_service: TransformService::new(),
            data_source_service: DataSourceService::new(db),
        }
    }

    pub async fn execute_plan(&self, plan: &PlanDag) -> Result<HashMap<String, Graph>> {
        let mut graph_registry: HashMap<String, Graph> = HashMap::new();
        let execution_order = self.calculate_execution_order(plan)?;

        for node_id in execution_order {
            let node = plan.nodes.iter().find(|n| n.id == node_id)
                .ok_or_else(|| anyhow::anyhow!("Node not found: {}", node_id))?;

            match node.node_type {
                PlanDagNodeType::DataSource => {
                    self.execute_datasource_node(node, &mut graph_registry).await?;
                }
                PlanDagNodeType::Transform => {
                    self.execute_transform_node(node, &mut graph_registry).await?;
                }
                PlanDagNodeType::Merge => {
                    self.execute_merge_node(node, &mut graph_registry).await?;
                }
                PlanDagNodeType::Copy => {
                    self.execute_copy_node(node, &mut graph_registry).await?;
                }
                PlanDagNodeType::Output => {
                    self.execute_output_node(node, &graph_registry).await?;
                }
                _ => {} // GraphNode is passive - just contains graph data
            }
        }

        Ok(graph_registry)
    }

    fn calculate_execution_order(&self, plan: &PlanDag) -> Result<Vec<String>> {
        // Topological sort to determine execution order
        // This ensures dependencies are processed before dependents

        use std::collections::{HashMap, VecDeque, HashSet};

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in &plan.nodes {
            in_degree.insert(node.id.clone(), 0);
            adjacency.insert(node.id.clone(), Vec::new());
        }

        // Build graph
        for edge in &plan.edges {
            adjacency.get_mut(&edge.source).unwrap().push(edge.target.clone());
            *in_degree.get_mut(&edge.target).unwrap() += 1;
        }

        // Topological sort
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        for (node_id, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());

            for neighbor in &adjacency[&node_id] {
                let neighbor_degree = in_degree.get_mut(neighbor).unwrap();
                *neighbor_degree -= 1;
                if *neighbor_degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }

        if result.len() != plan.nodes.len() {
            return Err(anyhow::anyhow!("Cyclic dependency detected in plan"));
        }

        Ok(result)
    }

    async fn execute_datasource_node(
        &self,
        node: &crate::graphql::types::plan_dag::PlanDagNode,
        graph_registry: &mut HashMap<String, Graph>,
    ) -> Result<()> {
        let config: serde_json::Value = serde_json::from_str(&node.config)?;
        let data_source_id = config["dataSourceId"].as_i64()
            .ok_or_else(|| anyhow::anyhow!("Missing dataSourceId in DataSource node config"))?;

        let data_source = self.data_source_service.get_by_id(data_source_id as i32).await?
            .ok_or_else(|| anyhow::anyhow!("DataSource not found: {}", data_source_id))?;

        let graph: Graph = serde_json::from_str(&data_source.graph_json)?;
        let output_ref = config["outputGraphRef"].as_str()
            .unwrap_or(&format!("datasource_{}_output", data_source_id));

        graph_registry.insert(output_ref.to_string(), graph);
        Ok(())
    }

    // ... implement other node execution methods
}
```

## Phase 5: Output Generation System (Priority: Medium)
**Estimated Effort**: 6-8 hours

#### 5.1 Output Service Implementation
**File**: `layercake-core/src/services/output_service.rs`

```rust
use anyhow::Result;
use crate::graph::Graph;
use crate::graphql::types::plan_dag::{RenderTarget, RenderConfig, GraphConfig};
use crate::export::{to_dot, to_gml, to_plantuml, to_mermaid, to_custom};

pub struct OutputService;

impl OutputService {
    pub fn new() -> Self {
        Self
    }

    pub async fn generate_output(
        &self,
        graph: &Graph,
        render_target: RenderTarget,
        render_config: Option<RenderConfig>,
        graph_config: Option<GraphConfig>,
        output_path: &str,
    ) -> Result<String> {
        let processed_graph = self.apply_graph_config(graph, graph_config)?;

        match render_target {
            RenderTarget::Dot => {
                to_dot::export_dot(&processed_graph, output_path, render_config)
            }
            RenderTarget::Gml => {
                to_gml::export_gml(&processed_graph, output_path)
            }
            RenderTarget::PlantUml => {
                to_plantuml::export_plantuml(&processed_graph, output_path, render_config)
            }
            RenderTarget::Mermaid => {
                to_mermaid::export_mermaid(&processed_graph, output_path)
            }
            RenderTarget::Json => {
                let json = serde_json::to_string_pretty(&processed_graph)?;
                std::fs::write(output_path, json)?;
                Ok(format!("JSON exported to {}", output_path))
            }
            RenderTarget::CsvNodes => {
                self.export_csv_nodes(&processed_graph, output_path)
            }
            RenderTarget::CsvEdges => {
                self.export_csv_edges(&processed_graph, output_path)
            }
            _ => Err(anyhow::anyhow!("Unsupported render target: {:?}", render_target))
        }
    }

    fn apply_graph_config(&self, graph: &Graph, config: Option<GraphConfig>) -> Result<Graph> {
        let mut result = graph.clone();

        if let Some(config) = config {
            if let Some(max_node_label_len) = config.node_label_max_length {
                for node in &mut result.nodes {
                    if node.label.len() > max_node_label_len as usize {
                        node.label.truncate(max_node_label_len as usize);
                    }
                }
            }

            if let Some(max_edge_label_len) = config.edge_label_max_length {
                for edge in &mut result.edges {
                    if edge.label.len() > max_edge_label_len as usize {
                        edge.label.truncate(max_edge_label_len as usize);
                    }
                }
            }

            if config.invert_graph == Some(true) {
                result.invert_graph()?;
            }

            if let Some(max_depth) = config.max_partition_depth {
                result.modify_graph_limit_partition_depth(max_depth as usize)?;
            }

            if let Some(max_width) = config.max_partition_width {
                result.modify_graph_limit_partition_width(max_width as usize)?;
            }
        }

        Ok(result)
    }

    fn export_csv_nodes(&self, graph: &Graph, output_path: &str) -> Result<String> {
        use csv::Writer;
        use std::fs::File;

        let file = File::create(output_path)?;
        let mut writer = Writer::from_writer(file);

        writer.write_record(&["id", "label", "layer_id", "properties"])?;

        for node in &graph.nodes {
            let properties = serde_json::to_string(&node.properties)?;
            writer.write_record(&[
                &node.id,
                &node.label,
                &node.layer_id.unwrap_or_default(),
                &properties,
            ])?;
        }

        writer.flush()?;
        Ok(format!("CSV nodes exported to {}", output_path))
    }

    fn export_csv_edges(&self, graph: &Graph, output_path: &str) -> Result<String> {
        use csv::Writer;
        use std::fs::File;

        let file = File::create(output_path)?;
        let mut writer = Writer::from_writer(file);

        writer.write_record(&["id", "source", "target", "label", "properties"])?;

        for edge in &graph.edges {
            let properties = serde_json::to_string(&edge.properties)?;
            writer.write_record(&[
                &edge.id,
                &edge.source,
                &edge.target,
                &edge.label,
                &properties,
            ])?;
        }

        writer.flush()?;
        Ok(format!("CSV edges exported to {}", output_path))
    }
}
```

## Phase 6: Integration and Testing (Priority: High)
**Estimated Effort**: 4-5 hours

#### 6.1 Plan Execution Mutation
**File**: `layercake-core/src/graphql/mutations/mod.rs`

Add after line 1115:

```rust
/// Execute a complete Plan DAG
async fn execute_plan_dag(&self, ctx: &Context<'_>, project_id: i32) -> Result<PlanExecutionResult> {
    let context = ctx.data::<GraphQLContext>()?;

    // Get the Plan DAG
    let plan_dag = self.get_plan_dag(ctx, project_id).await?
        .ok_or_else(|| Error::new("No Plan DAG found for project"))?;

    // Execute the plan
    let execution_service = crate::services::plan_execution_service::PlanExecutionService::new(context.db.clone());

    match execution_service.execute_plan(&plan_dag).await {
        Ok(graph_registry) => {
            Ok(PlanExecutionResult {
                success: true,
                message: format!("Plan executed successfully. Generated {} graphs.", graph_registry.len()),
                output_files: vec![], // TODO: Collect actual output file paths
            })
        }
        Err(e) => {
            Ok(PlanExecutionResult {
                success: false,
                message: format!("Plan execution failed: {}", e),
                output_files: vec![],
            })
        }
    }
}
```

#### 6.2 Frontend Integration
**File**: `frontend/src/components/editors/PlanVisualEditor/PlanVisualEditor.tsx`

Add plan execution functionality:

```tsx
const [executePlanDag] = useMutation(gql`
  mutation ExecutePlanDag($projectId: Int!) {
    executePlanDag(projectId: $projectId) {
      success
      message
      outputFiles
    }
  }
`)

const handleExecutePlan = async () => {
  try {
    const result = await executePlanDag({
      variables: { projectId: parseInt(projectId) }
    })

    if (result.data?.executePlanDag.success) {
      notifications.show({
        title: 'Plan Executed',
        message: result.data.executePlanDag.message,
        color: 'green',
      })
    } else {
      notifications.show({
        title: 'Execution Failed',
        message: result.data?.executePlanDag.message || 'Unknown error',
        color: 'red',
      })
    }
  } catch (error) {
    notifications.show({
      title: 'Execution Error',
      message: 'Failed to execute plan',
      color: 'red',
    })
  }
}
```

## Implementation Order & Timeline

### Week 1: Core Configuration System
1. **Days 1-2**: Backend node configuration types and DataSource integration
2. **Days 3-4**: Frontend configuration dialogs and forms
3. **Day 5**: Testing and integration

### Week 2: Interactive Interface
1. **Days 1-2**: Toolbar with draggable nodes and drop functionality
2. **Days 3-4**: Node validation and visual status indicators
3. **Day 5**: UI polish and testing

### Week 3: Backend Processing
1. **Days 1-3**: Transform service and rule engine
2. **Days 4-5**: Plan execution service and topological sorting

### Week 4: Output System
1. **Days 1-3**: Output service and render targets
2. **Days 4-5**: Integration testing and documentation

## Success Criteria

### ✅ Phase 1 Complete When:
- All node types have functional configuration dialogs
- DataSource nodes can select from available project DataSources
- Transform nodes can define multiple transformation rules
- Configuration state is properly validated and saved

### ✅ Phase 2 Complete When:
- Toolbar displays all 6 node types as draggable icons
- Drag and drop creates new nodes on canvas
- Auto-opening configuration dialog for new nodes
- Visual feedback during drag operations

### ✅ Phase 3 Complete When:
- Unconfigured nodes are visually highlighted (orange border)
- Error states are clearly indicated with red styling
- Node status badges show configuration state
- Validation tooltips provide helpful error messages

### ✅ Phase 4 Complete When:
- Transform service can apply all rule types from specification
- Plan execution service correctly handles dependency order
- Cyclic dependency detection prevents infinite loops
- Graph transformations produce expected results

### ✅ Phase 5 Complete When:
- Output nodes can generate all supported formats
- Render configuration options are respected
- Output files are created at specified paths
- Error handling for file I/O operations

### ✅ Complete Implementation When:
- All node types are fully functional
- Plan DAG can be executed end-to-end
- DataSource → Transform → Output pipeline works
- Real-time collaboration features are maintained
- No regressions in existing functionality

## Notes
- **Backward Compatibility**: All changes maintain existing GraphQL API compatibility
- **Performance**: Plan execution is async and reports progress
- **Error Handling**: Comprehensive error messages for debugging
- **Testing**: Each phase includes unit and integration tests
- **Documentation**: Update API documentation for new features

---
*Plan created: 2025-01-21*
*Implementation ready: All phases defined with specific deliverables*