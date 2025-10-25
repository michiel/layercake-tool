import { useEffect, useMemo, useState } from 'react';
import {
  Card,
  Stack,
  MultiSelect,
  SegmentedControl,
  Select,
  Group,
  Text,
  TextInput,
  Button,
  Textarea,
  Divider,
} from '@mantine/core';
import { IconPlus } from '@tabler/icons-react';
import { QueryBuilder, type Field, type RuleGroupType } from 'react-querybuilder';
import 'react-querybuilder/dist/query-builder.css';
import {
  QueryFilterConfig,
  QueryFilterTarget,
  QueryLinkPruningMode,
} from '../../../../types/plan-dag';

type EntityField = Field & { entity: QueryFilterTarget };

const BOOLEAN_VALUES = [
  { name: 'true', label: 'True' },
  { name: 'false', label: 'False' },
];

const TEXT_OPERATORS: string[] = ['=', '!=', 'contains', 'beginsWith', 'endsWith', 'in'];
const NUMBER_OPERATORS: string[] = ['=', '!=', '<', '<=', '>', '>=', 'between', 'in'];
const BOOLEAN_OPERATORS: string[] = ['=', '!='];

const BASE_FIELDS: EntityField[] = [
  // Node fields
  { name: 'node.id', label: 'Node ID', entity: 'nodes', operators: TEXT_OPERATORS },
  { name: 'node.label', label: 'Node Label', entity: 'nodes', operators: TEXT_OPERATORS },
  { name: 'node.layer', label: 'Node Layer', entity: 'nodes', operators: TEXT_OPERATORS },
  {
    name: 'node.weight',
    label: 'Node Weight',
    entity: 'nodes',
    inputType: 'number',
    operators: NUMBER_OPERATORS,
  },
  {
    name: 'node.belongs_to',
    label: 'Node Parent',
    entity: 'nodes',
    operators: TEXT_OPERATORS,
  },
  {
    name: 'node.is_partition',
    label: 'Node Is Partition',
    entity: 'nodes',
    valueEditorType: 'select',
    values: BOOLEAN_VALUES,
    operators: BOOLEAN_OPERATORS,
  },
  {
    name: 'node.datasource_id',
    label: 'Node Datasource ID',
    entity: 'nodes',
    inputType: 'number',
    operators: NUMBER_OPERATORS,
  },
  { name: 'node.comment', label: 'Node Comment', entity: 'nodes', operators: TEXT_OPERATORS },

  // Edge fields
  { name: 'edge.id', label: 'Edge ID', entity: 'edges', operators: TEXT_OPERATORS },
  { name: 'edge.label', label: 'Edge Label', entity: 'edges', operators: TEXT_OPERATORS },
  { name: 'edge.source', label: 'Edge Source Node', entity: 'edges', operators: TEXT_OPERATORS },
  { name: 'edge.target', label: 'Edge Target Node', entity: 'edges', operators: TEXT_OPERATORS },
  { name: 'edge.layer', label: 'Edge Layer', entity: 'edges', operators: TEXT_OPERATORS },
  {
    name: 'edge.weight',
    label: 'Edge Weight',
    entity: 'edges',
    inputType: 'number',
    operators: NUMBER_OPERATORS,
  },
  {
    name: 'edge.datasource_id',
    label: 'Edge Datasource ID',
    entity: 'edges',
    inputType: 'number',
    operators: NUMBER_OPERATORS,
  },

  // Layer fields
  { name: 'layer.layer_id', label: 'Layer ID', entity: 'layers', operators: TEXT_OPERATORS },
  { name: 'layer.name', label: 'Layer Name', entity: 'layers', operators: TEXT_OPERATORS },
  {
    name: 'layer.background_color',
    label: 'Layer Background Color',
    entity: 'layers',
    operators: TEXT_OPERATORS,
  },
  {
    name: 'layer.text_color',
    label: 'Layer Text Color',
    entity: 'layers',
    operators: TEXT_OPERATORS,
  },
  {
    name: 'layer.border_color',
    label: 'Layer Border Color',
    entity: 'layers',
    operators: TEXT_OPERATORS,
  },
];

const TARGET_OPTIONS = [
  { value: 'nodes', label: 'Nodes' },
  { value: 'edges', label: 'Edges' },
  { value: 'layers', label: 'Layers' },
];

