import React, { useState } from 'react';
import { Group, Stack, Text, Badge, Box, ActionIcon, Popover, ColorPicker } from '@mantine/core';
import { IconEye, IconEyeOff } from '@tabler/icons-react';
import { Layer } from '../../graphql/graphs';

interface LayerListItemProps {
  layer: Layer;
  nodeCount: number;
  edgeCount: number;
  isVisible: boolean;
  onVisibilityToggle: () => void;
  onColorChange?: (layerId: string, colorType: 'background' | 'border' | 'text', color: string) => void;
}

export const LayerListItem: React.FC<LayerListItemProps> = ({
  layer,
  nodeCount,
  edgeCount,
  isVisible,
  onVisibilityToggle,
  onColorChange,
}) => {
  const [bgPickerOpen, setBgPickerOpen] = useState(false);
  const [borderPickerOpen, setBorderPickerOpen] = useState(false);
  const [textPickerOpen, setTextPickerOpen] = useState(false);

  const backgroundColor = layer.properties?.background_color
    ? `#${layer.properties.background_color}`
    : '#f0f0f0';
  const borderColor = layer.properties?.border_color
    ? `#${layer.properties.border_color}`
    : '#999';
  const textColor = layer.properties?.text_color
    ? `#${layer.properties.text_color}`
    : '#000';

  const handleColorChange = (colorType: 'background' | 'border' | 'text', color: string) => {
    if (onColorChange) {
      // Remove # prefix for storage
      const hexColor = color.replace('#', '');
      onColorChange(layer.layerId, colorType, hexColor);
    }
  };

  return (
    <Box
      p="xs"
      style={{
        borderRadius: '4px',
        border: '1px solid #e9ecef',
        marginBottom: '8px',
        opacity: isVisible ? 1 : 0.5,
        backgroundColor: isVisible ? 'white' : '#f8f9fa',
      }}
    >
      <Stack gap="xs">
        <Group justify="space-between" align="center">
          <Group gap="sm">
            {/* Visibility toggle */}
            <ActionIcon
              variant="subtle"
              color="gray"
              size="sm"
              onClick={onVisibilityToggle}
              title={isVisible ? 'Hide layer' : 'Show layer'}
            >
              {isVisible ? <IconEye size={16} /> : <IconEyeOff size={16} />}
            </ActionIcon>

            {/* Color swatches with pickers */}
            <Group gap={4}>
              {/* Background & Border color picker */}
              <Popover opened={bgPickerOpen} onChange={setBgPickerOpen} position="bottom" withArrow>
                <Popover.Target>
                  <Box
                    onClick={() => setBgPickerOpen(!bgPickerOpen)}
                    style={{
                      width: '20px',
                      height: '20px',
                      backgroundColor: backgroundColor,
                      border: `2px solid ${borderColor}`,
                      borderRadius: '3px',
                      cursor: 'pointer',
                    }}
                    title="Click to change background color"
                  />
                </Popover.Target>
                <Popover.Dropdown>
                  <Stack gap="xs">
                    <Text size="xs" fw={500}>Background Color</Text>
                    <ColorPicker
                      format="hex"
                      value={backgroundColor}
                      onChange={(color) => handleColorChange('background', color)}
                    />
                  </Stack>
                </Popover.Dropdown>
              </Popover>

              {/* Border color picker */}
              <Popover opened={borderPickerOpen} onChange={setBorderPickerOpen} position="bottom" withArrow>
                <Popover.Target>
                  <Box
                    onClick={() => setBorderPickerOpen(!borderPickerOpen)}
                    style={{
                      width: '20px',
                      height: '20px',
                      backgroundColor: borderColor,
                      border: '1px solid #ddd',
                      borderRadius: '3px',
                      cursor: 'pointer',
                    }}
                    title="Click to change border color"
                  />
                </Popover.Target>
                <Popover.Dropdown>
                  <Stack gap="xs">
                    <Text size="xs" fw={500}>Border Color</Text>
                    <ColorPicker
                      format="hex"
                      value={borderColor}
                      onChange={(color) => handleColorChange('border', color)}
                    />
                  </Stack>
                </Popover.Dropdown>
              </Popover>

              {/* Text color picker */}
              <Popover opened={textPickerOpen} onChange={setTextPickerOpen} position="bottom" withArrow>
                <Popover.Target>
                  <Box
                    onClick={() => setTextPickerOpen(!textPickerOpen)}
                    style={{
                      width: '20px',
                      height: '20px',
                      backgroundColor: textColor,
                      border: '1px solid #ddd',
                      borderRadius: '3px',
                      cursor: 'pointer',
                    }}
                    title="Click to change text color"
                  />
                </Popover.Target>
                <Popover.Dropdown>
                  <Stack gap="xs">
                    <Text size="xs" fw={500}>Text Color</Text>
                    <ColorPicker
                      format="hex"
                      value={textColor}
                      onChange={(color) => handleColorChange('text', color)}
                    />
                  </Stack>
                </Popover.Dropdown>
              </Popover>
            </Group>

            <Text size="sm" fw={500}>
              {layer.name || layer.layerId}
            </Text>
          </Group>

          {/* Statistics badges */}
          <Group gap={6}>
            <Badge size="xs" variant="light" color="blue">
              {nodeCount}N
            </Badge>
            <Badge size="xs" variant="light" color="grape">
              {edgeCount}E
            </Badge>
          </Group>
        </Group>

        {/* Layer ID if different from name */}
        {layer.name && layer.name !== layer.layerId && (
          <Text size="xs" c="dimmed" style={{ fontFamily: 'monospace' }}>
            {layer.layerId}
          </Text>
        )}
      </Stack>
    </Box>
  );
};
