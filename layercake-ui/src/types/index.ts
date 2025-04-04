export interface Project {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Node {
  id: string;
  label: string;
  layer: string;
  isPartition: boolean;
  belongsTo?: string;
  weight: number;
  comment?: string;
}

export interface Edge {
  id: string;
  source: string;
  target: string;
  label: string;
  layer: string;
  weight: number;
  comment?: string;
}

export interface Layer {
  id: string;
  label: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
}

export interface Graph {
  id: string;
  projectId: string;
  nodes: Node[];
  edges: Edge[];
  layers: Layer[];
}

export interface PlanMeta {
  name?: string;
}

export interface ImportProfile {
  filename: string;
  filetype: string;
}

export interface ImportConfig {
  profiles: ImportProfile[];
}

export interface ExportProfileGraphConfig {
  generateHierarchy?: boolean;
  maxPartitionDepth?: number;
  maxPartitionWidth?: number;
  invertGraph?: boolean;
  nodeLabelMaxLength?: number;
  nodeLabelInsertNewlinesAt?: number;
  edgeLabelMaxLength?: number;
  edgeLabelInsertNewlinesAt?: number;
}

export interface ExportProfileItem {
  filename: string;
  exporter: string;
  graphConfig?: ExportProfileGraphConfig;
}

export interface ExportProfile {
  profiles: ExportProfileItem[];
}

export interface Plan {
  meta?: PlanMeta;
  import: ImportConfig;
  export: ExportProfile;
}
