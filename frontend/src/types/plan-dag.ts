/**
 * Plan DAG TypeScript Interfaces
 *
 * Based on the Plan DAG JSON schema design, these interfaces represent
 * the visual editing data structures for Plan DAGs in ReactFlow.
 */

import type { RuleGroupType } from 'react-querybuilder';

// Position interface for ReactFlow nodes
export interface Position {
  x: number;
  y: number;
}

// Base metadata for all nodes and edges
export interface NodeMetadata {
  label: string;
  description?: string;
}

export interface EdgeMetadata {
  label?: string;
  dataType: 'GRAPH_DATA' | 'GRAPH_REFERENCE' | 'SEQUENCE_DATA';
}

// Plan DAG Node Types
export enum PlanDagNodeType {
  DATA_SOURCE = 'DataSetNode',
  GRAPH = 'GraphNode',
  TRANSFORM = 'TransformNode',
  FILTER = 'FilterNode',
  MERGE = 'MergeNode',
  GRAPH_ARTEFACT = 'GraphArtefactNode',
  TREE_ARTEFACT = 'TreeArtefactNode',
  PROJECTION = 'ProjectionNode',
  STORY = 'StoryNode',
  SEQUENCE_ARTEFACT = 'SequenceArtefactNode',
}

// Data Source Node Configuration
export interface DataSetNodeConfig {
  dataSetId?: number; // Reference to DataSet entity (new)
  displayMode?: 'summary' | 'detailed' | 'preview'; // Optional for backward compatibility
  // Removed: outputGraphRef - output connections handled by visual edges in DAG

  // Legacy support (to be deprecated after migration)
  inputType?: 'CSVNodesFromFile' | 'CSVEdgesFromFile' | 'CSVLayersFromFile';
  source?: string;
  dataType?: 'Nodes' | 'Edges' | 'Layers';
}

// Graph Node Configuration
export interface GraphNodeConfig {
  // Removed: graphId - graph connections handled by visual edges in DAG
  metadata: {
    nodeCount?: number;
    edgeCount?: number;
    lastModified?: string;
  };
}

// Transform Node Configuration
export type GraphTransformKind =
  | 'PartitionDepthLimit'
  | 'PartitionWidthLimit'
  | 'DropUnconnectedNodes'
  | 'NodeLabelMaxLength'
  | 'NodeLabelInsertNewlines'
  | 'EdgeLabelMaxLength'
  | 'EdgeLabelInsertNewlines'
  | 'InvertGraph'
  | 'GenerateHierarchy'
  | 'AggregateLayerNodes'
  | 'AggregateEdges';

export interface GraphTransformParams {
  maxPartitionDepth?: number;
  maxPartitionWidth?: number;
  nodeLabelMaxLength?: number;
  nodeLabelInsertNewlinesAt?: number;
  edgeLabelMaxLength?: number;
  edgeLabelInsertNewlinesAt?: number;
  enabled?: boolean;
  layerConnectionsThreshold?: number;
  excludePartitionNodes?: boolean;
}

export interface GraphTransform {
  kind: GraphTransformKind;
  params: GraphTransformParams;
}

export interface TransformNodeConfig {
  transforms: GraphTransform[];
}

// Filter Node Configuration
export type QueryFilterTarget = 'nodes' | 'edges' | 'layers';
export type QueryLinkPruningMode = 'autoDropDanglingEdges' | 'retainEdges' | 'dropOrphanNodes';

export interface QueryFilterConfig {
  targets: QueryFilterTarget[];
  mode: 'include' | 'exclude';
  linkPruningMode: QueryLinkPruningMode;
  ruleGroup: RuleGroupType;
  fieldMetadataVersion: string;
  notes?: string;
}

export interface FilterNodeConfig {
  query: QueryFilterConfig;
}

// Merge Node Configuration
export interface MergeNodeConfig {
  // Removed: inputRefs - inputs come from incoming edges
  // Removed: outputGraphRef - output goes to outgoing edge
  mergeStrategy: 'Union' | 'Intersection' | 'Difference';
  conflictResolution: 'PreferFirst' | 'PreferLast' | 'Manual';
}

// Graph Artefact Node Configuration
export type GraphArtefactRenderTarget =
  | 'DOT'
  | 'GML'
  | 'JSON'
  | 'PlantUML'
  | 'CSVNodes'
  | 'CSVEdges'
  | 'Mermaid'
  | 'Custom';

