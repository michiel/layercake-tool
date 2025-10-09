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
      size="90%"
      padding={0}
      styles={{
        body: {
          padding: 0,
          height: '75vh',
        },
        content: {
          maxHeight: '90vh',
        },
      }}
    >
      {data && <GraphPreview data={data} />}
    </Modal>
  );
};
