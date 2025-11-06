import React, { useState, useEffect } from 'react';
import { GraphNode, Layer } from '../../graphql/graphs';
import { IconCheck } from '@tabler/icons-react';
import { Stack } from '../layout-primitives';
import { Input } from '../ui/input';
import { Label } from '../ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Spinner } from '../ui/spinner';

interface NodePropertiesFormProps {
  node: GraphNode;
  layers: Layer[];
  onUpdate: (updates: Partial<GraphNode>) => void;
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

  // Reset form when node changes
  useEffect(() => {
    setLabel(node.label || '');
    setLayer(node.layer || null);
  }, [node.id, node.label, node.layer]);

  const handleLabelBlur = () => {
    if (label !== node.label) {
      // Update happens optimistically in parent, just track save state
      setIsSaving(true);
      onUpdate({ label });
      // Mark as saved immediately since update is optimistic
      setTimeout(() => {
        setIsSaving(false);
        setLastSaved(new Date());
      }, 100);
    }
  };

  const handleLayerChange = (value: string | null) => {
    setLayer(value);
    // Update happens optimistically in parent, just track save state
    setIsSaving(true);
    onUpdate({ layer: value || undefined });
    // Mark as saved immediately since update is optimistic
    setTimeout(() => {
      setIsSaving(false);
      setLastSaved(new Date());
    }, 100);
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
        {!isSaving && lastSaved && (
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
