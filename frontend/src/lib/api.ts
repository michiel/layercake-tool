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

  getById: async (projectId: number, planId: number): Promise<Plan> => {
    const response = await api.get(`/projects/${projectId}/plans/${planId}`);
    return response.data;
  },

  create: async (projectId: number, data: CreatePlanRequest): Promise<Plan> => {
    const response = await api.post(`/projects/${projectId}/plans`, data);
    return response.data;
  },

  update: async (projectId: number, planId: number, data: UpdatePlanRequest): Promise<Plan> => {
    const response = await api.put(`/projects/${projectId}/plans/${planId}`, data);
    return response.data;
  },

  delete: async (projectId: number, planId: number): Promise<void> => {
    await api.delete(`/projects/${projectId}/plans/${planId}`);
  },

  execute: async (projectId: number, planId: number): Promise<ExecutePlanResponse> => {
    const response = await api.post(`/projects/${projectId}/plans/${planId}/execute`);
    return response.data;
  },
};

// Graph data API
export const graphDataApi = {
  getNodes: async (projectId: number): Promise<Node[]> => {
    const response = await api.get(`/projects/${projectId}/nodes`);
    return response.data;
  },

  getEdges: async (projectId: number): Promise<Edge[]> => {
    const response = await api.get(`/projects/${projectId}/edges`);
    return response.data;
  },

  getLayers: async (projectId: number): Promise<Layer[]> => {
    const response = await api.get(`/projects/${projectId}/layers`);
    return response.data;
  },
};

export default api;