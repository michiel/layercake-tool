import React from 'react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Spinner } from '@/components/ui/spinner';
import { Stack } from '@/components/layout-primitives';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import {
  GET_GRAPH_DETAILS,
  Graph,
  BULK_UPDATE_GRAPH_DATA,
  ADD_GRAPH_NODE,
  ADD_GRAPH_EDGE,
  DELETE_GRAPH_NODE,
  DELETE_GRAPH_EDGE,
  CREATE_LAYER
} from '../../../../graphql/graphs';
import { GraphSpreadsheetEditor, GraphData } from '../../../editors/GraphSpreadsheetEditor/GraphSpreadsheetEditor';

interface GraphDataDialogProps {
  opened: boolean;
  onClose: () => void;
  graphId: number | null;
  title?: string;
}

export const GraphDataDialog: React.FC<GraphDataDialogProps> = ({
  opened,
  onClose,
  graphId,
  title = 'Graph Data'
}) => {
  const { data, loading, error, refetch } = useQuery<{ graph: Graph }>(GET_GRAPH_DETAILS, {
    variables: { id: graphId },
    skip: !opened || !graphId,
    fetchPolicy: 'network-only'
  });

  const [bulkUpdateGraphData] = useMutation(BULK_UPDATE_GRAPH_DATA);
  const [addGraphNode] = useMutation(ADD_GRAPH_NODE);
  const [addGraphEdge] = useMutation(ADD_GRAPH_EDGE);
  const [deleteGraphNode] = useMutation(DELETE_GRAPH_NODE);
  const [deleteGraphEdge] = useMutation(DELETE_GRAPH_EDGE);
  const [createLayer] = useMutation(CREATE_LAYER);

  const getGraphData = (): GraphData | null => {
    if (!data?.graph) return null;

    return {
      nodes: data.graph.graphNodes.map(node => ({
        id: node.id,
        label: node.label || '',
        layer: node.layer,
        is_partition: node.isPartition,
        belongs_to: node.belongsTo,
        ...node.attrs
      })),
      edges: data.graph.graphEdges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label,
        layer: edge.layer,
        ...edge.attrs
      })),
      layers: data.graph.layers.map(layer => ({
        id: layer.layerId,
        label: layer.name,
        background_color: layer.backgroundColor,
        text_color: layer.textColor,
        border_color: layer.borderColor,
        comment: layer.comment,
        ...layer.properties
      }))
    };
  };

  const handleSave = async (newGraphData: GraphData) => {
    if (!data?.graph || !graphId) return;

    try {
      const oldGraph = data.graph;

      // Helper to normalize values for comparison
      const normalizeValue = (val: any) => {
        if (val === '' || val === null || val === undefined) return undefined;
        return val;
      };

      const oldNodeIds = new Set(oldGraph.graphNodes.map(n => n.id));
      const newNodeIds = new Set(newGraphData.nodes.map(n => n.id));
      const oldEdgeIds = new Set(oldGraph.graphEdges.map(e => e.id));
      const newEdgeIds = new Set(newGraphData.edges.map(e => e.id));
      const oldLayerIds = new Set(oldGraph.layers.map(l => l.layerId));

      // Identify added nodes
      const addedNodes = newGraphData.nodes.filter(n => !oldNodeIds.has(n.id));

      // Identify deleted nodes
      const deletedNodeIds = Array.from(oldNodeIds).filter(id => !newNodeIds.has(id));

      // Identify updated nodes
      const updatedNodes: any[] = [];
      for (const newNode of newGraphData.nodes) {
        const oldNode = oldGraph.graphNodes.find(n => n.id === newNode.id);
        if (!oldNode) continue;

        const oldLabel = normalizeValue(oldNode.label);
        const newLabel = normalizeValue(newNode.label);
        const oldLayer = normalizeValue(oldNode.layer);
        const newLayer = normalizeValue(newNode.layer);

        const { id, label, layer, is_partition, belongs_to, comment, ...customAttrs } = newNode;
        const cleanedCustomAttrs: Record<string, any> = {};
        for (const [key, value] of Object.entries(customAttrs)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedCustomAttrs[key] = normalized;
          }
        }

        const oldAttrs = oldNode.attrs || {};
        const attrsChanged = JSON.stringify(cleanedCustomAttrs) !== JSON.stringify(oldAttrs);

        if (oldLabel !== newLabel || oldLayer !== newLayer || attrsChanged) {
          updatedNodes.push({
            nodeId: newNode.id,
            label: oldLabel !== newLabel ? newLabel : null,
            layer: oldLayer !== newLayer ? newLayer : null,
            attrs: attrsChanged ? cleanedCustomAttrs : null,
          });
        }
      }

      // Identify added edges
      const addedEdges = newGraphData.edges.filter(e => !oldEdgeIds.has(e.id));

      // Identify deleted edges
      const deletedEdgeIds = Array.from(oldEdgeIds).filter(id => !newEdgeIds.has(id));

      // Identify added layers (will create them)
      const addedLayers = newGraphData.layers.filter(l => !oldLayerIds.has(l.id));

      // Identify updated layers
      const updatedLayers: any[] = [];
      for (const newLayer of newGraphData.layers) {
        const oldLayer = oldGraph.layers.find(l => l.layerId === newLayer.id);
        if (!oldLayer) continue;

        const oldName = normalizeValue(oldLayer.name);
        const newName = normalizeValue(newLayer.label);

        const { id, label, ...properties } = newLayer;
        const cleanedProperties: Record<string, any> = {};
        for (const [key, value] of Object.entries(properties)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedProperties[key] = normalized;
          }
        }

        const oldProperties = oldLayer.properties || {};
        const propertiesChanged = JSON.stringify(cleanedProperties) !== JSON.stringify(oldProperties);

        if (oldName !== newName || propertiesChanged) {
          updatedLayers.push({
            id: oldLayer.id,
            name: oldName !== newName ? newName : null,
            properties: propertiesChanged ? cleanedProperties : null,
          });
        }
      }

      console.log(`Changes: ${addedNodes.length} new nodes, ${deletedNodeIds.length} deleted nodes, ${updatedNodes.length} updated nodes`);
      console.log(`Changes: ${addedEdges.length} new edges, ${deletedEdgeIds.length} deleted edges`);
      console.log(`Changes: ${addedLayers.length} new layers, ${updatedLayers.length} updated layers`);

      // Delete nodes first (to avoid constraint issues)
      for (const nodeId of deletedNodeIds) {
        await deleteGraphNode({
          variables: { graphId: parseInt(String(graphId)), nodeId }
        });
      }

      // Delete edges
      for (const edgeId of deletedEdgeIds) {
        await deleteGraphEdge({
          variables: { graphId: parseInt(String(graphId)), edgeId }
        });
      }

      // Add new nodes
      for (const node of addedNodes) {
        const { id, label, layer, is_partition, belongs_to, comment, ...customAttrs } = node;
        const cleanedAttrs: Record<string, any> = {};
        for (const [key, value] of Object.entries(customAttrs)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedAttrs[key] = normalized;
          }
        }

        await addGraphNode({
          variables: {
            graphId: parseInt(String(graphId)),
            id,
            label: label || id,
            layer,
            isPartition: is_partition || false,
            belongsTo: belongs_to,
            attrs: Object.keys(cleanedAttrs).length > 0 ? cleanedAttrs : null,
          }
        });
      }

      // Add new edges
      for (const edge of addedEdges) {
        const { id, source, target, label, layer, comment, ...customAttrs } = edge;
        const cleanedAttrs: Record<string, any> = {};
        for (const [key, value] of Object.entries(customAttrs)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedAttrs[key] = normalized;
          }
        }

        await addGraphEdge({
          variables: {
            graphId: parseInt(String(graphId)),
            id,
            source,
            target,
            label: label || '',
            layer,
            attrs: Object.keys(cleanedAttrs).length > 0 ? cleanedAttrs : null,
          }
        });
      }

      // Add new layers
      for (const layer of addedLayers) {
        const { id, label, ...properties } = layer;
        const cleanedProperties: Record<string, any> = {};
        for (const [key, value] of Object.entries(properties)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedProperties[key] = normalized;
          }
        }

        await createLayer({
          variables: {
            input: {
              graphId: parseInt(String(graphId)),
              layerId: id,
              name: label || id,
              properties: Object.keys(cleanedProperties).length > 0 ? cleanedProperties : null,
            }
          }
        });
      }

      // Update existing nodes/layers in bulk if there are any
      if (updatedNodes.length > 0 || updatedLayers.length > 0) {
        await bulkUpdateGraphData({
          variables: {
            graphId: parseInt(String(graphId)),
            nodes: updatedNodes.length > 0 ? updatedNodes : null,
            layers: updatedLayers.length > 0 ? updatedLayers : null,
          }
        });
      }

      // Refetch the graph data to show updated values
      await refetch();

      console.log(`Save completed successfully`);
    } catch (error) {
      console.error('Failed to save graph data:', error);
      throw error;
    }
  };

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-[90vw] max-h-[90vh] p-0 flex flex-col">
        <DialogHeader className="px-6 py-4">
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-hidden px-6 pb-4">
          <Stack gap="md">
            {loading && (
              <Stack align="center" className="py-12">
                <Spinner size="lg" />
                <p className="text-sm text-muted-foreground">Loading graph data...</p>
              </Stack>
            )}

            {error && (
              <Alert variant="destructive">
                <IconAlertCircle className="h-4 w-4" />
                <AlertTitle>Error Loading Graph Data</AlertTitle>
                <AlertDescription>{error.message}</AlertDescription>
              </Alert>
            )}

            {data?.graph && (() => {
              const graphData = getGraphData();
              if (!graphData) {
                return (
                  <Alert variant="destructive">
                    <IconAlertCircle className="h-4 w-4" />
                    <AlertTitle>Invalid Graph Data</AlertTitle>
                    <AlertDescription>Failed to load graph data</AlertDescription>
                  </Alert>
                );
              }

              return (
                <GraphSpreadsheetEditor
                  graphData={graphData}
                  onSave={handleSave}
                  readOnly={false}
                />
              );
            })()}
          </Stack>
        </div>
      </DialogContent>
    </Dialog>
  );
};
