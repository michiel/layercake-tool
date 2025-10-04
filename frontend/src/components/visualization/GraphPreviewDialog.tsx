import { Modal } from '@mantine/core';
import { GraphPreview, GraphData } from './GraphPreview';

interface GraphPreviewDialogProps {
  opened: boolean;
  onClose: () => void;
  data: GraphData | null;
  title?: string;
}

export const GraphPreviewDialog = ({ opened, onClose, data, title }: GraphPreviewDialogProps) => {
  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={title || 'Graph Preview'}
      size="100%"
      fullScreen
      padding={0}
      styles={{
        body: {
          height: 'calc(100vh - 60px)',
          padding: 0,
        },
      }}
    >
      {data && <GraphPreview data={data} />}
    </Modal>
  );
};
