import { gql } from '@apollo/client';

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
