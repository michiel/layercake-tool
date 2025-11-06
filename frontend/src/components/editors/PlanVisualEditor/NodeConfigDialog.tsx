import React, { useCallback } from 'react';
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { PlanDagNodeType, NodeMetadata } from '../../../types/plan-dag';
import { DataSourceNodeConfigForm } from './forms/DataSourceNodeConfigForm';
import { TransformNodeConfigForm } from './forms/TransformNodeConfigForm';
import { FilterNodeConfigForm } from './forms/FilterNodeConfigForm';
import { MergeNodeConfigForm } from './forms/MergeNodeConfigForm';
import { OutputNodeConfigForm } from './forms/OutputNodeConfigForm';
import { GraphNodeConfigForm } from './forms/GraphNodeConfigForm';

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
  const [config, setConfigState] = React.useState(initialConfig);
  const [metadata, setMetadataState] = React.useState<NodeMetadata>(sanitizeMetadata(initialMetadata));
  const [isValid, setIsValidState] = React.useState(false);

  // Memoize setters to prevent infinite loops in child components
  const setConfig = useCallback((newConfig: React.SetStateAction<any>) => setConfigState(newConfig), []);
  const setMetadata = useCallback((newMetadata: React.SetStateAction<NodeMetadata>) => setMetadataState(newMetadata), []);
  const setIsValid = useCallback((valid: React.SetStateAction<boolean>) => setIsValidState(valid), []);

  React.useEffect(() => {
    if (opened) {
      setConfigState(initialConfig);
      setMetadataState(sanitizeMetadata(initialMetadata));
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
      case PlanDagNodeType.FILTER:
        return <FilterNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.MERGE:
        return <MergeNodeConfigForm {...commonProps} />;
      case PlanDagNodeType.OUTPUT:
        return <OutputNodeConfigForm {...commonProps} />;
      default:
        return <p className="text-destructive">Unknown node type: {nodeType}</p>;
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
      case PlanDagNodeType.FILTER:
        return 'Filter';
      case PlanDagNodeType.MERGE:
        return 'Merge';
      case PlanDagNodeType.OUTPUT:
        return 'Output';
      default:
        return 'Unknown';
    }
  };

  return (
    <Dialog
      open={opened}
      onOpenChange={(open) => {
        // Only allow closing via Cancel button (closeOnClickOutside={false}, closeOnEscape={false})
        if (!open) return;
      }}
    >
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Configure {getNodeTypeName()}</DialogTitle>
        </DialogHeader>
        <div className="py-4">
          {renderConfigForm()}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={!isValid}
          >
            Save Configuration
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
