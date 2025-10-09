import React from 'react';
import { Modal, Stack, Alert, Loader, Text } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery } from '@apollo/client/react';
import { GET_GRAPH_DETAILS, Graph } from '../../../../graphql/graphs';
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
  const { data, loading, error } = useQuery<{ graph: Graph }>(GET_GRAPH_DETAILS, {
    variables: { id: graphId },
    skip: !opened || !graphId,
    fetchPolicy: 'network-only'
  });

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

  const handleSave = async (_graphData: GraphData) => {
    // Read-only for now - graph nodes are generated from upstream sources
    // If we want to support editing, we would need to implement a mutation here
    console.log('Save not implemented for graph nodes (read-only)');
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
              readOnly={true}
            />
          );
        })()}
      </Stack>
    </Modal>
  );
};