const MODE_OPTIONS = [
  { label: 'Include matches', value: 'include' },
  { label: 'Exclude matches', value: 'exclude' },
];

const LINK_PRUNING_OPTIONS: { value: QueryLinkPruningMode; label: string; description: string }[] = [
  {
    value: 'autoDropDanglingEdges',
    label: 'Auto-drop dangling edges',
    description: 'Remove edges referencing removed nodes automatically.',
  },
  {
    value: 'retainEdges',
    label: 'Retain edges',
    description: 'Keep original edges even if their endpoints are removed.',
  },
  {
    value: 'dropOrphanNodes',
    label: 'Drop orphan nodes',
    description: 'When filtering edges, remove nodes that lose all incident edges.',
  },
];

export const createDefaultQueryFilterConfig = (): QueryFilterConfig => ({
  targets: ['nodes'],
  mode: 'include',
  linkPruningMode: 'autoDropDanglingEdges',
  ruleGroup: { combinator: 'and', rules: [] },
  fieldMetadataVersion: 'v1',
});

const ensureConfig = (config?: QueryFilterConfig): QueryFilterConfig => {
  if (!config) {
    return createDefaultQueryFilterConfig();
  }

  const needsTargets = !config.targets || config.targets.length === 0;
  const needsRuleGroup = !config.ruleGroup;
  const needsLinkPruning = !config.linkPruningMode;
  const needsMode = !config.mode;
  const needsVersion = !config.fieldMetadataVersion;

  if (!needsTargets && !needsRuleGroup && !needsLinkPruning && !needsMode && !needsVersion) {
    return config;
  }

  const defaults = createDefaultQueryFilterConfig();

  return {
    ...config,
    targets: needsTargets ? defaults.targets : config.targets,
    ruleGroup: needsRuleGroup ? defaults.ruleGroup : config.ruleGroup,
    linkPruningMode: needsLinkPruning ? defaults.linkPruningMode : config.linkPruningMode,
    mode: needsMode ? defaults.mode : config.mode,
    fieldMetadataVersion: needsVersion
      ? defaults.fieldMetadataVersion
      : config.fieldMetadataVersion,
  };
};

const BASE_FIELD_NAMES = new Set(BASE_FIELDS.map(field => field.name));

const guessEntityFromField = (name: string): QueryFilterTarget => {
  if (name.startsWith('edge.')) {
    return 'edges';
  }
  if (name.startsWith('layer.')) {
    return 'layers';
  }
  return 'nodes';
};

