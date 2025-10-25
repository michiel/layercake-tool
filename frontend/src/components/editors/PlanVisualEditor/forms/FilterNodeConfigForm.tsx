import React, { useEffect, useState } from 'react';
import { Stack, Select, Switch, Alert, Text, Card, Group, ActionIcon, Button, Tooltip, Divider } from '@mantine/core';
import { IconInfoCircle, IconTrash, IconPlus, IconArrowUp, IconArrowDown } from '@tabler/icons-react';
import { GraphFilter, GraphFilterKind, FilterPresetType, FilterNodeConfig } from '../../../../types/plan-dag';
import { QueryFilterBuilder, createDefaultQueryFilterConfig } from './QueryFilterBuilder';

interface FilterNodeConfigFormProps {
  config: FilterNodeConfig;
  setConfig: (config: FilterNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

type FilterParamKey = keyof GraphFilter['params'];

const FILTER_KIND_OPTIONS: { value: GraphFilterKind; label: string }[] = [
  { value: 'Preset', label: 'Preset Filter' },
  { value: 'Query', label: 'Query Filter' },
];

const PRESET_OPTIONS: { value: FilterPresetType; label: string }[] = [
  { value: 'RemoveUnconnectedNodes', label: 'Remove Unconnected Nodes' },
  { value: 'RemoveDanglingEdges', label: 'Remove Dangling Edges' },
];

const getDefaultParams = (kind: GraphFilterKind): GraphFilter['params'] => {
  switch (kind) {
    case 'Preset':
      return { preset: 'RemoveUnconnectedNodes', enabled: true };
    case 'Query':
      return { queryConfig: createDefaultQueryFilterConfig(), enabled: true };
    default:
      return { enabled: true };
  }
};

const normalizeFilters = (filters: GraphFilter[]): GraphFilter[] => {
  return filters
    .filter(Boolean)
    .map(filter => {
      const params = filter.params || {};
      const normalizedParams =
        filter.kind === 'Query'
          ? {
              ...params,
              queryConfig: params.queryConfig ?? createDefaultQueryFilterConfig(),
            }
          : params;
      return {
        kind: filter.kind,
        params: {
          ...normalizedParams,
          enabled: params.enabled ?? true,
        },
      };
    });
};

const filtersEqual = (a: GraphFilter[], b: GraphFilter[]): boolean =>
  JSON.stringify(a) === JSON.stringify(b);

const coerceFilterConfig = (raw: any): FilterNodeConfig => {
  if (raw && Array.isArray(raw.filters)) {
    return { filters: raw.filters as GraphFilter[] };
  }

  // Default to empty filters array
  return { filters: [] };
};

const isFilterValid = (filter: GraphFilter): boolean => {
  switch (filter.kind) {
    case 'Preset':
      return Boolean(filter.params.preset);
    case 'Query':
      return !!filter.params.queryConfig;
    default:
      return true;
  }
};

export const FilterNodeConfigForm: React.FC<FilterNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const initialConfig = coerceFilterConfig(config);
  const [localFilters, setLocalFilters] = useState<GraphFilter[]>(
    normalizeFilters(initialConfig.filters ?? [])
  );

  // Sync incoming config (e.g. when switching nodes)
  useEffect(() => {
    const normalized = normalizeFilters(coerceFilterConfig(config).filters ?? []);
    if (!filtersEqual(normalized, localFilters)) {
      setLocalFilters(normalized);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config]);

  // Update parent config when local config changes
  useEffect(() => {
    setConfig({ filters: localFilters });
  }, [localFilters, setConfig]);

  // Validate configuration
  useEffect(() => {
    const isValid =
      localFilters.length >= 0 && localFilters.every(filter => isFilterValid(filter));
    setIsValid(isValid);
  }, [localFilters, setIsValid]);

  const updateFilterParam = <K extends FilterParamKey>(
    index: number,
    key: K,
    value: GraphFilter['params'][K]
  ) => {
    setLocalFilters(prev => {
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

    const kind = value as GraphFilterKind;
    setLocalFilters(prev => {
      const copy = [...prev];
      copy[index] = {
        kind,
        params: getDefaultParams(kind),
      };
      return normalizeFilters(copy);
    });
  };

  const moveFilter = (index: number, direction: 'up' | 'down') => {
    setLocalFilters(prev => {
      const targetIndex = direction === 'up' ? index - 1 : index + 1;
      if (targetIndex < 0 || targetIndex >= prev.length) {
        return prev;
      }
      const copy = [...prev];
      [copy[index], copy[targetIndex]] = [copy[targetIndex], copy[index]];
      return copy;
    });
  };

  const removeFilter = (index: number) => {
    setLocalFilters(prev => {
      const filtered = prev.filter((_, idx) => idx !== index);
      return normalizeFilters(filtered);
    });
  };

  const addFilter = () => {
    setLocalFilters(prev => {
      const newFilter: GraphFilter = {
        kind: 'Preset',
        params: getDefaultParams('Preset'),
      };
      return [...prev, newFilter];
    });
  };

  const renderFilterFields = (filter: GraphFilter, index: number) => {
    switch (filter.kind) {
      case 'Preset':
        return (
          <Stack gap="xs">
            <Select
              label="Preset Type"
              description="Select a predefined filter operation"
              data={PRESET_OPTIONS}
              value={filter.params.preset ?? null}
              onChange={value => updateFilterParam(index, 'preset', value as FilterPresetType)}
              required
            />
            <Switch
              label="Enabled"
              description="Enable or disable this filter"
              checked={filter.params.enabled ?? true}
              onChange={event => updateFilterParam(index, 'enabled', event.currentTarget.checked)}
            />
          </Stack>
        );
      case 'Query':
        return (
          <Stack gap="xs">
            <QueryFilterBuilder
              value={filter.params.queryConfig}
              onChange={config => updateFilterParam(index, 'queryConfig', config)}
            />
            <Switch
              label="Enabled"
              description="Enable or disable this filter"
              checked={filter.params.enabled ?? true}
              onChange={event => updateFilterParam(index, 'enabled', event.currentTarget.checked)}
            />
          </Stack>
        );
      default:
        return null;
    }
  };

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Filter Configuration">
        <Text size="sm">
          Configure the ordered list of filters applied to this graph. Filters are applied in the order shown.
        </Text>
      </Alert>

      <Stack gap="sm">
        {localFilters.length === 0 ? (
          <Alert color="gray">
            <Text size="sm">No filters configured. Add a filter to get started.</Text>
          </Alert>
        ) : (
          localFilters.map((filter, index) => {
            return (
              <Card withBorder key={`${index}-${filter.kind}`}>
                <Stack gap="sm">
                  <Group justify="space-between" align="flex-start">
                    <Select
                      label={`Filter ${index + 1}`}
                      placeholder="Select a filter type"
                      data={FILTER_KIND_OPTIONS}
                      value={filter.kind}
                      onChange={value => handleKindChange(index, value)}
                      required
                    />
                    <Group gap="xs">
                      <Tooltip label="Move up" withArrow disabled={index === 0}>
                        <ActionIcon
                          variant="subtle"
                          aria-label="Move filter up"
                          onClick={() => moveFilter(index, 'up')}
                          disabled={index === 0}
                        >
                          <IconArrowUp size={16} />
                        </ActionIcon>
                      </Tooltip>
                      <Tooltip
                        label="Move down"
                        withArrow
                        disabled={index === localFilters.length - 1}
                      >
                        <ActionIcon
                          variant="subtle"
                          aria-label="Move filter down"
                          onClick={() => moveFilter(index, 'down')}
                          disabled={index === localFilters.length - 1}
                        >
                          <IconArrowDown size={16} />
                        </ActionIcon>
                      </Tooltip>
                      <Tooltip label="Remove this filter" withArrow>
                        <ActionIcon
                          color="red"
                          variant="subtle"
                          aria-label="Remove filter"
                          onClick={() => removeFilter(index)}
                        >
                          <IconTrash size={16} />
                        </ActionIcon>
                      </Tooltip>
                    </Group>
                  </Group>

                  <Divider />
                  {renderFilterFields(filter, index)}
                </Stack>
              </Card>
            );
          })
        )}
      </Stack>

      <Button
        variant="default"
        leftSection={<IconPlus size={16} />}
        onClick={addFilter}
        disabled={localFilters.length >= 12}
      >
        Add filter
      </Button>
    </Stack>
  );
};
