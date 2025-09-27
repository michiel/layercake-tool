import React, { useEffect, useState } from 'react';
import { Stack, Select, TextInput, NumberInput, Switch, Textarea, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { TransformNodeConfig } from '../../../../types/plan-dag';

interface TransformNodeConfigFormProps {
  config: TransformNodeConfig;
  setConfig: (config: TransformNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const TransformNodeConfigForm: React.FC<TransformNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<TransformNodeConfig>({
    ...config,
    inputGraphRef: config.inputGraphRef || '',
    outputGraphRef: config.outputGraphRef || '',
    transformType: config.transformType || 'PartitionDepthLimit',
    transformConfig: config.transformConfig || {},
  });

  // Update parent config when local config changes
  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  // Validate configuration
  useEffect(() => {
    const isValid = !!(
      localConfig.inputGraphRef.trim() &&
      localConfig.outputGraphRef.trim() &&
      localConfig.transformType
    );
    setIsValid(isValid);
  }, [localConfig, setIsValid]);

  const handleInputGraphRefChange = (value: string) => {
    setLocalConfig(prev => ({
      ...prev,
      inputGraphRef: value,
    }));
  };

  const handleOutputGraphRefChange = (value: string) => {
    setLocalConfig(prev => ({
      ...prev,
      outputGraphRef: value,
    }));
  };

  const handleTransformTypeChange = (value: string | null) => {
    if (value) {
      setLocalConfig(prev => ({
        ...prev,
        transformType: value as TransformNodeConfig['transformType'],
        // Reset transform config when type changes
        transformConfig: {},
      }));
    }
  };

  const handleTransformConfigChange = (key: string, value: any) => {
    setLocalConfig(prev => ({
      ...prev,
      transformConfig: {
        ...prev.transformConfig,
        [key]: value,
      },
    }));
  };

  const renderTransformSpecificFields = () => {
    switch (localConfig.transformType) {
      case 'PartitionDepthLimit':
        return (
          <Stack gap="sm">
            <NumberInput
              label="Max Partition Depth"
              description="Maximum depth for partitioning the graph"
              placeholder="Enter max depth"
              value={localConfig.transformConfig.maxPartitionDepth || 0}
              onChange={(value) => handleTransformConfigChange('maxPartitionDepth', value)}
              min={1}
              max={20}
            />
            <NumberInput
              label="Max Partition Width"
              description="Maximum width for partitioning the graph"
              placeholder="Enter max width"
              value={localConfig.transformConfig.maxPartitionWidth || 0}
              onChange={(value) => handleTransformConfigChange('maxPartitionWidth', value)}
              min={1}
              max={100}
            />
            <Switch
              label="Generate Hierarchy"
              description="Whether to generate hierarchical partitions"
              checked={localConfig.transformConfig.generateHierarchy || false}
              onChange={(event) => handleTransformConfigChange('generateHierarchy', event.currentTarget.checked)}
            />
          </Stack>
        );

      case 'InvertGraph':
        return (
          <Stack gap="sm">
            <Switch
              label="Invert Graph"
              description="Reverse the direction of all edges in the graph"
              checked={localConfig.transformConfig.invertGraph || false}
              onChange={(event) => handleTransformConfigChange('invertGraph', event.currentTarget.checked)}
            />
            <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Information">
              This transformation will reverse the direction of all edges, effectively inverting the graph structure.
            </Alert>
          </Stack>
        );

      case 'FilterNodes':
        return (
          <Stack gap="sm">
            <Textarea
              label="Node Filter"
              description="Filter expression for nodes (e.g., label.contains('server'))"
              placeholder="Enter node filter expression"
              value={localConfig.transformConfig.nodeFilter || ''}
              onChange={(event) => handleTransformConfigChange('nodeFilter', event.currentTarget.value)}
              rows={3}
            />
            <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Filter Syntax">
              <Text size="sm">
                Use expressions like:
                <br />• <code>label.contains('text')</code>
                <br />• <code>id.startsWith('prefix')</code>
                <br />• <code>metadata.type == 'server'</code>
              </Text>
            </Alert>
          </Stack>
        );

      case 'FilterEdges':
        return (
          <Stack gap="sm">
            <Textarea
              label="Edge Filter"
              description="Filter expression for edges (e.g., label.contains('connects'))"
              placeholder="Enter edge filter expression"
              value={localConfig.transformConfig.edgeFilter || ''}
              onChange={(event) => handleTransformConfigChange('edgeFilter', event.currentTarget.value)}
              rows={3}
            />
            <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Filter Syntax">
              <Text size="sm">
                Use expressions like:
                <br />• <code>label.contains('text')</code>
                <br />• <code>source.startsWith('prefix')</code>
                <br />• <code>metadata.weight {'>'} 5</code>
              </Text>
            </Alert>
          </Stack>
        );

      default:
        return null;
    }
  };

  return (
    <Stack gap="md">
      <TextInput
        label="Input Graph Reference"
        placeholder="Enter the reference to the input graph"
        description="Reference to the graph that will be transformed"
        value={localConfig.inputGraphRef}
        onChange={(event) => handleInputGraphRefChange(event.currentTarget.value)}
        required
      />

      <TextInput
        label="Output Graph Reference"
        placeholder="Enter a reference name for the transformed graph"
        description="This reference will be used by other nodes to connect to the transformed graph"
        value={localConfig.outputGraphRef}
        onChange={(event) => handleOutputGraphRefChange(event.currentTarget.value)}
        required
      />

      <Select
        label="Transform Type"
        placeholder="Choose the type of transformation"
        description="Select how the input graph should be transformed"
        data={[
          { value: 'PartitionDepthLimit', label: 'Partition Depth Limit' },
          { value: 'InvertGraph', label: 'Invert Graph' },
          { value: 'FilterNodes', label: 'Filter Nodes' },
          { value: 'FilterEdges', label: 'Filter Edges' },
        ]}
        value={localConfig.transformType}
        onChange={handleTransformTypeChange}
        required
      />

      {renderTransformSpecificFields()}
    </Stack>
  );
};