import React, { useEffect, useState } from 'react';
import {
  Stack,
  Select,
  NumberInput,
  Switch,
  Alert,
  Text,
  Card,
  Group,
  ActionIcon,
  Button,
  Tooltip,
  Divider,
} from '@mantine/core';
import { IconInfoCircle, IconTrash, IconPlus, IconArrowUp, IconArrowDown } from '@tabler/icons-react';
import {
  GraphTransform,
  GraphTransformKind,
  TransformNodeConfig,
} from '../../../../types/plan-dag';

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
  { value: 'NodeLabelMaxLength', label: 'Truncate Node Labels' },
  { value: 'NodeLabelInsertNewlines', label: 'Wrap Node Labels' },
  { value: 'EdgeLabelMaxLength', label: 'Truncate Edge Labels' },
  { value: 'EdgeLabelInsertNewlines', label: 'Wrap Edge Labels' },
  { value: 'InvertGraph', label: 'Invert Graph' },
  { value: 'GenerateHierarchy', label: 'Generate Hierarchy Metadata' },
  { value: 'AggregateEdges', label: 'Aggregate Duplicate Edges' },
];

const getDefaultParams = (kind: GraphTransformKind): GraphTransform['params'] => {
  switch (kind) {
    case 'PartitionDepthLimit':
      return { maxPartitionDepth: 2 };
    case 'PartitionWidthLimit':
      return { maxPartitionWidth: 3 };
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
    case 'AggregateEdges':
      return { enabled: true };
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
        transform.kind === 'InvertGraph' ||
        transform.kind === 'GenerateHierarchy' ||
        transform.kind === 'AggregateEdges';

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

  if (!aggregatorSeen) {
    deduped.push({
      kind: 'AggregateEdges',
      params: { enabled: true },
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
    transforms.push({
      kind: 'AggregateEdges',
      params: { enabled: true },
    });
  } else if (!transforms.some(t => t.kind === 'AggregateEdges')) {
    transforms.push({
      kind: 'AggregateEdges',
      params: { enabled: true },
    });
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
    case 'AggregateEdges':
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

  const handleKindChange = (index: number, value: string | null) => {
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
          <NumberInput
            label="Max Partition Depth"
            description="Limit the hierarchy depth explored for partitions"
            min={1}
            max={25}
            value={transform.params.maxPartitionDepth ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'maxPartitionDepth',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'PartitionWidthLimit':
        return (
          <NumberInput
            label="Max Partition Width"
            description="Restrict how many non-partition nodes remain within a partition"
            min={1}
            max={200}
            value={transform.params.maxPartitionWidth ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'maxPartitionWidth',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'NodeLabelMaxLength':
        return (
          <NumberInput
            label="Max Node Label Length"
            description="Truncate node labels to a fixed number of characters"
            min={1}
            max={200}
            value={transform.params.nodeLabelMaxLength ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'nodeLabelMaxLength',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'NodeLabelInsertNewlines':
        return (
          <NumberInput
            label="Wrap Node Labels At"
            description="Insert newlines into node labels every N characters"
            min={1}
            max={200}
            value={transform.params.nodeLabelInsertNewlinesAt ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'nodeLabelInsertNewlinesAt',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'EdgeLabelMaxLength':
        return (
          <NumberInput
            label="Max Edge Label Length"
            description="Truncate edge labels to a fixed number of characters"
            min={1}
            max={200}
            value={transform.params.edgeLabelMaxLength ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'edgeLabelMaxLength',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'EdgeLabelInsertNewlines':
        return (
          <NumberInput
            label="Wrap Edge Labels At"
            description="Insert newlines into edge labels every N characters"
            min={1}
            max={200}
            value={transform.params.edgeLabelInsertNewlinesAt ?? undefined}
            onChange={value =>
              updateTransformParam(
                index,
                'edgeLabelInsertNewlinesAt',
                typeof value === 'number' ? value : undefined
              )
            }
          />
        );
      case 'InvertGraph':
        return (
          <Stack gap="xs">
            <Switch
              label="Invert graph connections"
              description="Flip partitions into edges and edges into intermediates"
              checked={transform.params.enabled ?? true}
              onChange={event => updateTransformParam(index, 'enabled', event.currentTarget.checked)}
            />
            <Alert icon={<IconInfoCircle size="1rem" />} color="blue">
              <Text size="sm">
                Inversion creates a new graph where original edges become nodes, enabling alternative
                dependency visualisations.
              </Text>
            </Alert>
          </Stack>
        );
      case 'GenerateHierarchy':
        return (
          <Switch
            label="Generate hierarchy metadata"
            description="Annotate graph nodes with computed hierarchy details"
            checked={transform.params.enabled ?? true}
            onChange={event => updateTransformParam(index, 'enabled', event.currentTarget.checked)}
          />
        );
      case 'AggregateEdges':
        return (
          <Stack gap="xs">
            <Switch
              label="Aggregate duplicate edges"
              description="Combine edges that share the same source and target"
              checked={transform.params.enabled ?? true}
              onChange={event => updateTransformParam(index, 'enabled', event.currentTarget.checked)}
            />
            <Alert icon={<IconInfoCircle size="1rem" />} color="blue">
              <Text size="sm">
                Aggregation is applied by default to keep graphs readable. Disable to preserve duplicate
                edges for analysis.
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
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Transform Configuration">
        <Text size="sm">
          Configure the ordered list of transformations applied to this graph. Inputs and outputs are
          determined by the DAG connections.
        </Text>
      </Alert>

      <Stack gap="sm">
        {localTransforms.map((transform, index) => {
          const disableRemove = localTransforms.length === 1;
          return (
            <Card withBorder key={`${index}-${transform.kind}`}>
              <Stack gap="sm">
                <Group justify="space-between" align="flex-start">
                  <Select
                    label={`Step ${index + 1}`}
                    placeholder="Select a transformation"
                    data={TRANSFORM_OPTIONS}
                    value={transform.kind}
                    onChange={value => handleKindChange(index, value)}
                    required
                  />
                  <Group gap="xs">
                    <Tooltip label="Move up" withArrow disabled={index === 0}>
                      <ActionIcon
                        variant="subtle"
                        aria-label="Move transform up"
                        onClick={() => moveTransform(index, 'up')}
                        disabled={index === 0}
                      >
                        <IconArrowUp size={16} />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip
                      label="Move down"
                      withArrow
                      disabled={index === localTransforms.length - 1}
                    >
                      <ActionIcon
                        variant="subtle"
                        aria-label="Move transform down"
                        onClick={() => moveTransform(index, 'down')}
                        disabled={index === localTransforms.length - 1}
                      >
                        <IconArrowDown size={16} />
                      </ActionIcon>
                    </Tooltip>
                    <Tooltip
                      label={
                        disableRemove
                          ? 'At least one transform is required'
                          : 'Remove this transform'
                      }
                      withArrow
                    >
                      <ActionIcon
                        color="red"
                        variant="subtle"
                        aria-label="Remove transform"
                        disabled={disableRemove}
                        onClick={() => removeTransform(index)}
                      >
                        <IconTrash size={16} />
                      </ActionIcon>
                    </Tooltip>
                  </Group>
                </Group>

                <Divider />
                {renderTransformFields(transform, index)}
              </Stack>
            </Card>
          );
        })}
      </Stack>

      <Button
        variant="default"
        leftSection={<IconPlus size={16} />}
        onClick={addTransform}
        disabled={localTransforms.length >= 12}
      >
        Add transform
      </Button>
    </Stack>
  );
};
