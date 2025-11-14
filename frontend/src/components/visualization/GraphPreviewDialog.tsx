import { useMemo, useState, useEffect } from 'react';
import { IconLayout2, IconHierarchy, IconAlertCircle, IconMaximize, IconMinimize } from '@tabler/icons-react';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Spinner } from '@/components/ui/spinner';
import { Stack } from '@/components/layout-primitives';
import { cn } from '@/lib/utils';
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
  const [isFullscreen, setIsFullscreen] = useState(false);

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
      setIsFullscreen(false);
    }
  }, [opened]);

  return (
    <Dialog open={opened} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        className={cn(
          'max-w-[90vw] h-[90vh] flex flex-col',
          isFullscreen && 'max-w-[100vw] w-screen h-screen sm:rounded-none !left-0 !top-0 !translate-x-0 !translate-y-0'
        )}
      >
        <button
          type="button"
          onClick={() => setIsFullscreen(prev => !prev)}
          className="absolute right-12 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
          title={isFullscreen ? 'Exit full screen' : 'Enter full screen'}
          aria-label={isFullscreen ? 'Exit full screen' : 'Enter full screen'}
        >
          {isFullscreen ? <IconMinimize className="h-4 w-4" /> : <IconMaximize className="h-4 w-4" />}
        </button>
        <DialogHeader>
          <DialogTitle>{title || 'Graph Preview'}</DialogTitle>
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

export default GraphPreviewDialog
