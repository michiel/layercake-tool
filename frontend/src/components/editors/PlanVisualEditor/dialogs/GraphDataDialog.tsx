import React from 'react';
import { Modal, Stack, Alert, Loader, Text } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import { GET_GRAPH_DETAILS, Graph, UPDATE_GRAPH_NODE, UPDATE_LAYER_PROPERTIES } from '../../../../graphql/graphs';
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

  const [updateGraphNode] = useMutation(UPDATE_GRAPH_NODE);
  const [updateLayerProperties] = useMutation(UPDATE_LAYER_PROPERTIES);

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
        background_color: layer.properties?.background_color,
        text_color: layer.properties?.text_color,
        border_color: layer.properties?.border_color,
        ...layer.properties
      }))
    };
  };

  const handleSave = async (newGraphData: GraphData) => {
    if (!data?.graph || !graphId) return;

    try {
      const oldGraph = data.graph;

      // Update changed nodes
      const promises: Promise<any>[] = [];

      // Helper to normalize values for comparison (treats empty string and undefined as same)
      const normalizeValue = (val: any) => {
        if (val === '' || val === null || val === undefined) return undefined;
        return val;
      };

      for (const newNode of newGraphData.nodes) {
        const oldNode = oldGraph.graphNodes.find(n => n.id === newNode.id);
        if (!oldNode) continue;

        // Normalize values for comparison
        const oldLabel = normalizeValue(oldNode.label);
        const newLabel = normalizeValue(newNode.label);
        const oldLayer = normalizeValue(oldNode.layer);
        const newLayer = normalizeValue(newNode.layer);

        // Check if fields actually changed
        const labelChanged = oldLabel !== newLabel;
        const layerChanged = oldLayer !== newLayer;

        // Build attrs object excluding standard fields
        const { id, label, layer, is_partition, belongs_to, comment, ...customAttrs } = newNode;

        // Remove empty/undefined values from custom attrs
        const cleanedCustomAttrs: Record<string, any> = {};
        for (const [key, value] of Object.entries(customAttrs)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedCustomAttrs[key] = normalized;
          }
        }

        const oldAttrs = oldNode.attrs || {};
        const attrsChanged = JSON.stringify(cleanedCustomAttrs) !== JSON.stringify(oldAttrs);

        if (labelChanged || layerChanged || attrsChanged) {
          console.log(`Updating node ${newNode.id}:`, { labelChanged, layerChanged, attrsChanged });
          promises.push(
            updateGraphNode({
              variables: {
                graphId: graphId,
                nodeId: newNode.id,
                label: labelChanged ? newLabel : undefined,
                layer: layerChanged ? newLayer : undefined,
                attrs: attrsChanged ? cleanedCustomAttrs : undefined,
              }
            })
          );
        }
      }

      // Update changed layers
      for (const newLayer of newGraphData.layers) {
        const oldLayer = oldGraph.layers.find(l => l.layerId === newLayer.id);
        if (!oldLayer) continue;

        // Normalize values for comparison
        const oldName = normalizeValue(oldLayer.name);
        const newName = normalizeValue(newLayer.label);
        const nameChanged = oldName !== newName;

        // Build properties object, excluding id and label
        const { id, label, ...properties } = newLayer;

        // Clean properties - remove empty/undefined values
        const cleanedProperties: Record<string, any> = {};
        for (const [key, value] of Object.entries(properties)) {
          const normalized = normalizeValue(value);
          if (normalized !== undefined) {
            cleanedProperties[key] = normalized;
          }
        }

        const oldProperties = oldLayer.properties || {};
        const propertiesChanged = JSON.stringify(cleanedProperties) !== JSON.stringify(oldProperties);

        if (nameChanged || propertiesChanged) {
          console.log(`Updating layer ${newLayer.id}:`, { nameChanged, propertiesChanged });
          const layerDbId = oldLayer.id;
          promises.push(
            updateLayerProperties({
              variables: {
                id: layerDbId,
                name: nameChanged ? newName : undefined,
                properties: propertiesChanged ? cleanedProperties : undefined,
              }
            })
          );
        }
      }

      if (promises.length === 0) {
        console.log('No changes detected');
        return;
      }

      console.log(`Saving ${promises.length} changes...`);

      // Wait for all updates to complete
      await Promise.all(promises);

      // Refetch the graph data to show updated values
      await refetch();

      console.log(`Updated ${promises.length} items successfully`);
    } catch (error) {
      console.error('Failed to save graph data:', error);
      throw error;
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={title}
      size="90%"
      styles={{
        body: { padding: 0 },
        content: { maxHeight: '90vh' }
      }}
    >
      <Stack p="md" gap="md">
        {loading && (
          <Stack align="center" py="xl">
            <Loader />
            <Text c="dimmed">Loading graph data...</Text>
          </Stack>
        )}

        {error && (
          <Alert
            icon={<IconAlertCircle size={16} />}
            title="Error Loading Graph Data"
            color="red"
          >
            {error.message}
          </Alert>
        )}

        {data?.graph && (() => {
          const graphData = getGraphData();
          if (!graphData) {
            return (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Invalid Graph Data"
                color="red"
              >
                Failed to load graph data
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
    </Modal>
  );
};
