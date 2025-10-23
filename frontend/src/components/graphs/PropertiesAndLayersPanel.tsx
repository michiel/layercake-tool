import React from 'react';
import { Accordion, Paper, Group, Button, Slider, Stack, Text } from '@mantine/core';
import { NodePropertiesForm } from './NodePropertiesForm';
import { LayersAccordionPanel } from './LayersAccordionPanel';
import { Graph, GraphNode } from '../../graphql/graphs';
import { GraphViewMode, GraphOrientation, HierarchyViewMode } from './LayercakeGraphEditor';

interface PropertiesAndLayersPanelProps {
  graph: Graph;
  selectedNodeId: string | null;
  onNodeUpdate: (nodeId: string, updates: Partial<GraphNode>) => void;
  layerVisibility: Map<string, boolean>;
  onLayerVisibilityToggle: (layerId: string) => void;
  onShowAllLayers: () => void;
  onHideAllLayers: () => void;
  onLayerColorChange?: (layerId: string, colorType: 'background' | 'border' | 'text', color: string) => void;
  onAddLayer?: () => void;
  viewMode: GraphViewMode;
  onToggleViewMode: () => void;
  orientation: GraphOrientation;
  onToggleOrientation: () => void;
  flowGroupingEnabled: boolean;
  onToggleFlowGrouping: () => void;
  hierarchyViewMode: HierarchyViewMode;
  onToggleHierarchyViewMode: () => void;
  nodeSpacing: number;
  onNodeSpacingChange: (value: number) => void;
  rankSpacing: number;
  onRankSpacingChange: (value: number) => void;
  minEdgeLength: number;
  onMinEdgeLengthChange: (value: number) => void;
}

