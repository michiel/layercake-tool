import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Stack, Alert, Text } from '@mantine/core';
import { IconInfoCircle } from '@tabler/icons-react';
import { FilterNodeConfig, QueryFilterConfig } from '../../../../types/plan-dag';
import {
  QueryFilterBuilder,
} from './QueryFilterBuilder';
import { extractQueryConfigFromRaw } from './filterConfigUtils';

interface FilterNodeConfigFormProps {
  config: FilterNodeConfig;
  setConfig: (config: FilterNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

const configsEqual = (a: QueryFilterConfig, b: QueryFilterConfig): boolean =>
  JSON.stringify(a) === JSON.stringify(b);

export const FilterNodeConfigForm: React.FC<FilterNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const initialQuery = useMemo(() => extractQueryConfigFromRaw(config), [config]);
  const [localQueryConfig, setLocalQueryConfig] = useState<QueryFilterConfig>(initialQuery);
  const lastSentConfigRef = useRef<QueryFilterConfig>(initialQuery);

  // Sync when a new node is selected / config is externally updated
  useEffect(() => {
    const nextQuery = extractQueryConfigFromRaw(config);
    if (!configsEqual(nextQuery, localQueryConfig)) {
      setLocalQueryConfig(nextQuery);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config]);

  // Persist upstream when local state changes
  useEffect(() => {
    if (!configsEqual(localQueryConfig, lastSentConfigRef.current)) {
      setConfig({ query: localQueryConfig });
      lastSentConfigRef.current = localQueryConfig;
    }
  }, [localQueryConfig, setConfig]);

  useEffect(() => {
    setIsValid(true);
  }, [setIsValid]);

  return (
    <Stack gap="md">
      <Alert icon={<IconInfoCircle size="1rem" />} color="blue" title="Query Filter">
        <Text size="sm">
          This Filter node runs a single query builder rule-set against the upstream graph. Adjust the
          targets, match mode, pruning behavior, and rule groups below.
        </Text>
      </Alert>

      <QueryFilterBuilder value={localQueryConfig} onChange={setLocalQueryConfig} />
    </Stack>
  );
};
