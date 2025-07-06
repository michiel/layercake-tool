/**
 * Sample data loader for Storybook stories
 * Provides realistic data based on the sample projects in /sample/ directory
 */

export interface SampleNode {
  id: string;
  label: string;
  layer: string;
  isPartition: boolean;
  belongsTo?: string;
  weight: number;
  comment?: string;
  x?: number;
  y?: number;
  degree?: number;
  inDegree?: number;
  outDegree?: number;
}

export interface SampleEdge {
  id: string;
  source: string;
  target: string;
  label: string;
  layer: string;
  weight: number;
  comment?: string;
}

export interface SampleLayer {
  id: string;
  label: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
  nodeCount: number;
  edgeCount: number;
  visible: boolean;
}

export interface SampleProject {
  id: number;
  name: string;
  description: string;
  created_at: string;
  updated_at: string;
}

export interface SamplePlan {
  id: number;
  project_id: number;
  name: string;
  description: string;
  content: string;
  created_at: string;
  updated_at: string;
}

// Sample data based on /sample/ref/ project
export const refModelNodes: SampleNode[] = [
  {
    id: 'root',
    label: 'Root',
    layer: 'global',
    isPartition: true,
    weight: 1,
    x: 400,
    y: 200,
    degree: 12,
    inDegree: 0,
    outDegree: 12
  },
  {
    id: 'mgmt_a',
    label: 'Management A',
    layer: 'mgmt',
    isPartition: true,
    belongsTo: 'root',
    weight: 1,
    x: 200,
    y: 100,
    degree: 8,
    inDegree: 2,
    outDegree: 6
  },
  {
    id: 'mgmt_b',
    label: 'Management B',
    layer: 'mgmt',
    isPartition: true,
    belongsTo: 'root',
    weight: 1,
    x: 600,
    y: 100,
    degree: 10,
    inDegree: 3,
    outDegree: 7
  },
  {
    id: 'drone_01',
    label: 'Drone 01',
    layer: 'drone',
    isPartition: false,
    belongsTo: 'mgmt_a',
    weight: 2,
    x: 150,
    y: 300,
    degree: 5,
    inDegree: 2,
    outDegree: 3
  },
  {
    id: 'drone_02',
    label: 'Drone 02',
    layer: 'drone',
    isPartition: false,
    belongsTo: 'mgmt_a',
    weight: 2,
    x: 250,
    y: 300,
    degree: 4,
    inDegree: 1,
    outDegree: 3
  },
  {
    id: 'drone_03',
    label: 'Drone 03',
    layer: 'drone2',
    isPartition: false,
    belongsTo: 'mgmt_b',
    weight: 3,
    comment: 'High priority drone',
    x: 550,
    y: 300,
    degree: 6,
    inDegree: 3,
    outDegree: 3
  },
  {
    id: 'drone_04',
    label: 'Drone 04',
    layer: 'drone2',
    isPartition: false,
    belongsTo: 'mgmt_b',
    weight: 2,
    x: 650,
    y: 300,
    degree: 3,
    inDegree: 1,
    outDegree: 2
  }
];

export const refModelEdges: SampleEdge[] = [
  {
    id: 'root_mgmt_a',
    source: 'root',
    target: 'mgmt_a',
    label: 'manages',
    layer: 'mgmt',
    weight: 1,
    comment: 'Management connection'
  },
  {
    id: 'root_mgmt_b',
    source: 'root',
    target: 'mgmt_b',
    label: 'manages',
    layer: 'mgmt',
    weight: 1,
    comment: 'Management connection'
  },
  {
    id: 'mgmt_a_drone_01',
    source: 'mgmt_a',
    target: 'drone_01',
    label: 'controls',
    layer: 'connection',
    weight: 2
  },
  {
    id: 'mgmt_a_drone_02',
    source: 'mgmt_a',
    target: 'drone_02',
    label: 'controls',
    layer: 'connection',
    weight: 2
  },
  {
    id: 'mgmt_b_drone_03',
    source: 'mgmt_b',
    target: 'drone_03',
    label: 'controls',
    layer: 'connection',
    weight: 3
  },
  {
    id: 'mgmt_b_drone_04',
    source: 'mgmt_b',
    target: 'drone_04',
    label: 'controls',
    layer: 'connection',
    weight: 2
  },
  {
    id: 'drone_01_drone_02',
    source: 'drone_01',
    target: 'drone_02',
    label: 'link',
    layer: 'connection',
    weight: 1,
    comment: 'Inter-drone communication'
  },
  {
    id: 'drone_03_drone_04',
    source: 'drone_03',
    target: 'drone_04',
    label: 'link',
    layer: 'connection',
    weight: 1,
    comment: 'Inter-drone communication'
  }
];

