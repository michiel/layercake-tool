import React from 'react';
import { Modal, Button, Group, Text } from '@mantine/core';
import { PlanDagNodeType } from '../../../types/plan-dag';
import { DataSourceNodeConfigForm } from './forms/DataSourceNodeConfigForm';
import { TransformNodeConfigForm } from './forms/TransformNodeConfigForm';
import { MergeNodeConfigForm } from './forms/MergeNodeConfigForm';
import { OutputNodeConfigForm } from './forms/OutputNodeConfigForm';
import { GraphNodeConfigForm } from './forms/GraphNodeConfigForm';
import { CopyNodeConfigForm } from './forms/CopyNodeConfigForm';

interface NodeConfigDialogProps {
  opened: boolean;
  onClose: () => void;
  nodeType: PlanDagNodeType;
  currentConfig?: string; // JSON string
  projectId: number;
  onSave: (nodeId: string, config: any, metadata: any) => void;
  nodeId: string;
  config?: any;
  metadata?: any;
}

export const NodeConfigDialog: React.FC<NodeConfigDialogProps> = ({
  opened,
  onClose,
  nodeType,
  currentConfig,
  projectId,
  onSave,
  nodeId,
}) => {
  const [config, setConfig] = React.useState<any>({});
  const [isValid, setIsValid] = React.useState(false);

  // Parse current config when dialog opens
  React.useEffect(() => {
    if (currentConfig) {
      try {
        const parsed = JSON.parse(currentConfig);
        setConfig(parsed);
      } catch (e) {
        console.error('Failed to parse current config:', e);
        setConfig({});
      }
    } else {
      setConfig({});
    }
  }, [currentConfig]);

  const handleSave = () => {
    if (isValid) {
      // Create dummy metadata for now
      const metadata = { label: `${getNodeTypeName()} Node`, description: undefined };
      onSave(nodeId, config, metadata);
      onClose();
    }
  };

  const renderConfigForm = () => {
    const commonProps = {
      config,
      setConfig,
      setIsValid,
      projectId,
    };

    switch (nodeType) {
      case PlanDagNodeType.DATA_SOURCE:
        return <DataSourceNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.GRAPH:
        return <GraphNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.TRANSFORM:
        return <TransformNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.MERGE:
        return <MergeNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.COPY:
        return <CopyNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.OUTPUT:
        return <OutputNodeConfigForm {...commonProps} />;
      default:
        return <Text color="red">Unknown node type: {nodeType}</Text>;
    }
  };

  const getNodeTypeName = () => {
    switch (nodeType) {
      case PlanDagNodeType.DATA_SOURCE:
        return 'Data Source';
      case PlanDagNodeType.GRAPH:
        return 'Graph';
      case PlanDagNodeType.TRANSFORM:
        return 'Transform';
      case PlanDagNodeType.MERGE:
        return 'Merge';
      case PlanDagNodeType.COPY:
        return 'Copy';
      case PlanDagNodeType.OUTPUT:
        return 'Output';
      default:
        return 'Unknown';
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={`Configure ${getNodeTypeName()} Node`}
      size="lg"
      closeOnClickOutside={false}
      closeOnEscape={false}
    >
      {renderConfigForm()}

      <Group mt="xl" justify="flex-end">
        <Button variant="subtle" onClick={onClose}>
          Cancel
        </Button>
        <Button
          onClick={handleSave}
          disabled={!isValid}
          variant="filled"
        >
          Save Configuration
        </Button>
      </Group>
    </Modal>
  );
};