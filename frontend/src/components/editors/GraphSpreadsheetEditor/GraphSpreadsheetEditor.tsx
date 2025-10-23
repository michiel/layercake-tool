import React, { useState } from 'react';
import { Tabs, Stack, Button, Group, Text, Table, TextInput, ScrollArea } from '@mantine/core';
import { IconTable, IconDeviceFloppy, IconClipboard } from '@tabler/icons-react';

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
  comment?: string;
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
  const layerColumnDefs = ['id', 'label', 'background_color', 'text_color', 'border_color', 'comment'];

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

  // Parse CSV from clipboard
  const parseCSV = (csvText: string): Record<string, any>[] => {
    const lines = csvText.trim().split('\n');
    if (lines.length < 2) return [];

    // Parse header
    const headers = lines[0].split(/,(?=(?:(?:[^"]*"){2})*[^"]*$)/).map(h =>
      h.trim().replace(/^"|"$/g, '')
    );

    // Parse rows
    const rows: Record<string, any>[] = [];
    for (let i = 1; i < lines.length; i++) {
      const values = lines[i].split(/,(?=(?:(?:[^"]*"){2})*[^"]*$)/).map(v =>
        v.trim().replace(/^"|"$/g, '')
      );

      if (values.length !== headers.length) continue;

      const row: Record<string, any> = {};
      headers.forEach((header, idx) => {
        row[header] = values[idx];
      });
      rows.push(row);
    }

    return rows;
  };

  const handlePasteNodes = async () => {
    try {
      const text = await navigator.clipboard.readText();
      const parsedData = parseCSV(text);

      if (parsedData.length === 0) {
        alert('No valid CSV data found in clipboard');
        return;
      }

      const newNodes: GraphNode[] = parsedData.map(row => ({
        id: row.id || '',
        label: row.label || '',
        layer: row.layer,
        is_partition: row.is_partition === 'true' || row.is_partition === '1',
        belongs_to: row.belongs_to,
        comment: row.comment,
        ...row
      }));

      setLocalNodes(newNodes);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste nodes:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  const handlePasteEdges = async () => {
    try {
      const text = await navigator.clipboard.readText();
      const parsedData = parseCSV(text);

      if (parsedData.length === 0) {
        alert('No valid CSV data found in clipboard');
        return;
      }

      const newEdges: GraphEdge[] = parsedData.map(row => ({
        id: row.id || '',
        source: row.source || '',
        target: row.target || '',
        label: row.label,
        layer: row.layer,
        comment: row.comment,
        ...row
      }));

      setLocalEdges(newEdges);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste edges:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  const handlePasteLayers = async () => {
    try {
      const text = await navigator.clipboard.readText();
      const parsedData = parseCSV(text);

      if (parsedData.length === 0) {
        alert('No valid CSV data found in clipboard');
        return;
      }

      const newLayers: GraphLayer[] = parsedData.map(row => ({
        id: row.id || '',
        label: row.label || '',
        background_color: row.background_color,
        text_color: row.text_color,
        border_color: row.border_color,
        comment: row.comment,
        ...row
      }));

      setLocalLayers(newLayers);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste layers:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  return (
    <Stack gap="md">
      <Group justify="space-between">
        <Text size="sm" c="dimmed">
          {activeTab === 'nodes' && `${localNodes.length} nodes`}
          {activeTab === 'edges' && `${localEdges.length} edges`}
          {activeTab === 'layers' && `${localLayers.length} layers`}
        </Text>

        <Group gap="xs">
          {!readOnly && (
            <>
              {activeTab === 'nodes' && (
                <Button
                  leftSection={<IconClipboard size={16} />}
                  onClick={handlePasteNodes}
                  variant="light"
                  size="sm"
                >
                  Paste Nodes
                </Button>
              )}
              {activeTab === 'edges' && (
                <Button
                  leftSection={<IconClipboard size={16} />}
                  onClick={handlePasteEdges}
                  variant="light"
                  size="sm"
                >
                  Paste Edges
                </Button>
              )}
              {activeTab === 'layers' && (
                <Button
                  leftSection={<IconClipboard size={16} />}
                  onClick={handlePasteLayers}
                  variant="light"
                  size="sm"
                >
                  Paste Layers
                </Button>
              )}
              <Button
                leftSection={<IconDeviceFloppy size={16} />}
                onClick={handleSave}
                disabled={!hasChanges}
                loading={saving}
              >
                Save Changes
              </Button>
            </>
          )}
        </Group>
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