const prettifyLabel = (name: string): string => {
  const withoutPrefix = name.replace(/^node\.|^edge\.|^layer\./, '');
  return withoutPrefix
    .split('.')
    .map(segment => segment.replace(/_/g, ' '))
    .map(segment => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join(' ');
};

const collectFieldNames = (group: RuleGroupType): string[] => {
  const names: string[] = [];
  group.rules?.forEach(rule => {
    if (typeof rule === 'object' && rule !== null) {
      if (Array.isArray((rule as RuleGroupType).rules)) {
        names.push(...collectFieldNames(rule as RuleGroupType));
      } else if ('field' in rule && typeof rule.field === 'string') {
        names.push(rule.field);
      }
    }
  });
  return names;
};

interface QueryFilterBuilderProps {
  value?: QueryFilterConfig;
  onChange: (config: QueryFilterConfig) => void;
}

export const QueryFilterBuilder: React.FC<QueryFilterBuilderProps> = ({ value, onChange }) => {
  const mergedConfig = useMemo(() => ensureConfig(value), [value]);
  const [customFieldInput, setCustomFieldInput] = useState('');
  const [customFields, setCustomFields] = useState<EntityField[]>([]);

  useEffect(() => {
    setCustomFields(prev => {
      const next = [...prev];
      const existingNames = new Set(next.map(field => field.name));
      const referencedNames = collectFieldNames(mergedConfig.ruleGroup);
      let mutated = false;
      referencedNames.forEach(name => {
        if (!BASE_FIELD_NAMES.has(name) && !existingNames.has(name)) {
          next.push({
            name,
            label: prettifyLabel(name),
            entity: guessEntityFromField(name),
            operators: TEXT_OPERATORS,
          });
          existingNames.add(name);
          mutated = true;
        }
      });
      return mutated ? next : prev;
    });
  }, [mergedConfig.ruleGroup]);

  const handleConfigChange = (partial: Partial<QueryFilterConfig>) => {
    onChange({
      ...mergedConfig,
      ...partial,
    });
  };

  const safeTargets: QueryFilterTarget[] = mergedConfig.targets?.length
    ? mergedConfig.targets
    : (['nodes'] as QueryFilterTarget[]);
  const availableFields = useMemo(() => {
    const allowed = new Set<QueryFilterTarget>(safeTargets);
    return [...BASE_FIELDS, ...customFields].filter(field => allowed.has(field.entity));
  }, [customFields, safeTargets]);

  const handleAddCustomField = () => {
    const trimmed = customFieldInput.trim();
    if (!trimmed || BASE_FIELD_NAMES.has(trimmed)) {
      setCustomFieldInput('');
      return;
    }
    setCustomFields(prev => {
      if (prev.some(field => field.name === trimmed)) {
        return prev;
      }
      return [
        ...prev,
        {
          name: trimmed,
          label: prettifyLabel(trimmed),
          entity: guessEntityFromField(trimmed),
          operators: TEXT_OPERATORS,
        },
      ];
    });
    setCustomFieldInput('');
  };

  const customFieldExists =
    BASE_FIELD_NAMES.has(customFieldInput.trim()) ||
    customFields.some(field => field.name === customFieldInput.trim());

  return (
    <Card withBorder radius="md" padding="md">
      <Stack gap="sm">
        <Group align="flex-end">
          <MultiSelect
            label="Entity targets"
            description="Apply this query to one or more entity types"
            data={TARGET_OPTIONS}
            value={safeTargets}
            onChange={next =>
              handleConfigChange({
                targets: (next.length ? next : ['nodes']) as QueryFilterTarget[],
              })
            }
            searchable
            style={{ flex: 1 }}
          />
          <Stack gap={4} style={{ minWidth: 220 }}>
            <Text size="sm" fw={500}>
              Match mode
            </Text>
            <SegmentedControl
              data={MODE_OPTIONS}
              value={mergedConfig.mode}
              onChange={value => handleConfigChange({ mode: value as QueryFilterConfig['mode'] })}
            />
          </Stack>
        </Group>

        <Select
          label="Link pruning mode"
          description={
            LINK_PRUNING_OPTIONS.find(option => option.value === mergedConfig.linkPruningMode)
              ?.description
          }
          data={LINK_PRUNING_OPTIONS.map(option => ({
            value: option.value,
            label: option.label,
          }))}
          value={mergedConfig.linkPruningMode}
          onChange={value =>
            handleConfigChange({
              linkPruningMode: (value as QueryLinkPruningMode) ?? mergedConfig.linkPruningMode,
            })
          }
        />

        <Stack gap={4}>
          <Text size="sm" fw={500}>
            Query rules
          </Text>
          <Text size="xs" c="dimmed">
            Build nested AND/OR groups to match attributes across the selected entities. Use custom
            fields for JSON attributes such as <code>node.attrs.priority</code>.
          </Text>
          <QueryBuilder
            fields={availableFields}
            query={mergedConfig.ruleGroup}
            onQueryChange={ruleGroup => handleConfigChange({ ruleGroup })}
            controlClassnames={{ queryBuilder: 'lc-query-builder' }}
          />
        </Stack>

        <Group align="flex-end">
          <TextInput
            label="Custom field"
            placeholder="node.attrs.priority"
            value={customFieldInput}
            onChange={event => setCustomFieldInput(event.currentTarget.value)}
            style={{ flex: 1 }}
          />
          <Button
            variant="light"
            leftSection={<IconPlus size={16} />}
            onClick={handleAddCustomField}
            disabled={!customFieldInput.trim() || customFieldExists}
          >
            Add field
          </Button>
        </Group>

        <Divider />

        <Textarea
          label="Notes"
          description="Optional context or TODOs for this query filter."
          value={mergedConfig.notes ?? ''}
          onChange={event => handleConfigChange({ notes: event.currentTarget.value })}
          autosize
          minRows={2}
        />
      </Stack>
    </Card>
  );
};
