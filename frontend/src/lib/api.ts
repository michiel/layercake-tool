import axios from 'axios';
import type {
  Project,
  Plan,
  Node,
  Edge,
  Layer,
  CreateProjectRequest,
  UpdateProjectRequest,
  CreatePlanRequest,
  UpdatePlanRequest,
  ExecutePlanResponse,
  HealthResponse,
} from '@/types/api';

// Create axios instance with default configuration
const api = axios.create({
  baseURL: '/api/v1',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add request interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error);
    return Promise.reject(error);
  }
);

// Health check
export const healthCheck = async (): Promise<HealthResponse> => {
  const response = await axios.get('/health');
  return response.data;
};

// Projects API
export const projectsApi = {
  getAll: async (): Promise<Project[]> => {
    const response = await api.get('/projects');
    return response.data;
  },

  getById: async (id: number): Promise<Project> => {
    const response = await api.get(`/projects/${id}`);
    return response.data;
  },

  create: async (data: CreateProjectRequest): Promise<Project> => {
    const response = await api.post('/projects', data);
    return response.data;
  },

  update: async (id: number, data: UpdateProjectRequest): Promise<Project> => {
    const response = await api.put(`/projects/${id}`, data);
    return response.data;
  },

  delete: async (id: number): Promise<void> => {
    await api.delete(`/projects/${id}`);
  },
};

// Plans API
export const plansApi = {
  getByProject: async (projectId: number): Promise<Plan[]> => {
    const response = await api.get(`/projects/${projectId}/plans`);
    return response.data;
  },

  getById: async (planId: number): Promise<Plan> => {
    const response = await api.get(`/plans/${planId}`);
    return response.data;
  },

  create: async (projectId: number, data: CreatePlanRequest): Promise<Plan> => {
    const response = await api.post(`/projects/${projectId}/plans`, data);
    return response.data;
  },

  update: async (planId: number, data: UpdatePlanRequest): Promise<Plan> => {
    const response = await api.put(`/plans/${planId}`, data);
    return response.data;
  },

  delete: async (planId: number): Promise<void> => {
    await api.delete(`/plans/${planId}`);
  },

  execute: async (planId: number): Promise<ExecutePlanResponse> => {
    const response = await api.post(`/plans/${planId}/execute`);
    return response.data;
  },
};

// Graph data API
export const graphDataApi = {
  // Nodes
  getNodes: async (projectId: number): Promise<Node[]> => {
    const response = await api.get(`/projects/${projectId}/nodes`);
    return response.data;
  },

  createNode: async (projectId: number, data: Partial<Node>): Promise<Node> => {
    const response = await api.post(`/projects/${projectId}/nodes`, data);
    return response.data;
  },

  updateNode: async (projectId: number, nodeId: string, data: Partial<Node>): Promise<Node> => {
    const response = await api.put(`/projects/${projectId}/nodes/${nodeId}`, data);
    return response.data;
  },

  deleteNode: async (projectId: number, nodeId: string): Promise<void> => {
    await api.delete(`/projects/${projectId}/nodes/${nodeId}`);
  },

  // Edges
  getEdges: async (projectId: number): Promise<Edge[]> => {
    const response = await api.get(`/projects/${projectId}/edges`);
    return response.data;
  },

  createEdge: async (projectId: number, data: Partial<Edge>): Promise<Edge> => {
    const response = await api.post(`/projects/${projectId}/edges`, data);
    return response.data;
  },

  updateEdge: async (projectId: number, edgeId: string, data: Partial<Edge>): Promise<Edge> => {
    const response = await api.put(`/projects/${projectId}/edges/${edgeId}`, data);
    return response.data;
  },

  deleteEdge: async (projectId: number, edgeId: string): Promise<void> => {
    await api.delete(`/projects/${projectId}/edges/${edgeId}`);
  },

  // Layers
  getLayers: async (projectId: number): Promise<Layer[]> => {
    const response = await api.get(`/projects/${projectId}/layers`);
    return response.data;
  },

  createLayer: async (projectId: number, data: Partial<Layer>): Promise<Layer> => {
    const response = await api.post(`/projects/${projectId}/layers`, data);
    return response.data;
  },

  updateLayer: async (projectId: number, layerId: string, data: Partial<Layer>): Promise<Layer> => {
    const response = await api.put(`/projects/${projectId}/layers/${layerId}`, data);
    return response.data;
  },

  deleteLayer: async (projectId: number, layerId: string): Promise<void> => {
    await api.delete(`/projects/${projectId}/layers/${layerId}`);
  },
};

export default api;