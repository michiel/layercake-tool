import React, { useState } from 'react';
import { Tabs, Stack, Button, Group, Text, Alert, Table, TextInput, ScrollArea } from '@mantine/core';
import { IconTable, IconDeviceFloppy, IconAlertCircle } from '@tabler/icons-react';

export interface GraphNode {
  id: string;
  label: string;
  layer?: string;
  is_partition?: boolean;
  belongs_to?: string;
  comment?: string;
  [key: string]: any;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  comment?: string;
  [key: string]: any;
}

export interface GraphLayer {
  id: string;
  label: string;
  background_color?: string;
  text_color?: string;
  border_color?: string;
  [key: string]: any;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
  layers: GraphLayer[];
}

interface GraphSpreadsheetEditorProps {
  graphData: GraphData;
  onSave: (graphData: GraphData) => Promise<void>;
  readOnly?: boolean;
}

export const GraphSpreadsheetEditor: React.FC<GraphSpreadsheetEditorProps> = ({
  graphData,
  onSave,
  readOnly = false
}) => {
  const [activeTab, setActiveTab] = useState<string | null>('nodes');
  const [localNodes, setLocalNodes] = useState<GraphNode[]>(graphData.nodes || []);
  const [localEdges, setLocalEdges] = useState<GraphEdge[]>(graphData.edges || []);
  const [localLayers, setLocalLayers] = useState<GraphLayer[]>(graphData.layers || []);
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);

  const nodeColumnDefs = ['id', 'label', 'layer', 'is_partition', 'belongs_to', 'comment'];
  const edgeColumnDefs = ['id', 'source', 'target', 'label', 'layer', 'comment'];
  const layerColumnDefs = ['id', 'label', 'background_color', 'text_color', 'border_color'];

  const handleNodeChange = (rowIdx: number, field: string, value: string) => {
    setLocalNodes(prevNodes => {
      const newNodes = [...prevNodes];
      newNodes[rowIdx] = {
        ...newNodes[rowIdx],
        [field]: value
      };
      return newNodes;
    });
    setHasChanges(true);
  };

  const handleEdgeChange = (rowIdx: number, field: string, value: string) => {
    setLocalEdges(prevEdges => {
      const newEdges = [...prevEdges];
      newEdges[rowIdx] = {
        ...newEdges[rowIdx],
        [field]: value
      };
      return newEdges;
    });
    setHasChanges(true);
  };

  const handleLayerChange = (rowIdx: number, field: string, value: string) => {
    setLocalLayers(prevLayers => {
      const newLayers = [...prevLayers];
      newLayers[rowIdx] = {
        ...newLayers[rowIdx],
        [field]: value
      };
      return newLayers;
    });
    setHasChanges(true);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await onSave({
        nodes: localNodes,
        edges: localEdges,
        layers: localLayers
      });
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to save graph data:', error);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Stack gap="md">
      {hasChanges && !readOnly && (
        <Alert icon={<IconAlertCircle size={16} />} color="blue">
          You have unsaved changes
        </Alert>
      )}

      <Group justify="space-between">
        <Text size="sm" c="dimmed">
          {activeTab === 'nodes' && `${localNodes.length} nodes`}
          {activeTab === 'edges' && `${localEdges.length} edges`}
          {activeTab === 'layers' && `${localLayers.length} layers`}
        </Text>

        {!readOnly && (
          <Button
            leftSection={<IconDeviceFloppy size={16} />}
            onClick={handleSave}
            disabled={!hasChanges}
            loading={saving}
          >
            Save Changes
          </Button>
        )}
      </Group>

      <Tabs value={activeTab} onChange={setActiveTab}>
        <Tabs.List>
          <Tabs.Tab value="nodes" leftSection={<IconTable size={16} />}>
            Nodes ({localNodes.length})
          </Tabs.Tab>
          <Tabs.Tab value="edges" leftSection={<IconTable size={16} />}>
            Edges ({localEdges.length})
          </Tabs.Tab>
          <Tabs.Tab value="layers" leftSection={<IconTable size={16} />}>
            Layers ({localLayers.length})
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="nodes" pt="md">
          <ScrollArea h={600}>
            <Table striped highlightOnHover withTableBorder withColumnBorders>
              <Table.Thead>
                <Table.Tr>
                  {nodeColumnDefs.map(col => (
                    <Table.Th key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </Table.Th>
                  ))}
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {localNodes.map((node, rowIdx) => (
                  <Table.Tr key={rowIdx}>
                    {nodeColumnDefs.map(col => (
                      <Table.Td key={col}>
                        {readOnly ? (
                          <Text size="sm">{String(node[col] ?? '')}</Text>
                        ) : (
                          <TextInput
                            value={String(node[col] ?? '')}
                            onChange={(e) => handleNodeChange(rowIdx, col, e.currentTarget.value)}
                            size="xs"
                            styles={{ input: { border: 'none', padding: '4px 8px' } }}
                          />
                        )}
                      </Table.Td>
                    ))}
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </ScrollArea>
        </Tabs.Panel>

        <Tabs.Panel value="edges" pt="md">
          <ScrollArea h={600}>
            <Table striped highlightOnHover withTableBorder withColumnBorders>
              <Table.Thead>
                <Table.Tr>
                  {edgeColumnDefs.map(col => (
                    <Table.Th key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </Table.Th>
                  ))}
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {localEdges.map((edge, rowIdx) => (
                  <Table.Tr key={rowIdx}>
                    {edgeColumnDefs.map(col => (
                      <Table.Td key={col}>
                        {readOnly ? (
                          <Text size="sm">{String(edge[col] ?? '')}</Text>
                        ) : (
                          <TextInput
                            value={String(edge[col] ?? '')}
                            onChange={(e) => handleEdgeChange(rowIdx, col, e.currentTarget.value)}
                            size="xs"
                            styles={{ input: { border: 'none', padding: '4px 8px' } }}
                          />
                        )}
                      </Table.Td>
                    ))}
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </ScrollArea>
        </Tabs.Panel>

        <Tabs.Panel value="layers" pt="md">
          <ScrollArea h={600}>
            <Table striped highlightOnHover withTableBorder withColumnBorders>
              <Table.Thead>
                <Table.Tr>
                  {layerColumnDefs.map(col => (
                    <Table.Th key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </Table.Th>
                  ))}
                </Table.Tr>
              </Table.Thead>
              <Table.Tbody>
                {localLayers.map((layer, rowIdx) => (
                  <Table.Tr key={rowIdx}>
                    {layerColumnDefs.map(col => (
                      <Table.Td key={col}>
                        {readOnly ? (
                          <Text size="sm">{String(layer[col] ?? '')}</Text>
                        ) : (
                          <TextInput
                            value={String(layer[col] ?? '')}
                            onChange={(e) => handleLayerChange(rowIdx, col, e.currentTarget.value)}
                            size="xs"
                            styles={{ input: { border: 'none', padding: '4px 8px' } }}
                          />
                        )}
                      </Table.Td>
                    ))}
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </ScrollArea>
        </Tabs.Panel>
      </Tabs>
    </Stack>
  );
};
