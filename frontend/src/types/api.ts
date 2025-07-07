// Core API types for Layercake (GraphQL compatible)
export interface Project {
  id: number;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Plan {
  id: number;
  projectId: number;
  name: string;
  planContent: string;
  planFormat: 'json' | 'yaml';
  planSchemaVersion: string;
  dependencies?: number[];
  status: 'pending' | 'running' | 'completed' | 'failed';
  createdAt: string;
  updatedAt: string;
}

export interface Node {
  id: number;
  projectId: number;
  nodeId: string;
  label: string;
  layerId?: string;
  properties?: Record<string, any>;
}

export interface Edge {
  id: number;
  projectId: number;
  sourceNodeId: string;
  targetNodeId: string;
  properties?: Record<string, any>;
}

export interface Layer {
  id: number;
  projectId: number;
  layerId: string;
  name: string;
  color?: string;
  properties?: Record<string, any>;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
}

export interface UpdateProjectRequest {
  name?: string;
  description?: string;
}

export interface CreatePlanRequest {
  name: string;
  planContent: string;
  dependencies?: number[];
}

export interface UpdatePlanRequest {
  name?: string;
  planContent?: string;
  dependencies?: number[];
}

export interface ExecutePlanRequest {
  planId: number;
}

export interface ExecutePlanResponse {
  status: string;
  planId: number;
  message?: string;
}

export interface HealthResponse {
  service: string;
  status: string;
  version: string;
  timestamp: string;
}