import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import { MergeNodeConfig } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

interface MergeNodeConfigFormProps {
  config: MergeNodeConfig;
  setConfig: (config: MergeNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const MergeNodeConfigForm: React.FC<MergeNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<MergeNodeConfig>({
    ...config,
    mergeStrategy: config.mergeStrategy || 'Union',
    conflictResolution: config.conflictResolution || 'PreferFirst',
  });

  useEffect(() => {
    setConfig(localConfig);
  }, [localConfig, setConfig]);

  useEffect(() => {
    // Always valid - connections handled by edges
    setIsValid(true);
  }, [localConfig, setIsValid]);

  return (
    <Stack gap="md">
      <Alert>
        <IconInfoCircle className="h-4 w-4" />
        <AlertTitle>Merge Configuration</AlertTitle>
        <AlertDescription>
          Configure merge behavior. Inputs and output are determined by edge connections in the DAG.
        </AlertDescription>
      </Alert>

      <div className="space-y-2">
        <Label htmlFor="merge-strategy">Merge Strategy</Label>
        <Select
          value={localConfig.mergeStrategy}
          onValueChange={(value) => setLocalConfig(prev => ({ ...prev, mergeStrategy: value as any }))}
        >
          <SelectTrigger id="merge-strategy">
            <SelectValue placeholder="Select merge strategy" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="Union">Union</SelectItem>
            <SelectItem value="Intersection">Intersection</SelectItem>
            <SelectItem value="Difference">Difference</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </Stack>
  );
};