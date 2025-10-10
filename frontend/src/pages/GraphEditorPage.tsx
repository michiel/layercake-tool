import React, { useState, useCallback, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Container, Title, Alert, LoadingOverlay, Button, Stack, Flex } from '@mantine/core';
import { IconAlertCircle, IconArrowLeft } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { Breadcrumbs } from '../components/common/Breadcrumbs';
import { LayercakeGraphEditor } from '../components/graphs/LayercakeGraphEditor';
import { PropertiesAndLayersPanel } from '../components/graphs/PropertiesAndLayersPanel';
import { ReactFlowProvider, Node as FlowNode, Edge as FlowEdge } from 'reactflow';
import { Graph, GraphNode, UPDATE_GRAPH_NODE, UPDATE_LAYER_PROPERTIES } from '../graphql/graphs';

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

  // Store references to ReactFlow setters for optimistic updates
  const setNodesRef = useRef<React.Dispatch<React.SetStateAction<FlowNode[]>> | null>(null);
  const setEdgesRef = useRef<React.Dispatch<React.SetStateAction<FlowEdge[]>> | null>(null);

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS);
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'));

  const { data: graphData, loading: graphLoading, error: graphError } = useQuery<{ graph: Graph }, { id: number }>(GET_GRAPH_DETAILS, {
    variables: { id: parseInt(graphId || '0') },
    skip: !graphId,
  });

  const [updateGraphNode] = useMutation(UPDATE_GRAPH_NODE);
  const [updateLayerProperties] = useMutation(UPDATE_LAYER_PROPERTIES);

  const graph: Graph | null = graphData?.graph || null;

  const handleNavigate = (route: string) => {
    navigate(route);
  };

  const handleBack = () => {
    navigate(`/projects/${projectId}/graphs`);
  };

  // Callback to capture ReactFlow setters for optimistic updates
  const handleNodesInitialized = useCallback((
    setNodes: React.Dispatch<React.SetStateAction<FlowNode[]>>,
    setEdges: React.Dispatch<React.SetStateAction<FlowEdge[]>>
  ) => {
    setNodesRef.current = setNodes;
    setEdgesRef.current = setEdges;
  }, []);

  const handleNodeUpdate = useCallback((nodeId: string, updates: Partial<GraphNode>) => {
    if (!graphId) return;

    // Optimistic update: immediately update the node in ReactFlow
    if (setNodesRef.current) {
      setNodesRef.current(currentNodes => {
        return currentNodes.map(node => {
          if (node.id === nodeId) {
            // Update node data and style
            const updatedNode = { ...node };

            if (updates.label !== undefined) {
              updatedNode.data = { ...node.data, label: updates.label };
            }

            if (updates.layer !== undefined) {
              // Update layer in data
              updatedNode.data = { ...updatedNode.data, layer: updates.layer };

              // Update style if layer changed (will be applied when graph refetches)
              // For now, just store the layer change
            }

            return updatedNode;
          }
          return node;
        });
      });
    }

    // Send mutation to server (no refetch)
    updateGraphNode({
      variables: {
        graphId: parseInt(graphId),
        nodeId,
        label: updates.label,
        layer: updates.layer,
        attrs: updates.attrs,
      },
    }).catch(error => {
      console.error('Failed to update node:', error);
      // TODO: Rollback optimistic update on error
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

  const handleLayerColorChange = useCallback((layerId: string, colorType: 'background' | 'border' | 'text', color: string) => {
    if (!graph) return;

    // Find the layer
    const layer = graph.layers.find(l => l.layerId === layerId);
    if (!layer) return;

    // Build updated properties
    const updatedProperties = {
      ...layer.properties,
      [`${colorType}_color`]: color,
    };

    // Optimistic update: immediately update node styles in ReactFlow
    if (setNodesRef.current) {
      setNodesRef.current(currentNodes => {
        return currentNodes.map(node => {
          // Find graph node to check its layer
          const graphNode = graph.graphNodes.find(gn => gn.id === node.id);
          if (!graphNode || graphNode.layer !== layerId) return node;

          // Update node style based on color type
          const newStyle = { ...node.style };

          if (colorType === 'background') {
            newStyle.backgroundColor = `#${color}`;
          } else if (colorType === 'border') {
            newStyle.borderColor = `#${color}`;
            newStyle.border = `${node.type === 'group' ? '2px' : '1px'} solid #${color}`;
          } else if (colorType === 'text') {
            newStyle.color = `#${color}`;
          }

          return { ...node, style: newStyle };
        });
      });
    }

    // Update edges if they have this layer
    if (setEdgesRef.current && (colorType === 'border' || colorType === 'text')) {
      setEdgesRef.current(currentEdges => {
        return currentEdges.map(edge => {
          const graphEdge = graph.graphEdges.find(ge => ge.id === edge.id);
          if (!graphEdge || graphEdge.layer !== layerId) return edge;

          const newStyle = { ...edge.style };
          if (colorType === 'border' || colorType === 'text') {
            newStyle.stroke = `#${color}`;
          }

          return { ...edge, style: newStyle };
        });
      });
    }

    // Send mutation to server
    updateLayerProperties({
      variables: {
        id: layer.id,
        properties: updatedProperties,
      },
    }).catch(error => {
      console.error('Failed to update layer properties:', error);
      // TODO: Rollback optimistic update on error
    });
  }, [graph, updateLayerProperties]);

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
              onNodesInitialized={handleNodesInitialized}
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
          onLayerColorChange={handleLayerColorChange}
        />
      </Flex>
    </Stack>
  );
};
