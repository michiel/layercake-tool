import React, { useEffect, useState } from 'react';
import { IconInfoCircle } from '@tabler/icons-react';
import { CopyNodeConfig } from '../../../../types/plan-dag';
import { Stack } from '@/components/layout-primitives';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';

interface CopyNodeConfigFormProps {
  config: CopyNodeConfig;
  setConfig: (config: CopyNodeConfig) => void;
  setIsValid: (isValid: boolean) => void;
  projectId: number;
}

export const CopyNodeConfigForm: React.FC<CopyNodeConfigFormProps> = ({
  config,
  setConfig,
  setIsValid,
  projectId: _projectId,
}) => {
  const [localConfig, setLocalConfig] = useState<CopyNodeConfig>({
    ...config,
    copyType: config.copyType || 'DeepCopy',
    preserveMetadata: config.preserveMetadata ?? true,
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
        <AlertTitle>Copy Configuration</AlertTitle>
        <AlertDescription>
          Configure copy behavior. Source and output are determined by edge connections in the DAG.
        </AlertDescription>
      </Alert>

      <div className="space-y-2">
        <Label htmlFor="copy-type">Copy Type</Label>
        <Select
          value={localConfig.copyType}
          onValueChange={(value) => setLocalConfig(prev => ({ ...prev, copyType: value as any }))}
        >
          <SelectTrigger id="copy-type">
            <SelectValue placeholder="Select copy type" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="DeepCopy">Deep Copy</SelectItem>
            <SelectItem value="ShallowCopy">Shallow Copy</SelectItem>
            <SelectItem value="Reference">Reference</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="flex items-center space-x-2">
        <Switch
          id="preserve-metadata"
          checked={localConfig.preserveMetadata}
          onCheckedChange={(checked) => setLocalConfig(prev => ({ ...prev, preserveMetadata: checked }))}
        />
        <Label htmlFor="preserve-metadata">Preserve Metadata</Label>
      </div>
    </Stack>
  );
};