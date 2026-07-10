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
  DELETE_GRAPH_EDGE
} from '../../../../graphql/graphs';
import { GraphSpreadsheetEditor, GraphData } from '../../../editors/GraphSpreadsheetEditor/GraphSpreadsheetEditor';
import { sanitizeAttributes } from '@/utils/attributes';

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

  const getGraphData = (): GraphData | null => {
    if (!data?.graph) return null;

    const normalizeAttributes = (attrs?: any) => sanitizeAttributes(attrs);

    return {
      nodes: data.graph.graphNodes.map(node => ({
        id: node.id,
        label: node.label || '',
        layer: node.layer,
        weight: node.weight,
        is_partition: node.isPartition,
        belongs_to: node.belongsTo,
        attributes: normalizeAttributes((node as any).attributes ?? node.attrs),
      })),
      edges: data.graph.graphEdges.map(edge => ({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        label: edge.label,
        layer: edge.layer,
        weight: edge.weight,
        attributes: normalizeAttributes((edge as any).attributes ?? edge.attrs),
      })),
      layers: data.graph.layers.map(layer => ({
        id: layer.layerId,
        label: layer.name,
        alias: layer.alias ?? '',
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

        const newAttributes = sanitizeAttributes(newNode.attributes);
        const oldAttrs = sanitizeAttributes((oldNode as any).attributes ?? oldNode.attrs);
        const attrsChanged = JSON.stringify(newAttributes) !== JSON.stringify(oldAttrs);

        if (oldLabel !== newLabel || oldLayer !== newLayer || attrsChanged) {
          updatedNodes.push({
            nodeId: newNode.id,
            label: oldLabel !== newLabel ? newLabel : null,
            layer: oldLayer !== newLayer ? newLayer : null,
            attributes: attrsChanged ? newAttributes : null,
          });
        }
      }

      // Identify added edges
      const addedEdges = newGraphData.edges.filter(e => !oldEdgeIds.has(e.id));

      // Identify deleted edges
      const deletedEdgeIds = Array.from(oldEdgeIds).filter(id => !newEdgeIds.has(id));

      // Layers are read-only in this dialog (project-scoped layers are edited via
      // the project layer palette), so no layer create/update is performed here.

      console.log(`Changes: ${addedNodes.length} new nodes, ${deletedNodeIds.length} deleted nodes, ${updatedNodes.length} updated nodes`);
      console.log(`Changes: ${addedEdges.length} new edges, ${deletedEdgeIds.length} deleted edges`);

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
        const { id, label, layer, is_partition, belongs_to } = node;
        const cleanedAttrs = sanitizeAttributes(node.attributes);

        await addGraphNode({
          variables: {
            graphId: parseInt(String(graphId)),
            id,
            label: label || id,
            layer,
            isPartition: is_partition || false,
            belongsTo: belongs_to,
            attributes: Object.keys(cleanedAttrs).length > 0 ? cleanedAttrs : null,
          }
        });
      }

      // Add new edges
      for (const edge of addedEdges) {
        const { id, source, target, label, layer } = edge;
        const cleanedAttrs = sanitizeAttributes(edge.attributes);

        await addGraphEdge({
          variables: {
            graphId: parseInt(String(graphId)),
            id,
            source,
            target,
            label: label || '',
            layer,
            attributes: Object.keys(cleanedAttrs).length > 0 ? cleanedAttrs : null,
          }
        });
      }

      // Update existing nodes in bulk if there are any
      if (updatedNodes.length > 0) {
        await bulkUpdateGraphData({
          variables: {
            graphId: parseInt(String(graphId)),
            nodes: updatedNodes,
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
                  layersReadOnly
                />
              );
            })()}
          </Stack>
        </div>
      </DialogContent>
    </Dialog>
  );
};
