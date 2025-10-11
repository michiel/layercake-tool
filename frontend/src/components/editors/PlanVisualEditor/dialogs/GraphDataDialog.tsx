import React from 'react';
import { Modal, Stack, Alert, Loader, Text } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery, useMutation } from '@apollo/client/react';
import { GET_GRAPH_DETAILS, Graph, BULK_UPDATE_GRAPH_DATA } from '../../../../graphql/graphs';
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

      // Helper to normalize values for comparison (treats empty string and undefined as same)
      const normalizeValue = (val: any) => {
        if (val === '' || val === null || val === undefined) return undefined;
        return val;
      };

      const changedNodes: any[] = [];
      const changedLayers: any[] = [];

      // Collect changed nodes
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
          changedNodes.push({
            nodeId: newNode.id,
            label: labelChanged ? newLabel : null,
            layer: layerChanged ? newLayer : null,
            attrs: attrsChanged ? cleanedCustomAttrs : null,
          });
        }
      }

      // Collect changed layers
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
          changedLayers.push({
            id: oldLayer.id,
            name: nameChanged ? newName : null,
            properties: propertiesChanged ? cleanedProperties : null,
          });
        }
      }

      if (changedNodes.length === 0 && changedLayers.length === 0) {
        console.log('No changes detected');
        return;
      }

      console.log(`Saving ${changedNodes.length} node(s) and ${changedLayers.length} layer(s) in bulk...`);

      // Single bulk update call
      await bulkUpdateGraphData({
        variables: {
          graphId: graphId,
          nodes: changedNodes.length > 0 ? changedNodes : null,
          layers: changedLayers.length > 0 ? changedLayers : null,
        }
      });

      // Refetch the graph data to show updated values
      await refetch();

      console.log(`Updated successfully`);
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
