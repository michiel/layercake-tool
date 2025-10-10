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
      setIsSaving(true);
      onUpdate({ label });
      // Simulate save completion
      setTimeout(() => {
        setIsSaving(false);
        setLastSaved(new Date());
      }, 300);
    }
  };

  const handleLayerChange = (value: string | null) => {
    setLayer(value);
    setIsSaving(true);
    onUpdate({ layer: value || undefined });
    // Simulate save completion
    setTimeout(() => {
      setIsSaving(false);
      setLastSaved(new Date());
    }, 300);
  };

  // Build layer options with "None" as first option
  const layerOptions = [
    { value: '', label: 'None' },
    ...layers.map(l => ({
      value: l.layerId,
      label: l.name || l.layerId
    }))
  ];

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
        value={layer || ''}
        onChange={handleLayerChange}
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