export const refModelLayers: SampleLayer[] = [
  {
    id: 'mgmt',
    label: 'Management',
    backgroundColor: '#8194a0',
    textColor: '#ffffff',
    borderColor: '#dddddd',
    nodeCount: 2,
    edgeCount: 2,
    visible: true
  },
  {
    id: 'global',
    label: 'Global Ring',
    backgroundColor: '#426070',
    textColor: '#ffffff',
    borderColor: '#dddddd',
    nodeCount: 1,
    edgeCount: 0,
    visible: true
  },
  {
    id: 'drone',
    label: 'Drone',
    backgroundColor: '#002a41',
    textColor: '#ffffff',
    borderColor: '#dddddd',
    nodeCount: 2,
    edgeCount: 3,
    visible: true
  },
  {
    id: 'drone2',
    label: 'Drone 2',
    backgroundColor: '#224558',
    textColor: '#ffffff',
    borderColor: '#dddddd',
    nodeCount: 2,
    edgeCount: 3,
    visible: true
  },
  {
    id: 'connection',
    label: 'Connection',
    backgroundColor: '#e5e7eb',
    textColor: '#374151',
    borderColor: '#9ca3af',
    nodeCount: 0,
    edgeCount: 6,
    visible: true
  }
];

// Sample data based on /sample/distributed-monolith/ project
export const distributedMonolithNodes: SampleNode[] = [
  {
    id: 'project',
    label: 'Project',
    layer: 'root',
    isPartition: true,
    weight: 1,
    x: 400,
    y: 100,
    degree: 15,
    inDegree: 0,
    outDegree: 15
  },
  {
    id: 'api_gateway_instance',
    label: 'API Gateway',
    layer: 'api_gateway',
    isPartition: false,
    belongsTo: 'project',
    weight: 5,
    comment: 'Main entry point',
    x: 400,
    y: 200,
    degree: 12,
    inDegree: 0,
    outDegree: 12
  },
  {
    id: 'lambda_17',
    label: 'User Authentication Lambda',
    layer: 'lambda',
    isPartition: false,
    belongsTo: 'project',
    weight: 3,
    x: 200,
    y: 300,
    degree: 4,
    inDegree: 2,
    outDegree: 2
  },
  {
    id: 'lambda_2',
    label: 'Data Processing Lambda',
    layer: 'lambda',
    isPartition: false,
    belongsTo: 'project',
    weight: 4,
    x: 300,
    y: 350,
    degree: 5,
    inDegree: 2,
    outDegree: 3
  },
  {
    id: 'container_1',
    label: 'Main Application Container',
    layer: 'container',
    isPartition: false,
    belongsTo: 'project',
    weight: 6,
    comment: 'Core business logic',
    x: 500,
    y: 300,
    degree: 8,
    inDegree: 3,
    outDegree: 5
  },
  {
    id: 'mysql',
    label: 'MySQL Database',
    layer: 'database',
    isPartition: true,
    belongsTo: 'project',
    weight: 4,
    x: 600,
    y: 400,
    degree: 6,
    inDegree: 4,
    outDegree: 2
  },
  {
    id: 's3_instance',
    label: 'S3 Storage',
    layer: 's3',
    isPartition: false,
    belongsTo: 'project',
    weight: 2,
    x: 100,
    y: 400,
    degree: 3,
    inDegree: 2,
    outDegree: 1
  }
];

export const distributedMonolithEdges: SampleEdge[] = [
  {
    id: 'edge_1',
    source: 'api_gateway_instance',
    target: 'lambda_17',
    label: 'Auth Route',
    layer: 'api_gateway',
    weight: 2,
    comment: 'Authentication endpoint'
  },
  {
    id: 'edge_2',
    source: 'api_gateway_instance',
    target: 'lambda_2',
    label: 'Data Route',
    layer: 'api_gateway',
    weight: 3,
    comment: 'Data processing endpoint'
  },
  {
    id: 'edge_7',
    source: 'api_gateway_instance',
    target: 'container_1',
    label: 'Main Route',
    layer: 'api_gateway',
    weight: 4,
    comment: 'Primary application endpoint'
  },
  {
    id: 'lambda_db_1',
    source: 'lambda_17',
    target: 'mysql',
    label: 'User Query',
    layer: 'database',
    weight: 2
  },
  {
    id: 'lambda_db_2',
    source: 'lambda_2',
    target: 'mysql',
    label: 'Data Query',
    layer: 'database',
    weight: 3
  },
  {
    id: 'container_db_1',
    source: 'container_1',
    target: 'mysql',
    label: 'App Query',
    layer: 'database',
    weight: 4
  },
  {
    id: 'lambda_s3_1',
    source: 'lambda_2',
    target: 's3_instance',
    label: 'File Storage',
    layer: 's3',
    weight: 2
  },
  {
    id: 'container_s3_1',
    source: 'container_1',
    target: 's3_instance',
    label: 'Asset Storage',
    layer: 's3',
    weight: 3
  }
];

