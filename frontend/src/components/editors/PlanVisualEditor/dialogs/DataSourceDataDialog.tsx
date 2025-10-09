import React from 'react';
import { Modal, Stack, Alert, Loader, Text } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery } from '@apollo/client/react';
import { GET_DATASOURCE, DataSource } from '../../../../graphql/datasources';
import { GraphSpreadsheetEditor, GraphData } from '../../../editors/GraphSpreadsheetEditor/GraphSpreadsheetEditor';

interface DataSourceDataDialogProps {
  opened: boolean;
  onClose: () => void;
  dataSourceId: number | null;
  title?: string;
}

export const DataSourceDataDialog: React.FC<DataSourceDataDialogProps> = ({
  opened,
  onClose,
  dataSourceId,
  title = 'Data Source Data'
}) => {
  const { data, loading, error } = useQuery<{ dataSource: DataSource }>(GET_DATASOURCE, {
    variables: { id: dataSourceId },
    skip: !opened || !dataSourceId,
    fetchPolicy: 'network-only'
  });

  const getGraphData = (): GraphData | null => {
    if (!data?.dataSource) return null;

    try {
      const graphJson = JSON.parse(data.dataSource.graphJson);

      return {
        nodes: (graphJson.nodes || []).map((node: any) => ({
          id: node.id,
          label: node.label || '',
          layer: node.layer,
          is_partition: node.is_partition,
          belongs_to: node.belongs_to,
          ...node
        })),
        edges: (graphJson.edges || []).map((edge: any) => ({
          id: edge.id,
          source: edge.source,
          target: edge.target,
          label: edge.label,
          layer: edge.layer,
          ...edge
        })),
        layers: (graphJson.layers || []).map((layer: any) => ({
          id: layer.id,
          label: layer.label,
          background_color: layer.background_color,
          text_color: layer.text_color,
          border_color: layer.border_color,
          ...layer
        }))
      };
    } catch (err) {
      console.error('Failed to parse graph JSON:', err);
      return null;
    }
  };

  const handleSave = async (graphData: GraphData) => {
    // Update graph JSON for datasource
    // This would require a mutation to update the datasource's graphJson field
    console.log('Save datasource data:', graphData);
    // TODO: Implement UPDATE_GRAPH_JSON mutation call here
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
            <Text c="dimmed">Loading datasource data...</Text>
          </Stack>
        )}

        {error && (
          <Alert
            icon={<IconAlertCircle size={16} />}
            title="Error Loading Data Source"
            color="red"
          >
            {error.message}
          </Alert>
        )}

        {data?.dataSource && (() => {
          const graphData = getGraphData();
          if (!graphData) {
            return (
              <Alert
                icon={<IconAlertCircle size={16} />}
                title="Invalid Data"
                color="red"
              >
                Failed to parse datasource graph JSON data
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
