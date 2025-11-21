import React from 'react';
import { NodePropertiesForm } from './NodePropertiesForm';
import { Graph, GraphNode } from '../../graphql/graphs';
import { DataSet } from '../../graphql/datasets';
import { GraphViewMode, GraphOrientation, HierarchyViewMode } from './LayercakeGraphEditor';
import { Stack, Group } from '@/components/layout-primitives';
import { Paper } from '@/components/layout-primitives';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Slider } from '@/components/ui/slider';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion';

interface PropertiesAndLayersPanelProps {
  graph: Graph;
  selectedNodeId: string | null;
  onNodeUpdate: (nodeId: string, updates: Partial<GraphNode>) => void;
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
  datasets: DataSet[];
  targetNodeDatasetId: number | null;
  onTargetNodeDatasetChange: (id: number | null) => void;
  targetEdgeDatasetId: number | null;
  onTargetEdgeDatasetChange: (id: number | null) => void;
}

export const PropertiesAndLayersPanel: React.FC<PropertiesAndLayersPanelProps> = ({
  graph,
  selectedNodeId,
  onNodeUpdate,
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
  datasets,
  targetNodeDatasetId,
  onTargetNodeDatasetChange,
  targetEdgeDatasetId,
  onTargetEdgeDatasetChange,
}) => {
  const selectedNode = selectedNodeId
    ? graph.graphNodes.find(n => n.id === selectedNodeId)
    : null;

  return (
    <Paper
      className="shadow-sm p-4"
      style={{
        width: '320px',
        height: '100%',
        overflow: 'auto',
        borderLeft: '1px solid #e9ecef'
      }}
    >
      <Accordion
        type="multiple"
        defaultValue={['target-datasets', 'add-nodes', 'layout-options', 'node-properties']}
        className="space-y-2"
      >
        <AccordionItem value="target-datasets">
          <AccordionTrigger>Target Datasets</AccordionTrigger>
          <AccordionContent>
            <Stack gap="sm">
              <div className="space-y-1">
                <Label className="text-xs">Node dataset</Label>
                <Select
                  value={targetNodeDatasetId?.toString() || 'none'}
                  onValueChange={(value) => onTargetNodeDatasetChange(value === 'none' ? null : parseInt(value))}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="None" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">None</SelectItem>
                    {datasets.filter(d => d.status === 'active').map(dataset => (
                      <SelectItem key={dataset.id} value={dataset.id.toString()}>
                        {dataset.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label className="text-xs">Edge dataset</Label>
                <Select
                  value={targetEdgeDatasetId?.toString() || 'none'}
                  onValueChange={(value) => onTargetEdgeDatasetChange(value === 'none' ? null : parseInt(value))}
                >
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue placeholder="None" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">None</SelectItem>
                    {datasets.filter(d => d.status === 'active').map(dataset => (
                      <SelectItem key={dataset.id} value={dataset.id.toString()}>
                        {dataset.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <p className="text-xs text-muted-foreground">
                New nodes and edges will be added to selected datasets
              </p>
            </Stack>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="add-nodes">
          <AccordionTrigger>Add Nodes</AccordionTrigger>
          <AccordionContent>
            <Stack gap="xs">
              <Paper
                className="p-3"
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
                <p className="text-sm font-medium">Node</p>
                <p className="text-xs text-muted-foreground">Regular node</p>
              </Paper>

              <Paper
                className="p-3"
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
                <p className="text-sm font-medium">Container</p>
                <p className="text-xs text-muted-foreground">Partition node</p>
              </Paper>
            </Stack>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="layout-options">
          <AccordionTrigger>Layout Options</AccordionTrigger>
          <AccordionContent>
            <Stack gap="md">
              <Group gap="xs" wrap={true}>
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={onToggleViewMode}
                >
                  {viewMode === 'flow' ? 'Hierarchy' : 'Flow'}
                </Button>

                <Button
                  size="sm"
                  variant="secondary"
                  onClick={onToggleOrientation}
                >
                  {orientation === 'vertical' ? 'LR' : 'TD'}
                </Button>

                {viewMode === 'flow' && (
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={onToggleFlowGrouping}
                  >
                    {flowGroupingEnabled ? 'Disable Groupings' : 'Enable Groupings'}
                  </Button>
                )}

                {viewMode === 'hierarchy' && (
                  <Button
                    size="sm"
                    variant="secondary"
                    onClick={onToggleHierarchyViewMode}
                  >
                    {hierarchyViewMode === 'graph' ? 'As Containers' : 'As Graph'}
                  </Button>
                )}
              </Group>

              <div>
                <Group justify="between" className="mb-1">
                  <p className="text-xs text-muted-foreground">Node Spacing</p>
                  <p className="text-xs font-medium">{nodeSpacing}</p>
                </Group>
                <Slider
                  value={[nodeSpacing]}
                  onValueChange={([value]) => onNodeSpacingChange(value)}
                  min={20}
                  max={200}
                  step={5}
                />
              </div>

              <div>
                <Group justify="between" className="mb-1">
                  <p className="text-xs text-muted-foreground">Rank Spacing</p>
                  <p className="text-xs font-medium">{rankSpacing}</p>
                </Group>
                <Slider
                  value={[rankSpacing]}
                  onValueChange={([value]) => onRankSpacingChange(value)}
                  min={20}
                  max={200}
                  step={5}
                />
              </div>

              <div>
                <Group justify="between" className="mb-1">
                  <p className="text-xs text-muted-foreground">Min Edge Length</p>
                  <p className="text-xs font-medium">{minEdgeLength}</p>
                </Group>
                <Slider
                  value={[minEdgeLength]}
                  onValueChange={([value]) => onMinEdgeLengthChange(value)}
                  min={20}
                  max={200}
                  step={5}
                />
              </div>
            </Stack>
          </AccordionContent>
        </AccordionItem>

        <AccordionItem value="node-properties">
          <AccordionTrigger>Node Properties</AccordionTrigger>
          <AccordionContent>
            {selectedNode ? (
              <NodePropertiesForm
                node={selectedNode}
                layers={graph.layers}
                onUpdate={(updates) => onNodeUpdate(selectedNode.id, updates)}
              />
            ) : (
              <div className="text-muted-foreground text-sm text-center py-5">
                Select a node to view properties
              </div>
            )}
          </AccordionContent>
        </AccordionItem>

      </Accordion>
    </Paper>
  );
};
