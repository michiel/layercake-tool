import React, { useState, useCallback, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { IconAlertCircle, IconArrowLeft, IconHistory, IconEdit, IconDownload, IconRoute, IconZoomScan, IconChevronLeft, IconChevronRight } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { Breadcrumbs } from '../components/common/Breadcrumbs';
import { LayercakeGraphEditor, GraphViewMode, GraphOrientation, HierarchyViewMode } from '../components/graphs/LayercakeGraphEditor';
import { PropertiesAndLayersPanel } from '../components/graphs/PropertiesAndLayersPanel';
import EditHistoryModal from '../components/graphs/EditHistoryModal';
import { ReactFlowProvider, Node as FlowNode, Edge as FlowEdge } from 'reactflow';
import { Graph, GraphNode, UPDATE_GRAPH_NODE, GET_GRAPH_EDIT_COUNT, ADD_GRAPH_NODE, ADD_GRAPH_EDGE, UPDATE_GRAPH_EDGE, DELETE_GRAPH_EDGE, DELETE_GRAPH_NODE } from '../graphql/graphs';
import { GET_DATASOURCES, UPDATE_DATASOURCE_GRAPH_DATA, DataSet } from '../graphql/datasets';
import { Stack, Group } from '../components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '../components/ui/alert';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../components/ui/dropdown-menu';
import { Spinner } from '../components/ui/spinner';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '../components/ui/tooltip';
import { useRegisterChatContext } from '../hooks/useRegisterChatContext';

declare global {
  interface Window {
    htmlToImage?: {
      toPng: (node: HTMLElement, options?: any) => Promise<string>;
      toSvg: (node: HTMLElement, options?: any) => Promise<string>;
    };
  }
}

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
        backgroundColor
        textColor
        borderColor
        comment
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
  const [editHistoryOpen, setEditHistoryOpen] = useState(false);
  const [propertiesPanelCollapsed, setPropertiesPanelCollapsed] = useState(false);
  const [viewMode, setViewMode] = useState<GraphViewMode>('flow');
  const [orientation, setOrientation] = useState<GraphOrientation>('vertical');
  const [flowGroupingEnabled, setFlowGroupingEnabled] = useState(true);
  const [hierarchyViewMode, setHierarchyViewMode] = useState<HierarchyViewMode>('graph');
  const [fitViewTrigger, setFitViewTrigger] = useState(0);
  const [nodeSpacing, setNodeSpacing] = useState(40);
  const [rankSpacing, setRankSpacing] = useState(50);
  const [minEdgeLength, setMinEdgeLength] = useState(30);
  const [targetNodeDatasetId, setTargetNodeDatasetId] = useState<number | null>(null);
  const [targetEdgeDatasetId, setTargetEdgeDatasetId] = useState<number | null>(null);
  const reactFlowWrapperRef = useRef<HTMLDivElement>(null);
  const htmlToImagePromiseRef = useRef<Promise<any> | null>(null);

  // Store references to ReactFlow setters for optimistic updates
  const setNodesRef = useRef<React.Dispatch<React.SetStateAction<FlowNode[]>> | null>(null);
  const setEdgesRef = useRef<React.Dispatch<React.SetStateAction<FlowEdge[]>> | null>(null);

  const { data: projectsData } = useQuery<{ projects: Array<{ id: number; name: string }> }>(GET_PROJECTS);
  const selectedProject = projectsData?.projects.find((p: { id: number; name: string }) => p.id === parseInt(projectId || '0'));

  // Fetch datasets for target dataset selection
  const { data: datasetsData } = useQuery<{ dataSets: DataSet[] }>(GET_DATASOURCES, {
    variables: { projectId: parseInt(projectId || '0') },
    skip: !projectId,
  });
  const datasets = datasetsData?.dataSets || [];

  const [updateDatasetGraphData] = useMutation(UPDATE_DATASOURCE_GRAPH_DATA);

  const { data: graphData, loading: graphLoading, error: graphError } = useQuery<{ graph: Graph }, { id: number }>(GET_GRAPH_DETAILS, {
    variables: { id: parseInt(graphId || '0') },
    skip: !graphId,
  });

  const { data: editCountData, refetch: refetchEditCount } = useQuery<{ graphEditCount: number }, { graphId: number; unappliedOnly: boolean }>(
    GET_GRAPH_EDIT_COUNT,
    {
      variables: { graphId: parseInt(graphId || '0'), unappliedOnly: true },
      skip: !graphId,
      pollInterval: 10000, // Poll every 10 seconds
    }
  );

  const [updateGraphNode] = useMutation(UPDATE_GRAPH_NODE, {
    update(cache, { data }: any) {
      if (!data?.updateGraphNode) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphNodes: existingData.graph.graphNodes.map(n =>
              n.id === data.updateGraphNode.id ? { ...n, ...data.updateGraphNode } : n
            )
          }
        }
      });
    }
  });

  const [updateGraphEdge] = useMutation(UPDATE_GRAPH_EDGE, {
    update(cache, { data }: any) {
      if (!data?.updateGraphEdge) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphEdges: existingData.graph.graphEdges.map(e =>
              e.id === data.updateGraphEdge.id ? { ...e, ...data.updateGraphEdge } : e
            )
          }
        }
      });
    }
  });

  useRegisterChatContext(
    selectedProject && graphData?.graph
      ? `Editing graph "${graphData.graph.name}" (#${graphData.graph.id}) for project ${selectedProject.name} (#${selectedProject.id})`
      : 'Editing graph',
    selectedProject?.id,
  );

  const [addGraphNode] = useMutation(ADD_GRAPH_NODE, {
    update(cache, { data }: any) {
      if (!data?.addGraphNode) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphNodes: [...existingData.graph.graphNodes, data.addGraphNode],
            nodeCount: existingData.graph.nodeCount + 1
          }
        }
      });
    }
  });

  const [addGraphEdge] = useMutation(ADD_GRAPH_EDGE, {
    update(cache, { data }: any) {
      if (!data?.addGraphEdge) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphEdges: [...existingData.graph.graphEdges, data.addGraphEdge],
            edgeCount: existingData.graph.edgeCount + 1
          }
        }
      });
    }
  });

  const [deleteGraphEdge] = useMutation(DELETE_GRAPH_EDGE, {
    update(cache, { data: mutationData }: any) {
      if (!mutationData?.deleteGraphEdge) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      const edgeId = mutationData.deleteGraphEdge;
      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphEdges: existingData.graph.graphEdges.filter(e => e.id !== edgeId),
            edgeCount: existingData.graph.edgeCount - 1
          }
        }
      });
    }
  });

  const [deleteGraphNode] = useMutation(DELETE_GRAPH_NODE, {
    update(cache, { data: mutationData }: any) {
      if (!mutationData?.deleteGraphNode) return;
      const existingData = cache.readQuery<{ graph: Graph }>({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') }
      });
      if (!existingData?.graph) return;

      const nodeId = mutationData.deleteGraphNode;
      cache.writeQuery({
        query: GET_GRAPH_DETAILS,
        variables: { id: parseInt(graphId || '0') },
        data: {
          graph: {
            ...existingData.graph,
            graphNodes: existingData.graph.graphNodes.filter(n => n.id !== nodeId),
            nodeCount: existingData.graph.nodeCount - 1
          }
        }
      });
    }
  });

  const graph: Graph | null = graphData?.graph || null;

  const handleNavigate = (route: string) => {
    navigate(route);
  };

  const handleBack = () => {
    navigate(`/projects/${projectId}/plan-nodes`);
  };

  // Callback to capture ReactFlow setters for optimistic updates
  const handleNodesInitialized = useCallback((
    setNodes: React.Dispatch<React.SetStateAction<FlowNode[]>>,
    setEdges: React.Dispatch<React.SetStateAction<FlowEdge[]>>
  ) => {
    setNodesRef.current = setNodes;
    setEdgesRef.current = setEdges;
  }, []);

  const requestFitView = useCallback(() => {
    setFitViewTrigger(prev => prev + 1);
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
        belongsTo: updates.belongsTo,
      },
    }).catch(error => {
      console.error('Failed to update node:', error);
      // TODO: Rollback optimistic update on error
    });
  }, [graphId, updateGraphNode]);

  // Helper to sync node addition to target dataset
  const syncNodeToDataset = useCallback(async (node: FlowNode, operation: 'add' | 'delete') => {
    if (!targetNodeDatasetId) return;

    const dataset = datasets.find(d => d.id === targetNodeDatasetId);
    if (!dataset) return;

    try {
      const graphJson = JSON.parse(dataset.graphJson || '{"nodes":[],"edges":[],"layers":[]}');

      if (operation === 'add') {
        const newNode = {
          id: node.id,
          label: node.data?.label || 'New Node',
          layer: node.data?.layer,
          weight: node.data?.weight || 1,
          is_partition: node.data?.isPartition || false,
          belongs_to: node.parentNode || node.data?.belongsTo,
        };
        graphJson.nodes = [...(graphJson.nodes || []), newNode];
      } else {
        graphJson.nodes = (graphJson.nodes || []).filter((n: any) => n.id !== node.id);
      }

      await updateDatasetGraphData({
        variables: { id: targetNodeDatasetId, graphJson: JSON.stringify(graphJson) }
      });
    } catch (error) {
      console.error('Failed to sync node to dataset:', error);
    }
  }, [targetNodeDatasetId, datasets, updateDatasetGraphData]);

  // Helper to sync edge addition to target dataset
  const syncEdgeToDataset = useCallback(async (edge: FlowEdge, operation: 'add' | 'delete') => {
    if (!targetEdgeDatasetId) return;

    const dataset = datasets.find(d => d.id === targetEdgeDatasetId);
    if (!dataset) return;

    try {
      const graphJson = JSON.parse(dataset.graphJson || '{"nodes":[],"edges":[],"layers":[]}');

      if (operation === 'add') {
        const newEdge = {
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label || edge.data?.label,
          layer: edge.data?.layer,
          weight: edge.data?.weight || 1,
        };
        graphJson.edges = [...(graphJson.edges || []), newEdge];
      } else {
        graphJson.edges = (graphJson.edges || []).filter((e: any) => e.id !== edge.id);
      }

      await updateDatasetGraphData({
        variables: { id: targetEdgeDatasetId, graphJson: JSON.stringify(graphJson) }
      });
    } catch (error) {
      console.error('Failed to sync edge to dataset:', error);
    }
  }, [targetEdgeDatasetId, datasets, updateDatasetGraphData]);

  const handleEdgeAdd = useCallback((edge: FlowEdge) => {
    if (!graphId) return;

    addGraphEdge({
      variables: {
        graphId: parseInt(graphId),
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label,
        layer: edge.data?.layer,
        weight: edge.data?.weight,
        attrs: edge.data?.attrs,
      },
    }).catch(error => {
      console.error('Failed to add edge:', error);
      // TODO: Rollback optimistic update on error
    });

    // Sync to target dataset
    syncEdgeToDataset(edge, 'add');
  }, [graphId, addGraphEdge, syncEdgeToDataset]);

  const handleEdgeDelete = useCallback((edgeId: string) => {
    if (!graphId) return;

    deleteGraphEdge({
      variables: {
        graphId: parseInt(graphId),
        edgeId,
      },
    }).catch(error => {
      console.error('Failed to delete edge:', error);
      // TODO: Rollback optimistic update on error
    });

    // Sync to target dataset
    syncEdgeToDataset({ id: edgeId } as FlowEdge, 'delete');
  }, [graphId, deleteGraphEdge, syncEdgeToDataset]);

  const handleNodeAdd = useCallback((node: FlowNode) => {
    if (!graphId) return;

    // Extract belongsTo from either node.parentNode or node.data.belongsTo
    const belongsTo = node.parentNode || node.data?.belongsTo;

    addGraphNode({
      variables: {
        graphId: parseInt(graphId),
        id: node.id,
        label: node.data?.label || 'New Node',
        layer: node.data?.layer,
        isPartition: node.data?.isPartition || false,
        belongsTo: belongsTo,
        weight: node.data?.weight,
        attrs: node.data?.attrs,
      },
    }).catch(error => {
      console.error('Failed to add node:', error);
      // TODO: Rollback optimistic update on error
    });

    // Sync to target dataset
    syncNodeToDataset(node, 'add');
  }, [graphId, addGraphNode, syncNodeToDataset]);

  const handleNodeDelete = useCallback((nodeId: string) => {
    if (!graphId) return;

    deleteGraphNode({
      variables: {
        graphId: parseInt(graphId),
        nodeId,
      },
    }).catch(error => {
      console.error('Failed to delete node:', error);
      // TODO: Rollback optimistic update on error
    });

    // Sync to target dataset
    syncNodeToDataset({ id: nodeId } as FlowNode, 'delete');
  }, [graphId, deleteGraphNode, syncNodeToDataset]);

  const handleNodeLabelChange = useCallback((nodeId: string, newLabel: string) => {
    if (!graphId) return;

    // Optimistic update: immediately update the node label in ReactFlow
    if (setNodesRef.current) {
      setNodesRef.current(currentNodes => {
        return currentNodes.map(node => {
          // Update the main node or its label node
          if (node.id === nodeId || node.id === `${nodeId}-label`) {
            return {
              ...node,
              data: { ...node.data, label: newLabel }
            };
          }
          return node;
        });
      });
    }

    // Send mutation to server without cache update to avoid viewport shift
    // Cache will be updated by polling (every 10s) or manual refetch
    updateGraphNode({
      variables: {
        graphId: parseInt(graphId),
        nodeId,
        label: newLabel,
      },
      update: () => {
        // Skip cache update for label-only changes to prevent viewport shift
      }
    }).catch(error => {
      console.error('Failed to update node label:', error);
      // TODO: Rollback optimistic update on error
    });
  }, [graphId, updateGraphNode]);

  const handleEdgeLabelChange = useCallback((edgeId: string, newLabel: string) => {
    if (!graphId) return;

    // Optimistic update: immediately update the edge label in ReactFlow
    if (setEdgesRef.current) {
      setEdgesRef.current(currentEdges => {
        return currentEdges.map(edge => {
          if (edge.id === edgeId) {
            return {
              ...edge,
              label: newLabel,
              data: { ...edge.data, label: newLabel }
            };
          }
          return edge;
        });
      });
    }

    // Send mutation to server without cache update to avoid viewport shift
    // Cache will be updated by polling (every 10s) or manual refetch
    updateGraphEdge({
      variables: {
        graphId: parseInt(graphId),
        edgeId,
        label: newLabel,
      },
      update: () => {
        // Skip cache update for label-only changes to prevent viewport shift
      }
    }).catch(error => {
      console.error('Failed to update edge label:', error);
      // TODO: Rollback optimistic update on error
    });
  }, [graphId, updateGraphEdge]);

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

  const handleToggleViewMode = useCallback(() => {
    setViewMode(prev => (prev === 'flow' ? 'hierarchy' : 'flow'));
    requestFitView();
  }, [requestFitView]);

  const handleToggleOrientation = useCallback(() => {
    setOrientation(prev => (prev === 'vertical' ? 'horizontal' : 'vertical'));
    requestFitView();
  }, [requestFitView]);

  const handleToggleFlowGrouping = useCallback(() => {
    setFlowGroupingEnabled(prev => !prev);
    requestFitView();
  }, [requestFitView]);

  const handleToggleHierarchyViewMode = useCallback(() => {
    setHierarchyViewMode(prev => (prev === 'graph' ? 'containers' : 'graph'));
    requestFitView();
  }, [requestFitView]);

  const handleNodeSpacingChange = useCallback((value: number) => {
    setNodeSpacing(value);
    requestFitView();
  }, [requestFitView]);

  const handleRankSpacingChange = useCallback((value: number) => {
    setRankSpacing(value);
    requestFitView();
  }, [requestFitView]);

  const handleMinEdgeLengthChange = useCallback((value: number) => {
    setMinEdgeLength(value);
    requestFitView();
  }, [requestFitView]);

  const ensureHtmlToImage = useCallback((): Promise<any> => {
    if (typeof window !== 'undefined' && window.htmlToImage) {
      return Promise.resolve(window.htmlToImage);
    }
    if (htmlToImagePromiseRef.current) {
      return htmlToImagePromiseRef.current;
    }
    htmlToImagePromiseRef.current = new Promise((resolve, reject) => {
      if (typeof document === 'undefined') {
        reject(new Error('Document is not available'));
        return;
      }
      const script = document.createElement('script');
      script.src = 'https://unpkg.com/html-to-image@1.11.11/dist/html-to-image.js';
      script.async = true;
      script.onload = () => {
        if (window.htmlToImage) {
          resolve(window.htmlToImage);
        } else {
          reject(new Error('html-to-image failed to load'));
        }
      };
      script.onerror = () => reject(new Error('Failed to load html-to-image script'));
      document.body.appendChild(script);
    });
    return htmlToImagePromiseRef.current;
  }, []);

  const handleDownload = useCallback(async (format: 'png' | 'svg') => {
    try {
      const htmlToImage = await ensureHtmlToImage();
      if (!reactFlowWrapperRef.current) {
        throw new Error('Graph wrapper not available');
      }
      const target = reactFlowWrapperRef.current.querySelector('.react-flow__viewport') as HTMLElement | null;
      if (!target) {
        throw new Error('Unable to find graph viewport');
      }
      const fileNameBase = graph?.name?.replace(/\s+/g, '_').toLowerCase() || 'graph';
      if (format === 'png') {
        const dataUrl = await htmlToImage.toPng(target, {
          backgroundColor: '#ffffff',
          pixelRatio: 2,
        });
        const link = document.createElement('a');
        link.download = `${fileNameBase}.png`;
        link.href = dataUrl;
        link.click();
      } else {
        const dataUrl = await htmlToImage.toSvg(target, {
          backgroundColor: '#ffffff',
        });
        const link = document.createElement('a');
        link.download = `${fileNameBase}.svg`;
        link.href = dataUrl;
        link.click();
      }
    } catch (error) {
      console.error('Failed to download graph image:', error);
    }
  }, [ensureHtmlToImage, graph?.name]);

  // Apply edits to canvas without full re-render
  const handleApplyEdits = useCallback((edits: any[]) => {
    if (!graph) return;

    edits.forEach(edit => {
      const { targetType, targetId, operation, fieldName, newValue } = edit;

      if (targetType === 'node' && operation === 'update') {
        // Apply node update optimistically
        if (setNodesRef.current) {
          setNodesRef.current(currentNodes => {
            return currentNodes.map(node => {
              if (node.id !== targetId) return node;

              const updatedNode = { ...node };

              if (fieldName === 'label' && newValue !== undefined) {
                updatedNode.data = { ...node.data, label: newValue };
              } else if (fieldName === 'layer' && newValue !== undefined) {
                updatedNode.data = { ...updatedNode.data, layer: newValue };

                // Update node style based on new layer
                const newLayer = graph.layers.find(l => l.layerId === newValue);
                if (newLayer) {
                  const newStyle = { ...node.style };

                  if (newLayer.backgroundColor) {
                    newStyle.backgroundColor = `#${newLayer.backgroundColor}`;
                  }
                  if (newLayer.borderColor) {
                    newStyle.borderColor = `#${newLayer.borderColor}`;
                    newStyle.border = `${node.type === 'group' ? '2px' : '1px'} solid #${newLayer.borderColor}`;
                  }
                  if (newLayer.textColor) {
                    newStyle.color = `#${newLayer.textColor}`;
                  }

                  updatedNode.style = newStyle;
                }
              }

              return updatedNode;
            });
          });
        }
      } else if (targetType === 'layer' && operation === 'update' && fieldName === 'properties' && newValue) {
        // Apply layer property changes to all nodes/edges with this layer (legacy format)
        const layerId = targetId;
        const newProperties = newValue;

        if (setNodesRef.current) {
          setNodesRef.current(currentNodes => {
            return currentNodes.map(node => {
              const graphNode = graph.graphNodes.find(gn => gn.id === node.id);
              if (!graphNode || graphNode.layer !== layerId) return node;

              const newStyle = { ...node.style };

              if (newProperties.background_color) {
                newStyle.backgroundColor = `#${newProperties.background_color}`;
              }
              if (newProperties.border_color) {
                newStyle.borderColor = `#${newProperties.border_color}`;
                newStyle.border = `${node.type === 'group' ? '2px' : '1px'} solid #${newProperties.border_color}`;
              }
              if (newProperties.text_color) {
                newStyle.color = `#${newProperties.text_color}`;
              }

              return { ...node, style: newStyle };
            });
          });
        }

        if (setEdgesRef.current) {
          setEdgesRef.current(currentEdges => {
            return currentEdges.map(edge => {
              const graphEdge = graph.graphEdges.find(ge => ge.id === edge.id);
              if (!graphEdge || graphEdge.layer !== layerId) return edge;

              const newStyle = { ...edge.style };
              if (newProperties.border_color || newProperties.text_color) {
                newStyle.stroke = `#${newProperties.border_color || newProperties.text_color}`;
              }

              return { ...edge, style: newStyle };
            });
          });
        }
      } else if (targetType === 'layer' && operation === 'update' && (fieldName === 'background_color' || fieldName === 'text_color' || fieldName === 'border_color')) {
        // Apply individual color field changes to all nodes/edges with this layer
        const layerId = targetId;
        const layer = graph.layers.find(l => l.layerId === layerId);

        if (setNodesRef.current && layer) {
          setNodesRef.current(currentNodes => {
            return currentNodes.map(node => {
              const graphNode = graph.graphNodes.find(gn => gn.id === node.id);
              if (!graphNode || graphNode.layer !== layerId) return node;

              const newStyle = { ...node.style };

              if (layer.backgroundColor) {
                newStyle.backgroundColor = `#${layer.backgroundColor}`;
              }
              if (layer.borderColor) {
                newStyle.borderColor = `#${layer.borderColor}`;
                newStyle.border = `${node.type === 'group' ? '2px' : '1px'} solid #${layer.borderColor}`;
              }
              if (layer.textColor) {
                newStyle.color = `#${layer.textColor}`;
              }

              return { ...node, style: newStyle };
            });
          });
        }

        if (setEdgesRef.current && layer) {
          setEdgesRef.current(currentEdges => {
            return currentEdges.map(edge => {
              const graphEdge = graph.graphEdges.find(ge => ge.id === edge.id);
              if (!graphEdge || graphEdge.layer !== layerId) return edge;

              const newStyle = { ...edge.style };
              if (layer.borderColor || layer.textColor) {
                newStyle.stroke = `#${layer.borderColor || layer.textColor}`;
              }

              return { ...edge, style: newStyle };
            });
          });
        }
      }
    });
  }, [graph]);

  if (!selectedProject) {
    return (
      <div className="container mx-auto max-w-7xl p-4">
        <h1 className="text-3xl font-bold">Project Not Found</h1>
        <Button onClick={() => navigate('/projects')} className="mt-4">
          Back to Projects
        </Button>
      </div>
    );
  }

  if (graphLoading) {
    return (
      <div className="container mx-auto max-w-7xl p-4 relative">
        <div className="absolute inset-0 flex items-center justify-center bg-background/80">
          <Spinner className="h-8 w-8" />
        </div>
        <div style={{ height: '400px' }} />
      </div>
    );
  }

  if (graphError || !graph) {
    return (
      <div className="container mx-auto max-w-7xl p-4">
        <Alert variant="destructive" className="mb-4">
          <IconAlertCircle className="h-4 w-4" />
          <AlertTitle>Error Loading Graph</AlertTitle>
          <AlertDescription>
            {graphError?.message || 'Graph not found'}
          </AlertDescription>
        </Alert>
        <Button onClick={handleBack}>
          <IconArrowLeft className="mr-2 h-4 w-4" />
          Back to Graphs
        </Button>
      </div>
    );
  }

  const editCount = editCountData?.graphEditCount || 0;
  const hasEdits = editCount > 0;

  return (
    <TooltipProvider>
      <Stack gap="none" style={{ height: 'calc(100vh - 60px)', width: '100%', margin: '-16px' }}>
        <div style={{ padding: '8px 16px', borderBottom: '1px solid #e9ecef' }}>
          <div className="flex justify-between items-center">
            <Breadcrumbs
              projectName={selectedProject.name}
              projectId={selectedProject.id}
              sections={[
                { title: 'Workbench', href: `/projects/${projectId}/plan` },
                { title: 'Graphs', href: `/projects/${projectId}/graphs` },
              ]}
              currentPage={graph.name}
              onNavigate={handleNavigate}
            />

            <Group gap="xs">
              {hasEdits && (
                <Badge variant="secondary" className="gap-1">
                  <IconEdit className="h-3 w-3" />
                  {editCount} pending {editCount === 1 ? 'edit' : 'edits'}
                </Badge>
              )}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={requestFitView}
                  >
                    <IconZoomScan className="h-[18px] w-[18px]" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>Zoom to Fit</TooltipContent>
              </Tooltip>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon">
                    <IconDownload className="h-[18px] w-[18px]" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent>
                  <DropdownMenuItem onClick={() => handleDownload('png')}>
                    Download PNG
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => handleDownload('svg')}>
                    Download SVG
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
              {projectId && graph?.nodeId && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => navigate(`/projects/${projectId}/plan?focusNode=${graph.nodeId}`)}
                    >
                      <IconRoute className="h-[18px] w-[18px]" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>View in Plan DAG</TooltipContent>
                </Tooltip>
              )}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => setEditHistoryOpen(true)}
                  >
                    <IconHistory className="h-[18px] w-[18px]" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>View edit history</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    onClick={() => setPropertiesPanelCollapsed(!propertiesPanelCollapsed)}
                  >
                    {propertiesPanelCollapsed ? <IconChevronLeft className="h-[18px] w-[18px]" /> : <IconChevronRight className="h-[18px] w-[18px]" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>{propertiesPanelCollapsed ? "Show properties panel" : "Hide properties panel"}</TooltipContent>
              </Tooltip>
            </Group>
          </div>
        </div>

        <div className="flex flex-1 overflow-hidden">
        <div style={{ flex: 1, position: 'relative' }}>
          <ReactFlowProvider>
            <LayercakeGraphEditor
              graph={graph}
              onNodeSelect={setSelectedNodeId}
              layerVisibility={layerVisibility}
              onNodesInitialized={handleNodesInitialized}
              mode={viewMode}
              orientation={orientation}
              groupingEnabled={viewMode === 'flow' ? flowGroupingEnabled : false}
              hierarchyViewMode={hierarchyViewMode}
              fitViewTrigger={fitViewTrigger}
              wrapperRef={reactFlowWrapperRef}
              nodeSpacing={nodeSpacing}
              rankSpacing={rankSpacing}
              minEdgeLength={minEdgeLength}
              onNodeUpdate={handleNodeUpdate}
              onNodeAdd={handleNodeAdd}
              onNodeDelete={handleNodeDelete}
              onEdgeAdd={handleEdgeAdd}
              onEdgeDelete={handleEdgeDelete}
              onNodeLabelChange={handleNodeLabelChange}
              onEdgeLabelChange={handleEdgeLabelChange}
            />
          </ReactFlowProvider>
        </div>

        {!propertiesPanelCollapsed && (
          <PropertiesAndLayersPanel
            graph={graph}
            selectedNodeId={selectedNodeId}
            onNodeUpdate={handleNodeUpdate}
            viewMode={viewMode}
            onToggleViewMode={handleToggleViewMode}
            orientation={orientation}
            onToggleOrientation={handleToggleOrientation}
            flowGroupingEnabled={flowGroupingEnabled}
            onToggleFlowGrouping={handleToggleFlowGrouping}
            hierarchyViewMode={hierarchyViewMode}
            onToggleHierarchyViewMode={handleToggleHierarchyViewMode}
            nodeSpacing={nodeSpacing}
            onNodeSpacingChange={handleNodeSpacingChange}
            rankSpacing={rankSpacing}
            onRankSpacingChange={handleRankSpacingChange}
            minEdgeLength={minEdgeLength}
            onMinEdgeLengthChange={handleMinEdgeLengthChange}
            datasets={datasets}
            targetNodeDatasetId={targetNodeDatasetId}
            onTargetNodeDatasetChange={setTargetNodeDatasetId}
            targetEdgeDatasetId={targetEdgeDatasetId}
            onTargetEdgeDatasetChange={setTargetEdgeDatasetId}
          />
        )}
        </div>

        <EditHistoryModal
          opened={editHistoryOpen}
          onClose={() => {
            setEditHistoryOpen(false);
            refetchEditCount();
          }}
          graphId={parseInt(graphId || '0')}
          graphName={graph.name}
          onApplyEdits={handleApplyEdits}
        />
      </Stack>
    </TooltipProvider>
  );
};
