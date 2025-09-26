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
  dataSourceId?: number; // Reference to DataSource entity (new)
  displayMode?: 'summary' | 'detailed' | 'preview'; // Optional for backward compatibility
  outputGraphRef: string;

  // Legacy support (to be deprecated after migration)
  inputType?: 'CSVNodesFromFile' | 'CSVEdgesFromFile' | 'CSVLayersFromFile';
  source?: string;
  dataType?: 'Nodes' | 'Edges' | 'Layers';
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

// DataSource types (new)
export type DataSourceType = 'csv_nodes' | 'csv_edges' | 'csv_layers' | 'json_graph';

export interface DataSource {
  id: number;
  projectId: number;
  name: string;
  description?: string;
  sourceType: DataSourceType;
  filename: string;
  graphJson: string;
  status: 'active' | 'processing' | 'error';
  errorMessage?: string;
  fileSize: number;
  processedAt?: string;
  createdAt: string;
  updatedAt: string;
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
export interface DataSourceUpload {
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