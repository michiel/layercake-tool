import React, { useState } from 'react';
import { Accordion, Paper, ActionIcon, Tooltip } from '@mantine/core';
import { IconChevronLeft, IconChevronRight } from '@tabler/icons-react';
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
}) => {
  const [collapsed, setCollapsed] = useState(false);

  const selectedNode = selectedNodeId
    ? graph.graphNodes.find(n => n.id === selectedNodeId)
    : null;

  if (collapsed) {
    return (
      <Tooltip label="Show Properties Panel" position="left">
        <ActionIcon
          variant="filled"
          size="lg"
          onClick={() => setCollapsed(false)}
          style={{
            position: 'absolute',
            top: '8px',
            right: '8px',
            zIndex: 1000,
            boxShadow: '0 2px 8px rgba(0, 0, 0, 0.15)'
          }}
        >
          <IconChevronLeft size={18} />
        </ActionIcon>
      </Tooltip>
    );
  }

  return (
    <Paper
      shadow="sm"
      p="md"
      style={{
        width: '320px',
        height: '100%',
        overflow: 'auto',
        borderLeft: '1px solid #e9ecef',
        position: 'relative'
      }}
    >
      <Tooltip label="Hide Properties Panel" position="left">
        <ActionIcon
          variant="filled"
          size="lg"
          onClick={() => setCollapsed(true)}
          style={{
            position: 'absolute',
            top: '8px',
            right: '8px',
            zIndex: 10,
            boxShadow: '0 2px 8px rgba(0, 0, 0, 0.15)'
          }}
        >
          <IconChevronRight size={18} />
        </ActionIcon>
      </Tooltip>

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
            />
          </Accordion.Panel>
        </Accordion.Item>
      </Accordion>
    </Paper>
  );
};
