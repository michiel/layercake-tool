import { useEffect, useMemo, useRef, useState, useCallback } from 'react';
import { IconPlus } from '@tabler/icons-react';
import { QueryBuilder, type Field, type RuleGroupType } from 'react-querybuilder';
import 'react-querybuilder/dist/query-builder.css';
import {
  QueryFilterConfig,
  QueryFilterTarget,
  QueryLinkPruningMode,
} from '../../../../types/plan-dag';
import { Stack, Group } from '@/components/layout-primitives';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';

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

export const normalizeQueryFilterConfig = (config?: QueryFilterConfig): QueryFilterConfig => {
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
  const mergedConfig = useMemo(() => normalizeQueryFilterConfig(value), [value]);
  const mergedConfigRef = useRef(mergedConfig);
  const onChangeRef = useRef(onChange);
  const lastEmittedConfigRef = useRef<string>(JSON.stringify(mergedConfig));
  const [customFieldInput, setCustomFieldInput] = useState('');
  const [customFields, setCustomFields] = useState<EntityField[]>([]);

  // Keep refs in sync with latest props
  useEffect(() => {
    mergedConfigRef.current = mergedConfig;
    // Update lastEmittedConfigRef when value prop changes from parent
    lastEmittedConfigRef.current = JSON.stringify(mergedConfig);
  }, [mergedConfig]);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

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

  // Stable callback - no dependencies that change frequently
  const handleConfigChange = useCallback((partial: Partial<QueryFilterConfig>) => {
    const nextConfig = {
      ...mergedConfigRef.current,
      ...partial,
    };
    const nextJson = JSON.stringify(nextConfig);

    // Only call onChange if config actually changed
    if (nextJson !== lastEmittedConfigRef.current) {
      lastEmittedConfigRef.current = nextJson;
      onChangeRef.current(nextConfig);
    }
  }, []);

  const ruleGroupJsonRef = useRef(JSON.stringify(mergedConfig.ruleGroup));

  useEffect(() => {
    ruleGroupJsonRef.current = JSON.stringify(mergedConfig.ruleGroup);
  }, [mergedConfig.ruleGroup]);

  const handleRuleGroupChange = useCallback((ruleGroup: RuleGroupType) => {
    const nextJson = JSON.stringify(ruleGroup);
    if (nextJson === ruleGroupJsonRef.current) {
      return;
    }
    ruleGroupJsonRef.current = nextJson;
    handleConfigChange({ ruleGroup });
  }, [handleConfigChange]);

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
    <Card className="border">
      <CardContent className="pt-6">
        <Stack gap="sm">
          <Group align="end">
            <div className="space-y-2 flex-1">
              <Label>Entity targets</Label>
              <div className="space-y-2">
                {TARGET_OPTIONS.map(option => (
                  <div key={option.value} className="flex items-center space-x-2">
                    <Checkbox
                      id={`target-${option.value}`}
                      checked={safeTargets.includes(option.value as QueryFilterTarget)}
                      onCheckedChange={checked => {
                        const next = checked
                          ? [...safeTargets, option.value as QueryFilterTarget]
                          : safeTargets.filter(t => t !== option.value);
                        handleConfigChange({
                          targets: (next.length ? next : ['nodes']) as QueryFilterTarget[],
                        });
                      }}
                    />
                    <Label htmlFor={`target-${option.value}`} className="font-normal cursor-pointer">
                      {option.label}
                    </Label>
                  </div>
                ))}
              </div>
              <p className="text-sm text-muted-foreground">
                Apply this query to one or more entity types
              </p>
            </div>
            <div className="space-y-2" style={{ minWidth: 220 }}>
              <Label>Match mode</Label>
              <Tabs
                value={mergedConfig.mode}
                onValueChange={value => handleConfigChange({ mode: value as QueryFilterConfig['mode'] })}
              >
                <TabsList className="grid w-full grid-cols-2">
                  {MODE_OPTIONS.map(option => (
                    <TabsTrigger key={option.value} value={option.value}>
                      {option.label}
                    </TabsTrigger>
                  ))}
                </TabsList>
              </Tabs>
            </div>
          </Group>

          <div className="space-y-2">
            <Label htmlFor="link-pruning-mode">Link pruning mode</Label>
            <Select
              value={mergedConfig.linkPruningMode}
              onValueChange={value =>
                handleConfigChange({
                  linkPruningMode: (value as QueryLinkPruningMode) ?? mergedConfig.linkPruningMode,
                })
              }
            >
              <SelectTrigger id="link-pruning-mode">
                <SelectValue placeholder="Select pruning mode" />
              </SelectTrigger>
              <SelectContent>
                {LINK_PRUNING_OPTIONS.map(option => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <p className="text-sm text-muted-foreground">
              {LINK_PRUNING_OPTIONS.find(option => option.value === mergedConfig.linkPruningMode)
                ?.description}
            </p>
          </div>

          <div className="space-y-1">
            <Label>Query rules</Label>
            <p className="text-xs text-muted-foreground">
              Build nested AND/OR groups to match attributes across the selected entities. Use custom
              fields for JSON attributes such as <code>node.attrs.priority</code>.
            </p>
            <QueryBuilder
              fields={availableFields}
              query={mergedConfig.ruleGroup}
              onQueryChange={handleRuleGroupChange}
              controlClassnames={{ queryBuilder: 'lc-query-builder' }}
            />
          </div>

          <Group align="end">
            <div className="space-y-2 flex-1">
              <Label htmlFor="custom-field">Custom field</Label>
              <Input
                id="custom-field"
                placeholder="node.attrs.priority"
                value={customFieldInput}
                onChange={event => setCustomFieldInput(event.currentTarget.value)}
              />
            </div>
            <Button
              variant="secondary"
              onClick={handleAddCustomField}
              disabled={!customFieldInput.trim() || customFieldExists}
            >
              <IconPlus className="mr-2 h-4 w-4" />
              Add field
            </Button>
          </Group>

          <Separator />

          <div className="space-y-2">
            <Label htmlFor="notes">Notes</Label>
            <Textarea
              id="notes"
              placeholder="Optional context or TODOs for this query filter"
              value={mergedConfig.notes ?? ''}
              onChange={event => handleConfigChange({ notes: event.currentTarget.value })}
              rows={2}
            />
            <p className="text-sm text-muted-foreground">
              Optional context or TODOs for this query filter.
            </p>
          </div>
        </Stack>
      </CardContent>
    </Card>
  );
};
