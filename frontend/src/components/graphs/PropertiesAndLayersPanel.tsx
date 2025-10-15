import React from 'react';
import { Accordion, Paper, Button, Group } from '@mantine/core';
import { NodePropertiesForm } from './NodePropertiesForm';
import { LayersAccordionPanel } from './LayersAccordionPanel';
import { Graph, GraphNode } from '../../graphql/graphs';

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
  viewMode: 'flow' | 'hierarchy';
  onToggleViewMode: () => void;
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
      <Group justify="flex-end" mb="sm">
        <Button
          size="xs"
          variant="light"
          onClick={onToggleViewMode}
        >
          {viewMode === 'flow' ? 'Switch to Hierarchy' : 'Switch to Flow'}
        </Button>
      </Group>

      <Accordion
        multiple
        variant="separated"
        defaultValue={['node-properties']}
      >
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