export interface GraphArtefactNodeConfig {
  renderTarget: GraphArtefactRenderTarget;
  outputPath: string;
  renderConfig?: {
    containNodes?: boolean;
    orientation?: 'LR' | 'TB';
    applyLayers?: boolean;
    useNodeWeight?: boolean;
    useEdgeWeight?: boolean;
    builtInStyles?: 'none' | 'light' | 'dark';
    targetOptions?: RenderTargetOptions;
    addNodeCommentsAsNotes?: boolean;
    notePosition?: 'left' | 'right' | 'top' | 'bottom';
    layerSourceStyles?: LayerSourceStyleOverride[];
  };
    graphConfig?: {
    generateHierarchy?: boolean;
    maxPartitionDepth?: number | null;
    maxPartitionWidth?: number | null;
    invertGraph?: boolean;
    nodeLabelMaxLength?: number;
    nodeLabelInsertNewlinesAt?: number;
    edgeLabelMaxLength?: number;
    edgeLabelInsertNewlinesAt?: number;
  };
}

// Tree Artefact Node Configuration
export type TreeArtefactRenderTarget = 'PlantUmlMindmap' | 'PlantUmlWbs' | 'MermaidMindmap' | 'MermaidTreemap';

export interface TreeArtefactNodeConfig {
  renderTarget: TreeArtefactRenderTarget;
  outputPath: string;
  renderConfig?: {
    containNodes?: boolean;
    orientation?: 'LR' | 'TB';
    applyLayers?: boolean;
    useNodeWeight?: boolean;
    useEdgeWeight?: boolean;
    builtInStyles?: 'none' | 'light' | 'dark';
    targetOptions?: RenderTargetOptions;
    addNodeCommentsAsNotes?: boolean;
    notePosition?: 'left' | 'right' | 'top' | 'bottom';
    layerSourceStyles?: LayerSourceStyleOverride[];
  };
  graphConfig?: {
    generateHierarchy?: boolean;
    maxPartitionDepth?: number | null;
    maxPartitionWidth?: number | null;
    invertGraph?: boolean;
    nodeLabelMaxLength?: number;
    nodeLabelInsertNewlinesAt?: number;
    edgeLabelMaxLength?: number;
    edgeLabelInsertNewlinesAt?: number;
  };
}

// Projection Node Configuration
export interface ProjectionNodeConfig {
  projectionId?: number; // Reference to Projection entity
}

// Story Node Configuration
export interface StoryNodeConfig {
  storyId?: number; // Reference to Story entity
}

// Sequence Artefact Node Configuration
export type SequenceArtefactRenderTarget = 'MermaidSequence' | 'PlantUmlSequence';

export interface SequenceArtefactNodeConfig {
  renderTarget: SequenceArtefactRenderTarget;
  outputPath: string;
  renderConfig?: {
    containNodes?: 'one' | 'all';
    builtInStyles?: 'none' | 'light' | 'dark';
    showNotes?: boolean;
    renderAllSequences?: boolean;
    enabledSequenceIds?: number[];
  };
  useStoryLayers?: boolean;
}

// Union type for all node configurations
export type NodeConfig =
  | DataSetNodeConfig
  | GraphNodeConfig
  | TransformNodeConfig
  | FilterNodeConfig
  | MergeNodeConfig
  | GraphArtefactNodeConfig
  | TreeArtefactNodeConfig
  | ProjectionNodeConfig
  | StoryNodeConfig
  | SequenceArtefactNodeConfig;

export interface RenderTargetOptions {
  graphviz?: GraphvizRenderOptions;
  mermaid?: MermaidRenderOptions;
}

export interface GraphvizRenderOptions {
  layout?: 'dot' | 'neato' | 'fdp' | 'circo';
  overlap?: boolean;
  splines?: boolean;
  nodesep?: number;
  ranksep?: number;
   commentStyle?: 'label' | 'tooltip';
}

export interface MermaidRenderOptions {
  look?: 'default' | 'handDrawn';
  display?: 'full' | 'compact';
}

export type LayerSourceStyleMode = 'default' | 'light' | 'dark';

export interface LayerSourceStyleOverride {
  sourceDatasetId?: number | null;
  mode: LayerSourceStyleMode;
}

