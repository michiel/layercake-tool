import type { Node, Edge, Layer } from '@/types/api';

// Basic mock data for testing
export const mockLayers: Layer[] = [
  {
    id: 1,
    project_id: 1,
    layer_id: 'services',
    name: 'Services',
    color: '#3B82F6',
    properties: JSON.stringify({ description: 'Microservices layer' }),
  },
  {
    id: 2,
    project_id: 1,
    layer_id: 'data',
    name: 'Data Layer',
    color: '#10B981',
    properties: JSON.stringify({ description: 'Database and storage layer' }),
  },
  {
    id: 3,
    project_id: 1,
    layer_id: 'gateway',
    name: 'API Gateway',
    color: '#F59E0B',
    properties: JSON.stringify({ description: 'API gateway and routing layer' }),
  },
  {
    id: 4,
    project_id: 1,
    layer_id: 'ui',
    name: 'User Interface',
    color: '#8B5CF6',
    properties: JSON.stringify({ description: 'Frontend and UI components' }),
  },
];

export const mockNodes: Node[] = [
  {
    id: 1,
    project_id: 1,
    node_id: 'api_gateway',
    label: 'API Gateway',
    layer_id: 'gateway',
    properties: JSON.stringify({ 
      type: 'gateway',
      instances: 2,
      health: 'healthy'
    }),
  },
  {
    id: 2,
    project_id: 1,
    node_id: 'user_service',
    label: 'User Service',
    layer_id: 'services',
    properties: JSON.stringify({ 
      type: 'microservice',
      language: 'Node.js',
      version: '1.2.3'
    }),
  },
  {
    id: 3,
    project_id: 1,
    node_id: 'order_service',
    label: 'Order Service',
    layer_id: 'services',
    properties: JSON.stringify({ 
      type: 'microservice',
      language: 'Python',
      version: '2.1.0'
    }),
  },
  {
    id: 4,
    project_id: 1,
    node_id: 'user_db',
    label: 'User Database',
    layer_id: 'data',
    properties: JSON.stringify({ 
      type: 'database',
      engine: 'PostgreSQL',
      size: '100GB'
    }),
  },
  {
    id: 5,
    project_id: 1,
    node_id: 'order_db',
    label: 'Order Database',
    layer_id: 'data',
    properties: JSON.stringify({ 
      type: 'database',
      engine: 'MongoDB',
      size: '50GB'
    }),
  },
  {
    id: 6,
    project_id: 1,
    node_id: 'web_app',
    label: 'Web Application',
    layer_id: 'ui',
    properties: JSON.stringify({ 
      type: 'frontend',
      framework: 'React',
      version: '18.2.0'
    }),
  },
];

export const mockEdges: Edge[] = [
  {
    id: 1,
    project_id: 1,
    source_node_id: 'api_gateway',
    target_node_id: 'user_service',
    properties: JSON.stringify({ 
      protocol: 'HTTP',
      method: 'REST',
      latency: '15ms'
    }),
  },
  {
    id: 2,
    project_id: 1,
    source_node_id: 'api_gateway',
    target_node_id: 'order_service',
    properties: JSON.stringify({ 
      protocol: 'HTTP',
      method: 'REST',
      latency: '12ms'
    }),
  },
  {
    id: 3,
    project_id: 1,
    source_node_id: 'user_service',
    target_node_id: 'user_db',
    properties: JSON.stringify({ 
      protocol: 'TCP',
      connection_pool: 20,
      latency: '5ms'
    }),
  },
  {
    id: 4,
    project_id: 1,
    source_node_id: 'order_service',
    target_node_id: 'order_db',
    properties: JSON.stringify({ 
      protocol: 'TCP',
      connection_pool: 15,
      latency: '3ms'
    }),
  },
  {
    id: 5,
    project_id: 1,
    source_node_id: 'order_service',
    target_node_id: 'user_service',
    properties: JSON.stringify({ 
      protocol: 'HTTP',
      method: 'REST',
      async: true
    }),
  },
  {
    id: 6,
    project_id: 1,
    source_node_id: 'web_app',
    target_node_id: 'api_gateway',
    properties: JSON.stringify({ 
      protocol: 'HTTPS',
      method: 'REST',
      cors: true
    }),
  },
];

// Generate larger datasets for performance testing
export function generateMockNodes(count: number, projectId: number = 1): Node[] {
  const nodes: Node[] = [];
  const layers = ['services', 'data', 'gateway', 'ui'];
  
  for (let i = 0; i < count; i++) {
    nodes.push({
      id: i + 1,
      project_id: projectId,
      node_id: `node_${i + 1}`,
      label: `Service ${i + 1}`,
      layer_id: layers[i % layers.length],
      properties: JSON.stringify({
        type: 'generated',
        index: i,
        timestamp: new Date().toISOString(),
      }),
    });
  }
  
  return nodes;
}

export function generateMockEdges(nodeCount: number, edgeCount: number, projectId: number = 1): Edge[] {
  const edges: Edge[] = [];
  
  for (let i = 0; i < edgeCount; i++) {
    const sourceIndex = Math.floor(Math.random() * nodeCount) + 1;
    let targetIndex = Math.floor(Math.random() * nodeCount) + 1;
    
    // Ensure source and target are different
    while (targetIndex === sourceIndex) {
      targetIndex = Math.floor(Math.random() * nodeCount) + 1;
    }
    
    edges.push({
      id: i + 1,
      project_id: projectId,
      source_node_id: `node_${sourceIndex}`,
      target_node_id: `node_${targetIndex}`,
      properties: JSON.stringify({
        weight: Math.random(),
        generated: true,
        index: i,
      }),
    });
  }
  
  return edges;
}

// Predefined datasets for different scenarios
export const mockGraphDatasets = {
  small: {
    nodes: mockNodes.slice(0, 3),
    edges: mockEdges.slice(0, 2),
    layers: mockLayers.slice(0, 2),
  },
  
  medium: {
    nodes: generateMockNodes(30),
    edges: generateMockEdges(30, 45),
    layers: mockLayers,
  },
  
  large: {
    nodes: generateMockNodes(150),
    edges: generateMockEdges(150, 300),
    layers: mockLayers,
  },
  
  empty: {
    nodes: [],
    edges: [],
    layers: mockLayers,
  },
  
  default: {
    nodes: mockNodes,
    edges: mockEdges,
    layers: mockLayers,
  },
};

// Mock project data
export const mockProject = {
  id: 1,
  name: 'Example Microservices Architecture',
  description: 'A sample microservices architecture demonstrating service communication patterns',
  created_at: new Date('2025-01-01').toISOString(),
  updated_at: new Date().toISOString(),
};

// Mock plan data
export const mockPlan = {
  id: 1,
  project_id: 1,
  name: 'Architecture Analysis Plan',
  plan_content: JSON.stringify({
    meta: {
      name: 'Architecture Analysis',
      description: 'Analyze microservices architecture patterns',
    },
    export: {
      profiles: [
        { filename: 'architecture.dot', exporter: 'DOT' },
        { filename: 'services.json', exporter: 'JSON' },
      ],
    },
  }),
  plan_format: 'json',
  plan_schema_version: '1.0.0',
  dependencies: null,
  status: 'pending',
  created_at: new Date('2025-01-01').toISOString(),
  updated_at: new Date().toISOString(),
};