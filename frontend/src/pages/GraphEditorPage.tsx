import React, { useState, useCallback, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Title, Alert, LoadingOverlay, Button, Stack, Flex } from '@mantine/core';
import { IconAlertCircle, IconArrowLeft } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { Breadcrumbs } from '../components/common/Breadcrumbs';
import { LayercakeGraphEditor } from '../components/graphs/LayercakeGraphEditor';
import { PropertiesAndLayersPanel } from '../components/graphs/PropertiesAndLayersPanel';
import { ReactFlowProvider } from 'reactflow';
import { Graph, GraphNode, UPDATE_GRAPH_NODE } from '../graphql/graphs';

const GET_PROJECTS = gql`
  query GetProjects {
    projects {
      id
      name
    }
  }
`;

const GET_GRAPH_DETAILS = gql`
  query GetGraphDetails($id: Int!) {
    graph(id: $id) {
      id
      name
      nodeId
      executionState
      nodeCount
      edgeCount
      createdAt
      updatedAt
      layers {
        id
        layerId
        name
        color
        properties
      }
      graphNodes {
        id
        label
        layer
        weight
        isPartition
        belongsTo
        attrs
      }
      graphEdges {
        id
        source
        target
        label
        layer
        weight
        attrs
      }
    }
  }
`;

interface GraphEditorPageProps {}

export const GraphEditorPage: React.FC<GraphEditorPageProps> = () => {
  const navigate = useNavigate();
  const { projectId, graphId } = useParams<{ projectId: string; graphId: string }>();
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [layerVisibility, setLayerVisibility] = useState<Map<string, boolean>>(new Map());

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS);
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'));

  const { data: graphData, loading: graphLoading, error: graphError } = useQuery<{ graph: Graph }, { id: number }>(GET_GRAPH_DETAILS, {
    variables: { id: parseInt(graphId || '0') },
    skip: !graphId,
  });

  const [updateGraphNode] = useMutation(UPDATE_GRAPH_NODE, {
    refetchQueries: ['GetGraphDetails'],
  });

  const graph: Graph | null = graphData?.graph || null;

  const handleNavigate = (route: string) => {
    navigate(route);
  };

  const handleBack = () => {
    navigate(`/projects/${projectId}/graphs`);
  };

  const handleNodeUpdate = useCallback((nodeId: string, updates: Partial<GraphNode>) => {
    if (!graphId) return;

    updateGraphNode({
      variables: {
        graphId: parseInt(graphId),
        nodeId,
        label: updates.label,
        layer: updates.layer,
        attrs: updates.attrs,
      },
    });
  }, [graphId, updateGraphNode]);

  // Initialize layer visibility when graph loads
  useEffect(() => {
    if (graph) {
      const initialVisibility = new Map<string, boolean>();
      graph.layers.forEach(layer => {
        initialVisibility.set(layer.layerId, true); // All visible by default
      });
      setLayerVisibility(initialVisibility);
    }
  }, [graph?.id]); // Only re-run when graph ID changes

  const handleLayerVisibilityToggle = useCallback((layerId: string) => {
    setLayerVisibility(prev => {
      const newMap = new Map(prev);
      newMap.set(layerId, !prev.get(layerId));
      return newMap;
    });
  }, []);

  const handleShowAllLayers = useCallback(() => {
    setLayerVisibility(prev => {
      const newMap = new Map(prev);
      newMap.forEach((_, layerId) => newMap.set(layerId, true));
      return newMap;
    });
  }, []);

  const handleHideAllLayers = useCallback(() => {
    setLayerVisibility(prev => {
      const newMap = new Map(prev);
      newMap.forEach((_, layerId) => newMap.set(layerId, false));
      return newMap;
    });
  }, []);

  if (!selectedProject) {
    return (
      <Container size="xl">
        <Title order={1}>Project Not Found</Title>
        <Button onClick={() => navigate('/projects')} mt="md">
          Back to Projects
        </Button>
      </Container>
    );
  }

  if (graphLoading) {
    return (
      <Container size="xl">
        <LoadingOverlay visible />
        <div style={{ height: '400px' }} />
      </Container>
    );
  }

  if (graphError || !graph) {
    return (
      <Container size="xl">
        <Alert
          icon={<IconAlertCircle size={16} />}
          title="Error Loading Graph"
          color="red"
          mb="md"
        >
          {graphError?.message || 'Graph not found'}
        </Alert>
        <Button onClick={handleBack} leftSection={<IconArrowLeft size={16} />}>
          Back to Graphs
        </Button>
      </Container>
    );
  }

  return (
    <Stack gap={0} style={{ height: 'calc(100vh - 60px)', width: '100%', margin: '-16px' }}>
      <div style={{ padding: '8px 16px', borderBottom: '1px solid #e9ecef' }}>
        <Breadcrumbs
          projectName={selectedProject.name}
          projectId={selectedProject.id}
          currentPage={`Graphs > ${graph.name}`}
          onNavigate={handleNavigate}
        />
      </div>

      <Flex style={{ flex: 1, overflow: 'hidden' }}>
        <div style={{ flex: 1, position: 'relative' }}>
          <ReactFlowProvider>
            <LayercakeGraphEditor
              graph={graph}
              onNodeSelect={setSelectedNodeId}
              layerVisibility={layerVisibility}
            />
          </ReactFlowProvider>
        </div>

        <PropertiesAndLayersPanel
          graph={graph}
          selectedNodeId={selectedNodeId}
          onNodeUpdate={handleNodeUpdate}
          layerVisibility={layerVisibility}
          onLayerVisibilityToggle={handleLayerVisibilityToggle}
          onShowAllLayers={handleShowAllLayers}
          onHideAllLayers={handleHideAllLayers}
        />
      </Flex>
    </Stack>
  );
};
