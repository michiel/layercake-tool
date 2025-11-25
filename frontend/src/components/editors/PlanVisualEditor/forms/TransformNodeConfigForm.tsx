import React, { useEffect, useState } from 'react';
import { IconInfoCircle, IconTrash, IconPlus, IconArrowUp, IconArrowDown } from '@tabler/icons-react';
import {
  GraphTransform,
  GraphTransformKind,
  TransformNodeConfig,
} from '../../../../types/plan-dag';
import { Stack, Group } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Switch } from '@/components/ui/switch';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';

interface TransformNodeConfigFormProps {
  config: TransformNodeConfig;
  setConfig: (config: TransformNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

type TransformParamKey = keyof GraphTransform['params'];

const TRANSFORM_OPTIONS: { value: GraphTransformKind; label: string }[] = [
  { value: 'PartitionDepthLimit', label: 'Limit Partition Depth' },
  { value: 'PartitionWidthLimit', label: 'Limit Partition Width' },
  { value: 'DropUnconnectedNodes', label: 'Drop Unconnected Nodes' },
  { value: 'NodeLabelMaxLength', label: 'Truncate Node Labels' },
  { value: 'NodeLabelInsertNewlines', label: 'Wrap Node Labels' },
  { value: 'EdgeLabelMaxLength', label: 'Truncate Edge Labels' },
  { value: 'EdgeLabelInsertNewlines', label: 'Wrap Edge Labels' },
  { value: 'InvertGraph', label: 'Invert Graph' },
  { value: 'GenerateHierarchy', label: 'Hierarchy to flow' },
  { value: 'AggregateLayerNodes', label: 'Aggregate Nodes by Layer' },
  { value: 'AggregateEdges', label: 'Aggregate Duplicate Edges' },
];

const getDefaultParams = (kind: GraphTransformKind): GraphTransform['params'] => {
  switch (kind) {
    case 'PartitionDepthLimit':
      return { maxPartitionDepth: 2 };
    case 'PartitionWidthLimit':
      return { maxPartitionWidth: 3 };
    case 'DropUnconnectedNodes':
      return { enabled: true, excludePartitionNodes: true };
    case 'NodeLabelMaxLength':
      return { nodeLabelMaxLength: 32 };
    case 'NodeLabelInsertNewlines':
      return { nodeLabelInsertNewlinesAt: 16 };
    case 'EdgeLabelMaxLength':
      return { edgeLabelMaxLength: 32 };
    case 'EdgeLabelInsertNewlines':
      return { edgeLabelInsertNewlinesAt: 16 };
    case 'InvertGraph':
    case 'GenerateHierarchy':
      return { enabled: true };
    case 'AggregateLayerNodes':
      return { layerConnectionsThreshold: 3 };
    case 'AggregateEdges':
      return {};
    default:
      return {};
  }
};

const normalizeTransforms = (transforms: GraphTransform[]): GraphTransform[] => {
  const sanitized = transforms
    .filter(Boolean)
    .map(transform => {
      const params = transform.params || {};
      const needsEnabled =
        transform.kind === 'InvertGraph' || transform.kind === 'GenerateHierarchy';

      return {
        kind: transform.kind,
        params: {
          ...params,
          enabled: needsEnabled ? params.enabled ?? true : params.enabled,
        },
      };
    });

  const deduped: GraphTransform[] = [];
  let aggregatorSeen = false;

  sanitized.forEach(transform => {
    if (transform.kind === 'AggregateEdges') {
      if (aggregatorSeen) {
        return;
      }
      aggregatorSeen = true;
    }
    deduped.push(transform);
  });

  if (deduped.length === 0) {
    deduped.push({
      kind: 'PartitionDepthLimit',
      params: getDefaultParams('PartitionDepthLimit'),
    });
  }

  return deduped;
};

const transformsEqual = (a: GraphTransform[], b: GraphTransform[]): boolean =>
  JSON.stringify(a) === JSON.stringify(b);

const isPositive = (value?: number) => typeof value === 'number' && value > 0;

const coerceTransformConfig = (raw: any): TransformNodeConfig => {
  if (raw && Array.isArray(raw.transforms)) {
    return { transforms: raw.transforms as GraphTransform[] };
  }

  const transforms: GraphTransform[] = [];
  const legacyType = raw?.transformType;
  const legacyConfig = raw?.transformConfig || {};

  if (legacyType === 'PartitionDepthLimit') {
    if (isPositive(legacyConfig.maxPartitionDepth)) {
      transforms.push({
        kind: 'PartitionDepthLimit',
        params: { maxPartitionDepth: legacyConfig.maxPartitionDepth },
      });
    }
    if (isPositive(legacyConfig.maxPartitionWidth)) {
      transforms.push({
        kind: 'PartitionWidthLimit',
        params: { maxPartitionWidth: legacyConfig.maxPartitionWidth },
      });
    }
    if (legacyConfig.generateHierarchy) {
      transforms.push({
        kind: 'GenerateHierarchy',
        params: { enabled: true },
      });
    }
  } else if (legacyType === 'InvertGraph') {
    transforms.push({
      kind: 'InvertGraph',
      params: { enabled: legacyConfig.invertGraph ?? true },
    });
  }

  if (transforms.length === 0) {
    return {
      transforms: [
        {
          kind: 'PartitionDepthLimit',
          params: getDefaultParams('PartitionDepthLimit'),
        },
      ],
    };
  }

  return { transforms };
};

const isTransformValid = (transform: GraphTransform): boolean => {
  switch (transform.kind) {
    case 'PartitionDepthLimit':
      return isPositive(transform.params.maxPartitionDepth);
    case 'PartitionWidthLimit':
      return isPositive(transform.params.maxPartitionWidth);
    case 'NodeLabelMaxLength':
      return isPositive(transform.params.nodeLabelMaxLength);
    case 'NodeLabelInsertNewlines':
      return isPositive(transform.params.nodeLabelInsertNewlinesAt);
    case 'EdgeLabelMaxLength':
      return isPositive(transform.params.edgeLabelMaxLength);
    case 'EdgeLabelInsertNewlines':
      return isPositive(transform.params.edgeLabelInsertNewlinesAt);
    case 'InvertGraph':
    case 'GenerateHierarchy':
    case 'DropUnconnectedNodes':
    case 'AggregateEdges':
    case 'AggregateLayerNodes':
      return true;
    default:
      return true;
  }
};

export const TransformNodeConfigForm: React.FC<TransformNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const initialConfig = coerceTransformConfig(config);
  const [localTransforms, setLocalTransforms] = useState<GraphTransform[]>(
    normalizeTransforms(initialConfig.transforms ?? [])
  );
  const lastSentConfigRef = React.useRef<GraphTransform[]>(localTransforms);

  // Sync incoming config (e.g. when switching nodes)
  useEffect(() => {
    const normalized = normalizeTransforms(coerceTransformConfig(config).transforms ?? []);
    if (!transformsEqual(normalized, localTransforms)) {
      setLocalTransforms(normalized);
      lastSentConfigRef.current = normalized;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config]);

  // Update parent config when local config changes (but avoid loops)
  useEffect(() => {
    if (!transformsEqual(localTransforms, lastSentConfigRef.current)) {
      setConfig({ transforms: localTransforms });
      lastSentConfigRef.current = localTransforms;
    }
  }, [localTransforms, setConfig]);

  // Validate configuration
  useEffect(() => {
    const isValid =
      localTransforms.length > 0 && localTransforms.every(transform => isTransformValid(transform));
    setIsValid(isValid);
  }, [localTransforms, setIsValid]);

  const updateTransformParam = (
    index: number,
    key: TransformParamKey,
    value: number | boolean | undefined
  ) => {
    setLocalTransforms(prev => {
      const copy = [...prev];
      copy[index] = {
        ...copy[index],
        params: {
          ...copy[index].params,
          [key]: value,
        },
      };
      return copy;
    });
  };

  const handleKindChange = (index: number, value: string | undefined) => {
    if (!value) {
      return;
    }

    const kind = value as GraphTransformKind;
    setLocalTransforms(prev => {
      const copy = [...prev];
      copy[index] = {
        kind,
        params: getDefaultParams(kind),
      };
      return normalizeTransforms(copy);
    });
  };

  const moveTransform = (index: number, direction: 'up' | 'down') => {
    setLocalTransforms(prev => {
      const targetIndex = direction === 'up' ? index - 1 : index + 1;
      if (targetIndex < 0 || targetIndex >= prev.length) {
        return prev;
      }
      const copy = [...prev];
      [copy[index], copy[targetIndex]] = [copy[targetIndex], copy[index]];
      return copy;
    });
  };

  const removeTransform = (index: number) => {
    setLocalTransforms(prev => {
      const filtered = prev.filter((_, idx) => idx !== index);
      return normalizeTransforms(filtered);
    });
  };

  const addTransform = () => {
    setLocalTransforms(prev => {
      const newTransform: GraphTransform = {
        kind: 'PartitionDepthLimit',
        params: getDefaultParams('PartitionDepthLimit'),
      };
      const aggregatorIndex = prev.findIndex(t => t.kind === 'AggregateEdges');
      if (aggregatorIndex === -1) {
        return [...prev, newTransform];
      }
      const copy = [...prev];
      copy.splice(aggregatorIndex, 0, newTransform);
      return copy;
    });
  };

  const renderTransformFields = (transform: GraphTransform, index: number) => {
    switch (transform.kind) {
      case 'PartitionDepthLimit':
        return (
          <div className="space-y-2">
            <Label htmlFor={`max-partition-depth-${index}`}>Max Partition Depth</Label>
            <Input
              id={`max-partition-depth-${index}`}
              type="number"
              min={1}
              max={25}
              value={transform.params.maxPartitionDepth ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'maxPartitionDepth', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Limit the hierarchy depth explored for partitions
            </p>
          </div>
        );
      case 'PartitionWidthLimit':
        return (
          <div className="space-y-2">
            <Label htmlFor={`max-partition-width-${index}`}>Max Partition Width</Label>
            <Input
              id={`max-partition-width-${index}`}
              type="number"
              min={1}
              max={200}
              value={transform.params.maxPartitionWidth ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'maxPartitionWidth', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Restrict how many non-partition nodes remain within a partition
            </p>
          </div>
        );
      case 'DropUnconnectedNodes':
        return (
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Switch
                id={`drop-unconnected-partitions-${index}`}
                checked={transform.params.excludePartitionNodes ?? true}
                onCheckedChange={checked =>
                  updateTransformParam(index, 'excludePartitionNodes', checked)
                }
              />
              <Label htmlFor={`drop-unconnected-partitions-${index}`}>
                Exclude partition nodes
              </Label>
            </div>
            <p className="text-sm text-muted-foreground">
              Keep partition nodes even if no edges reference them. Disable to drop unconnected
              partitions as well.
            </p>
          </div>
        );
      case 'NodeLabelMaxLength':
        return (
          <div className="space-y-2">
            <Label htmlFor={`node-label-max-length-${index}`}>Max Node Label Length</Label>
            <Input
              id={`node-label-max-length-${index}`}
              type="number"
              min={1}
              max={200}
              value={transform.params.nodeLabelMaxLength ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'nodeLabelMaxLength', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Truncate node labels to a fixed number of characters
            </p>
          </div>
        );
      case 'NodeLabelInsertNewlines':
        return (
          <div className="space-y-2">
            <Label htmlFor={`node-label-insert-newlines-${index}`}>Wrap Node Labels At</Label>
            <Input
              id={`node-label-insert-newlines-${index}`}
              type="number"
              min={1}
              max={200}
              value={transform.params.nodeLabelInsertNewlinesAt ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'nodeLabelInsertNewlinesAt', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Insert newlines into node labels every N characters
            </p>
          </div>
        );
      case 'EdgeLabelMaxLength':
        return (
          <div className="space-y-2">
            <Label htmlFor={`edge-label-max-length-${index}`}>Max Edge Label Length</Label>
            <Input
              id={`edge-label-max-length-${index}`}
              type="number"
              min={1}
              max={200}
              value={transform.params.edgeLabelMaxLength ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'edgeLabelMaxLength', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Truncate edge labels to a fixed number of characters
            </p>
          </div>
        );
      case 'EdgeLabelInsertNewlines':
        return (
          <div className="space-y-2">
            <Label htmlFor={`edge-label-insert-newlines-${index}`}>Wrap Edge Labels At</Label>
            <Input
              id={`edge-label-insert-newlines-${index}`}
              type="number"
              min={1}
              max={200}
              value={transform.params.edgeLabelInsertNewlinesAt ?? ''}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'edgeLabelInsertNewlinesAt', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Insert newlines into edge labels every N characters
            </p>
          </div>
        );
      case 'InvertGraph':
        return (
          <Stack gap="xs">
            <div className="space-y-2">
              <div className="flex items-center space-x-2">
                <Switch
                  id={`invert-graph-${index}`}
                  checked={transform.params.enabled ?? true}
                  onCheckedChange={checked => updateTransformParam(index, 'enabled', checked)}
                />
                <Label htmlFor={`invert-graph-${index}`}>Invert graph connections</Label>
              </div>
              <p className="text-sm text-muted-foreground">
                Flip partitions into edges and edges into intermediates
              </p>
            </div>
            <Alert>
              <IconInfoCircle className="h-4 w-4" />
              <AlertDescription>
                Inversion creates a new graph where original edges become nodes, enabling alternative
                dependency visualisations.
              </AlertDescription>
            </Alert>
          </Stack>
        );
      case 'GenerateHierarchy':
        return (
          <div className="space-y-2">
            <div className="flex items-center space-x-2">
              <Switch
                id={`generate-hierarchy-${index}`}
                checked={transform.params.enabled ?? true}
                onCheckedChange={checked => updateTransformParam(index, 'enabled', checked)}
              />
              <Label htmlFor={`generate-hierarchy-${index}`}>Generate hierarchy metadata</Label>
            </div>
            <p className="text-sm text-muted-foreground">
              Annotate graph nodes with computed hierarchy details
            </p>
          </div>
        );
      case 'AggregateEdges':
        return (
          <Alert>
            <IconInfoCircle className="h-4 w-4" />
            <AlertTitle>Aggregate duplicate edges</AlertTitle>
            <AlertDescription>
              Combine edges that share the same source and target. Remove this transform to keep
              duplicate edges separate.
            </AlertDescription>
          </Alert>
        );
      case 'AggregateLayerNodes':
        return (
          <div className="space-y-2">
            <Label htmlFor={`layer-aggregation-threshold-${index}`}>
              Shared connections threshold
            </Label>
            <Input
              id={`layer-aggregation-threshold-${index}`}
              type="number"
              min={1}
              max={100}
              value={transform.params.layerConnectionsThreshold ?? 3}
              onChange={e => {
                const value = e.target.value ? parseInt(e.target.value, 10) : undefined;
                updateTransformParam(index, 'layerConnectionsThreshold', value);
              }}
            />
            <p className="text-sm text-muted-foreground">
              Merge sibling nodes in the same layer when at least this many nodes connect to the same
              external node.
            </p>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <Stack gap="md">
      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Transform Configuration</AlertTitle>
        <AlertDescription>
          Configure the ordered list of transformations applied to this graph. Inputs and outputs are
          determined by the DAG connections.
        </AlertDescription>
      </Alert>

      <Stack gap="sm">
        {localTransforms.map((transform, index) => {
          const disableRemove = localTransforms.length === 1;
          return (
            <Card key={`${index}-${transform.kind}`} className="border">
              <CardContent className="pt-6">
                <Stack gap="sm">
                  <Group justify="between" align="start">
                    <div className="space-y-2 flex-1">
                      <Label htmlFor={`transform-kind-${index}`}>Step {index + 1}</Label>
                      <Select
                        value={transform.kind}
                        onValueChange={value => handleKindChange(index, value)}
                      >
                        <SelectTrigger id={`transform-kind-${index}`}>
                          <SelectValue placeholder="Select a transformation" />
                        </SelectTrigger>
                        <SelectContent>
                          {TRANSFORM_OPTIONS.map(option => (
                            <SelectItem key={option.value} value={option.value}>
                              {option.label}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <TooltipProvider>
                      <Group gap="xs" className="pt-8">
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              aria-label="Move transform up"
                              onClick={() => moveTransform(index, 'up')}
                              disabled={index === 0}
                            >
                              <IconArrowUp className="h-4 w-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>Move up</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              aria-label="Move transform down"
                              onClick={() => moveTransform(index, 'down')}
                              disabled={index === localTransforms.length - 1}
                            >
                              <IconArrowDown className="h-4 w-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>Move down</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger asChild>
                            <Button
                              variant="ghost"
                              size="icon"
                              aria-label="Remove transform"
                              disabled={disableRemove}
                              onClick={() => removeTransform(index)}
                              className="text-destructive hover:text-destructive"
                            >
                              <IconTrash className="h-4 w-4" />
                            </Button>
                          </TooltipTrigger>
                          <TooltipContent>
                            {disableRemove
                              ? 'At least one transform is required'
                              : 'Remove this transform'}
                          </TooltipContent>
                        </Tooltip>
                      </Group>
                    </TooltipProvider>
                  </Group>

                  <Separator />
                  {renderTransformFields(transform, index)}
                </Stack>
              </CardContent>
            </Card>
          );
        })}
      </Stack>

      <Button
        variant="outline"
        onClick={addTransform}
        disabled={localTransforms.length >= 12}
        className="w-full"
      >
        <IconPlus className="mr-2 h-4 w-4" />
        Add transform
      </Button>
    </Stack>
  );
};
