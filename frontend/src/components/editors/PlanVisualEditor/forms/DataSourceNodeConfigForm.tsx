import React, { useEffect, useState } from 'react';
import { Stack, Select, Loader, Alert, Text } from '@mantine/core';
import { IconAlertCircle } from '@tabler/icons-react';
import { useQuery } from '@apollo/client/react';
import { gql } from '@apollo/client';
import { DataSourceNodeConfig, NodeMetadata } from '../../../../types/plan-dag';

// GraphQL query for available data sources
const GET_AVAILABLE_DATA_SOURCES = gql`
  query GetAvailableDataSources($projectId: Int!) {
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

interface DataSourceReference {
  id: number;
  name: string;
  description?: string;
  fileFormat: string;
  dataType: string;
  createdAt: string;
}

interface GetAvailableDataSourcesData {
  dataSources: DataSourceReference[];
}

interface DataSourceNodeConfigFormProps {
  config: DataSourceNodeConfig;
  setConfig: (config: DataSourceNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
  metadata: NodeMetadata;
  setMetadata: React.Dispatch<React.SetStateAction<NodeMetadata>>;
}

export const DataSourceNodeConfigForm: React.FC<DataSourceNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId,
  metadata: _metadata,
  setMetadata,
}) => {
  const [localConfig, setLocalConfig] = useState<DataSourceNodeConfig>({
    ...config,
  });

  const { data, loading, error } = useQuery<GetAvailableDataSourcesData>(GET_AVAILABLE_DATA_SOURCES, {
    variables: { projectId },
    skip: !projectId,
  });

  // Update parent config when local config changes
  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  // Validate configuration
  useEffect(() => {
    const isValid = !!localConfig.dataSourceId;
    setIsValid(isValid);
  }, [localConfig.dataSourceId, setIsValid]);

  const handleDataSourceChange = (value: string | null) => {
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

  const handleDisplayModeChange = (value: string | null) => {
    setLocalConfig(prev => ({
      ...prev,
      displayMode: value as 'summary' | 'detailed' | 'preview' | undefined,
    }));
  };

  if (loading) {
    return (
      <Stack align="center" p="md">
        <Loader size="sm" />
        <Text size="sm" c="dimmed">Loading data sources...</Text>
      </Stack>
    );
  }

  if (error) {
    return (
      <Alert icon={<IconAlertCircle size="1rem" />} color="red" title="Error">
        Failed to load data sources: {error.message}
      </Alert>
    );
  }

  const dataSourceOptions = data?.dataSources?.map((ds: DataSourceReference) => ({
    value: ds.id.toString(),
    label: ds.name,
    description: ds.description || `Type: ${ds.fileFormat}/${ds.dataType}`,
  })) || [];

  const selectedDataSource = data?.dataSources?.find(
    (ds: DataSourceReference) => ds.id === localConfig.dataSourceId
  );

  return (
    <Stack gap="md">
      <Select
        label="Data Source"
        placeholder="Select a data source"
        description="Choose an existing data source to reference in this plan"
        data={dataSourceOptions}
        value={localConfig.dataSourceId?.toString() || null}
        onChange={handleDataSourceChange}
        required
        searchable
        clearable
        maxDropdownHeight={200}
      />

      {selectedDataSource && (
        <Alert color="blue" radius="md" p="sm">
          <Stack gap="xs">
            <Text size="sm" fw={500}>Selected Data Source Details:</Text>
            <Text size="xs" c="dimmed">
              <strong>Type:</strong> {selectedDataSource.fileFormat}/{selectedDataSource.dataType}
            </Text>
            {selectedDataSource.description && (
              <Text size="xs" c="dimmed">
                <strong>Description:</strong> {selectedDataSource.description}
              </Text>
            )}
            <Text size="xs" c="dimmed">
              <strong>Created:</strong> {new Date(selectedDataSource.createdAt).toLocaleDateString()}
            </Text>
          </Stack>
        </Alert>
      )}

      <Select
        label="Display Mode"
        placeholder="Choose display mode"
        description="How this data source should be displayed in the plan view"
        data={[
          { value: 'summary', label: 'Summary' },
          { value: 'detailed', label: 'Detailed' },
          { value: 'preview', label: 'Preview' },
        ]}
        value={localConfig.displayMode || 'summary'}
        onChange={handleDisplayModeChange}
      />
    </Stack>
  );
};
