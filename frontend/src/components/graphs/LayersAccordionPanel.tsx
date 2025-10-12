import React, { useMemo } from 'react';
import { Stack, Text, Group, Divider, ActionIcon, Tooltip } from '@mantine/core';
import { IconEye, IconEyeOff, IconPlus } from '@tabler/icons-react';
import { Graph } from '../../graphql/graphs';
import { LayerListItem } from './LayerListItem';

interface LayersAccordionPanelProps {
  graph: Graph;
  layerVisibility: Map<string, boolean>;
  onLayerVisibilityToggle: (layerId: string) => void;
  onShowAllLayers: () => void;
  onHideAllLayers: () => void;
  onLayerColorChange?: (layerId: string, colorType: 'background' | 'border' | 'text', color: string) => void;
  onAddLayer?: () => void;
}

interface LayerStatistics {
  nodeCount: number;
  edgeCount: number;
}

export const LayersAccordionPanel: React.FC<LayersAccordionPanelProps> = ({
  graph,
  layerVisibility,
  onLayerVisibilityToggle,
  onShowAllLayers,
  onHideAllLayers,
  onLayerColorChange,
  onAddLayer,
}) => {
  // Calculate statistics for each layer
  const layerStats = useMemo(() => {
    const stats = new Map<string, LayerStatistics>();

    // Initialize stats for all layers
    graph.layers.forEach(layer => {
      stats.set(layer.layerId, { nodeCount: 0, edgeCount: 0 });
    });

    // Count nodes per layer
    graph.graphNodes.forEach(node => {
      if (node.layer) {
        const stat = stats.get(node.layer);
        if (stat) {
          stat.nodeCount++;
        } else {
          // Layer not in graph.layers but node references it
          stats.set(node.layer, { nodeCount: 1, edgeCount: 0 });
        }
      }
    });

    // Count edges per layer
    graph.graphEdges.forEach(edge => {
      if (edge.layer) {
        const stat = stats.get(edge.layer);
        if (stat) {
          stat.edgeCount++;
        } else {
          // Layer not in graph.layers but edge references it
          stats.set(edge.layer, { nodeCount: 0, edgeCount: 1 });
        }
      }
    });

    return stats;
  }, [graph.graphNodes, graph.graphEdges, graph.layers]);

  // Calculate totals
  const totalNodes = graph.nodeCount || graph.graphNodes.length;
  const totalEdges = graph.edgeCount || graph.graphEdges.length;
  const totalLayers = graph.layers.length;

  return (
    <Stack gap="md">
      {/* Summary statistics */}
      <Group justify="space-between" p="xs" style={{ backgroundColor: '#f8f9fa', borderRadius: '4px' }}>
        <Group gap="lg">
          <div>
            <Text size="xs" c="dimmed">Layers</Text>
            <Text size="sm" fw={600}>{totalLayers}</Text>
          </div>
          <div>
            <Text size="xs" c="dimmed">Nodes</Text>
            <Text size="sm" fw={600}>{totalNodes}</Text>
          </div>
          <div>
            <Text size="xs" c="dimmed">Edges</Text>
            <Text size="sm" fw={600}>{totalEdges}</Text>
          </div>
        </Group>
      </Group>

      {/* Bulk actions */}
      <Group gap="xs">
        <Tooltip label="Show all layers">
          <ActionIcon
            variant="light"
            color="blue"
            onClick={onShowAllLayers}
          >
            <IconEye size={16} />
          </ActionIcon>
        </Tooltip>
        <Tooltip label="Hide all layers">
          <ActionIcon
            variant="light"
            color="blue"
            onClick={onHideAllLayers}
          >
            <IconEyeOff size={16} />
          </ActionIcon>
        </Tooltip>
        {onAddLayer && (
          <Tooltip label="Add layer">
            <ActionIcon
              variant="light"
              color="green"
              onClick={onAddLayer}
            >
              <IconPlus size={16} />
            </ActionIcon>
          </Tooltip>
        )}
      </Group>

      <Divider />

      {/* Layer list */}
      <Stack gap="xs">
        {graph.layers.length === 0 ? (
          <Text size="sm" c="dimmed" ta="center" py="md">
            No layers defined
          </Text>
        ) : (
          graph.layers.map(layer => {
            const stats = layerStats.get(layer.layerId) || { nodeCount: 0, edgeCount: 0 };
            const isVisible = layerVisibility.get(layer.layerId) ?? true;
            return (
              <LayerListItem
                key={layer.id}
                layer={layer}
                nodeCount={stats.nodeCount}
                edgeCount={stats.edgeCount}
                isVisible={isVisible}
                onVisibilityToggle={() => onLayerVisibilityToggle(layer.layerId)}
                onColorChange={onLayerColorChange}
              />
            );
          })
        )}
      </Stack>
    </Stack>
  );
};
