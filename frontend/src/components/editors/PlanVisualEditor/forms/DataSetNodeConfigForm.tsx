import React, { useEffect, useState } from 'react';
import { IconAlertCircle, IconLoader2 } from '@tabler/icons-react';
import { useQuery } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { DataSetNodeConfig, NodeMetadata } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

// GraphQL query for available data sources
const GET_AVAILABLE_DATA_SOURCES = gql`
  query GetAvailableDataSets($projectId: Int!) {
    dataSources(projectId: $projectId) {
      id
      name
      description
      fileFormat
      dataType
      createdAt
    }
  }
`;

interface DataSetReference {
  id: number;
  name: string;
  description?: string;
  fileFormat: string;
  dataType: string;
  createdAt: string;
}

interface GetAvailableDataSetsData {
  dataSources: DataSetReference[];
}

interface DataSetNodeConfigFormProps {
  config: DataSetNodeConfig;
  setConfig: (config: DataSetNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
  metadata: NodeMetadata;
  setMetadata: React.Dispatch<React.SetStateAction<NodeMetadata>>;
}

export const DataSetNodeConfigForm: React.FC<DataSetNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId,
  metadata: _metadata,
  setMetadata,
}) => {
  const [localConfig, setLocalConfig] = useState<DataSetNodeConfig>({
    ...config,
  });
  const lastSentConfigRef = React.useRef<DataSetNodeConfig>(localConfig);

  const { data, loading, error } = useQuery<GetAvailableDataSetsData>(GET_AVAILABLE_DATA_SOURCES, {
    variables: { projectId },
    skip: !projectId,
  });

  // Update parent config when local config changes (but avoid loops)
  useEffect(() => {
    if (JSON.stringify(localConfig) !== JSON.stringify(lastSentConfigRef.current)) {
      setConfig(localConfig);
      lastSentConfigRef.current = localConfig;
    }
  }, [localConfig, setConfig]);

  // Validate configuration
  useEffect(() => {
    const isValid = !!localConfig.dataSourceId;
    setIsValid(isValid);
  }, [localConfig.dataSourceId, setIsValid]);

  const handleDataSetChange = (value: string | undefined) => {
    if (value) {
      const dataSourceId = parseInt(value, 10);
      const newSelection = data?.dataSources?.find(ds => ds.id === dataSourceId);

      setLocalConfig(prev => ({
        ...prev,
        dataSourceId,
      }));

      if (newSelection) {
        setMetadata(prev => ({
          ...prev,
          label: newSelection.name,
        }));
      }
    } else {
      setLocalConfig(prev => ({
        ...prev,
        dataSourceId: undefined,
      }));
      setMetadata(prev => ({
        ...prev,
        label: '',
      }));
    }
  };

  const handleDisplayModeChange = (value: string | undefined) => {
    setLocalConfig(prev => ({
      ...prev,
      displayMode: value as 'summary' | 'detailed' | 'preview' | undefined,
    }));
  };

  if (loading) {
    return (
      <Stack gap="md" className="items-center py-4">
        <IconLoader2 className="h-4 w-4 animate-spin" />
        <p className="text-sm text-muted-foreground">Loading data sources...</p>
      </Stack>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <IconAlertCircle className="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to load data sources: {error.message}
        </AlertDescription>
      </Alert>
    );
  }

  const dataSourceOptions = data?.dataSources?.map((ds: DataSetReference) => ({
    value: ds.id.toString(),
    label: ds.name,
    description: ds.description || `Type: ${ds.fileFormat}/${ds.dataType}`,
  })) || [];

  const selectedDataSet = data?.dataSources?.find(
    (ds: DataSetReference) => ds.id === localConfig.dataSourceId
  );

  return (
    <Stack gap="md">
      <div className="space-y-2">
        <Label htmlFor="data-source">
          Data Source <span className="text-red-600">*</span>
        </Label>
        <Select
          value={localConfig.dataSourceId?.toString() || undefined}
          onValueChange={handleDataSetChange}
        >
          <SelectTrigger id="data-source">
            <SelectValue placeholder="Select a data source" />
          </SelectTrigger>
          <SelectContent className="max-h-[200px]">
            {dataSourceOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                <div>
                  <div>{option.label}</div>
                  <div className="text-xs text-muted-foreground">{option.description}</div>
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <p className="text-sm text-muted-foreground">
          Choose an existing data source to reference in this plan
        </p>
      </div>

      {selectedDataSet && (
        <Alert>
          <AlertTitle>Selected Data Source Details</AlertTitle>
          <AlertDescription>
            <div className="space-y-1 text-xs">
              <div>
                <strong>Type:</strong> {selectedDataSet.fileFormat}/{selectedDataSet.dataType}
              </div>
              {selectedDataSet.description && (
                <div>
                  <strong>Description:</strong> {selectedDataSet.description}
                </div>
              )}
              <div>
                <strong>Created:</strong> {new Date(selectedDataSet.createdAt).toLocaleDateString()}
              </div>
            </div>
          </AlertDescription>
        </Alert>
      )}

      <div className="space-y-2">
        <Label htmlFor="display-mode">Display Mode</Label>
        <Select
          value={localConfig.displayMode || 'summary'}
          onValueChange={handleDisplayModeChange}
        >
          <SelectTrigger id="display-mode">
            <SelectValue placeholder="Choose display mode" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="summary">Summary</SelectItem>
            <SelectItem value="detailed">Detailed</SelectItem>
            <SelectItem value="preview">Preview</SelectItem>
          </SelectContent>
        </Select>
        <p className="text-sm text-muted-foreground">
          How this data source should be displayed in the plan view
        </p>
      </div>
    </Stack>
  );
};