export const PropertiesAndLayersPanel: React.FC<PropertiesAndLayersPanelProps> = ({
  graph,
  selectedNodeId,
  onNodeUpdate,
  layerVisibility,
  onLayerVisibilityToggle,
  onShowAllLayers,
  onHideAllLayers,
  onLayerColorChange,
  onAddLayer,
  viewMode,
  onToggleViewMode,
  orientation,
  onToggleOrientation,
  flowGroupingEnabled,
  onToggleFlowGrouping,
  hierarchyViewMode,
  onToggleHierarchyViewMode,
  nodeSpacing,
  onNodeSpacingChange,
  rankSpacing,
  onRankSpacingChange,
  minEdgeLength,
  onMinEdgeLengthChange,
}) => {
  const selectedNode = selectedNodeId
    ? graph.graphNodes.find(n => n.id === selectedNodeId)
    : null;

  return (
    <Paper
      shadow="sm"
      p="md"
      style={{
        width: '320px',
        height: '100%',
        overflow: 'auto',
        borderLeft: '1px solid #e9ecef'
      }}
    >
      <Accordion
        multiple
        variant="separated"
        defaultValue={['add-nodes', 'layout-options', 'node-properties', 'layers']}
      >
        <Accordion.Item value="add-nodes">
          <Accordion.Control>Add Nodes</Accordion.Control>
          <Accordion.Panel>
            <Stack gap="xs">
              <Paper
                p="sm"
                style={{
                  cursor: 'grab',
                  border: '2px dashed #dee2e6',
                  backgroundColor: '#f8f9fa',
                  textAlign: 'center',
                  userSelect: 'none',
                }}
                onDragStart={(e) => {
                  e.dataTransfer.setData('application/reactflow', 'node');
                  e.dataTransfer.setData('nodeType', 'regular');
                  e.dataTransfer.effectAllowed = 'move';
                }}
                draggable
              >
                <Text size="sm" fw={500}>Node</Text>
                <Text size="xs" c="dimmed">Regular node</Text>
              </Paper>

              <Paper
                p="sm"
                style={{
                  cursor: 'grab',
                  border: '2px dashed #dee2e6',
                  backgroundColor: '#f8f9fa',
                  textAlign: 'center',
                  userSelect: 'none',
                }}
                onDragStart={(e) => {
                  e.dataTransfer.setData('application/reactflow', 'node');
                  e.dataTransfer.setData('nodeType', 'container');
                  e.dataTransfer.effectAllowed = 'move';
                }}
                draggable
              >
                <Text size="sm" fw={500}>Container</Text>
                <Text size="xs" c="dimmed">Partition node</Text>
              </Paper>
            </Stack>
          </Accordion.Panel>
        </Accordion.Item>

        <Accordion.Item value="layout-options">
          <Accordion.Control>Layout Options</Accordion.Control>
          <Accordion.Panel>
            <Stack gap="md">
              <Group gap="xs">
                <Button
                  size="xs"
                  variant="light"
                  onClick={onToggleViewMode}
                >
                  {viewMode === 'flow' ? 'Hierarchy' : 'Flow'}
                </Button>

                <Button
                  size="xs"
                  variant="light"
                  onClick={onToggleOrientation}
                >
                  {orientation === 'vertical' ? 'LR' : 'TD'}
                </Button>

                {viewMode === 'flow' && (
                  <Button
                    size="xs"
                    variant="light"
                    onClick={onToggleFlowGrouping}
                  >
                    {flowGroupingEnabled ? 'Disable Groupings' : 'Enable Groupings'}
                  </Button>
                )}

                {viewMode === 'hierarchy' && (
                  <Button
                    size="xs"
                    variant="light"
                    onClick={onToggleHierarchyViewMode}
                  >
                    {hierarchyViewMode === 'graph' ? 'As Containers' : 'As Graph'}
                  </Button>
                )}
              </Group>

              <div>
                <Group justify="space-between" mb={4}>
                  <Text size="xs" c="dimmed">Node Spacing</Text>
                  <Text size="xs" fw={500}>{nodeSpacing}</Text>
                </Group>
                <Slider
                  value={nodeSpacing}
                  onChange={onNodeSpacingChange}
                  min={20}
                  max={200}
                  step={5}
                  size="sm"
                />
              </div>

              <div>
                <Group justify="space-between" mb={4}>
                  <Text size="xs" c="dimmed">Rank Spacing</Text>
                  <Text size="xs" fw={500}>{rankSpacing}</Text>
                </Group>
                <Slider
                  value={rankSpacing}
                  onChange={onRankSpacingChange}
                  min={20}
                  max={200}
                  step={5}
                  size="sm"
                />
              </div>

              <div>
                <Group justify="space-between" mb={4}>
                  <Text size="xs" c="dimmed">Min Edge Length</Text>
                  <Text size="xs" fw={500}>{minEdgeLength}</Text>
                </Group>
                <Slider
                  value={minEdgeLength}
                  onChange={onMinEdgeLengthChange}
                  min={20}
                  max={200}
                  step={5}
                  size="sm"
                />
              </div>
            </Stack>
          </Accordion.Panel>
        </Accordion.Item>

        <Accordion.Item value="node-properties">
          <Accordion.Control>Node Properties</Accordion.Control>
          <Accordion.Panel>
            {selectedNode ? (
              <NodePropertiesForm
                node={selectedNode}
                layers={graph.layers}
                onUpdate={(updates) => onNodeUpdate(selectedNode.id, updates)}
              />
            ) : (
              <div style={{ color: '#868e96', fontSize: '14px', textAlign: 'center', padding: '20px 0' }}>
                Select a node to view properties
              </div>
            )}
          </Accordion.Panel>
        </Accordion.Item>

        <Accordion.Item value="layers">
          <Accordion.Control>Layers</Accordion.Control>
          <Accordion.Panel>
            <LayersAccordionPanel
              graph={graph}
              layerVisibility={layerVisibility}
              onLayerVisibilityToggle={onLayerVisibilityToggle}
              onShowAllLayers={onShowAllLayers}
              onHideAllLayers={onHideAllLayers}
              onLayerColorChange={onLayerColorChange}
              onAddLayer={onAddLayer}
            />
          </Accordion.Panel>
        </Accordion.Item>
      </Accordion>
    </Paper>
  );
};
