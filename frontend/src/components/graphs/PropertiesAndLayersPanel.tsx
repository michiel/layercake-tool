import React from 'react';
import { Accordion, Paper } from '@mantine/core';
import { NodePropertiesForm } from './NodePropertiesForm';
import { Graph, GraphNode } from '../../graphql/graphs';

interface PropertiesAndLayersPanelProps {
  graph: Graph;
  selectedNodeId: string | null;
  onNodeUpdate: (nodeId: string, updates: Partial<GraphNode>) => void;
}

export const PropertiesAndLayersPanel: React.FC<PropertiesAndLayersPanelProps> = ({
  graph,
  selectedNodeId,
  onNodeUpdate,
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
            <div style={{ color: '#868e96', fontSize: '14px', textAlign: 'center', padding: '20px 0' }}>
              Layers panel coming soon
            </div>
          </Accordion.Panel>
        </Accordion.Item>
      </Accordion>
    </Paper>
  );
};