export const distributedMonolithLayers: SampleLayer[] = [
  {
    id: 'root',
    label: 'Root',
    backgroundColor: '#f3f4f6',
    textColor: '#111827',
    borderColor: '#d1d5db',
    nodeCount: 1,
    edgeCount: 0,
    visible: true
  },
  {
    id: 'api_gateway',
    label: 'API Gateway',
    backgroundColor: '#dbeafe',
    textColor: '#1e40af',
    borderColor: '#3b82f6',
    nodeCount: 1,
    edgeCount: 3,
    visible: true
  },
  {
    id: 'lambda',
    label: 'Lambda Functions',
    backgroundColor: '#dcfce7',
    textColor: '#166534',
    borderColor: '#22c55e',
    nodeCount: 2,
    edgeCount: 3,
    visible: true
  },
  {
    id: 'container',
    label: 'Containers',
    backgroundColor: '#fef3c7',
    textColor: '#92400e',
    borderColor: '#f59e0b',
    nodeCount: 1,
    edgeCount: 2,
    visible: true
  },
  {
    id: 'database',
    label: 'Database',
    backgroundColor: '#fce7f3',
    textColor: '#be185d',
    borderColor: '#ec4899',
    nodeCount: 1,
    edgeCount: 3,
    visible: true
  },
  {
    id: 's3',
    label: 'S3 Storage',
    backgroundColor: '#f0fdf4',
    textColor: '#15803d',
    borderColor: '#22c55e',
    nodeCount: 1,
    edgeCount: 2,
    visible: true
  }
];

// Sample projects data
export const sampleProjects: SampleProject[] = [
  {
    id: 1,
    name: 'Reference Model',
    description: 'A reference model demonstrating hierarchical graph structures with management layers and drone networks.',
    created_at: '2024-12-01T10:00:00Z',
    updated_at: '2024-12-15T14:30:00Z'
  },
  {
    id: 2,
    name: 'Distributed Monolith',
    description: 'Analysis of a distributed monolith architecture showing API gateways, lambdas, containers, and data storage.',
    created_at: '2024-11-20T09:15:00Z',
    updated_at: '2024-12-10T16:45:00Z'
  },
  {
    id: 3,
    name: 'Attack Tree Analysis',
    description: 'Security analysis project mapping potential attack vectors and defensive strategies.',
    created_at: '2024-11-15T11:30:00Z',
    updated_at: '2024-12-05T13:20:00Z'
  },
  {
    id: 4,
    name: 'KVM Control Flow',
    description: 'Kernel virtual machine control flow analysis for hypervisor security assessment.',
    created_at: '2024-10-30T08:45:00Z',
    updated_at: '2024-11-25T12:10:00Z'
  },
  {
    id: 5,
    name: 'Layercake Overview',
    description: 'Meta-analysis of the Layercake tool architecture and component relationships.',
    created_at: '2024-10-15T14:20:00Z',
    updated_at: '2024-11-30T17:55:00Z'
  }
];

