import React from 'react';
import { Modal, Button, Group, Text } from '@mantine/core';
import { PlanDagNodeType, NodeMetadata } from '../../../types/plan-dag';
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
  projectId: number;
  onSave: (nodeId: string, config: any, metadata: any) => void;
  nodeId: string;
  config: any;
  metadata: any;
}

const sanitizeMetadata = (raw: any): NodeMetadata => {
  if (raw && typeof raw === 'object') {
    const { label, description } = raw as any;
    const metadata: NodeMetadata = {
      label: typeof label === 'string' ? label : '',
    };
    if (typeof description === 'string' && description.length > 0) {
      metadata.description = description;
    }
    return metadata;
  }

  return { label: '' };
};

export const NodeConfigDialog: React.FC<NodeConfigDialogProps> = ({
  opened,
  onClose,
  nodeType,
  projectId,
  onSave,
  nodeId,
  config: initialConfig,
  metadata: initialMetadata,
}) => {
  const [config, setConfig] = React.useState(initialConfig);
  const [metadata, setMetadata] = React.useState<NodeMetadata>(sanitizeMetadata(initialMetadata));
  const [isValid, setIsValid] = React.useState(false);

  React.useEffect(() => {
    if (opened) {
      setConfig(initialConfig);
      setMetadata(sanitizeMetadata(initialMetadata));
    }
  }, [opened, initialConfig, initialMetadata]);

  const handleSave = () => {
    if (isValid) {
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
        return (
          <DataSourceNodeConfigForm
            {...commonProps}
            metadata={metadata}
            setMetadata={setMetadata}
          />
        );
      case PlanDagNodeType.GRAPH:
        return (
          <GraphNodeConfigForm
            {...commonProps}
            metadata={metadata}
            setMetadata={setMetadata}
          />
        );
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
      title={`Configure ${getNodeTypeName()}`}
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
