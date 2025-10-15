import { useMemo, useState, useEffect } from 'react';
import { Modal, Tabs, Group, Title, ActionIcon } from '@mantine/core';
import { IconLayout2, IconHierarchy, IconX } from '@tabler/icons-react';
import { GraphPreview, GraphData } from './GraphPreview';

interface GraphPreviewDialogProps {
  opened: boolean;
  onClose: () => void;
  data: GraphData | null;
  title?: string;
}

export const GraphPreviewDialog = ({ opened, onClose, data, title }: GraphPreviewDialogProps) => {
  const [tab, setTab] = useState<string | null>('flow');

  const normalizedData = useMemo(() => {
    if (!data) return { flow: null, hierarchy: null };

    const flowNodes = data.nodes.filter(node => {
      const flag = node.attrs?.is_partition ?? node.attrs?.isPartition;
      return !(flag === 'true' || flag === '1' || flag === 'yes' || flag === 'TRUE');
    });
    const flowIds = new Set(flowNodes.map(node => node.id));
    const flowEdges = data.links.filter(link => flowIds.has(link.source) && flowIds.has(link.target));

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
      .filter(({ parent }) => parent && data.nodes.some(candidate => candidate.id === parent))
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
    <Modal
      opened={opened}
      onClose={onClose}
      withCloseButton={false}
      size="90%"
      withinPortal
      padding="md"
    >
      <Group justify="space-between" mb="md">
        <Title order={4}>{title || 'Graph Preview'}</Title>
        <ActionIcon variant="subtle" onClick={onClose}>
          <IconX size={18} />
        </ActionIcon>
      </Group>
      <Tabs value={tab} onChange={setTab}>
        <Tabs.List>
          <Tabs.Tab value="flow" leftSection={<IconLayout2 size={16} />}>
            Flow
          </Tabs.Tab>
          <Tabs.Tab value="hierarchy" leftSection={<IconHierarchy size={16} />}>
            Hierarchy
          </Tabs.Tab>
        </Tabs.List>

        <Tabs.Panel value="flow" pt="md" style={{ height: '75vh' }}>
          {normalizedData.flow && (
            <GraphPreview
              key={`flow-${normalizedData.flow.nodes.length}-${normalizedData.flow.links.length}-${tab}`}
              data={normalizedData.flow}
            />
          )}
        </Tabs.Panel>
        <Tabs.Panel value="hierarchy" pt="md" style={{ height: '75vh' }}>
          {normalizedData.hierarchy && (
            <GraphPreview
              key={`hierarchy-${normalizedData.hierarchy.nodes.length}-${normalizedData.hierarchy.links.length}-${tab}`}
              data={normalizedData.hierarchy}
            />
          )}
        </Tabs.Panel>
      </Tabs>
    </Modal>
  );
};
