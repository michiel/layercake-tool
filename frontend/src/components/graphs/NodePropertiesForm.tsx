import React, { useState, useEffect } from 'react';
import { TextInput, Select, Stack, Text, Loader } from '@mantine/core';
import { GraphNode, Layer } from '../../graphql/graphs';
import { IconCheck } from '@tabler/icons-react';

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
      <TextInput
        label="Label"
        value={label}
        onChange={(e) => setLabel(e.currentTarget.value)}
        onBlur={handleLabelBlur}
        placeholder="Enter node label"
      />

      <Select
        label="Layer"
        value={selectValue}
        onChange={handleSelectChange}
        data={layerOptions}
        placeholder="Select layer"
        clearable={false}
      />

      {/* Save indicator */}
      <div style={{
        display: 'flex',
        alignItems: 'center',
        gap: '6px',
        fontSize: '12px',
        color: '#868e96',
        minHeight: '20px'
      }}>
        {isSaving && (
          <>
            <Loader size="xs" />
            <Text size="xs">Saving...</Text>
          </>
        )}
        {!isSaving && lastSaved && (
          <>
            <IconCheck size={14} style={{ color: '#51cf66' }} />
            <Text size="xs" c="dimmed">
              Saved at {lastSaved.toLocaleTimeString()}
            </Text>
          </>
        )}
      </div>
    </Stack>
  );
};