// Sample plans data
export const samplePlans: SamplePlan[] = [
  {
    id: 1,
    project_id: 1,
    name: 'Reference Export Plan',
    description: 'Export reference model to multiple formats including GML and GraphML',
    content: JSON.stringify({
      meta: { name: 'Reference Model' },
      import: {
        profiles: [
          { filename: 'nodes.csv', filetype: 'Nodes' },
          { filename: 'links.csv', filetype: 'Edges' },
          { filename: 'layers.csv', filetype: 'Layers' }
        ]
      },
      export: {
        profiles: [
          { filename: 'out/ref-model.gml', exporter: 'GML' },
          { filename: 'out/ref-model.graphml', exporter: 'GraphML' }
        ]
      }
    }),
    created_at: '2024-12-01T10:30:00Z',
    updated_at: '2024-12-15T15:00:00Z'
  },
  {
    id: 2,
    project_id: 2,
    name: 'Distributed Monolith Analysis',
    description: 'Comprehensive analysis and visualization of distributed monolith architecture',
    content: JSON.stringify({
      meta: { name: 'Distributed Monolith' },
      import: {
        profiles: [
          { filename: 'nodes.csv', filetype: 'Nodes' },
          { filename: 'edges.csv', filetype: 'Edges' },
          { filename: 'layers.csv', filetype: 'Layers' }
        ]
      },
      export: {
        profiles: [
          { filename: 'out/distributed-monolith.gml', exporter: 'GML' },
          { filename: 'out/distributed-monolith.json', exporter: 'JSON' }
        ]
      }
    }),
    created_at: '2024-11-20T09:45:00Z',
    updated_at: '2024-12-10T17:15:00Z'
  },
  {
    id: 3,
    project_id: 1,
    name: 'Hierarchy Analysis',
    description: 'Analyze hierarchical structures and export as DOT format',
    content: JSON.stringify({
      meta: { name: 'Reference Model Hierarchy' },
      transformations: [
        { type: 'filter', condition: 'is_partition == true' },
        { type: 'layout', algorithm: 'hierarchical' }
      ],
      export: {
        profiles: [
          { filename: 'out/ref-hierarchy.dot', exporter: 'DOT' }
        ]
      }
    }),
    created_at: '2024-12-05T11:15:00Z',
    updated_at: '2024-12-12T14:30:00Z'
  }
];

// Utility functions for generating sample data
export const generateRandomNodes = (count: number, layers: string[] = ['layer1', 'layer2', 'layer3']): SampleNode[] => {
  const nodes: SampleNode[] = [];
  for (let i = 0; i < count; i++) {
    nodes.push({
      id: `node_${i + 1}`,
      label: `Node ${i + 1}`,
      layer: layers[i % layers.length],
      isPartition: Math.random() > 0.7,
      weight: Math.floor(Math.random() * 5) + 1,
      x: Math.random() * 800,
      y: Math.random() * 600,
      degree: Math.floor(Math.random() * 10) + 1,
      inDegree: Math.floor(Math.random() * 5),
      outDegree: Math.floor(Math.random() * 5)
    });
  }
  return nodes;
};

export const generateRandomEdges = (nodes: SampleNode[], count: number): SampleEdge[] => {
  const edges: SampleEdge[] = [];
  for (let i = 0; i < count; i++) {
    const source = nodes[Math.floor(Math.random() * nodes.length)];
    const target = nodes[Math.floor(Math.random() * nodes.length)];
    if (source.id !== target.id) {
      edges.push({
        id: `edge_${i + 1}`,
        source: source.id,
        target: target.id,
        label: `Connection ${i + 1}`,
        layer: 'connection',
        weight: Math.floor(Math.random() * 3) + 1
      });
    }
  }
  return edges;
};

export const generateLayersFromNodes = (nodes: SampleNode[]): SampleLayer[] => {
  const layerMap = new Map<string, { nodeCount: number; edgeCount: number }>();
  
  nodes.forEach(node => {
    if (!layerMap.has(node.layer)) {
      layerMap.set(node.layer, { nodeCount: 0, edgeCount: 0 });
    }
    layerMap.get(node.layer)!.nodeCount++;
  });

  const colors = [
    { bg: '#dbeafe', text: '#1e40af', border: '#3b82f6' },
    { bg: '#dcfce7', text: '#166534', border: '#22c55e' },
    { bg: '#fef3c7', text: '#92400e', border: '#f59e0b' },
    { bg: '#fce7f3', text: '#be185d', border: '#ec4899' },
    { bg: '#f0f9ff', text: '#0c4a6e', border: '#0284c7' }
  ];

  return Array.from(layerMap.entries()).map(([layerId, stats], index) => ({
    id: layerId,
    label: layerId.charAt(0).toUpperCase() + layerId.slice(1),
    backgroundColor: colors[index % colors.length].bg,
    textColor: colors[index % colors.length].text,
    borderColor: colors[index % colors.length].border,
    nodeCount: stats.nodeCount,
    edgeCount: stats.edgeCount,
    visible: true
  }));
};

// Export all sample data sets
export const sampleDataSets = {
  refModel: {
    nodes: refModelNodes,
    edges: refModelEdges,
    layers: refModelLayers
  },
  distributedMonolith: {
    nodes: distributedMonolithNodes,
    edges: distributedMonolithEdges,
    layers: distributedMonolithLayers
  }
};