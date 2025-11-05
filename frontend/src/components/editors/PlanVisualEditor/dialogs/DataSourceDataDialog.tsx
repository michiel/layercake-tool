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
import { GET_DATASOURCE, DataSource, UPDATE_DATASOURCE_GRAPH_DATA } from '../../../../graphql/datasources';
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
  const { data, loading, error, refetch } = useQuery<{ dataSource: DataSource }>(GET_DATASOURCE, {
    variables: { id: dataSourceId },
    skip: !opened || !dataSourceId,
    fetchPolicy: 'network-only'
  });

  const [updateDataSourceGraphData] = useMutation(UPDATE_DATASOURCE_GRAPH_DATA);

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
    if (!dataSourceId) return;

    try {
      // Convert GraphData back to the format expected by the backend
      const graphJson = JSON.stringify({
        nodes: graphData.nodes,
        edges: graphData.edges,
        layers: graphData.layers
      });

      await updateDataSourceGraphData({
        variables: {
          id: dataSourceId,
          graphJson
        }
      });

      // Refetch to show updated data
      await refetch();

      console.log('Datasource data saved successfully');
    } catch (error) {
      console.error('Failed to save datasource data:', error);
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
                <p className="text-sm text-muted-foreground">Loading datasource data...</p>
              </Stack>
            )}

            {error && (
              <Alert variant="destructive">
                <IconAlertCircle className="h-4 w-4" />
                <AlertTitle>Error Loading Data Source</AlertTitle>
                <AlertDescription>{error.message}</AlertDescription>
              </Alert>
            )}

            {data?.dataSource && (() => {
              const graphData = getGraphData();
              if (!graphData) {
                return (
                  <Alert variant="destructive">
                    <IconAlertCircle className="h-4 w-4" />
                    <AlertTitle>Invalid Data</AlertTitle>
                    <AlertDescription>Failed to parse datasource graph JSON data</AlertDescription>
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
