import { gql } from '@apollo/client';

// Input type interfaces
export interface NodeInput {
  label: string;
  layer: string;
  isPartition?: boolean;
  belongsTo?: string | null;
  weight?: number;
  comment?: string | null;
}

export interface EdgeInput {
  source: string;
  target: string;
  label: string;
  layer: string;
  weight?: number;
  comment?: string | null;
}

export interface LayerInput {
  label: string;
  backgroundColor: string;
  textColor: string;
  borderColor: string;
}

export const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`;

export const GET_PROJECT = gql`
  query GetProject($id: ID!) {
    project(id: $id) {
      id
      name
      description
      createdAt
      updatedAt
      graph {
        id
        nodes {
          id
          label
        }
        edges {
          id
          source
          target
        }
        layers {
          id
          label
        }
      }
      plan {
        meta {
          name
        }
        import {
          profiles {
            filename
            filetype
          }
        }
        export {
          profiles {
            filename
            exporter
          }
        }
      }
    }
  }
`;

export const GET_GRAPH = gql`
  query GetGraph($projectId: ID!) {
    graph(projectId: $projectId) {
      id
      projectId
      nodes {
        id
        label
        layer
        isPartition
        belongsTo
        weight
        comment
      }
      edges {
        id
        source
        target
        label
        layer
        weight
        comment
      }
      layers {
        id
        label
        backgroundColor
        textColor
        borderColor
      }
    }
  }
`;

export const GET_PLAN = gql`
  query GetPlan($projectId: ID!) {
    plan(projectId: $projectId) {
      meta {
        name
      }
      import {
        profiles {
          filename
          filetype
        }
      }
      export {
        profiles {
          filename
          exporter
          graphConfig {
            generateHierarchy
            maxPartitionDepth
            maxPartitionWidth
            invertGraph
            nodeLabelMaxLength
            nodeLabelInsertNewlinesAt
            edgeLabelMaxLength
            edgeLabelInsertNewlinesAt
          }
        }
      }
    }
  }
`;

// Graph Mutations
export const UPDATE_GRAPH = gql`
  mutation UpdateGraph($projectId: ID!, $graphData: String!) {
    updateGraph(projectId: $projectId, graphData: $graphData)
  }
`;

// Since we don't have granular mutations in the backend yet,
// we'll need to transform our changes into full graph updates
// These are the mutations we'd ideally have:

/*
export const ADD_NODE = gql`
  mutation AddNode($projectId: ID!, $node: NodeInput!) {
    addNode(projectId: $projectId, node: $node) {
      id
      label
      layer
      isPartition
      belongsTo
      weight
      comment
    }
  }
`;

export const UPDATE_NODE = gql`
  mutation UpdateNode($projectId: ID!, $nodeId: String!, $node: NodeInput!) {
    updateNode(projectId: $projectId, nodeId: $nodeId, node: $node) {
      id
      label
      layer
      isPartition
      belongsTo
      weight
      comment
    }
  }
`;

export const DELETE_NODE = gql`
  mutation DeleteNode($projectId: ID!, $nodeId: String!) {
    deleteNode(projectId: $projectId, nodeId: $nodeId)
  }
`;

export const ADD_EDGE = gql`
  mutation AddEdge($projectId: ID!, $edge: EdgeInput!) {
    addEdge(projectId: $projectId, edge: $edge) {
      id
      source
      target
      label
      layer
      weight
      comment
    }
  }
`;

export const UPDATE_EDGE = gql`
  mutation UpdateEdge($projectId: ID!, $edgeId: String!, $edge: EdgeInput!) {
    updateEdge(projectId: $projectId, edgeId: $edgeId, edge: $edge) {
      id
      source
      target
      label
      layer
      weight
      comment
    }
  }
`;

export const DELETE_EDGE = gql`
  mutation DeleteEdge($projectId: ID!, $edgeId: String!) {
    deleteEdge(projectId: $projectId, edgeId: $edgeId)
  }
`;

export const ADD_LAYER = gql`
  mutation AddLayer($projectId: ID!, $layer: LayerInput!) {
    addLayer(projectId: $projectId, layer: $layer) {
      id
      label
      backgroundColor
      textColor
      borderColor
    }
  }
`;

export const UPDATE_LAYER = gql`
  mutation UpdateLayer($projectId: ID!, $layerId: String!, $layer: LayerInput!) {
    updateLayer(projectId: $projectId, layerId: $layerId, layer: $layer) {
      id
      label
      backgroundColor
      textColor
      borderColor
    }
  }
`;

export const DELETE_LAYER = gql`
  mutation DeleteLayer($projectId: ID!, $layerId: String!) {
    deleteLayer(projectId: $projectId, layerId: $layerId)
  }
`;
*/
