import React, { useState, useEffect, useRef } from 'react';
import { GraphNode, Layer } from '../../graphql/graphs';
import { IconCheck, IconAlertTriangle } from '@tabler/icons-react';
import { Stack } from '../layout-primitives';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Spinner } from '../ui/spinner';

interface NodePropertiesFormProps {
  node: GraphNode;
  layers: Layer[];
  // May be synchronous (optimistic) or return a Promise; when a Promise is
  // returned the save indicator reflects its actual settled state.
  onUpdate: (updates: Partial<GraphNode>) => void | Promise<unknown>;
}

export const NodePropertiesForm: React.FC<NodePropertiesFormProps> = ({
  node,
  layers,
  onUpdate,
}) => {
  const [label, setLabel] = useState(node.label || '');
  const [layer, setLayer] = useState<string | null>(node.layer || null);
  const [isSaving, setIsSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Reset form when node changes
  useEffect(() => {
    setLabel(node.label || '');
    setLayer(node.layer || null);
  }, [node.id, node.label, node.layer]);

  // Drive the save indicator off the actual result of onUpdate rather than a
  // fixed timeout, so a failed save is reported as failed (not "Saved").
  const runUpdate = (updates: Partial<GraphNode>) => {
    setIsSaving(true);
    setSaveError(null);
    Promise.resolve(onUpdate(updates))
      .then(() => {
        setLastSaved(new Date());
      })
      .catch((err) => {
        console.error('Failed to save node properties:', err);
        setSaveError(err instanceof Error ? err.message : 'Save failed');
      })
      .finally(() => {
        setIsSaving(false);
      });
  };

  // Keep the latest pending label edit available to the unmount flush so a
  // label typed but not yet blurred is not lost when the panel closes.
  const pendingLabelRef = useRef<{ label: string; base: string } | null>(null);
  useEffect(() => {
    pendingLabelRef.current = label !== (node.label || '') ? { label, base: node.label || '' } : null;
  }, [label, node.label]);
  useEffect(() => {
    return () => {
      const pending = pendingLabelRef.current;
      if (pending && pending.label !== pending.base) {
        // Fire-and-forget flush of the unsaved label on unmount.
        void Promise.resolve(onUpdate({ label: pending.label }));
      }
    };
    // Intentionally run only on unmount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleLabelBlur = () => {
    if (label !== node.label) {
      runUpdate({ label });
    }
  };

  const handleLayerChange = (value: string | null) => {
    setLayer(value);
    runUpdate({ layer: value || undefined });
  };

  // Build layer options with "None" as first option
  // Use special sentinel value to avoid conflicts with empty layer IDs
  const layerOptions = [
    { value: '__none__', label: 'None' },
    ...layers.map(l => ({
      value: l.layerId,
      label: l.name || l.layerId
    }))
  ];

  // Convert between internal representation and select value
  const selectValue = layer || '__none__';
  const handleSelectChange = (value: string | null) => {
    const actualValue = value === '__none__' ? null : value;
    handleLayerChange(actualValue);
  };

  return (
    <Stack gap="md">
      <div className="space-y-2">
        <Label htmlFor="label">Label</Label>
        <Input
          id="label"
          value={label}
          onChange={(e) => setLabel(e.currentTarget.value)}
          onBlur={handleLabelBlur}
          placeholder="Enter node label"
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="layer">Layer</Label>
        <Select value={selectValue} onValueChange={handleSelectChange}>
          <SelectTrigger id="layer">
            <SelectValue placeholder="Select layer" />
          </SelectTrigger>
          <SelectContent>
            {layerOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* Save indicator */}
      <div className="flex items-center gap-1.5 text-xs text-muted-foreground min-h-[20px]">
        {isSaving && (
          <>
            <Spinner className="h-3.5 w-3.5" />
            <span>Saving...</span>
          </>
        )}
        {!isSaving && saveError && (
          <>
            <IconAlertTriangle className="h-3.5 w-3.5 text-red-500" />
            <span className="text-red-500">Not saved: {saveError}</span>
          </>
        )}
        {!isSaving && !saveError && lastSaved && (
          <>
            <IconCheck className="h-3.5 w-3.5 text-green-500" />
            <span>
              Saved at {lastSaved.toLocaleTimeString()}
            </span>
          </>
        )}
      </div>
    </Stack>
  );
};
