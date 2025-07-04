// Core API types for Layercake
export interface Project {
  id: number;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
}

export interface Plan {
  id: number;
  project_id: number;
  name: string;
  plan_content: string;
  plan_format: 'json' | 'yaml';
  plan_schema_version: string;
  dependencies?: number[];
  status: 'pending' | 'running' | 'completed' | 'failed';
  created_at: string;
  updated_at: string;
}

export interface Node {
  id: number;
  project_id: number;
  node_id: string;
  label: string;
  layer_id?: string;
  properties?: Record<string, any>;
}

export interface Edge {
  id: number;
  project_id: number;
  source_node_id: string;
  target_node_id: string;
  properties?: Record<string, any>;
}

export interface Layer {
  id: number;
  project_id: number;
  layer_id: string;
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
  plan_content: string;
  dependencies?: number[];
}

export interface UpdatePlanRequest {
  name?: string;
  plan_content?: string;
  dependencies?: number[];
}

export interface ExecutePlanRequest {
  plan_id: number;
}

export interface ExecutePlanResponse {
  status: string;
  plan_id: number;
  message?: string;
}

export interface HealthResponse {
  service: string;
  status: string;
  version: string;
  timestamp: string;
}