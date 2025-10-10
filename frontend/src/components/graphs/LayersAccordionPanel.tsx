import React, { useMemo } from 'react';
import { Stack, Text, Group, Divider } from '@mantine/core';
import { Graph } from '../../graphql/graphs';
import { LayerListItem } from './LayerListItem';

interface LayersAccordionPanelProps {
  graph: Graph;
}

interface LayerStatistics {
  nodeCount: number;
  edgeCount: number;
}

export const LayersAccordionPanel: React.FC<LayersAccordionPanelProps> = ({ graph }) => {
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
            return (
              <LayerListItem
                key={layer.id}
                layer={layer}
                nodeCount={stats.nodeCount}
                edgeCount={stats.edgeCount}
              />
            );
          })
        )}
      </Stack>
    </Stack>
  );
};
