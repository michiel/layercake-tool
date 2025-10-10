import React from 'react';
import { Group, Stack, Text, Badge, Box } from '@mantine/core';
import { Layer } from '../../graphql/graphs';

interface LayerListItemProps {
  layer: Layer;
  nodeCount: number;
  edgeCount: number;
}

export const LayerListItem: React.FC<LayerListItemProps> = ({
  layer,
  nodeCount,
  edgeCount,
}) => {
  const backgroundColor = layer.properties?.background_color
    ? `#${layer.properties.background_color}`
    : '#f0f0f0';
  const borderColor = layer.properties?.border_color
    ? `#${layer.properties.border_color}`
    : '#999';
  const textColor = layer.properties?.text_color
    ? `#${layer.properties.text_color}`
    : '#000';

  return (
    <Box
      p="xs"
      style={{
        borderRadius: '4px',
        border: '1px solid #e9ecef',
        marginBottom: '8px',
      }}
    >
      <Stack gap="xs">
        <Group justify="space-between" align="center">
          <Group gap="sm">
            {/* Color swatches */}
            <Group gap={4}>
              <Box
                style={{
                  width: '20px',
                  height: '20px',
                  backgroundColor: backgroundColor,
                  border: `2px solid ${borderColor}`,
                  borderRadius: '3px',
                }}
                title="Background & Border"
              />
              <Box
                style={{
                  width: '20px',
                  height: '20px',
                  backgroundColor: textColor,
                  border: '1px solid #ddd',
                  borderRadius: '3px',
                }}
                title="Text color"
              />
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
