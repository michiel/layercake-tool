import { useMemo, useState, useEffect } from 'react';
import { IconLayout2, IconHierarchy, IconX, IconAlertCircle } from '@tabler/icons-react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Spinner } from '@/components/ui/spinner';
import { Stack } from '@/components/layout-primitives';
import { GraphPreview, GraphData } from './GraphPreview';

interface GraphPreviewDialogProps {
  opened: boolean;
  onClose: () => void;
  data: GraphData | null;
  title?: string;
  loading?: boolean;
  error?: string | null;
}

export const GraphPreviewDialog = ({ opened, onClose, data, title, loading = false, error }: GraphPreviewDialogProps) => {
  const [tab, setTab] = useState<string | null>('flow');

  const normalizedData = useMemo(() => {
    if (!data) return { flow: null, hierarchy: null };

    const partitionFlags = new Set(['true', '1', 'yes', 'TRUE']);
    const flowNodes = data.nodes.filter(node => {
      const flag = node.attrs?.is_partition ?? node.attrs?.isPartition ?? '';
      return !partitionFlags.has(flag);
    });
    const flowIds = new Set(flowNodes.map(node => node.id));
    const flowEdges = data.links.filter(link => flowIds.has(link.source) && flowIds.has(link.target));
    const allNodeIds = new Set(data.nodes.map(node => node.id));

    const hierarchyNodes = data.nodes.map(node => {
      const attrs = { ...node.attrs };
      attrs.isPartition = 'false';
      return { ...node, attrs };
    });

    const hierarchyEdges = data.nodes
      .map(node => {
        const parent = node.attrs?.belongs_to ?? node.attrs?.belongsTo;
        return { node, parent };
      })
      .filter(({ parent }) => parent && allNodeIds.has(parent))
      .map(({ node, parent }) => ({
        id: `hierarchy-${parent}-${node.id}`,
        source: parent as string,
        target: node.id,
        name: '',
        layer: node.layer,
        attrs: {},
      }));

    return {
      flow: {
        nodes: flowNodes,
        links: flowEdges,
        layers: data.layers,
      },
      hierarchy: {
        nodes: hierarchyNodes,
        links: hierarchyEdges,
        layers: data.layers,
      },
    };
  }, [data]);

  useEffect(() => {
    if (opened) {
      setTab('flow');
    }
  }, [opened]);

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-[90vw] h-[90vh] flex flex-col">
        <DialogHeader className="flex flex-row items-center justify-between pr-10">
          <DialogTitle>{title || 'Graph Preview'}</DialogTitle>
          <Button
            variant="ghost"
            size="icon"
            className="absolute right-4 top-4"
            onClick={onClose}
          >
            <IconX size={18} />
          </Button>
        </DialogHeader>
        <div className="flex-1 overflow-hidden">
          {loading ? (
            <Stack align="center" justify="center" className="h-full">
              <Spinner size="lg" />
              <p className="text-sm text-muted-foreground">Loading graph preview…</p>
            </Stack>
          ) : error ? (
            <Alert variant="destructive">
              <IconAlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : data ? (
            <Tabs value={tab || 'flow'} onValueChange={setTab} className="h-full flex flex-col">
              <TabsList>
                <TabsTrigger value="flow" className="flex items-center gap-2">
                  <IconLayout2 size={16} />
                  Flow
                </TabsTrigger>
                <TabsTrigger value="hierarchy" className="flex items-center gap-2">
                  <IconHierarchy size={16} />
                  Hierarchy
                </TabsTrigger>
              </TabsList>

              <TabsContent value="flow" className="flex-1 mt-4" style={{ height: '75vh' }}>
                {normalizedData.flow ? (
                  <GraphPreview
                    key={`flow-${normalizedData.flow.nodes.length}-${normalizedData.flow.links.length}-${tab}`}
                    data={normalizedData.flow}
                  />
                ) : (
                  <p className="text-sm text-muted-foreground">No flow data available.</p>
                )}
              </TabsContent>
              <TabsContent value="hierarchy" className="flex-1 mt-4" style={{ height: '75vh' }}>
                {normalizedData.hierarchy ? (
                  <GraphPreview
                    key={`hierarchy-${normalizedData.hierarchy.nodes.length}-${normalizedData.hierarchy.links.length}-${tab}`}
                    data={normalizedData.hierarchy}
                  />
                ) : (
                  <p className="text-sm text-muted-foreground">No hierarchy data available.</p>
                )}
              </TabsContent>
            </Tabs>
          ) : (
            <p className="text-sm text-muted-foreground">No preview data available.</p>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
};
