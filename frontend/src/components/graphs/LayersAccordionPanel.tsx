import React, { useMemo } from 'react';
import { IconEye, IconEyeOff, IconPlus } from '@tabler/icons-react';
import { Graph } from '../../graphql/graphs';
import { LayerListItem } from './LayerListItem';
import { Stack, Group } from '../layout-primitives';
import { Button } from '../ui/button';
import { Separator } from '../ui/separator';
import { Tooltip, TooltipContent, TooltipTrigger, TooltipProvider } from '../ui/tooltip';

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
    <TooltipProvider>
      <Stack gap="md">
        {/* Summary statistics */}
        <div className="p-2 bg-muted rounded">
          <Group gap="lg">
            <div>
              <p className="text-xs text-muted-foreground">Layers</p>
              <p className="text-sm font-semibold">{totalLayers}</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">Nodes</p>
              <p className="text-sm font-semibold">{totalNodes}</p>
            </div>
            <div>
              <p className="text-xs text-muted-foreground">Edges</p>
              <p className="text-sm font-semibold">{totalEdges}</p>
            </div>
          </Group>
        </div>

        {/* Bulk actions */}
        <Group gap="xs">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="secondary"
                size="icon"
                onClick={onShowAllLayers}
              >
                <IconEye className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Show all layers</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="secondary"
                size="icon"
                onClick={onHideAllLayers}
              >
                <IconEyeOff className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>Hide all layers</TooltipContent>
          </Tooltip>
          {onAddLayer && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="secondary"
                  size="icon"
                  onClick={onAddLayer}
                  className="text-green-600"
                >
                  <IconPlus className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Add layer</TooltipContent>
            </Tooltip>
          )}
        </Group>

        <Separator />

        {/* Layer list */}
        <Stack gap="xs">
          {graph.layers.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-4">
              No layers defined
            </p>
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
    </TooltipProvider>
  );
};
