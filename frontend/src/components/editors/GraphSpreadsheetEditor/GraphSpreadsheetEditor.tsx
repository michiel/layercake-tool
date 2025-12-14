import React, { useEffect, useState } from 'react';
import { IconTable, IconDeviceFloppy, IconClipboard, IconClipboardCopy, IconTrash, IconPlus, IconX } from '@tabler/icons-react';
import { Stack, Group } from '@/components/layout-primitives';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { AttributesMap, attributesToInlineString, attributesToJson, parseAttributesInline, parseAttributesJson, sanitizeAttributes } from '@/utils/attributes';
import { AttributesEditorDialog } from '@/components/attributes/AttributesEditorDialog';
import { IconEdit } from '@tabler/icons-react';

export interface GraphNode {
  id: string;
  label: string;
  layer?: string;
  weight?: number;
  is_partition?: boolean;
  belongs_to?: string;
  comment?: string;
  attributes?: AttributesMap;
  [key: string]: any;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  label?: string;
  layer?: string;
  weight?: number;
  comment?: string;
  attributes?: AttributesMap;
  [key: string]: any;
}

export interface GraphLayer {
  id: string;
  label: string;
  alias?: string;
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
  layersReadOnly?: boolean;
}

export const GraphSpreadsheetEditor: React.FC<GraphSpreadsheetEditorProps> = ({
  graphData,
  onSave,
  readOnly = false,
  layersReadOnly = false
}) => {
  const coerceBoolean = (value: any): boolean => {
    if (typeof value === 'boolean') return value;
    if (typeof value === 'string') {
      const normalized = value.trim().toLowerCase();
      return ['true', '1', 'yes', 'y', 'on'].includes(normalized);
    }
    return Boolean(value);
  };

  const normalizeNode = (node: GraphNode): GraphNode => ({
    ...node,
    attributes: sanitizeAttributes(node.attributes || (node as any).attrs),
  });

  const normalizeEdge = (edge: GraphEdge): GraphEdge => ({
    ...edge,
    attributes: sanitizeAttributes(edge.attributes || (edge as any).attrs),
  });

  const [activeTab, setActiveTab] = useState<string>('nodes');
  const [localNodes, setLocalNodes] = useState<GraphNode[]>(
    (graphData.nodes || []).map(normalizeNode)
  );
  const [localEdges, setLocalEdges] = useState<GraphEdge[]>(
    (graphData.edges || []).map(normalizeEdge)
  );
  const [localLayers, setLocalLayers] = useState<GraphLayer[]>(graphData.layers || []);
  const [hasChanges, setHasChanges] = useState(false);
  const [saving, setSaving] = useState(false);
  const [attributesDialog, setAttributesDialog] = useState<{ type: 'node' | 'edge'; index: number } | null>(null);
  const [attributesTextNodes, setAttributesTextNodes] = useState<Record<string, string>>({});
  const [attributesTextEdges, setAttributesTextEdges] = useState<Record<string, string>>({});
  const [attributesErrors, setAttributesErrors] = useState<Record<string, string>>({});
  const layerEditingDisabled = readOnly || layersReadOnly;

  useEffect(() => {
    setLocalNodes((graphData.nodes || []).map(normalizeNode));
    setLocalEdges((graphData.edges || []).map(normalizeEdge));
    setLocalLayers(graphData.layers || []);
    setHasChanges(false);
    setAttributesTextNodes(
      Object.fromEntries(
        (graphData.nodes || []).map(n => [
          n.id,
          attributesToInlineString(sanitizeAttributes(n.attributes || (n as any).attrs)),
        ])
      )
    );
    setAttributesTextEdges(
      Object.fromEntries(
        (graphData.edges || []).map(e => [
          e.id,
          attributesToInlineString(sanitizeAttributes(e.attributes || (e as any).attrs)),
        ])
      )
    );
    setAttributesErrors({});
  }, [graphData]);

  const nodeColumnDefs = ['id', 'label', 'layer', 'weight', 'is_partition', 'belongs_to', 'comment', 'attributes'];
  const edgeColumnDefs = ['id', 'source', 'target', 'label', 'layer', 'weight', 'comment', 'attributes'];
  const layerColumnDefs = ['id', 'label', 'alias', 'background_color', 'text_color', 'border_color', 'comment'];

  const handleAddRecord = () => {
    if (readOnly) {
      return;
    }
    if (activeTab === 'nodes') {
      setLocalNodes(prev => [
        ...prev,
        { id: '', label: '', layer: '', weight: undefined, is_partition: false, belongs_to: '', comment: '', attributes: {} }
      ]);
      setHasChanges(true);
      return;
    }
    if (activeTab === 'edges') {
      setLocalEdges(prev => [
        ...prev,
        { id: '', source: '', target: '', label: '', layer: '', weight: undefined, comment: '', attributes: {} }
      ]);
      setHasChanges(true);
      return;
    }
    if (!layerEditingDisabled && activeTab === 'layers') {
      setLocalLayers(prev => [
        ...prev,
        {
          id: '',
          label: '',
          alias: '',
          background_color: '',
          text_color: '',
          border_color: '',
          comment: ''
        }
      ]);
      setHasChanges(true);
    }
  };

  const handleAttributesInlineChange = (
    type: 'node' | 'edge',
    id: string,
    text: string,
    rowIdx: number
  ) => {
    const parsed = parseAttributesInline(text);
    if (type === 'node') {
      setAttributesTextNodes(prev => ({ ...prev, [id]: text }));
      if (parsed.ok) {
        setAttributesErrors(prev => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
        setLocalNodes(prev => {
          const next = [...prev];
          const target = next[rowIdx];
          if (target) next[rowIdx] = { ...target, attributes: parsed.value };
          return next;
        });
      } else {
        setAttributesErrors(prev => ({ ...prev, [id]: parsed.error }));
      }
    } else {
      setAttributesTextEdges(prev => ({ ...prev, [id]: text }));
      if (parsed.ok) {
        setAttributesErrors(prev => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
        setLocalEdges(prev => {
          const next = [...prev];
          const target = next[rowIdx];
          if (target) next[rowIdx] = { ...target, attributes: parsed.value };
          return next;
        });
      } else {
        setAttributesErrors(prev => ({ ...prev, [id]: parsed.error }));
      }
    }
    setHasChanges(true);
  };

  const handleDeleteNode = (index: number) => {
    if (readOnly) {
      return;
    }
    setLocalNodes(prev => prev.filter((_, idx) => idx !== index));
    setHasChanges(true);
  };

  const handleDeleteEdge = (index: number) => {
    if (readOnly) {
      return;
    }
    setLocalEdges(prev => prev.filter((_, idx) => idx !== index));
    setHasChanges(true);
  };

  const handleDeleteLayer = (index: number) => {
    if (layerEditingDisabled) {
      return;
    }
    setLocalLayers(prev => prev.filter((_, idx) => idx !== index));
    setHasChanges(true);
  };

  const handleNodeChange = (rowIdx: number, field: string, value: string) => {
    setLocalNodes(prevNodes => {
      const newNodes = [...prevNodes];
      const current = newNodes[rowIdx];
      if (!current) return newNodes;

      // Normalize booleans for is_partition
      const nextValue =
        field === 'is_partition' ? value === 'true' || value === '1' || value === 'on' : value;

      newNodes[rowIdx] = {
        ...current,
        [field]: nextValue
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
    if (layerEditingDisabled) {
      return;
    }
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
      const normalizedNodes = localNodes.map(node => ({
        ...node,
        is_partition: coerceBoolean(node.is_partition),
        attributes: sanitizeAttributes(node.attributes),
      }));
      const normalizedEdges = localEdges.map(edge => ({
        ...edge,
        attributes: sanitizeAttributes(edge.attributes),
      }));
      await onSave({
        nodes: normalizedNodes,
        edges: normalizedEdges,
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

  // Convert data to CSV format
  const toCSV = (data: Record<string, any>[], columns: string[]): string => {
    if (data.length === 0) return columns.join(',') + '\n';

    const escapeCSV = (value: any): string => {
      const str = String(value ?? '');
      if (str.includes(',') || str.includes('"') || str.includes('\n')) {
        return `"${str.replace(/"/g, '""')}"`;
      }
      return str;
    };

    const header = columns.join(',');
    const formatValue = (row: Record<string, any>, col: string) => {
      if (col === 'attributes') {
        return attributesToJson(sanitizeAttributes(row.attributes));
      }
      return row[col];
    };
    const rows = data.map(row =>
      columns.map(col => escapeCSV(formatValue(row, col))).join(',')
    );

    return [header, ...rows].join('\n');
  };

  const handleCopyNodes = async () => {
    const csv = toCSV(localNodes, nodeColumnDefs);
    try {
      await navigator.clipboard.writeText(csv);
      alert(`Copied ${localNodes.length} nodes to clipboard`);
    } catch (error) {
      console.error('Failed to copy nodes:', error);
      alert('Failed to copy to clipboard');
    }
  };

  const handleCopyEdges = async () => {
    const csv = toCSV(localEdges, edgeColumnDefs);
    try {
      await navigator.clipboard.writeText(csv);
      alert(`Copied ${localEdges.length} edges to clipboard`);
    } catch (error) {
      console.error('Failed to copy edges:', error);
      alert('Failed to copy to clipboard');
    }
  };

  const handleCopyLayers = async () => {
    const csv = toCSV(localLayers, layerColumnDefs);
    try {
      await navigator.clipboard.writeText(csv);
      alert(`Copied ${localLayers.length} layers to clipboard`);
    } catch (error) {
      console.error('Failed to copy layers:', error);
      alert('Failed to copy to clipboard');
    }
  };

  const confirmDestructiveAction = (subject: string, existingCount: number) => {
    if (existingCount === 0) {
      return true;
    }

    return window.confirm(
      `This will replace the existing ${existingCount} ${subject}. Continue?`
    );
  };

  const handlePasteNodes = async () => {
    if (!confirmDestructiveAction('node records', localNodes.length)) {
      return;
    }

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
        weight: row.weight ? parseFloat(row.weight) : 1,
        is_partition: row.is_partition === 'true' || row.is_partition === '1',
        belongs_to: row.belongs_to,
        comment: row.comment,
        attributes: sanitizeAttributes(parseAttributesJson((row as any).attributes)),
      }));

      setLocalNodes(() => [...newNodes]);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste nodes:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  const handlePasteEdges = async () => {
    if (!confirmDestructiveAction('edge records', localEdges.length)) {
      return;
    }

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
        weight: row.weight ? parseFloat(row.weight) : 1,
        comment: row.comment,
        attributes: sanitizeAttributes(parseAttributesJson((row as any).attributes)),
      }));

      setLocalEdges(() => [...newEdges]);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste edges:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  const handlePasteLayers = async () => {
    if (layerEditingDisabled) {
      return;
    }
    if (!confirmDestructiveAction('layer records', localLayers.length)) {
      return;
    }

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

      setLocalLayers(() => [...newLayers]);
      setHasChanges(true);
    } catch (error) {
      console.error('Failed to paste layers:', error);
      alert('Failed to read from clipboard. Please ensure you have copied CSV data.');
    }
  };

  const handleClearNodes = () => {
    if (!confirmDestructiveAction('node records', localNodes.length)) {
      return;
    }

    setLocalNodes([]);
    setAttributesTextNodes({});
    setHasChanges(true);
  };

  const handleClearEdges = () => {
    if (!confirmDestructiveAction('edge records', localEdges.length)) {
      return;
    }

    setLocalEdges([]);
    setAttributesTextEdges({});
    setHasChanges(true);
  };

  const handleClearLayers = () => {
    if (layerEditingDisabled) {
      return;
    }
    if (!confirmDestructiveAction('layer records', localLayers.length)) {
      return;
    }

    setLocalLayers([]);
    setHasChanges(true);
  };

  const handleAttributesSave = (next: AttributesMap) => {
    if (!attributesDialog) return;
    if (attributesDialog.type === 'node') {
      setLocalNodes(prev => {
        const updated = [...prev];
        const target = updated[attributesDialog.index];
        if (target) {
          updated[attributesDialog.index] = { ...target, attributes: sanitizeAttributes(next) };
        }
        return updated;
      });
    } else {
      setLocalEdges(prev => {
        const updated = [...prev];
        const target = updated[attributesDialog.index];
        if (target) {
          updated[attributesDialog.index] = { ...target, attributes: sanitizeAttributes(next) };
        }
        return updated;
      });
    }
    setHasChanges(true);
    setAttributesDialog(null);
  };

  const currentAttributes =
    attributesDialog?.type === 'node'
      ? sanitizeAttributes(localNodes[attributesDialog.index]?.attributes)
      : sanitizeAttributes(localEdges[attributesDialog?.index ?? 0]?.attributes);

  return (
    <Stack gap="md">
      <Group justify="between">
        <p className="text-sm text-muted-foreground">
          {activeTab === 'nodes' && `${localNodes.length} nodes`}
          {activeTab === 'edges' && `${localEdges.length} edges`}
          {activeTab === 'layers' && `${localLayers.length} layers`}
        </p>

        <Group gap="xs">
          {!readOnly && (
            <>
              <Button
                onClick={handleAddRecord}
                variant="secondary"
                size="icon"
                disabled={activeTab === 'layers' && layerEditingDisabled}
                title="Add record"
              >
                <IconPlus className="h-4 w-4" />
              </Button>
              {activeTab === 'nodes' && (
                <>
                  <Button
                    onClick={handleCopyNodes}
                    variant="outline"
                    size="sm"
                  >
                    <IconClipboardCopy className="mr-2 h-4 w-4" />
                    Copy Nodes
                  </Button>
                  <Button
                    onClick={handlePasteNodes}
                    variant="secondary"
                    size="sm"
                  >
                    <IconClipboard className="mr-2 h-4 w-4" />
                    Paste Nodes
                  </Button>
                  <Button
                    onClick={handleClearNodes}
                    variant="destructive"
                    size="sm"
                  >
                    <IconTrash className="mr-2 h-4 w-4" />
                    Clear Nodes
                  </Button>
                </>
              )}
              {activeTab === 'edges' && (
                <>
                  <Button
                    onClick={handleCopyEdges}
                    variant="outline"
                    size="sm"
                  >
                    <IconClipboardCopy className="mr-2 h-4 w-4" />
                    Copy Edges
                  </Button>
                  <Button
                    onClick={handlePasteEdges}
                    variant="secondary"
                    size="sm"
                  >
                    <IconClipboard className="mr-2 h-4 w-4" />
                    Paste Edges
                  </Button>
                  <Button
                    onClick={handleClearEdges}
                    variant="destructive"
                    size="sm"
                  >
                    <IconTrash className="mr-2 h-4 w-4" />
                    Clear Edges
                  </Button>
                </>
              )}
              {activeTab === 'layers' && (
                <>
                  <Button
                    onClick={handleCopyLayers}
                    variant="outline"
                    size="sm"
                  >
                    <IconClipboardCopy className="mr-2 h-4 w-4" />
                    Copy Layers
                  </Button>
                  {!layersReadOnly && (
                    <>
                      <Button
                        onClick={handlePasteLayers}
                        variant="secondary"
                        size="sm"
                      >
                        <IconClipboard className="mr-2 h-4 w-4" />
                        Paste Layers
                      </Button>
                      <Button
                        onClick={handleClearLayers}
                        variant="destructive"
                        size="sm"
                      >
                        <IconTrash className="mr-2 h-4 w-4" />
                        Clear Layers
                      </Button>
                    </>
                  )}
                </>
              )}
              <Button
                onClick={handleSave}
                disabled={!hasChanges || saving}
              >
                <IconDeviceFloppy className="mr-2 h-4 w-4" />
                {saving ? 'Saving...' : 'Save Changes'}
              </Button>
            </>
          )}
        </Group>
      </Group>

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList>
          <TabsTrigger value="nodes">
            <IconTable className="mr-2 h-4 w-4" />
            Nodes ({localNodes.length})
          </TabsTrigger>
          <TabsTrigger value="edges">
            <IconTable className="mr-2 h-4 w-4" />
            Edges ({localEdges.length})
          </TabsTrigger>
          <TabsTrigger value="layers">
            <IconTable className="mr-2 h-4 w-4" />
            Layers ({localLayers.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="nodes" className="pt-4">
          <ScrollArea className="h-[600px]">
            <Table>
              <TableHeader>
                <TableRow>
                  {nodeColumnDefs.map(col => (
                    <TableHead key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </TableHead>
                  ))}
                  {!readOnly && (
                    <TableHead className="w-12 text-right">
                      <span className="sr-only">Node Actions</span>
                    </TableHead>
                  )}
                </TableRow>
              </TableHeader>
              <TableBody>
                {localNodes.map((node, rowIdx) => (
                  <TableRow key={rowIdx}>
                    {nodeColumnDefs.map(col => (
                      <TableCell key={col}>
                        {col === 'attributes' ? (
                          <div className="flex items-center gap-2">
                            <div className="flex flex-col flex-1 gap-1">
                              <Input
                                value={
                                  attributesTextNodes[node.id] ??
                                  attributesToInlineString(node.attributes)
                                }
                                onChange={(e) =>
                                  handleAttributesInlineChange('node', node.id, e.target.value, rowIdx)
                                }
                                className={`h-7 text-xs px-2 ${attributesErrors[node.id] ? 'border-destructive' : ''}`}
                                readOnly={readOnly}
                                placeholder="key:val;priority:1"
                              />
                              {attributesErrors[node.id] && (
                                <p className="text-[11px] text-destructive">
                                  {attributesErrors[node.id]}
                                </p>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => setAttributesDialog({ type: 'node', index: rowIdx })}
                              title="Edit attributes"
                            >
                              <IconEdit className="h-4 w-4" />
                            </Button>
                          </div>
                        ) : readOnly ? (
                          <p className="text-sm">{String(node[col] ?? '')}</p>
                        ) : (
                          <Input
                            value={String(node[col] ?? '')}
                            onChange={(e) => handleNodeChange(rowIdx, col, e.currentTarget.value)}
                            className="h-7 text-xs border-none px-2"
                            readOnly={col === 'attributes'}
                          />
                        )}
                      </TableCell>
                    ))}
                    {!readOnly && (
                      <TableCell className="text-right">
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => handleDeleteNode(rowIdx)}
                          className="h-6 w-6 text-muted-foreground"
                          title="Delete node"
                        >
                          <IconX className="h-3.5 w-3.5" />
                        </Button>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        </TabsContent>

        <TabsContent value="edges" className="pt-4">
          <ScrollArea className="h-[600px]">
            <Table>
              <TableHeader>
                <TableRow>
                  {edgeColumnDefs.map(col => (
                    <TableHead key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </TableHead>
                  ))}
                  {!readOnly && (
                    <TableHead className="w-12 text-right">
                      <span className="sr-only">Edge Actions</span>
                    </TableHead>
                  )}
                </TableRow>
              </TableHeader>
              <TableBody>
                {localEdges.map((edge, rowIdx) => (
                  <TableRow key={rowIdx}>
                    {edgeColumnDefs.map(col => (
                      <TableCell key={col}>
                        {col === 'attributes' ? (
                          <div className="flex items-center gap-2">
                            <div className="flex flex-col flex-1 gap-1">
                              <Input
                                value={
                                  attributesTextEdges[edge.id] ??
                                  attributesToInlineString(edge.attributes)
                                }
                                onChange={(e) =>
                                  handleAttributesInlineChange('edge', edge.id, e.target.value, rowIdx)
                                }
                                className={`h-7 text-xs px-2 ${attributesErrors[edge.id] ? 'border-destructive' : ''}`}
                                readOnly={readOnly}
                                placeholder="key:val;priority:1"
                              />
                              {attributesErrors[edge.id] && (
                                <p className="text-[11px] text-destructive">
                                  {attributesErrors[edge.id]}
                                </p>
                              )}
                            </div>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => setAttributesDialog({ type: 'edge', index: rowIdx })}
                              title="Edit attributes"
                            >
                              <IconEdit className="h-4 w-4" />
                            </Button>
                          </div>
                        ) : readOnly ? (
                          <p className="text-sm">{String(edge[col] ?? '')}</p>
                        ) : (
                          <Input
                            value={String(edge[col] ?? '')}
                            onChange={(e) => handleEdgeChange(rowIdx, col, e.currentTarget.value)}
                            className="h-7 text-xs border-none px-2"
                          />
                        )}
                      </TableCell>
                    ))}
                    {!readOnly && (
                      <TableCell className="text-right">
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => handleDeleteEdge(rowIdx)}
                          className="h-6 w-6 text-muted-foreground"
                          title="Delete edge"
                        >
                          <IconX className="h-3.5 w-3.5" />
                        </Button>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        </TabsContent>

        <TabsContent value="layers" className="pt-4">
          <ScrollArea className="h-[600px]">
            <Table>
              <TableHeader>
                <TableRow>
                  {layerColumnDefs.map(col => (
                    <TableHead key={col} style={{ minWidth: 150 }}>
                      {col.replace(/_/g, ' ').toUpperCase()}
                    </TableHead>
                  ))}
                  {!layerEditingDisabled && (
                    <TableHead className="w-12 text-right">
                      <span className="sr-only">Layer Actions</span>
                    </TableHead>
                  )}
                </TableRow>
              </TableHeader>
              <TableBody>
                {localLayers.map((layer, rowIdx) => (
                  <TableRow key={rowIdx}>
                    {layerColumnDefs.map(col => (
                      <TableCell key={col}>
                        {layerEditingDisabled ? (
                          <p className="text-sm">{String(layer[col] ?? '')}</p>
                        ) : (
                          <Input
                            value={String(layer[col] ?? '')}
                            onChange={(e) => handleLayerChange(rowIdx, col, e.currentTarget.value)}
                            className="h-7 text-xs border-none px-2"
                          />
                        )}
                      </TableCell>
                    ))}
                    {!layerEditingDisabled && (
                      <TableCell className="text-right">
                        <Button
                          size="icon"
                          variant="ghost"
                          onClick={() => handleDeleteLayer(rowIdx)}
                          className="h-6 w-6 text-muted-foreground"
                          title="Delete layer"
                        >
                          <IconX className="h-3.5 w-3.5" />
                        </Button>
                      </TableCell>
                    )}
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </ScrollArea>
        </TabsContent>
      </Tabs>
      <AttributesEditorDialog
        open={!!attributesDialog}
        initialValue={currentAttributes}
        onClose={() => setAttributesDialog(null)}
        onSave={handleAttributesSave}
        title="Attributes"
      />
    </Stack>
  );
};
