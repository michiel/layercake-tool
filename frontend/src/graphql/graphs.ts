import { gql } from '@apollo/client';

export const GET_GRAPH_DATA = gql`
  query GetGraphData($projectId: Int!) {
    graphData(projectId: $projectId) {
      nodes {
        id
        label
        layer
        isPartition
        weight
      }
      edges {
        id
        source
        target
        label
        layer
        weight
      }
      layers {
        id
        name
        color
      }
    }
  }
`;

export interface GraphNode {
  id: string;
  label: string;
  layer: string;
  isPartition: boolean;
  weight: number;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label: string;
  layer: string;
  weight: number;
}

export interface GraphLayer {
  id: string;
  name: string;
  color: string;
}

export interface GraphDataResponse {
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: GraphLayer[];
}