export const DEFAULT_GRAPHVIZ_OPTIONS: GraphvizRenderOptions = {
  layout: 'dot',
  overlap: false,
  splines: true,
  nodesep: 0.3,
  ranksep: 1.3,
  commentStyle: 'label',
};

export const DEFAULT_MERMAID_OPTIONS: MermaidRenderOptions = {
  look: 'default',
  display: 'full',
};

// Execution metadata for DataSet nodes
export interface DataSetExecutionMetadata {
  dataSetId: number;
  filename: string;
  status: string;
  processedAt?: string;
  executionState: string;
  errorMessage?: string;
}

// Execution metadata for Graph nodes
export interface GraphExecutionMetadata {
  graphId: number;
  graphDataId?: number;
  nodeCount: number;
  edgeCount: number;
  executionState: string;
  computedDate?: string;
  errorMessage?: string;
  annotations?: string | null;
}

// Plan DAG Node Structure
export interface PlanDagNode {
  id: string;
  nodeType: PlanDagNodeType;
  position: Position;
  sourcePosition?: string;
  targetPosition?: string;
  metadata: NodeMetadata;
  config: NodeConfig | string; // Can be object (internal) or JSON string (from GraphQL)
  datasetExecution?: DataSetExecutionMetadata;
  graphExecution?: GraphExecutionMetadata;
}

// Plan DAG Edge Structure
export interface PlanDagEdge {
  id: string;
  source: string;
  target: string;
  // Removed sourceHandle and targetHandle for floating edges
  metadata: EdgeMetadata;
}

// Plan DAG Metadata
export interface PlanDagMetadata {
  version: string;
  name?: string;
  description?: string;
  created?: string;
  lastModified?: string;
  author?: string;
}

// Complete Plan DAG Structure
export interface PlanDag {
  version: string;
  nodes: PlanDagNode[];
  edges: PlanDagEdge[];
  metadata: PlanDagMetadata;
}



export interface ProcessedGraphData {
  nodes: any[]; // Will be properly typed when we have GraphNode interface
  edges: any[]; // Will be properly typed when we have GraphEdge interface
  layers: any[]; // Will be properly typed when we have GraphLayer interface
}

// CSV format specifications
export interface CSVNodeRow {
  id: string;
  label: string;
  layer?: string;
  x?: number;
  y?: number;
  [key: string]: any; // additional metadata
}

export interface CSVEdgeRow {
  id: string;
  source: string;
  target: string;
  label?: string;
  [key: string]: any; // additional metadata
}

export interface CSVLayerRow {
  id: string;
  label: string;
  color?: string;
  [key: string]: any; // additional metadata
}

// File upload types
export interface DataSetUpload {
  file: File;
  name: string;
  description?: string;
  projectId: number;
}

// ReactFlow-specific types for rendering
export interface ReactFlowNode extends PlanDagNode {
  data: {
    label: string;
    nodeType: PlanDagNodeType;
    config: NodeConfig;
    metadata: NodeMetadata;
    datasetExecution?: DataSetExecutionMetadata;
    graphExecution?: GraphExecutionMetadata;
    hasValidConfig?: boolean;
    projectId?: number;
    edges?: any[];
    [key: string]: any; // Allow additional properties
  };
  draggable?: boolean;
  selectable?: boolean;
}

export interface ReactFlowEdge extends PlanDagEdge {
  // Removed sourceHandle and targetHandle for floating edges
  type?: string;
  animated?: boolean;
  style?: Record<string, any>;
  labelStyle?: Record<string, any>;
  label?: string;
}

// Connection validation types
export interface ConnectionType {
  sourceType: PlanDagNodeType;
  targetType: PlanDagNodeType;
  dataType: EdgeMetadata['dataType'];
  isValid: boolean;
  errorMessage?: string;
}

// Node creation templates
export interface NodeTemplate {
  type: PlanDagNodeType;
  defaultConfig: NodeConfig;
  defaultMetadata: NodeMetadata;
  requiredInputs: string[];
  outputs: string[];
}

// Validation result for Plan DAG
export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  nodeId?: string;
  edgeId?: string;
  type: 'MissingInput' | 'InvalidConnection' | 'CyclicDependency' | 'InvalidConfig';
  message: string;
}

export interface ValidationWarning {
  nodeId?: string;
  edgeId?: string;
  type: 'UnusedOutput' | 'PerformanceImpact' | 'ConfigurationSuggestion';
  message: string;
}
