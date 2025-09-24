/**
 * Plan DAG TypeScript Interfaces
 *
 * Based on the Plan DAG JSON schema design, these interfaces represent
 * the visual editing data structures for Plan DAGs in ReactFlow.
 */

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
  dataType: 'GraphData' | 'GraphReference';
}

// Plan DAG Node Types
export enum PlanDagNodeType {
  DATA_SOURCE = 'DataSourceNode',
  GRAPH = 'GraphNode',
  TRANSFORM = 'TransformNode',
  MERGE = 'MergeNode',
  COPY = 'CopyNode',
  OUTPUT = 'OutputNode'
}

// Data Source Node Configuration
export interface DataSourceNodeConfig {
  inputType: 'CSVNodesFromFile' | 'CSVEdgesFromFile' | 'CSVLayersFromFile';
  source: string;
  dataType: 'Nodes' | 'Edges' | 'Layers';
  outputGraphRef: string;
}

// Graph Node Configuration
export interface GraphNodeConfig {
  graphId: number;
  isReference: boolean;
  metadata: {
    nodeCount?: number;
    edgeCount?: number;
    lastModified?: string;
  };
}

// Transform Node Configuration
export interface TransformNodeConfig {
  inputGraphRef: string;
  outputGraphRef: string;
  transformType: 'PartitionDepthLimit' | 'InvertGraph' | 'FilterNodes' | 'FilterEdges';
  transformConfig: {
    maxPartitionDepth?: number;
    maxPartitionWidth?: number;
    generateHierarchy?: boolean;
    invertGraph?: boolean;
    nodeFilter?: string;
    edgeFilter?: string;
  };
}

// Merge Node Configuration
export interface MergeNodeConfig {
  inputRefs: string[];
  outputGraphRef: string;
  mergeStrategy: 'Union' | 'Intersection' | 'Difference';
  conflictResolution: 'PreferFirst' | 'PreferLast' | 'Manual';
}

// Copy Node Configuration
export interface CopyNodeConfig {
  sourceGraphRef: string;
  outputGraphRef: string;
  copyType: 'DeepCopy' | 'ShallowCopy' | 'Reference';
  preserveMetadata: boolean;
}

// Output Node Configuration
export interface OutputNodeConfig {
  sourceGraphRef: string;
  renderTarget: 'DOT' | 'GML' | 'JSON' | 'PlantUML' | 'CSVNodes' | 'CSVEdges' | 'Mermaid' | 'Custom';
  outputPath: string;
  renderConfig?: {
    containNodes?: boolean;
    orientation?: 'LR' | 'TB';
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

// Union type for all node configurations
export type NodeConfig =
  | DataSourceNodeConfig
  | GraphNodeConfig
  | TransformNodeConfig
  | MergeNodeConfig
  | CopyNodeConfig
  | OutputNodeConfig;

// Plan DAG Node Structure
export interface PlanDagNode {
  id: string;
  nodeType: PlanDagNodeType;
  position: Position;
  metadata: NodeMetadata;
  config: NodeConfig | string; // Can be object (internal) or JSON string (from GraphQL)
}

// Plan DAG Edge Structure
export interface PlanDagEdge {
  id: string;
  source: string;
  target: string;
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

// ReactFlow-specific types for rendering
export interface ReactFlowNode extends PlanDagNode {
  data: {
    label: string;
    nodeType: PlanDagNodeType;
    config: NodeConfig;
    metadata: NodeMetadata;
  };
  draggable?: boolean;
  selectable?: boolean;
}

export interface ReactFlowEdge extends PlanDagEdge {
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